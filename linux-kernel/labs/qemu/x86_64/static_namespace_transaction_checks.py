"""Independent user-mode rename/link callers; never linked into the kernel."""
import struct
from static_named_file_checks import NamedWitness, PATH, DATA, STAT, SAVED

ENTRY = 0x510000
NEW = PATH + 512


class TransactionWitness(NamedWitness):
    def call(self, number, args, expected):
        # The older process witness emits at most four arguments. These calls
        # also need the fifth native argument (flags) in R8, and its complete
        # instruction length must agree with the observed user return PC.
        assert len(args) <= 6
        returned = self.entry + len(self.code) + 10 * len(args) + 22
        self.calls.append(dict(returned=returned, number=number,
                               arguments=list(args), expected=expected))
        for prefix, value in zip(('48bf', '48be', '48ba', '49ba', '49b8', '49b9'), args):
            self.imm(prefix, value)
        self.imm('49bc', 0x51ab)
        self.imm('48b8', number)
        self.raw('0f05')
        self.expect(expected)
        self.raw('4981fcab51000075fe')


def links(w, fd, count):
    w.call(5, (fd, STAT), 0)
    w.memory(STAT + 16, count)


def names(w, old, new):
    w.text(PATH, old)
    w.text(NEW, new)


def full_entries_and_reclamation(w):
    w.open('/tmp/left', 0x42, 4, 0o600)
    w.store(DATA, int.from_bytes(b'left!!!!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    links(w, 4, 1)
    w.save(STAT + 8, SAVED)
    w.open('/tmp/right', 0x42, 5, 0o640)
    w.store(DATA, int.from_bytes(b'right!!!', 'little'))
    w.call(1, (5, DATA, 8), 8)
    links(w, 5, 1)
    w.save(STAT + 8, SAVED + 8)
    names(w, '/tmp/left', '/tmp/alias')
    w.call(86, (PATH, NEW), 0)
    w.open('/tmp/fourth', 0x42, 6)
    w.call(3, (6,), 0)
    names(w, '/tmp/left', '/tmp/fifth')
    w.call(86, (PATH, NEW), -28)
    names(w, '/tmp/left', '/tmp/right')
    w.call(316, (-100, PATH, -100, NEW, 2), 0)
    w.calls[-1]['linux_case'] = 'renameat2-exchange-unequal-link-counts'
    links(w, 4, 2)
    w.same(STAT + 8, SAVED)
    links(w, 5, 1)
    w.same(STAT + 8, SAVED + 8)
    w.open('/tmp/left', 0, 6)
    links(w, 6, 1)
    w.same(STAT + 8, SAVED + 8)
    w.open('/tmp/right', 0, 7)
    links(w, 7, 2)
    w.same(STAT + 8, SAVED)
    for fd, payload in ((6, b'right!!!'), (7, b'left!!!!')):
        w.zero(DATA, 16)
        w.call(0, (fd, DATA, 8), 8)
        w.memory(DATA, int.from_bytes(payload, 'little'))
    w.call(8, (4, 0, 1), 8)
    w.call(8, (5, 0, 1), 8)
    names(w, '/tmp/fourth', '/tmp/new-fourth')
    w.call(82, (PATH, NEW), 0)
    w.calls[-1]['linux_case'] = 'rename-full-entry-budget-to-absent'
    for name in ('left', 'right', 'alias', 'new-fourth'):
        w.unlink('/tmp/' + name)
    for fd in (4, 5, 6, 7):
        w.call(3, (fd,), 0)

    # A closed replacement victim must release its inode and resident page.
    for name, fd in (('source', 4), ('victim', 5)):
        w.open('/tmp/' + name, 0x42, fd)
        w.store(DATA, int.from_bytes(b'resident', 'little'))
        w.call(1, (fd, DATA, 8), 8)
    w.call(3, (5,), 0)
    names(w, '/tmp/source', '/tmp/victim')
    w.call(82, (PATH, NEW), 0)
    w.zero(DATA, 3 * 4096)
    for index, fd in enumerate((5, 6, 7)):
        w.open('/tmp/fill' + str(index), 0x42, fd)
        # The surviving source owns one page. Reusing all seven remaining
        # pages proves the closed victim's resident page was reclaimed too.
        size = (3 if index == 0 else 2) * 4096
        for _ in range(size // 256):
            w.call(1, (fd, DATA, 256), 256)
        w.call(5, (fd, STAT), 0)
        w.memory(STAT + 48, size)
    w.call(8, (4, 0, 0), 0)
    w.zero(DATA, 16)
    w.call(0, (4, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'resident', 'little'))
    for name in ('victim', 'fill0', 'fill1', 'fill2'):
        w.unlink('/tmp/' + name)
    for fd in (4, 5, 6, 7):
        w.call(3, (fd,), 0)

    raw = bytes(128 + i % 128 for i in range(255))
    w.open('/tmp/raw', 0x42, 4)
    w.bytes(NEW, b'/tmp/' + raw)
    w.call(82, (PATH, NEW), 0)
    w.calls[-1]['linux_case'] = 'rename-long-raw-component'
    w.bytes(PATH, b'/tmp/' + raw)
    w.text(NEW, '/tmp/raw-alias')
    w.call(265, (999, PATH, 999, NEW, 0), 0)
    w.calls[-1]['linux_case'] = 'linkat-empty-descriptor-to-raw-name'
    links(w, 4, 2)
    w.bytes(PATH, b'/tmp/' + raw)
    w.call(87, (PATH,), 0)
    w.unlink('/tmp/raw-alias')
    w.call(3, (4,), 0)


def fixture(original):
    w = TransactionWitness(ENTRY)
    w.imm('48b8', 0x4ac000)
    w.raw('ffd0')  # Retain the preceding standard ABI/control witnesses.
    w.open('/tmp/a', 0x42, 4, 0o640)
    links(w, 4, 1)
    w.save(STAT + 8, SAVED)
    w.store(DATA, int.from_bytes(b'payload!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    names(w, '/tmp/a', '/tmp/b')
    w.call(86, (PATH, NEW), 0)
    links(w, 4, 2)
    names(w, '/tmp/a', '/tmp/c')
    w.call(82, (PATH, NEW), 0)
    w.open('/tmp/a', 0, -2)
    w.open('/tmp', 0x10000, 5)
    names(w, 'c', 'd')
    w.call(264, ((1 << 32) | 5, PATH, (1 << 32) | 5, NEW), 0)
    names(w, '', 'e')
    w.call(265, ((1 << 32) | 4, PATH, (1 << 32) | 5, NEW, (1 << 32) | 0x1000), 0)
    links(w, 4, 3)
    names(w, 'd', 'f')
    w.call(316, (5, PATH, 5, NEW, (1 << 32) | 1), 0)
    names(w, 'f', 'b')
    w.call(316, (5, PATH, 5, NEW, 1), -17)
    w.call(316, (5, PATH, 5, NEW, 0), 0)  # Distinct same-inode aliases.
    w.calls[-1]['no_namespace_action'] = True
    w.call(316, (5, PATH, 5, NEW, 2), 0)
    w.calls[-1]['no_namespace_action'] = True
    links(w, 4, 3)
    w.same(STAT + 8, SAVED)
    w.call(8, (4, 0, 1), 8)
    w.open('/tmp/b', 0, 6)
    w.call(1, (6, DATA, 1), -9)
    w.zero(DATA, 16)
    w.call(0, (6, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'payload!', 'little'))
    w.open('/tmp/victim', 0x42, 7, 0o600)  # Four names, two owned inodes.
    w.store(DATA, int.from_bytes(b'victim!!', 'little'))
    w.call(1, (7, DATA, 8), 8)
    names(w, '/tmp/f', '/tmp/victim')
    w.call(82, (PATH, NEW), 0)
    links(w, 7, 0)
    links(w, 4, 3)
    w.call(8, (7, 0, 0), 0)
    w.zero(DATA, 16)
    w.call(0, (7, DATA, 8), 8)
    w.memory(DATA, int.from_bytes(b'victim!!', 'little'))
    for name in ('b', 'e', 'victim'):
        w.unlink('/tmp/' + name)
    links(w, 4, 0)
    names(w, '', '/tmp/resurrect')
    w.call(265, (4, PATH, 999, NEW, 0x1000), -2)
    for fd in (4, 6, 7):
        w.call(3, (fd,), 0)
    w.open('/tmp', 0x410002, 4, 0o600)
    names(w, '', '/tmp/initial')
    w.call(265, (4, PATH, 999, NEW, 0x1000), 0)
    links(w, 4, 1)
    w.unlink('/tmp/initial')
    names(w, '', '/tmp/consumed')
    w.call(265, (4, PATH, 999, NEW, 0x1000), -2)
    w.call(3, (4,), 0)
    w.open('/tmp', 0x410082, 4, 0o600)
    names(w, '', '/tmp/exclusive')
    w.call(265, (4, PATH, 999, NEW, 0x1000), -2)
    w.open('/tmp/existing', 0x42, 6, 0o600)
    names(w, '', '/tmp/existing')
    w.call(265, (4, PATH, 999, NEW, 0x1000), -17)
    w.unlink('/tmp/existing')
    for fd in (6, 4, 5):
        w.call(3, (fd,), 0)
    full_entries_and_reclamation(w)
    w.zero(PATH, 0x2000)
    w.raw('c3')
    assert len(w.code) < 0x8000
    image = bytearray(original)
    start = ENTRY - 0x400000
    assert not any(image[start:start + 0x8000])
    image[start:start + len(w.code)] = w.code
    runtime = bytes(image[0xaa000:0xab000])
    pattern = bytes.fromhex('b800c04a00ffd0')
    assert runtime.count(pattern) == 1
    at = 0xaa000 + runtime.index(pattern)
    image[at:at + 7] = b'\xb8' + struct.pack('<I', ENTRY) + b'\xff\xd0'
    return bytes(image), w.calls
