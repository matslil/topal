"""Read saved boot structures; no guest state or publication authority is created.

TOPAL-SYSTEMS-USER-EXTENT-001. This reader deliberately accepts only the current
private component layout. Unsupported layouts need their own qualification.
"""
import struct

from inspect_call_inventory import digest


BASE = 0x100000
GDT = 0x104000
IDT = 0x105000
IDTR = 0x106000
TABLES = (0x101000, 0x102000, 0x103000, 0x107000)
GATES = {
    3: ('topal_x86_systems_debug_break_entry', 15),
    14: ('topal_x86_systems_user_copy_page_fault', 14),
    0xf1: ('topal_x86_systems_local_notification_entry', 14),
    0xf2: ('topal_x86_systems_deadline_entry', 14),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def take(raw, at, size):
    require(type(at) is int and type(size) is int and 0 <= at <= at + size <= len(raw),
            'saved structure outside file')
    return raw[at:at + size]


def number(raw, at, size):
    return int.from_bytes(take(raw, at, size), 'little')


def load_segments(raw):
    require(take(raw, 0, 7) == b'\x7fELF\x02\x01\x01'
            and number(raw, 16, 2) == 2 and number(raw, 18, 2) == 62,
            'expected executable little-endian x86-64 ELF')
    require(number(raw, 52, 2) == 64 and number(raw, 54, 2) == 56, 'ELF header widths')
    offset, count = number(raw, 32, 8), number(raw, 56, 2)
    require(count > 0, 'missing ELF program headers')
    headers = take(raw, offset, count * 56)
    result = []
    for index in range(count):
        kind, flags, file_at, virtual, physical, size, memory, alignment = struct.unpack_from(
            '<II6Q', headers, index * 56)
        if kind != 1:
            continue
        require(0x200000 <= virtual < virtual + memory <= 0x40000000
                and physical == virtual and size <= memory, 'unsupported ELF load placement')
        require(alignment >= 4096 and alignment & (alignment - 1) == 0
                and virtual % alignment == file_at % alignment, 'ELF load alignment')
        data = take(raw, file_at, size)
        result.append(dict(index=index, flags=flags, file_offset=file_at, address=virtual,
                           file_bytes=size, memory_bytes=memory, alignment=alignment,
                           file_sha256=digest(data)))
    result.sort(key=lambda row: row['address'])
    require(bool(result), 'missing ELF load segments')
    for first, second in zip(result, result[1:]):
        require(first['address'] + first['memory_bytes'] <= second['address'], 'ELF loads overlap')
    return result


def bind_loads(kernel, payload, segments):
    end = max(row['address'] + row['memory_bytes'] for row in segments) - BASE
    require(len(payload) == (end + 15) // 16 * 16, 'protected payload size/load extent')
    require(not any(take(payload, end, len(payload) - end)), 'nonzero payload alignment tail')
    for row in segments:
        offset, size = row['address'] - BASE, row['file_bytes']
        require(take(payload, offset, size) == take(kernel, row['file_offset'], size),
                'packaged ELF load bytes differ')
        require(not any(take(payload, offset + size, row['memory_bytes'] - size)),
                'packaged BSS is not zero')


def descriptor(word):
    base = ((word >> 16) & 0xffffff) | ((word >> 32) & 0xff000000)
    limit = (word & 0xffff) | ((word >> 32) & 0xf0000)
    granular = (word >> 55) & 1
    return dict(base=base, limit=((limit << 12) | 0xfff) if granular else limit,
                present=bool(word & (1 << 47)), dpl=(word >> 45) & 3,
                application=bool(word & (1 << 44)), type=(word >> 40) & 15,
                long=bool(word & (1 << 53)), default32=bool(word & (1 << 54)))


def gdt(payload):
    raw = take(payload, GDT - BASE, 48)
    result = [descriptor(number(raw, index * 8, 8)) for index in range(6)]
    require(number(raw, 0, 8) == 0, 'GDT null descriptor')
    for index, kind, long, default in [(1, 10, False, True), (2, 10, True, False), (3, 2, False, True)]:
        row = result[index]
        require(row == dict(base=0, limit=0xffffffff, present=True, dpl=0, application=True,
                            type=kind, long=long, default32=default), 'GDT transition descriptor')
    return dict(address=GDT, sha256=digest(raw), descriptors=result,
                scope='Only selectors 8, 16 and 24 are checked for this transition; TSS/runtime GDT use remains separate.')


def idt(payload, targets):
    limit = number(payload, IDTR - BASE, 2)
    address = number(payload, IDTR - BASE + 2, 8)
    require(address == IDT and limit == 4095, 'IDTR base/limit')
    raw = take(payload, address - BASE, limit + 1)
    result, empty = [], []
    for vector in range(256):
        gate = raw[vector * 16:(vector + 1) * 16]
        if vector not in GATES:
            require(not any(gate), 'unexpected nonzero IDT gate')
            empty.append(vector)
            continue
        name, kind = GATES[vector]
        low, selector, ist, attributes, middle, high, reserved = struct.unpack('<HHBBHII', gate)
        target = low | (middle << 16) | (high << 32)
        require(ist == 0 and reserved == 0 and attributes == (0x80 | kind)
                and selector == 16, 'IDT selector/type/privilege/IST/reserved bits')
        require(target == targets[name], 'IDT target differs from linked executable symbol')
        result.append(dict(vector=vector, address=address + vector * 16, target=target,
                           symbol=name, selector=selector, ist=ist, dpl=0,
                           kind='trap' if kind == 15 else 'interrupt', bytes=gate.hex()))
    return dict(address=address, limit=limit, sha256=digest(raw), gates=result,
                empty_vectors=empty,
                scope='Initial saved gates only. Empty vectors do not establish exclusion or safe handling of those events.')


def paging(payload):
    expected = [{0: TABLES[1] | 3}, {0: TABLES[2] | 3, 3: TABLES[3] | 3},
                {i: i * 0x200000 | 0x83 for i in range(512)},
                {502: 0xfec00000 | 0x9b, 503: 0xfee00000 | 0x9b}]
    records = []
    for address, wanted in zip(TABLES, expected):
        raw = take(payload, address - BASE, 4096)
        require(all(number(raw, i * 8, 8) == wanted.get(i, 0) for i in range(512)),
                'initial paging table differs from selected layout')
        records.append(dict(address=address, sha256=digest(raw), populated_entries=len(wanted)))
    return dict(tables=records, identity_bytes=0x40000000, leaf_bytes=0x200000,
                mmio_leaf_addresses=[0xfec00000, 0xfee00000], user_access=False,
                writable=True, nx=False,
                scope='Saved initial table bytes only; broad supervisor writable/executable identity map, not W^X, runtime mode, ownership or continuous lifetime proof.')
