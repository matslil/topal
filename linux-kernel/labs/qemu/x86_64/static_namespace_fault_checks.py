"""Real user calls preparing one admitted namespace operation that must not return."""
import struct

from static_namespace_transaction_checks import TransactionWitness, ENTRY, PATH, NEW, DATA, names

OPERATIONS = {'link': 0, 'linkat': 0, 'move': 1, 'replace': 2, 'exchange': 3}


def fixture(original, operation):
    assert operation in OPERATIONS
    w = TransactionWitness(ENTRY)
    w.imm('48b8', 0x4ac000)
    w.raw('ffd0')
    w.open('/tmp' if operation == 'linkat' else '/tmp/source',
           0x410002 if operation == 'linkat' else 0x42, 4, 0o640)
    w.store(DATA, int.from_bytes(b'source!!', 'little'))
    w.call(1, (4, DATA, 8), 8)
    if operation in ('replace', 'exchange'):
        w.open('/tmp/victim', 0x42, 5, 0o600)
        w.store(DATA, int.from_bytes(b'victim!!', 'little'))
        w.call(1, (5, DATA, 8), 8)
    names(w, '' if operation == 'linkat' else '/tmp/source',
          '/tmp/victim' if operation in ('replace', 'exchange') else '/tmp/alias')
    number, arguments = {
        'link': (86, (PATH, NEW)),
        'linkat': (265, (4, PATH, 999, NEW, 0x1000)),
        'move': (82, (PATH, NEW)),
        'replace': (264, (999, PATH, 999, NEW)),
        'exchange': (316, (999, PATH, 999, NEW, 2)),
    }[operation]
    w.call(number, arguments, 0)
    w.calls[-1].update(must_not_return=True, namespace_operation=OPERATIONS[operation])
    w.raw('0f0b')
    assert len(w.code) < 0x8000
    image = bytearray(original)
    start = ENTRY-0x400000
    assert not any(image[start:start+0x8000])
    image[start:start+len(w.code)] = w.code
    pattern = bytes.fromhex('b800c04a00ffd0')
    runtime = bytes(image[0xaa000:0xab000])
    assert runtime.count(pattern) == 1
    at = 0xaa000+runtime.index(pattern)
    image[at:at+7] = b'\xb8'+struct.pack('<I', ENTRY)+b'\xff\xd0'
    return bytes(image), w.calls
