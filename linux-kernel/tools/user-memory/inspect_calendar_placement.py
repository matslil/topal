"""Check separated RTC data and an exact saved process-layout transformation.

ELF read-only/non-executable flags do not prove hardware permissions, stable
contents, complete entries or production qualification. No bytes are executed.
"""
import json
import struct

from calendar_data import YEAR_BYTES, calendar_payload, rip_references, year_guard, require
from inspect_call_inventory import Elf, digest
from inspect_callback_bindings import relocation_entries

FUNCTION = 'topal_x86_systems_static_image_process_enter'
YEAR = 'topal_x86_realtime_years_v1'
MONTH = 'topal_x86_realtime_months_v1'
HEADERS = 'topal_x86_systems_static_image_remove_stack_program_headers'
REFERENCE = 'linux-kernel/tools/process-memory/results/stage-3-exec-failures/native'


def symbol_data(elf, name):
    symbol = elf.symbols[name]
    section = elf.sections[symbol['section']]
    offset = symbol['value'] - section[3]
    require(0 <= offset and offset + symbol['size'] <= section[5], 'calendar symbol extent')
    require(section[2] & (7 | 0x400) == 2, 'calendar ELF data flags')
    return symbol, elf.section_bytes(symbol['section'])[offset:offset + symbol['size']]


def load_flags(elf, symbol):
    phoff = struct.unpack_from('<Q', elf.data, 32)[0]
    width, count = struct.unpack_from('<HH', elf.data, 54)
    require(width == 56, 'ELF program header width')
    matches = []
    for i in range(count):
        p = struct.unpack_from('<IIQQQQQQ', elf.data, phoff + i * width)
        if p[0] == 1 and p[3] <= symbol['value'] and symbol['value'] + symbol['size'] <= p[3] + p[5]:
            require(p[1] == 4, 'calendar initialized load must be ELF read-only/non-executable')
            matches.append(p[1])
    require(len(matches) == 1, 'calendar data lacks one initialized read-only load')
    return matches[0]


def layout(old, new, old_origins, new_origins, old_relocations, new_relocations):
    require(len(old) - len(new) == len(calendar_payload()), 'exact process data removal size')
    begin = old_origins['labels']['realtime-year-table']
    end = old_origins['labels']['realtime-month-table'] + len(calendar_payload()) - YEAR_BYTES
    require(old[begin:end] == calendar_payload(), 'removed bytes are Gregorian data')
    removed = end - begin
    def old_offset(at):
        return at if at < begin else at + removed
    labels = {k: v for k, v in old_origins['labels'].items()
              if k not in ['realtime-year-table', 'realtime-month-table']}
    require(set(labels) == set(new_origins['labels']), 'retained process label set')
    for name, at in new_origins['labels'].items():
        require(labels[name] == old_offset(at), 'retained process label offset')
    expected = bytearray(old[:begin] + old[end:])
    fixed = set()
    for at, name in new_origins['fixups']:
        require(name in labels and at not in fixed, 'unique named local fixup')
        before = old_offset(at)
        require(old[before:before + 4] == struct.pack('<i', labels[name] - before - 4), 'original local fixup bytes')
        require(0 <= at and at + 4 <= len(new), 'new local fixup extent')
        expected[at:at + 4] = struct.pack('<i', new_origins['labels'][name] - at - 4)
        fixed.add(at)
    calendars = {YEAR: 'realtime-year-table', MONTH: 'realtime-month-table'}
    replaced = []
    retained = []
    for at, (target, kind, addend) in new_relocations.items():
        if target in calendars:
            before = old_offset(at)
            require(kind == 2 and addend == -4, 'calendar PC-relative relocation')
            require(old[before:before + 4] == struct.pack('<i', old_origins['labels'][calendars[target]] - before - 4), 'old calendar reference')
            expected[at:at + 4] = bytes(4)
            replaced.append(target)
        else:
            require(old_relocations.get(old_offset(at)) == (target, kind, addend), 'retained external relocation')
            retained.append(old_offset(at))
    require(set(replaced) == set(calendars) and len(replaced) == 2
            and set(retained) == set(old_relocations), 'complete external relocation transformation')
    require(bytes(expected) == new, 'unexpected process instruction or literal change')
    return dict(removed_bytes=removed, local_fixups=len(fixed), new_data_relocations=replaced,
                retained_external_relocations=len(retained), exact_transformation=True,
                scope='Bound compiler label/fixup metadata and exact bytes; not independent whole-program semantics, CFI or lifetime proof')


def inspect(build, reference):
    elf, provider = Elf(build / 'kernel.elf'), Elf(build / 'provider.o')
    process, body = elf.function(FUNCTION)
    original_symbol, original = provider.function(FUNCTION)
    origins = json.loads((build / 'process-byte-origins.json').read_bytes())
    require(origins['function_bytes'] == len(original) == len(body)
            and origins['function_sha256'] == digest(original), 'new origin/object binding')
    old_provider = Elf(reference / 'provider.o')
    old_symbol, old_body = old_provider.function(FUNCTION)
    old_origins = json.loads((reference / 'process-byte-origins.json').read_bytes())
    require(old_origins['function_sha256'] == digest(old_body), 'old origin/object binding')
    def relocations(obj, symbol, size):
        return {at - symbol['value']: (target['name'], kind, addend)
                for at, (target, kind, addend) in relocation_entries(obj, symbol['section']).items()
                if symbol['value'] <= at < symbol['value'] + size}
    new_relocations = relocations(provider, original_symbol, len(original))
    transformation = layout(old_body, original, old_origins, origins,
                            relocations(old_provider, old_symbol, len(old_body)), new_relocations)
    patched = bytearray(original)
    for at, (target, kind, addend) in new_relocations.items():
        value = elf.symbols[target]['value'] + addend
        if kind in [2, 4]:
            raw = struct.pack('<i', value - process['value'] - at)
        elif kind in [1, 10, 11]:
            raw = struct.pack({1: '<Q', 10: '<I', 11: '<i'}[kind], value)
        else:
            raise ValueError('unsupported process relocation')
        patched[at:at + len(raw)] = raw
    require(bytes(patched) == body, 'new complete object-to-linked reconstruction')
    payload = calendar_payload()
    constants = {}
    for name, expected in [(YEAR, payload[:YEAR_BYTES]), (MONTH, payload[YEAR_BYTES:])]:
        symbol, data = symbol_data(elf, name)
        _, object_data = symbol_data(provider, name)
        require(data == object_data == expected, 'independent Gregorian object/linked payload')
        require(not (process['value'] < symbol['value'] + len(data) and symbol['value'] < process['value'] + len(body)), 'calendar overlaps process instructions')
        refs = rip_references(body, process['value'], symbol['value'])
        require(len(refs) == 1, 'unique calendar RIP consumer')
        constants[name] = dict(symbol=symbol, bytes=len(data), sha256=digest(data),
                               reference=process['value'] + refs[0], ELF_load_flags=load_flags(elf, symbol))
    years = constants[YEAR]['symbol']
    guard = year_guard(body, process['value'], constants[YEAR]['reference'] - process['value'], years['value'], years['value'] + years['size'])
    old_kernel = Elf(reference / 'kernel.elf')
    headers, header_body = elf.function(HEADERS)
    require(header_body == old_kernel.function(HEADERS)[1], 'unchanged containing program-header function')
    at = header_body.find(bytes.fromhex('0f850f300000'))
    require(at >= 0 and header_body.find(bytes.fromhex('0f850f300000'), at + 1) == -1, 'unique embedded WRMSR candidate branch')
    candidate = dict(address=headers['value'] + at + 2, containing_instruction=headers['value'] + at,
                     instruction_bytes='0f850f300000', field='signed rel32 JNE displacement',
                     displacement=0x300f, branch_target=headers['value'] + at + 6 + 0x300f,
                     containing_function_bytes_unchanged=True, global_unreachability_proved=False)
    return dict(passed=True, kernel_sha256=digest(elf.data), provider_sha256=digest(provider.data),
                transformation=transformation, constants=constants, year_consumer=guard,
                relocated_CF_candidate=dict(address=years['value'] + 105 * 16, bytes='cf',
                    field='2075 Gregorian day count low byte', hardware_NX_proved=False),
                embedded_MSR_candidate=candidate, complete_object_to_linked_reconstruction=True,
                qualification_complete=False, hardware_permissions_proved=False, runtime_immutability_proved=False)
