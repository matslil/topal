"""Independent native Linux mode callers; no code is linked into the kernel."""
import struct
from static_named_file_checks import NamedWitness, PATH, DATA, STAT, SAVED, MAX

ENTRY = 0x510000
PRE_ENTRY, POST_ENTRY = 0x518000, 0x518800


def stat(w, fd, mode):
    w.call(5, (fd, STAT), 0)
    w.memory(STAT + 24, mode)


def regular(w):
    w.open('/tmp/mode', 0x42, 4, 0o666)
    w.store(DATA, int.from_bytes(b'metadata', 'little'))
    w.call(1, (4, DATA, 8), 8)
    w.open('/tmp/mode', 0, 5)
    w.call(32, (5,), 6)
    stat(w, 4, 0o100644)
    for offset in (8, 72, 88):
        w.save(STAT + offset, SAVED + offset)
    # Type bits and high mode bits are removed at the Linux ABI boundary.
    w.call(91, ((1 << 32) + 5, (1 << 63) | 0o4777), 0)
    for fd in (4, 5, 6):
        stat(w, fd, 0o104777)
        for offset in (8, 72, 88):
            w.same(STAT + offset, SAVED + offset)
    w.text(PATH, '/tmp/mode')
    w.call(90, (PATH, 0o20654), 0)
    stat(w, 6, 0o100654)
    # Absolute names ignore an invalid directory fd; fchmodat ignores arg four.
    w.call(268, (999, PATH, 0o2751, MAX), 0)
    stat(w, 5, 0o102751)
    w.call(452, (999, PATH, 0o7777, (1 << 32) | 0x100), 0)
    stat(w, 4, 0o107777)
    w.text(PATH, '')
    w.call(452, (6, PATH, 0, 0x1000), 0)
    stat(w, 5, 0o100000)
    # A mode change does not revoke granted description rights or positions.
    w.call(8, (4, 0, 1), 8)
    w.call(8, (5, 0, 1), 0)
    w.call(1, (4, DATA, 1), 1)
    w.call(1, (5, DATA, 1), -9)
    w.zero(DATA, 16)
    w.call(0, (6, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'metadata', 'little'))
    # An unlinked inode and a recreated spelling keep independent metadata.
    w.unlink('/tmp/mode')
    w.open('/tmp/mode', 0x42, 7, 0o600)
    stat(w, 7, 0o100600)
    w.call(91, (4, 0o713), 0)
    stat(w, 6, 0o100713)
    stat(w, 7, 0o100600)
    for fd in (4, 5, 6, 7):
        w.call(3, (fd,), 0)
    w.text(PATH, '/tmp/mode')
    w.call(90, (PATH, 0o702), 0)  # linked inode with no open description
    w.open('/tmp/mode', 0, 4)
    stat(w, 4, 0o100702)
    w.call(3, (4,), 0)
    w.unlink('/tmp/mode')
    w.open('/tmp/reused', 0x42, 4, 0o644)
    stat(w, 4, 0o100644)
    w.call(3, (4,), 0)
    w.unlink('/tmp/reused')


def directories(w):
    w.open('/tmp', 0x10000, 4)
    w.text(PATH, '/tmp')
    w.call(90, (PATH, 0o2770), 0)
    stat(w, 4, 0o42770)
    w.call(81, (4,), 0)
    w.text(PATH, '')
    w.call(452, (-100, PATH, 0o1711, 0x1000), 0)
    stat(w, 4, 0o41711)
    w.text(PATH, 'relative')
    w.call(257, (4, PATH, 0x42, 0o600), 5)
    stat(w, 4, 0o41711)  # entry creation preserves the directory mode
    w.call(268, (4, PATH, 0o4671), 0)
    stat(w, 5, 0o104671)
    w.call(263, (4, PATH, 0), 0)
    stat(w, 4, 0o41711)
    w.call(3, (5,), 0)
    w.call(91, (4, 0o1777), 0)
    w.call(3, (4,), 0)
    w.text(PATH, '/bin')
    w.call(80, (PATH,), 0)


def pipes(w):
    w.call(22, (DATA,), 0)
    w.memory(DATA, (5 << 32) | 4)
    w.call(91, (4, 0o2751), 0)
    stat(w, 5, 0o12751)
    w.text(PATH, '')
    w.call(452, (5, PATH, 0o701, 0x1000), 0)
    stat(w, 4, 0o10701)
    w.call(32, (4,), 6)
    w.call(3, (4,), 0)
    w.call(91, (6, 0o600), 0)
    stat(w, 5, 0o10600)
    w.store(DATA, int.from_bytes(b'pipebits', 'little'))
    w.call(1, (5, DATA, 8), 8)
    w.zero(DATA, 16)
    w.call(0, (6, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'pipebits', 'little'))
    w.call(3, (5,), 0)
    w.call(3, (6,), 0)


def errors(w):
    w.text(PATH, '')
    for number, args, error in [
        (452, (999, MAX, 0, 2), 22),
        (452, (999, 0, 0, 0x1000), 14),
        (452, (999, PATH, 0, 0), 2),
        (452, (999, PATH, 0, 0x1000), 9),
        (91, (999, 0), 9),
        (90, (0, 0), 14),
    ]:
        w.call(number, args, -error)
    for name, number, args, error in [
        ('relative', 268, (999, PATH, 0), 9),
        ('relative', 268, (3, PATH, 0), 20),
        ('/bin/missing', 90, (PATH, 0), 2),
        ('/bin/busybox', 90, (PATH, 0), 30),
        ('/bin/busybox/', 90, (PATH, 0), 20),
        ('/tmp/missing', 90, (PATH, 0), 2),
    ]:
        w.text(PATH, name)
        w.call(number, args, -error)


def fork_lifetime(w):
    w.open('/tmp/inherited', 0x42, 4, 0o666)
    w.open('/tmp/inherited', 0, 5)
    w.call(32, (4,), 6)
    w.store(DATA, int.from_bytes(b'inherit!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    w.call(22, (DATA,), 0)
    w.memory(DATA, (8 << 32) | 7)
    w.imm('49bc', 0x51ab)
    w.imm('48b8', 57)
    w.raw('0f05')
    child = dict(returned=ENTRY+len(w.code), number=57, arguments=[], expected=0)
    w.calls.append(child)
    w.raw('4885c00f85')
    parent_jump = len(w.code)
    w.code.extend(bytes(4))
    w.call(91, (5, 0o750), 0)
    w.call(91, (8, 0o620), 0)
    w.unlink('/tmp/inherited')
    w.open('/tmp/inherited', 0x42, 9, 0o640)
    w.call(91, (9, 0o701), 0)
    w.call(3, (9,), 0)
    w.call(3, (4,), 0)
    w.imm('48bf', 9)
    w.imm('48b8', 60)
    w.raw('0f050f0b')
    struct.pack_into('<i', w.code, parent_jump, len(w.code)-parent_jump-4)
    w.calls.append(dict(child, expected=11))
    w.expect(11)
    w.call(61, (11, STAT, 0, 0), 11)
    w.memory(STAT, 9 << 8)
    for fd in (4, 5, 6): stat(w, fd, 0o100750)
    w.memory(STAT+16, 0)
    for fd in (7, 8): stat(w, fd, 0o10620)
    w.zero(DATA, 16)
    w.call(0, (5, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'inherit!', 'little'))
    w.open('/tmp/inherited', 0, 9)
    stat(w, 9, 0o100701)
    # Even equal mode/clock observations must execute all source stores.
    w.call(91, (9, 0o701), 0)
    stat(w, 9, 0o100701)
    for fd in (4, 5, 6, 7, 8, 9): w.call(3, (fd,), 0)
    w.unlink('/tmp/inherited')


def exec_lifetime():
    pre, post = NamedWitness(PRE_ENTRY), NamedWitness(POST_ENTRY)
    pre.imm('48b8', 0x4c4000)
    pre.raw('ffd0')
    path = 0x608200
    pre.text(path, '/tmp/mode-exec')
    pre.call(257, (-100, path, 0x42, 0o666), 5)
    pre.call(33, (5, 15), 15)
    pre.call(3, (5,), 0)
    pre.call(257, (-100, path, 0, 0), 5)
    pre.call(33, (5, 13), 13)
    pre.call(3, (5,), 0)
    pre.call(72, (15, 1030, 14), 14)
    pre.call(91, (13, 0o2750), 0)
    pre.call(59, (0, 0, MAX), -14)
    pre.raw('c3')
    # The existing fixture executes this post-check for its maximum environment.
    from static_environment_checks import ENVIRONMENT
    post.raw('0fb60425')
    post.code.extend(struct.pack('<I', ENVIRONMENT+1022))
    post.raw('3c780f85')
    skip = len(post.code)
    post.code.extend(bytes(4))
    post.call(72, (14, 1, 0), -9)
    stat(post, 15, 0o102750)
    stat(post, 13, 0o102750)
    post.call(91, (13, 0o614), 0)
    stat(post, 15, 0o100614)
    post.text(path, '/tmp/mode-exec')
    post.call(87, (path,), 0)
    post.call(3, (13,), 0)
    post.call(3, (15,), 0)
    struct.pack_into('<i', post.code, skip, len(post.code)-skip-4)
    post.imm('48b8', 0x4c8000)
    post.raw('ffd0c3')
    assert len(pre.code) < POST_ENTRY-PRE_ENTRY and len(post.code) < 0x800
    return pre, post


def fixture(original, lifetime=False):
    w = NamedWitness(ENTRY)
    w.imm('48b8', 0x4ac000)
    w.raw('ffd0')
    regular(w)
    directories(w)
    pipes(w)
    errors(w)
    if lifetime: fork_lifetime(w)
    w.zero(PATH, 0x2000)
    w.raw('c3')
    assert len(w.code) < 0x8000
    image = bytearray(original)
    at = ENTRY - 0x400000
    assert not any(image[at:at + 0x8000]), 'mode caller overlaps the existing fixture'
    image[at:at + len(w.code)] = w.code
    runtime = bytes(image[0xaa000:0xab000])
    pattern = bytes.fromhex('b800c04a00ffd0')
    assert runtime.count(pattern) == 1
    at = 0xaa000 + runtime.index(pattern)
    image[at:at + 7] = b'\xb8' + struct.pack('<I', ENTRY) + b'\xff\xd0'
    calls = w.calls
    if lifetime:
        pre, post = exec_lifetime()
        for address, payload in ((PRE_ENTRY, pre.code), (POST_ENTRY, post.code)):
            at = address-0x400000
            assert not any(image[at:at+len(payload)])
            image[at:at+len(payload)] = payload
        before = bytes(image[0xb2000:0xb4000])
        pattern = bytes.fromhex('48b800404c0000000000ffd0')
        assert before.count(pattern) == 1
        at = 0xb2000+before.index(pattern)+2
        image[at:at+8] = struct.pack('<Q', PRE_ENTRY)
        pattern = bytes.fromhex('b800804c00ffd0')
        assert runtime.count(pattern) == 1
        at = 0xaa000+runtime.index(pattern)
        image[at:at+7] = b'\xb8'+struct.pack('<I', POST_ENTRY)+b'\xff\xd0'
        calls = pre.calls+post.calls+calls
    return bytes(image), calls
