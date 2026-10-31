"""Independent executable ABI witnesses for file and signal process state."""
import struct
import runpy
from pathlib import Path

ENTRY, PRE_EXEC = 0x4AC000, 0x4B2000
BUF, STAT, ACTION, OLD = 0x7FE400, 0x7FE600, 0x7FE900, 0x7FE940
MAX = (1 << 64) - 1
MASK = MAX & ~0x40100


class Witness:
    def __init__(self):
        self.code = bytearray()

    def raw(self, value):
        self.code.extend(bytes.fromhex(value))

    def imm(self, prefix, value):
        self.raw(prefix)
        self.code.extend(struct.pack('<Q', value & MAX))

    def store(self, address, value):
        self.imm('48b8', value)
        self.imm('48a3', address)

    def expect(self, value):
        self.imm('49bb', value)
        self.raw('4c39d875fe')

    def memory(self, address, value):
        self.imm('48a1', address)
        self.expect(value)

    def call(self, number, args, expected):
        for prefix, value in zip(('48bf', '48be', '48ba', '49ba'), args):
            self.imm(prefix, value)
        self.imm('49bc', 0x51AB)
        self.imm('48b8', number)
        self.raw('0f05')
        self.expect(expected)
        self.raw('4981fcab51000075fe')

    def action(self, handler):
        for offset, value in ((0,handler), (8,MAX), (16,0xDECAFBAD), (24,MAX)):
            self.store(ACTION+offset, value)

    def query(self, signal, handler, flags, restorer, mask):
        self.call(13, (signal,0,OLD,8), 0)
        for offset, value in ((0,handler), (8,flags), (16,restorer), (24,mask)):
            self.memory(OLD+offset, value)

    def finish(self):
        self.imm('48bf', BUF)
        self.imm('48b9', 0x600 // 8)
        self.raw('31c0fcf348abc3')
        return bytes(self.code)


def pre_exec():
    w = Witness()
    w.imm('48b8', 0x4C0000)
    w.raw('ffd0')
    w.call(72, (3,1030,10), 10)
    w.call(32, (3,), 4)
    w.call(8, (4,7,0), 7)
    w.action(0x12345000)
    w.call(13, (10,ACTION,0,8), 0)
    w.action(1)
    w.call(13, (12,ACTION,0,8), 0)
    w.store(ACTION,64)
    w.call(14, (2,ACTION,0,8), 0)
    w.call(59, (0,0x5F9300,0x5F90A0), -14)
    w.call(8, (3,0,1), 7)
    w.call(72, (10,1,0), 1)
    w.query(10,0x12345000,0xDC000807,0xDECAFBAD,MASK)
    w.query(12,1,0xDC000807,0xDECAFBAD,MASK)
    w.call(14, (99,0,OLD,8), 0)
    w.memory(OLD,64)
    return w.finish()


def code():
    w = Witness()
    w.call(72, (10,1,0), -9)
    w.call(8, (4,0,1), 7)
    w.call(3, (4,), 0)
    w.call(8, (3,0,0), 0)
    w.query(10,0,0,0,0)
    w.query(12,1,0,0,0)
    w.call(14, (99,0,OLD,8), 0)
    w.memory(OLD,64)
    w.store(ACTION,0)
    w.call(14, (2,ACTION,0,8), 0)
    w.call(97, (7,STAT), 0)
    w.memory(STAT,16)
    w.memory(STAT+8,16)
    w.call(5, (3,STAT), 0)
    for offset, value in ((8,5), (24,0o100755), (48,2024544), (56,4096), (64,3955)):
        w.memory(STAT+offset,value)
    w.call(5, (99,MAX), -9)
    w.call(5, (3,0x7FFFF0), -14)
    w.call(0, (3,BUF,4), 4)
    w.raw('813c2500e47f007f454c4675fe')
    w.call(32, (3,), 4)
    w.call(0, (4,BUF,1), 1)
    w.raw('803c2500e47f000275fe')
    for number, args, result in (
        (8,(3,0,1),5), (72,(4,2,1),0), (72,(3,1,0),0),
        (72,(4,1,0),1), (72,(4,3,0),0x8000), (33,(99,3),-9),
        (8,(3,0,1),5), (33,(4,4),4), (72,(4,1,0),1),
        (292,(4,4,0),-22), (292,(99,3,1),-22), (292,(3,16,0),-9),
        (72,(3,0,16),-22), (0,(3,0x7FFFFE,4),-14), (0,(3,MAX,4),-14),
        (8,(3,0,1),5), (0,(3,0,0),0), (0,(3,MAX,0),-14),
        (8,(3,MAX,0),-22), (8,(3,MAX>>1,0),MAX>>1),
        (8,(3,1,1),-22), (0,(3,BUF,1),0), (8,(3,-2,2),2024542),
        (0,(3,BUF,256),2), (0,(3,BUF,256),0),
    ):
        w.call(number,args,result)
    w.raw('0fb6042500d05e00')
    w.imm('48a3',0x7FE800)
    w.raw('c6042500d05e00a5')
    w.call(8,(3,0x1ED000,0),0x1ED000)
    w.call(0,(3,BUF,1),1)
    w.raw('0fb6042500e47f003a042500e87f0075fe')
    w.imm('48a1',0x7FE800)
    w.raw('a200d05e0000000000')
    for fd in range(5,16):
        w.call(32,(3,),fd)
    w.call(32,(3,),-24)
    w.call(3,(7,),0)
    w.call(72,(3,0,4),7)
    for fd in range(4,16):
        w.call(3,(fd,),0)
    for number, args, result in (
        (33,(3,2),2), (8,(2,0,0),0), (0,(2,BUF,4),4),
        (33,(1,2),2), (32,(1,),4), (1,(4,0x4AB30E,14),14),
        (3,(4,),0), (1,(4,MAX,1),-9),
    ):
        w.call(number,args,result)
    signal_checks(w)
    w.imm('48b8', 0x4B4000)
    w.raw('ffd0')
    result = w.finish()
    assert len(result) < PRE_EXEC-ENTRY
    return result


def signal_checks(w):
    for args, result in (((99,MAX,0,7),-22), ((99,MAX,0,8),-14), ((9,0,OLD,8),0)):
        w.call(13,args,result)
    w.action(0xFEEDFACE)
    for signal in (9,19,0,65):
        w.call(13,(signal,ACTION,0,8),-22)
    w.call(13,(0x10000000A,ACTION,ACTION,8),0)
    for offset in range(0,32,8):
        w.memory(ACTION+offset,0)
    w.query(10,0xFEEDFACE,0xDC000807,0xDECAFBAD,MASK)
    w.action(1)
    w.call(13,(10,ACTION,0x7FFFF0,8),-14)
    w.query(10,1,0xDC000807,0xDECAFBAD,MASK)
    w.call(13,(10,MAX,0,8),-14)
    w.query(10,1,0xDC000807,0xDECAFBAD,MASK)
    w.call(14,(99,MAX,0,7),-22)
    w.call(14,(99,MAX,0,8),-14)
    w.store(ACTION,MAX)
    w.call(14,(99,ACTION,0,8),-22)
    w.call(14,(2,ACTION,ACTION,8),0)
    w.memory(ACTION,0)
    w.call(14,(99,0,OLD,8),0)
    w.memory(OLD,MASK)
    w.store(ACTION,2)
    w.call(14,(1,ACTION,OLD,8),0)
    w.memory(OLD,MASK)
    w.call(14,(99,0,OLD,8),0)
    w.memory(OLD,MASK & ~2)
    w.call(14,(0,ACTION,0,8),0)
    w.store(ACTION,32)
    w.call(14,(2,ACTION,0x7FFFFC,8),-14)
    w.call(14,(99,0,OLD,8),0)
    w.memory(OLD,32)


def namespace_code(stage):
    encoder = runpy.run_path(str(Path(__file__).parent / 'static_namespace_checks.py'))
    return encoder['code'](stage, Witness)
