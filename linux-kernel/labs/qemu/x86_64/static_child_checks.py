"""Independent executed child/wait, private-state and thread-boundary witnesses."""
import struct

ENTRY = 0x4F0000
BUF, STATUS, BASE, PRIVATE = 0x7FE400, 0x7FE410, 0x7FE420, 0x5F2000
MAX = (1 << 64) - 1


def code(Witness):
    w = Witness()

    def syscall(number, args=()):
        for prefix, value in zip(('48bf', '48be', '48ba', '49ba'), args):
            w.imm(prefix, value)
        w.imm('48b8', number)
        w.raw('0f05')

    def branch_zero():
        w.raw('4885c00f84')
        offset = len(w.code)
        w.code.extend(bytes(4))
        return offset

    def jump():
        w.raw('e9')
        offset = len(w.code)
        w.code.extend(bytes(4))
        return offset

    def bind(offset):
        struct.pack_into('<i', w.code, offset, len(w.code) - offset - 4)

    w.raw('53554154415541564157')
    w.imm('48bf', BUF + 0x400)
    w.raw('480fae07')
    w.imm('48a1', PRIVATE)
    w.imm('48a3', BUF + 0x100)
    syscall(8, (3, 0, 1))
    w.imm('48a3', BUF + 0x108)
    syscall(79, (BUF + 0x180, 64))
    w.raw('4885c078fe')
    for number, args, result in (
        (61, (MAX, MAX, 0, 0), -10),
        (56, (0x111, 0, MAX, MAX), -38), (56, (17, 1, MAX, MAX), -38),
        (218, (MAX,), 1), (273, (MAX, 24), 0), (273, (0, 23), -22),
        (158, (0x100001003, BASE), 0),
    ):
        w.call(number, args, result)
    w.memory(BASE, 0x5F7400)
    w.call(158, (0x1002, MAX), -1)
    w.call(158, (0x1003, MAX), -14)
    w.call(158, (0x1003, BASE), 0)
    w.memory(BASE, 0x5F7400)
    w.store(PRIVATE, 0x12345678)
    w.store(BUF, 0x87654321)
    w.call(8, (3, 0, 0), 0)
    w.imm('48b8', 0xABCDEF1234567890)
    w.raw('664c0f6ef8')
    syscall(57, (MAX, MAX, MAX, MAX))
    child = branch_zero()
    w.raw('4883f80176fe4889c5')
    w.memory(PRIVATE, 0x12345678)
    w.memory(BUF, 0x87654321)
    w.raw('664c0f7ef8')
    w.expect(0xABCDEF1234567890)
    w.call(158, (0x1003, BASE), 0)
    w.memory(BASE, 0x5F7400)
    for number, args, result in ((39, (), 1), (110, (), 0), (72, (3, 1), 0),
                                  (8, (3, 0, 1), 3)):
        w.call(number, args, result)
    syscall(79, (BUF + 64, 64))
    w.raw('4885c078fe')
    w.imm('48be', BUF + 64)
    w.imm('48bf', BUF + 0x180)
    w.imm('48b9', 64)
    w.raw('fcf3a675fe')
    w.call(57, (), -11)
    for args, result in (((1, MAX, 0, 0), -10), ((MAX, MAX, 0, 0), -14),
                         ((MAX, STATUS, 2, 0), -22), ((MAX, STATUS, 0, 1), -38)):
        w.call(61, args, result)
    w.store(STATUS, 0xDECAFBAD00000000)
    syscall(61, (MAX, STATUS, 1, 0))
    w.raw('4839e875fe')
    w.memory(STATUS, 0xDECAFBAD00000700)
    w.call(61, (MAX, STATUS, 0, 0), -10)
    after = jump()
    bind(child)
    syscall(39)
    w.raw('4883f80176fe')
    for number, args, result in ((110, (), 1), (57, (), -11), (61, (MAX, 0, 0, 0), -10)):
        w.call(number, args, result)
    w.store(PRIVATE, 0xBAD)
    w.store(BUF, 0xBAD)
    w.store(BUF + 64, int.from_bytes(b'/\0\0\0\0\0\0\0', 'little'))
    for number, args, result in ((80, (BUF + 64,), 0), (8, (3, 3, 1), 3),
                                  (72, (3, 2, 1), 0), (3, (3,), 0),
                                  (158, (0x100001002, 0x5F7500), 0), (158, (0x1003, BASE), 0)):
        w.call(number, args, result)
    w.memory(BASE, 0x5F7500)
    w.call(273, (0, 24), 0)
    w.raw('66450fefff')
    syscall(60, (0x107,))
    w.raw('0f0b')
    bind(after)

    syscall(56, (0x101200011, 0, MAX, MAX))
    child = branch_zero()
    w.raw('4839e876fe4889c5')
    syscall(61, (0, 0, 0, 0))
    w.raw('4839e875fe')
    w.call(61, (MAX, 0, 1, 0), -10)
    after = jump()
    bind(child)
    w.call(110, (), 1)
    syscall(231, (0,))
    w.raw('0f0b')
    bind(after)

    w.store(BUF + 128, 0xFACEFEEDDECAFBAD)
    syscall(56, (0x1200011, 0, MAX, BUF + 128))
    child = branch_zero()
    w.raw('4839e876fe4889c5')
    w.memory(BUF + 128, 0xFACEFEEDDECAFBAD)
    w.raw('4889ef')
    for prefix in ('48be', '48ba', '49ba'):
        w.imm(prefix, 0)
    w.imm('48b8', 61)
    w.raw('0f054839e875fe')
    after = jump()
    bind(child)
    syscall(186)
    w.raw('4889c3')
    w.imm('48be', BUF + 128)
    w.raw('8b064839d875fe')
    syscall(60, (0,))
    w.raw('0f0b')
    bind(after)

    w.store(BUF + 0x200, int.from_bytes(b'/bin/bus', 'little'))
    w.store(BUF + 0x208, int.from_bytes(b'ybox\0\0\0\0', 'little'))
    w.call(3, (4,), 0)
    for fd in range(4, 16):
        w.call(257, (-100, BUF + 0x200, 0, 0), fd)
    syscall(57)
    child = branch_zero()
    w.raw('4839e876fe4889c5')
    syscall(61, (MAX, 0, 0, 0))
    w.raw('4839e875fe')
    w.call(8, (3, 0, 1), 3)
    for fd in range(5, 16):
        w.call(3, (fd,), 0)
    w.call(3, (4,), 0)
    w.call(33, (3, 4), 4)
    after = jump()
    bind(child)
    for fd in range(16):
        w.call(3, (fd,), 0)
    for fd in range(16):
        w.call(257, (-100, BUF + 0x200, 0, 0), fd)
    for number, args, result in ((8, (15, 4, 0), 4), (3, (3,), 0), (32, (0,), 3),
                                  (8, (0, 6, 0), 6), (8, (3, 0, 1), 6),
                                  (292, (15, 3, 0x80000), 3), (72, (3, 1), 1),
                                  (72, (15, 1), 0), (8, (3, 0, 1), 4)):
        w.call(number, args, result)
    syscall(60, (0,))
    w.raw('0f0b')
    bind(after)

    w.imm('48a1', BUF + 0x100)
    w.imm('48a3', PRIVATE)
    w.imm('48a1', BUF + 0x108)
    w.raw('4889c6')
    w.imm('48bf', 3)
    w.imm('48ba', 0)
    w.imm('48b8', 8)
    w.raw('0f05')
    w.imm('48bf', BUF + 0x400)
    w.raw('480fae0f415f415e415d415c5d5b')
    result = w.finish()
    assert len(result) < 0x8000
    return result
