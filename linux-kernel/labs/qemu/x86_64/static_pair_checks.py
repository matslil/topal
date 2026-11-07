"""Independent executed user witness for resumable private process pairs."""
import struct
ENTRY=0x4F8000
BUF,PRIVATE=0x7FE400,0x5F2000
MAX=(1<<64)-1

def code(Witness):
    w=Witness()
    def syscall(number,args=()):
        for prefix,value in zip(('48bf','48be','48ba','49ba'),args):w.imm(prefix,value)
        w.imm('48b8',number);w.raw('0f05')
    def branch():
        w.raw('4885c00f84');offset=len(w.code);w.code.extend(bytes(4));return offset
    def jump():
        w.raw('e9');offset=len(w.code);w.code.extend(bytes(4));return offset
    def bind(offset):struct.pack_into('<i',w.code,offset,len(w.code)-offset-4)
    def fs(value):
        w.call(158,(0x1003,BUF+32),0);w.memory(BUF+32,value)
    def xmm(value):
        w.raw('664c0f7ef8');w.expect(value)
    def set_xmm(value):
        w.imm('48b8',value);w.raw('664c0f6ef8')
    def wait_status(expected):
        syscall(61,(MAX,BUF+16,0,0));w.raw('4839e875fe');w.memory(BUF+16,expected)
    w.raw('53554154415541564157')
    w.imm('48bf',BUF+0x400);w.raw('480fae07')
    w.imm('48a1',PRIVATE);w.imm('48a3',BUF+0x100)
    syscall(8,(3,0,1));w.imm('48a3',BUF+0x108)
    w.store(PRIVATE,0x12345678);w.store(BUF,0x87654321)
    w.raw('dbe3d9e8') # FNINIT; FLD1: a live parent x87 value
    set_xmm(0xABCDEF1234567890)
    syscall(57);child=branch()
    w.raw('4883f80176fe4889c5')
    w.memory(PRIVATE,0x12345678);w.memory(BUF,0x87654321)
    fs(0x5F7400);xmm(0xABCDEF1234567890)
    w.call(8,(3,0,1),11)
    w.call(61,(MAX,MAX,1,0),0)
    w.call(57,(),-11)
    w.call(24,(MAX,MAX,MAX,MAX),0)
    w.memory(PRIVATE,0x12345678);w.memory(BUF,0x87654321)
    fs(0x5F7400);xmm(0xABCDEF1234567890)
    w.call(8,(3,0,1),13);w.call(8,(3,17,0),17)
    w.call(24,(),0)
    w.memory(PRIVATE,0x12345678);w.memory(BUF,0x87654321)
    # This parks while the child is live; the failed status is checked only
    # after the child's exit, in the restored parent generation.
    w.call(61,(MAX,MAX,0,0),-14)
    w.call(57,(),-11)
    w.store(BUF+16,0x1122334400000000)
    wait_status(0x1122334400000900)
    w.call(61,(MAX,0,1,0),-10)
    after=jump();bind(child)
    w.store(PRIVATE,0x22222222);w.store(BUF,0xBBBBBBBB)
    w.raw('dbe3d9eb') # FNINIT; FLDPI: a distinct live child x87 value
    w.call(158,(0x1002,0x5F7500),0);set_xmm(0xDEADC0DE11112222)
    w.call(8,(3,11,0),11)
    w.call(24,(),0)
    w.memory(PRIVATE,0x22222222);w.memory(BUF,0xBBBBBBBB)
    fs(0x5F7500);xmm(0xDEADC0DE11112222)
    w.call(8,(3,13,0),13)
    w.call(24,(),0)
    w.memory(PRIVATE,0x22222222);w.memory(BUF,0xBBBBBBBB)
    w.call(8,(3,0,1),17)
    w.call(24,(),0)
    # Parent is now blocked: this yield must stay in the child.
    w.call(24,(),0);w.call(39,(),6)
    syscall(60,(0x109,));w.raw('0f0b');bind(after)

    # The parked child owns all original file descriptions while the parent
    # independently opens records from the second bank. Those records must
    # survive child exit/reap and be inheritable by the following child.
    w.store(BUF+0x200,int.from_bytes(b'/bin/bus','little'))
    w.store(BUF+0x208,int.from_bytes(b'ybox\0\0\0\0','little'))
    w.call(3,(4,),0)
    for fd in range(4,16):w.call(257,(-100,BUF+0x200,0,0),fd)
    syscall(57);child=branch()
    w.raw('4889c5')
    for fd in range(3,16):w.call(3,(fd,),0)
    for fd in range(3,16):w.call(257,(-100,BUF+0x200,0,0),fd)
    w.call(24,(),0)
    wait_status(0x1122334400000000)
    w.call(8,(3,0,1),0)
    for fd in range(5,16):w.call(3,(fd,),0)
    w.call(3,(4,),0);w.call(33,(3,4),4)
    after=jump();bind(child)
    w.call(24,(),0)
    syscall(60,(0,));w.raw('0f0b');bind(after)
    syscall(57);child=branch()
    w.raw('4889c5');w.call(8,(3,0,1),23)
    syscall(61,(MAX,0,0,0));w.raw('4839e875fe')
    w.call(8,(4,0,1),23)
    after=jump();bind(child)
    w.call(8,(3,23,0),23);w.call(24,(),0)
    w.call(24,(),0)
    syscall(60,(0,));w.raw('0f0b');bind(after)

    w.imm('48a1',BUF+0x100);w.imm('48a3',PRIVATE)
    w.imm('48a1',BUF+0x108);w.raw('4889c6')
    w.imm('48bf',3);w.imm('48ba',0);w.imm('48b8',8);w.raw('0f05')
    w.imm('48bf',BUF+0x400);w.raw('480fae0f415f415e415d415c5d5b')
    result=w.finish()
    assert len(result)<0x8000
    return result
