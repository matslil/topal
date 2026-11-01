"""Independent caller-issued RAM-file cases; never linked into the kernel.

The wrapper runs after the existing file/namespace witnesses. It preserves
their standard descriptions, cwd and umask and exercises actual native copies.
"""
from pathlib import Path
import runpy
import struct

LAB = Path(__file__).resolve().parent
Witness = runpy.run_path(str(LAB / 'static_process_checks.py'))['Witness']
ENTRY = 0x4D4000
PRE_ENTRY, POST_ENTRY = 0x4D7000, 0x4D7800
PATH, DATA, STAT = 0x607000, 0x608000, 0x608800
TMP = 0x410002
MAX = (1 << 64) - 1


class FileWitness(Witness):
    def __init__(self, entry=ENTRY):
        super().__init__()
        self.calls = []
        self.entry = entry

    def call(self, number, args, expected):
        returned = self.entry + len(self.code) + 10 * len(args) + 22
        self.calls.append(dict(returned=returned, number=number, arguments=list(args), expected=expected))
        super().call(number, args, expected)

    def text(self, address, value):
        data = value.encode() + b'\0'
        data += bytes((-len(data)) % 8)
        for offset in range(0, len(data), 8):
            self.store(address + offset, int.from_bytes(data[offset:offset + 8], 'little'))


def code(lifetime=False):
    w = FileWitness()
    w.imm('48b8', 0x4AC000)
    w.raw('ffd0')
    w.text(PATH, '/tmp')
    w.call(257, (-100, PATH, 0x10000, 0), 4)
    w.call(5, (4, STAT), 0)
    for offset, value in ((0, 2), (8, 6), (16, 2), (24, 0o41777)):
        w.memory(STAT + offset, value)
    w.call(217, (4, DATA, 256), 48)
    w.memory(DATA, 6)
    w.memory(DATA + 24, 3)
    w.call(217, (4, MAX, 256), 0)
    w.call(81, (4,), 0)
    w.store(DATA, 0)
    w.call(79, (DATA, 16), 5)
    w.memory(DATA, int.from_bytes(b'/tmp\0\0\0\0', 'little'))
    w.text(PATH, '.')
    w.call(257, (4, PATH, 0x10000, 0), 5)
    w.call(3, (5,), 0)
    w.text(PATH, '/bin')
    w.call(80, (PATH,), 0)
    w.call(3, (4,), 0)
    w.text(PATH, '/tmp')
    w.call(95, (0o077,), 0o022)
    for flags in (0x400002, 0x410000, 0x410003, 0x410042):
        w.call(257, (-100, MAX, flags, 0o666), -22)
    w.call(257, (-100, MAX, TMP, 0o666), -14)
    w.call(257, (-100, PATH, TMP, 0o666), 4)
    w.call(5, (4, STAT), 0)
    for offset, expected in ((0, 2), (8, 8 if lifetime else 7), (16, 0), (24, 0o100600), (48, 0), (56, 4096), (64, 0)):
        w.memory(STAT + offset, expected)
    w.call(72, (4, 3, 0), TMP | 0x8000)
    w.call(0, (4, DATA, 256), 0)
    w.call(0, (4, MAX, 0), -14)
    w.call(1, (4, MAX, 0), -14)
    w.call(1, (4, 0, 0), 0)
    w.store(DATA, int.from_bytes(b'abcdefgh', 'little'))
    w.call(1, (4, DATA, 8), 8)
    w.call(32, (4,), 5)
    w.call(8, (5, 0, 0), 0)
    w.store(DATA, 0)
    w.call(0, (4, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'abcdefgh', 'little'))
    w.call(8, (5, 0, 1), 8)
    for number, args in ((0, (4, MAX, 1)), (1, (4, MAX, 1)), (1, (4, 0x7FFFFF, 2))):
        w.call(number, args, -14)
    w.call(8, (4, 0, 1), 8)
    w.call(77, (99, -1), -22)
    w.call(77, (99, 0), -9)
    w.call(77, (3, 0), -22)
    w.call(77, (4, 3), 0)
    w.call(8, (5, 0, 1), 8)
    w.call(77, (4, 4097), 0)
    w.call(8, (4, 0, 0), 0)
    w.store(DATA, MAX)
    w.call(0, (5, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'abc\0\0\0\0\0', 'little'))
    w.call(8, (4, 4093, 0), 4093)
    w.call(0, (4, DATA, 256), 3)
    w.call(0, (4, DATA, 256), 1)
    w.call(0, (4, DATA, 256), 0)
    w.call(8, (4, 0, 3), 0)
    w.call(8, (4, 0, 4), 4096)
    w.call(8, (4, 4096, 3), -6)
    w.call(8, (4, -1, 4), -6)
    w.call(77, (4, 0), 0)
    w.call(5, (5, STAT), 0)
    w.memory(STAT + 48, 0)
    w.memory(STAT + 64, 0)
    # Logical range is independent of the eight-page physical budget.
    large = (1 << 48) + 13
    w.call(8, (4, large, 0), large)
    w.store(DATA, int.from_bytes(b'sparse!!', 'little'))
    w.call(1, (5, DATA, 8), 8)
    w.call(8, (4, 0, 3), 1 << 48)
    w.call(8, (4, large, 4), large + 8)
    w.call(5, (4, STAT), 0)
    w.memory(STAT + 48, large + 8)
    w.memory(STAT + 64, 8)
    w.call(3, (4,), 0)
    w.call(8, (5, large, 0), large)
    w.store(DATA, 0)
    w.call(0, (5, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'sparse!!', 'little'))
    w.call(3, (5,), 0)
    w.call(257, (-100, PATH, 0x10000, 0), 4)
    w.text(PATH, '.')
    w.call(257, (4, PATH, TMP, 0o600), 5)
    w.call(3, (5,), 0)
    w.call(3, (4,), 0)
    w.text(PATH, '/tmp')
    # All four anonymous object slots, and one shared eight-page pool.
    for fd in range(4, 8):
        w.call(257, (-100, PATH, TMP, 0o600), fd)
    w.call(257, (-100, PATH, TMP, 0o600), -28)
    for fd in range(4, 8):
        for offset in (0, 4096):
            w.call(8, (fd, offset, 0), offset)
            w.call(1, (fd, DATA, 8), 8)
    w.call(8, (4, 8192, 0), 8192)
    w.call(1, (4, DATA, 8), -28)
    w.call(8, (4, 0, 1), 8192)
    w.call(8, (4, 0, 0), 0)
    w.call(1, (4, DATA, 8), 8)
    for fd in range(4, 8):
        w.call(3, (fd,), 0)
    # Full signed offset boundary and the declared short-transfer quantum.
    w.call(257, (-100, PATH, TMP, 0o600), 4)
    limit = (1 << 63) - 1
    w.store(DATA, int.from_bytes(b'boundary', 'little'))
    w.call(8, (4, limit - 1, 0), limit - 1)
    w.call(1, (4, DATA, 2), -22)
    w.call(1, (4, DATA, 1), 1)
    w.call(1, (4, DATA, 1), -22)
    w.call(8, (4, 1, 1), -22)
    w.call(8, (4, limit - 1, 0), limit - 1)
    w.store(DATA, 0)
    w.call(0, (4, DATA, 8), -22)
    w.call(0, (4, DATA, 1), 1)
    w.memory(DATA, ord('b'))
    w.call(77, (4, 0), 0)
    w.call(8, (4, 0, 0), 0)
    w.imm('48bf', DATA)
    w.imm('48b9', 300)
    w.imm('48b8', 0xA5)
    w.raw('fcf3aa')
    w.call(1, (4, DATA, 300), 256)
    w.call(8, (4, 0, 0), 0)
    w.call(0, (4, 0x400000, 2), -14)
    w.call(0, (4, 0x7FFFFF, 2), -14)
    w.call(8, (4, 0, 1), 0)
    w.call(0, (4, DATA, 300), 256)
    w.memory(DATA, 0xA5A5A5A5A5A5A5A5)
    w.call(8, (4, 0, 1), 256)
    w.call(3, (4,), 0)
    # Append selects EOF for each write; access bits remain effective.
    w.call(257, (-100, PATH, TMP | 0x400, 0o600), 4)
    w.call(1, (4, DATA, 8), 8)
    w.call(8, (4, 0, 0), 0)
    w.call(1, (4, 0, 0), 0)
    w.call(8, (4, 0, 1), 0)
    w.call(1, (4, DATA, 8), 8)
    w.call(8, (4, 0, 1), 16)
    w.call(77, (4, limit - 1), 0)
    w.call(8, (4, 0, 0), 0)
    w.call(1, (4, DATA, 2), 1)
    w.call(8, (4, 0, 0), 0)
    w.call(1, (4, DATA, 1), -27)
    w.call(3, (4,), 0)
    w.call(257, (-100, PATH, 0x410001, 0o600), 4)
    w.call(0, (4, DATA, 0), -9)
    w.call(1, (4, DATA, 8), 8)
    w.call(3, (4,), 0)
    w.call(95, (0o022,), 0o077)
    if lifetime:
        fork(w)
        # Leave aliases and real contents live for ordinary process termination.
        w.call(257, (-100, PATH, TMP, 0o600), 4)
        w.call(32, (4,), 5)
        w.store(DATA, int.from_bytes(b'terminal', 'little'))
        w.call(1, (4, DATA, 8), 8)
    w.imm('48bf', PATH)
    w.imm('48b9', 0x2000 // 8)
    w.raw('31c0fcf348abc3')
    assert len(w.code) < (PRE_ENTRY - ENTRY if lifetime else 0x4000)
    return bytes(w.code), w.calls


def fork(w):
    w.call(257, (-100, PATH, TMP, 0o600), 4)
    w.call(32, (4,), 5)
    w.store(DATA, int.from_bytes(b'parent!!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    w.imm('49bc', 0x51AB)
    w.imm('48b8', 57)
    w.raw('0f05')
    returned = ENTRY + len(w.code)
    child = dict(returned=returned, number=57, arguments=[], expected=0)
    w.calls.append(child)
    w.raw('4885c00f85')
    parent_jump = len(w.code)
    w.code.extend(bytes(4))
    w.call(95, (0o777,), 0o022)
    w.store(DATA, int.from_bytes(b'child!!!', 'little'))
    w.call(1, (5, DATA, 8), 8)
    w.call(3, (4,), 0)
    w.imm('48bf', 11)
    w.imm('48b8', 60)
    w.raw('0f050f0b')
    struct.pack_into('<i', w.code, parent_jump, len(w.code) - parent_jump - 4)
    # The same actual fork-return site is observed again after child teardown.
    w.calls.append(dict(child, expected=11))
    w.expect(11)
    w.call(95, (0o022,), 0o022)
    w.call(61, (11, STAT, 0, 0), 11)
    w.memory(STAT, 11 << 8)
    w.call(8, (4, 0, 1), 16)
    w.call(8, (5, 0, 0), 0)
    w.call(0, (4, DATA, 16), 16)
    w.memory(DATA, int.from_bytes(b'parent!!', 'little'))
    w.memory(DATA + 8, int.from_bytes(b'child!!!', 'little'))
    w.call(3, (4,), 0)
    w.call(3, (5,), 0)


def exec_lifetime():
    pre = FileWitness(PRE_ENTRY)
    pre.imm('48b8', 0x4C4000)
    pre.raw('ffd0')
    # Preserve the existing envp source/vector in [0x607000,0x607488).
    pre_path, pre_data = 0x608200, 0x608300
    pre.text(pre_path, '/tmp')
    pre.call(95, (0o077,), 0o022)
    pre.call(257, (-100, pre_path, TMP, 0o666), 5)
    pre.call(33, (5, 15), 15)
    pre.call(3, (5,), 0)
    pre.call(72, (15, 1030, 14), 14)
    pre.store(pre_data, int.from_bytes(b'exec-data', 'little'))
    pre.call(1, (15, pre_data, 8), 8)
    pre.call(59, (0, 0, MAX), -14)
    pre.raw('c3')
    post = FileWitness(POST_ENTRY)
    environment = runpy.run_path(str(LAB / 'static_environment_checks.py'))['ENVIRONMENT']
    post.raw('0fb60425')
    post.code.extend(struct.pack('<I', environment + 1022))
    post.raw('3c780f85')
    skip = len(post.code)
    post.code.extend(bytes(4))
    post.call(72, (14, 1, 0), -9)
    post.call(95, (0o022,), 0o077)
    post.call(5, (15, STAT), 0)
    post.memory(STAT + 24, 0o100600)
    post.memory(STAT + 48, 8)
    post.call(8, (15, 0, 0), 0)
    post.store(DATA, 0)
    post.call(0, (15, DATA, 8), 8)
    post.memory(DATA, int.from_bytes(b'exec-data', 'little'))
    post.call(3, (15,), 0)
    struct.pack_into('<i', post.code, skip, len(post.code) - skip - 4)
    post.imm('48b8', 0x4C8000)
    post.raw('ffd0c3')
    assert len(pre.code) < POST_ENTRY - PRE_ENTRY
    assert len(post.code) < 0x4D8000 - POST_ENTRY
    return pre, post


def fixture(original, lifetime=False):
    image = bytearray(original)
    start, end = ENTRY - 0x400000, ENTRY - 0x400000 + 0x4000
    assert not any(image[start:end]), 'volatile witness overlaps existing source/data'
    payload, calls = code(lifetime)
    image[start:start + len(payload)] = payload
    pattern = bytes.fromhex('b800c04a00ffd0')
    runtime = bytes(image[0xAA000:0xAB000])
    assert runtime.count(pattern) == 1
    position = 0xAA000 + runtime.index(pattern)
    image[position:position + 7] = b'\xb8' + struct.pack('<I', ENTRY) + b'\xff\xd0'
    if lifetime:
        pre, post = exec_lifetime()
        for entry, value in ((PRE_ENTRY, pre.code), (POST_ENTRY, post.code)):
            offset = entry - 0x400000
            image[offset:offset + len(value)] = value
        before = bytes(image[0xB2000:0xB4000])
        pattern = bytes.fromhex('48b800404c0000000000ffd0')
        assert before.count(pattern) == 1
        position = 0xB2000 + before.index(pattern) + 2
        image[position:position + 8] = struct.pack('<Q', PRE_ENTRY)
        pattern = bytes.fromhex('b800804c00ffd0')
        assert runtime.count(pattern) == 1
        position = 0xAA000 + runtime.index(pattern)
        image[position:position + 7] = b'\xb8' + struct.pack('<I', POST_ENTRY) + b'\xff\xd0'
        calls = pre.calls + post.calls + calls
    return bytes(image), calls
