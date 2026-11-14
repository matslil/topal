"""Namespace aliases and epochs across the existing successful same-image exec chain."""
import struct

from static_namespace_transaction_checks import TransactionWitness,ENTRY,DATA,PATH,STAT,SAVED
from static_environment_checks import ENVIRONMENT

PRE,POST=0x518000,0x51a000
P,N,D=0x608200,0x608400,0x608600


def names(w,old,new):
    w.text(P,old)
    w.text(N,new)


def epoch(w,value):
    w.calls[-1]['namespace_epoch']=value


def pre():
    w=TransactionWitness(PRE)
    w.imm('48b8',0x4c4000)
    w.raw('ffd0')
    # The preceding environment builder owns 0x607000/0x607400 until exec;
    # these private paths never overwrite its still-live strings/vector.
    w.text(P,'/tmp/exec-a')
    w.call(257,(-100,P,0x42,0o600),5)
    w.call(33,(5,15),15)
    w.call(3,(5,),0)
    w.call(257,(-100,P,0,0),5)
    w.call(33,(5,13),13)
    w.call(3,(5,),0)
    w.call(72,(15,1030,14),14)
    w.store(D,int.from_bytes(b'before!!','little'))
    w.call(1,(15,D,8),8)
    names(w,'/tmp/exec-a','/tmp/exec-alias')
    w.call(86,(P,N),0)
    epoch(w,1)
    names(w,'/tmp/exec-a','/tmp/exec-moved')
    w.call(82,(P,N),0)
    epoch(w,2)
    w.call(59,(0,0,(1<<64)-1),-14)
    epoch(w,2)
    w.call(5,(15,STAT),0)
    epoch(w,2)
    w.calls[-1]['filesystem_checkpoint']=('first-exec','save')
    w.raw('c3')
    return w


def post():
    w=TransactionWitness(POST)
    w.raw('0fb60425')
    w.code.extend(struct.pack('<I',ENVIRONMENT+1022))
    w.raw('3c780f85')
    skip=len(w.code)
    w.code.extend(bytes(4))
    w.call(72,(14,1,0),-9)
    epoch(w,2)
    w.calls[-1]['filesystem_checkpoint']=('first-exec','compare')
    w.call(8,(15,0,1),8)
    w.call(8,(13,0,1),0)
    w.call(5,(15,STAT),0)
    w.memory(STAT+16,2)
    w.save(STAT+8,SAVED)
    w.text(P,'/tmp/exec-alias')
    w.call(257,(-100,P,0,0),5)
    w.call(5,(5,STAT),0)
    w.same(STAT+8,SAVED)
    w.memory(STAT+16,2)
    w.call(3,(5,),0)
    w.zero(D,16)
    w.call(0,(13,D,8),8)
    w.memory(D,int.from_bytes(b'before!!','little'))
    names(w,'/tmp/exec-alias','/tmp/exec-post')
    w.call(264,(-100,P,-100,N),0)
    epoch(w,3)
    for name in ('exec-moved','exec-post'):
        w.text(P,'/tmp/'+name)
        w.call(87,(P,),0)
    w.call(5,(15,STAT),0)
    w.memory(STAT+16,0)
    w.call(3,(15,),0)
    w.call(3,(13,),0)
    struct.pack_into('<i',w.code,skip,len(w.code)-skip-4)
    w.imm('48b8',0x4c8000)
    w.raw('ffd0c3')
    return w


def main():
    w=TransactionWitness(ENTRY)
    w.imm('48b8',0x4ac000)
    w.raw('ffd0')
    w.text(P,'/tmp/after-execs')
    w.call(257,(-100,P,0x42,0o640),4)
    epoch(w,3)
    w.call(5,(4,STAT),0)
    w.memory(STAT+8,8)  # The first image's inode identity seven was retired.
    w.store(D,int.from_bytes(b'final!!!','little'))
    w.call(1,(4,D,8),8)
    names(w,'/tmp/after-execs','/tmp/final-alias')
    w.call(86,(P,N),0)
    epoch(w,4)
    names(w,'/tmp/after-execs','/tmp/final-name')
    w.call(82,(P,N),0)
    epoch(w,5)
    w.call(5,(4,STAT),0)
    w.memory(STAT+16,2)
    w.text(P,'/tmp/final-alias')
    w.call(257,(-100,P,0,0),5)
    w.zero(D,16)
    w.call(0,(5,D,8),8)
    w.memory(D,int.from_bytes(b'final!!!','little'))
    for name in ('final-alias','final-name'):
        w.text(P,'/tmp/'+name)
        w.call(87,(P,),0)
    w.call(5,(4,STAT),0)
    w.memory(STAT+16,0)
    for fd in (5,4):
        w.call(3,(fd,),0)
    w.zero(PATH,0x2000)
    w.raw('c3')
    return w


def fixture(original):
    image=bytearray(original)
    before=bytes(image[0xb2000:0xb4000])
    runtime=bytes(image[0xaa000:0xab000])
    for pattern,region,start,offset,entry in (
        (bytes.fromhex('48b800404c0000000000ffd0'),before,0xb2000,2,PRE),
        (bytes.fromhex('b800804c00ffd0'),runtime,0xaa000,1,POST),
        (bytes.fromhex('b800c04a00ffd0'),runtime,0xaa000,1,ENTRY),
    ):
        assert region.count(pattern)==1
        at=start+region.index(pattern)+offset
        width=8 if offset==2 else 4
        image[at:at+width]=entry.to_bytes(width,'little')
    callers=(pre(),post(),main())
    assert len(callers[0].code)<POST-PRE and len(callers[1].code)<0x2000 and len(callers[2].code)<0x8000
    for w in callers:
        at=w.entry-0x400000
        assert not any(image[at:at+len(w.code)])
        image[at:at+len(w.code)]=w.code
    return bytes(image),[case for w in callers for case in w.calls]
