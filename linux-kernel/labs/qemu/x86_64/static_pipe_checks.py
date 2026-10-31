"""Independent native FIFO, failure, lifetime and blocking-progress witnesses."""
import struct
ENTRY=0x500000
BUF,IN,OUT,STAT,ACTION,OLD,PRIVATE=0x7FE400,0x60D000,0x60E000,0x7FE600,0x7FE700,0x7FE740,0x5F2000
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
    def fill(address,size,value):
        w.imm('48bf',address);w.imm('48b9',size);w.imm('48b8',value);w.raw('fcf3aa')
    def same(address,size,value):
        w.imm('48bf',address);w.imm('48b9',size);w.imm('48b8',value);w.raw('fcf3ae75fe')
    def pair(flags=0):
        w.call(293,(BUF+0x40,flags),0);w.memory(BUF+0x40,(6<<32)|5)
    def wait(status):
        syscall(61,(MAX,BUF+0x50,0,0));w.raw('4839e875fe');w.memory(BUF+0x50,status)
    def state(value,xmm,fs):
        w.memory(PRIVATE,value);w.raw('664c0f7ef8');w.expect(xmm)
        w.call(158,(0x1003,BUF+0x30),0);w.memory(BUF+0x30,fs)
    w.raw('53554154415541564157')
    w.imm('48bf',BUF+0x400);w.raw('480fae07')
    w.imm('48a1',PRIVATE);w.imm('48a3',BUF+0x100)
    # Native admission, exact-width publication, descriptor limits and rollback.
    for flags,result in ((4,-22),(0x4000,-38),(0x80,-38),(MAX,-22)):
        w.call(293,(MAX,flags),result)
    for fd in range(5,15):w.call(32,(3,),fd)
    w.call(293,(MAX,0),-24)
    for fd in range(5,15):w.call(3,(fd,),0)
    w.call(293,(MAX,0),-14)
    syscall(201,(0,));w.imm('48a3',BUF+0x170)
    w.store(BUF+0x48,0x123456789ABCDEF0)
    pair((1<<32)|0x80800)
    w.memory(BUF+0x48,0x123456789ABCDEF0)
    for number,args,result in ((72,(5,1),1),(72,(6,1),1),(72,(5,3),0x800),
                               (72,(6,3),0x801),(72,(5,1032),4096),
                               (72,(6,1032),4096),(8,(5,0,0),-29),
                               (8,(5,MAX,4),-29),(8,(5,MAX,(1<<32)|3),-29),
                               (8,(5,0,5),-22),(8,(6,0,MAX),-22),
                               (8,(1,0,MAX),-22),(8,(15,0,MAX),-9),
                               (0,(6,OUT,1),-9),(1,(5,IN,1),-9),
                               (16,(5,1,MAX),-25),(16,(5,0x541b,MAX),-14)):
        w.call(number,args,result)
    w.call(5,(5,STAT),0)
    for offset,value in ((0,2),(16,1),(24,0x1180),(32,0),(48,0),(56,4096),(64,0)):
        w.memory(STAT+offset,value)
    w.imm('48a1',STAT+8);w.raw('4885c074fe');w.imm('48a3',BUF+0x58)
    w.call(5,(6,STAT),0);w.imm('48a1',STAT+8);w.raw('4889c3')
    w.imm('48a1',BUF+0x58);w.raw('4839d875fe')
    syscall(201,(0,));w.raw('4989c5') # current time in r13
    w.imm('48a1',BUF+0x170);w.raw('4889c3')
    for offset in (72,88,104):
        w.imm('48a1',STAT+offset);w.raw('4839d872fe4c39e877fe')
        w.memory(STAT+offset+8,0)
    w.store(BUF+0x60,0x8765432100000000)
    w.call(16,(5,0x541b,BUF+0x60),0);w.memory(BUF+0x60,0x8765432100000000)
    w.call(0,(5,OUT,1),-11);w.call(0,(5,0x619000,1),-11)
    w.call(0,(5,MAX,0),-14);w.call(1,(6,MAX,0),-14)
    w.store(IN,0x434241);w.call(1,(6,IN,3),3)
    w.call(16,(6,0x541b,BUF+0x60),0);w.memory(BUF+0x60,0x8765432100000003)
    w.store(OUT,0);w.call(0,(5,OUT,2),2);w.memory(OUT,0x4241)
    w.call(0,(5,0x619000,1),-14);w.call(16,(5,0x541b,BUF+0x60),0)
    w.memory(BUF+0x60,0x8765432100000001)
    w.store(OUT,0);w.call(0,(5,OUT,8),1);w.memory(OUT,0x43)
    fill(IN,4096,0x5A);w.call(1,(6,IN,4096),4096)
    w.call(1,(6,IN,1),-11);w.call(1,(6,MAX,1),-14)
    w.call(0,(5,OUT,3000),3000);same(OUT,3000,0x5A)
    w.call(1,(6,IN,3001),-11)
    w.call(1,(6,IN,8192),3000)
    w.call(0,(5,OUT,4096),4096);same(OUT,4096,0x5A)
    w.call(1,(6,0x619000,1),-14)
    w.call(32,(6,),7);w.call(72,(7,1),0);w.call(72,(7,3),0x801)
    w.call(3,(6,),0);w.call(0,(5,OUT,1),-11)
    w.call(3,(7,),0);w.call(0,(5,0x619000,1),0);w.call(3,(5,),0)
    # Last-reader failure only under the qualified ignored-SIGPIPE disposition.
    pair()
    for offset,value in ((0,1),(8,0),(16,0),(24,0)):w.store(ACTION+offset,value)
    w.call(13,(13,ACTION,OLD,8),0)
    w.store(BUF+0x180,1<<12)
    w.call(14,(0,BUF+0x180,BUF+0x188,8),0);w.call(3,(5,),0)
    w.call(1,(6,IN,1),-32);w.call(1,(6,IN,0),0)
    w.call(14,(2,BUF+0x188,0,8),0)
    w.call(13,(13,OLD,0,8),0);w.call(1,(6,IN,1),-38);w.call(3,(6,),0)
    # Four independent FIFO owners; pool exhaustion precedes user publication.
    for index in range(4):
        w.call(22,(BUF+0x40,MAX,MAX,MAX),0)
        w.memory(BUF+0x40,((6+2*index)<<32)|(5+2*index))
    w.call(293,(MAX,0),-23)
    for fd in range(5,13):w.call(3,(fd,),0)

    # The child blocks in read. An unrelated FIFO cannot make it eligible.
    pair();w.call(293,(BUF+0x40,0),0);w.memory(BUF+0x40,(8<<32)|7)
    w.store(IN,0x434241);w.store(PRIVATE,0x11112222)
    w.imm('48b8',0xABCDEF1234567890);w.raw('664c0f6ef8')
    syscall(57);child=branch();w.raw('4883f80176fe4889c5')
    state(0x11112222,0xABCDEF1234567890,0x5F7400)
    w.call(1,(8,IN,3),3);w.call(24,(),0)
    w.call(3,(7,),0);w.call(3,(8,),0)
    w.call(1,(6,IN,3),3);w.call(61,(MAX,0,1,0),0)
    w.call(24,(),0)
    state(0x11112222,0xABCDEF1234567890,0x5F7400)
    w.call(3,(6,),0);w.store(OUT,0);w.call(0,(5,OUT,8),2);w.memory(OUT,0x4342)
    w.call(0,(5,OUT,1),0);w.call(3,(5,),0);wait(4<<8)
    after=jump();bind(child)
    w.store(PRIVATE,0x9999AAAA);w.imm('48b8',0xDEADC0DE11112222);w.raw('664c0f6ef8')
    w.call(158,(0x1002,0x5F7500),0);w.call(3,(6,),0)
    w.call(0,(5,0x619000,1),-14) # blocks first; permissions are checked after readiness
    w.store(OUT,0);w.call(0,(5,OUT,1),1);w.memory(OUT,0x41)
    state(0x9999AAAA,0xDEADC0DE11112222,0x5F7500)
    w.call(3,(5,),0);syscall(60,(0x104,));w.raw('0f0b');bind(after)

    # An atomic writer stays blocked until the complete requested space exists.
    pair();fill(IN,4096,0x58)
    syscall(57);child=branch();w.raw('4883f80176fe4889c5')
    w.call(3,(6,),0);w.call(0,(5,OUT,2048),2048);same(OUT,2048,0x58)
    w.call(24,(),0) # insufficient space must not make the child runnable
    w.call(0,(5,OUT,2048),2048);same(OUT,2048,0x58)
    w.call(0,(5,OUT,1),1)
    w.call(0,(5,OUT,4096),4095);same(OUT,4095,0x58)
    w.call(0,(5,OUT,1),0);w.call(3,(5,),0);wait(0)
    after=jump();bind(child)
    w.call(3,(5,),0);w.call(1,(6,IN,4096),4096)
    w.call(1,(6,0x619000,4096),-14) # full FIFO parks before the input-page fault
    w.call(1,(6,IN,4096),4096)
    syscall(60,(0,));w.raw('0f0b');bind(after)

    w.imm('48a1',BUF+0x100);w.imm('48a3',PRIVATE)
    fill(IN,8192,0)
    w.imm('48bf',BUF+0x400);w.raw('480fae0f415f415e415d415c5d5b')
    result=w.finish();assert len(result)<0x8000;return result
