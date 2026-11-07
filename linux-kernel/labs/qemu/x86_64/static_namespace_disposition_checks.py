"""Independent raw errno callers compared with the pinned Linux VM baseline."""
import json
from pathlib import Path
import struct

from static_namespace_transaction_checks import TransactionWitness, PATH, NEW, DATA, STAT

ENTRY = 0x520000
OLD, TARGET = PATH + 1024, PATH + 2048
SNAPSHOT = DATA + 0x1000
BASELINE = Path(__file__).resolve().parents[3]/'knowledge/x86_64/namespace-native-dispositions.json'


def snapshot(w, fd, compare=False):
    w.call(5, (fd, STAT), 0)
    w.imm('48be', STAT)
    w.imm('48bf', SNAPSHOT + (fd - 4)*144)
    w.imm('48b9', 18)
    w.raw('fcf348a775fe' if compare else 'fcf348a5')


def fixture(original):
    w = TransactionWitness(ENTRY)
    w.imm('48b8', 0x510000)
    w.raw('ffd0')
    w.open('/tmp/file', 0x42, 4, 0o640)
    w.store(DATA, int.from_bytes(b'payload!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    w.open('/tmp/exists', 0x42, 5, 0o600)
    w.open('/tmp', 0x10000, 6)
    w.open('/bin', 0x10000, 7)
    for fd in (4, 5, 6, 7):
        snapshot(w, fd)
    w.text(PATH, '/tmp')
    w.call(80, (PATH,), 0)
    cases = [
        ('rename-flags-before-bad-names',316,[-1,0,-1,0,8],22),
        ('rename-exchange-noreplace-invalid',316,[-1,0,-1,0,3],22),
        ('rename-exchange-whiteout-invalid',316,[-1,0,-1,0,6],22),
        ('link-flags-before-bad-names',265,[-1,0,-1,0,1],22),
        ('rename-old-null',82,[0,b'missing'],14),
        ('rename-new-null',82,[b'file',0],14),
        ('link-null-with-empty-flag',265,[4,0,6,b'new',0x1000],14),
        ('link-empty-without-flag-before-bad-fd',265,[-1,b'',6,b'new',0],2),
        ('link-empty-bad-fd',265,[-1,b'',6,b'new',0x1000],9),
        ('rename-bad-old-dirfd',264,[-1,b'file',6,b'new'],9),
        ('rename-bad-new-dirfd',264,[6,b'file',-1,b'new'],9),
        ('link-relative-nondirectory',265,[4,b'file',6,b'new',0],20),
        ('rename-old-parent-missing-before-new-copy',82,[b'missing-parent/file',0],2),
        ('rename-old-final-missing-before-new-copy',82,[b'missing',0],14),
        ('link-old-missing-before-new-copy',86,[b'missing',0],2),
        ('rename-old-missing-new-parent-missing',82,[b'missing',b'missing-parent/new'],2),
        ('rename-missing-source-existing-noreplace',316,[6,b'missing',6,b'exists',1],2),
        ('rename-missing-source-cross-mount',82,[b'missing',b'/bin/new'],18),
        ('rename-existing-source-cross-mount-existing-noreplace',316,[6,b'file',7,b'busybox',1],18),
        ('rename-readonly-missing-source',82,[b'/bin/missing',b'/bin/new'],30),
        ('rename-readonly-existing-noreplace',316,[7,b'busybox',7,b'busybox',1],30),
        ('rename-old-dot-cross-mount',82,[b'.',b'/bin/new'],18),
        ('rename-old-dot-same-mount',82,[b'.',b'new'],16),
        ('rename-new-dot-noreplace',316,[6,b'file',6,b'.',1],17),
        ('rename-new-dot-plain',82,[b'file',b'.'],16),
        ('rename-existing-old-trailing-cross-mount',82,[b'file/',b'/bin/new'],18),
        ('rename-existing-old-trailing',82,[b'file/',b'new'],20),
        ('rename-missing-old-trailing',82,[b'missing/',b'new'],2),
        ('rename-new-trailing-existing-regular',82,[b'file',b'exists/'],20),
        ('rename-new-trailing-missing',82,[b'file',b'new/'],20),
        ('link-missing-old-before-readonly-new',86,[b'missing',b'/bin/new'],2),
        ('link-cross-mount-new-missing-readonly',86,[b'file',b'/bin/new'],30),
        ('link-cross-mount-new-existing-readonly',86,[b'file',b'/bin/busybox'],17),
        ('link-readonly-old-to-mutable-missing',86,[b'/bin/busybox',b'new'],18),
        ('link-readonly-old-to-mutable-existing',86,[b'/bin/busybox',b'exists'],17),
        ('link-new-dot',86,[b'file',b'.'],17),
        ('link-new-trailing-missing',86,[b'file',b'new/'],2),
        ('link-new-trailing-existing',86,[b'file',b'exists/'],17),
        ('link-old-regular-trailing',86,[b'file/',b'new'],20),
        ('rename-long-final-cross-mount',82,[b'x'*256,b'/bin/new'],18),
        ('rename-readonly-long-final',82,[b'/bin/'+b'x'*256,b'/bin/new'],30),
        ('link-long-old-before-readonly-new',86,[b'x'*256,b'/bin/new'],36),
        ('link-readonly-new-long-final',86,[b'file',b'/bin/'+b'x'*256],36),
        ('rename-exchange-missing-target',316,[6,b'file',6,b'new',2],2),
        ('renameat-bad-old-fd-ignored-for-absolute-new-fault',264,[-1,b'/tmp/file',-1,0],14),
    ]
    baseline = {case['case']:case for case in json.loads(BASELINE.read_text())['events']}
    for tag, number, arguments, error in cases:
        assert (baseline[tag]['syscall'], baseline[tag]['errno']) == (number,error), tag
        addresses = iter((OLD, TARGET))
        adapted = []
        for arg in arguments:
            if isinstance(arg,bytes):
                pointer = next(addresses)
                w.bytes(pointer,arg)
                adapted.append(pointer)
            else:
                adapted.append(arg)
        w.call(number, adapted, -error)
        w.calls[-1]['linux_case'] = tag
        for fd in (4, 5, 6, 7):
            snapshot(w, fd, compare=True)
    w.call(8, (4,0,1), 8)
    w.call(8, (4,0,0), 0)
    w.zero(DATA,16)
    w.call(0, (4,DATA,8), 8)
    w.memory(DATA,int.from_bytes(b'payload!','little'))
    for fd in (4,5,6,7):
        w.call(3,(fd,),0)
    w.unlink('/tmp/file')
    w.unlink('/tmp/exists')
    w.text(PATH,'/bin')
    w.call(80,(PATH,),0)
    w.zero(PATH,0x4000)
    w.raw('c3')
    assert len(w.code) < 0x10000, len(w.code)
    image = bytearray(original)
    start = ENTRY-0x400000
    assert not any(image[start:start+0x10000])
    image[start:start+len(w.code)] = w.code
    pattern = bytes.fromhex('b800005100ffd0')
    at = bytes(image[0xaa000:0xab000]).index(pattern)+0xaa000
    image[at:at+7] = b'\xb8'+struct.pack('<I',ENTRY)+b'\xff\xd0'
    return bytes(image),w.calls
