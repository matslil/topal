"""Independent positional transfer caller; never kernel-linked."""
from pathlib import Path
import runpy
import struct

LAB=Path(__file__).resolve().parent
NAMED=runpy.run_path(str(LAB/'static_named_file_checks.py'))
ENTRY=0x520000
PRE_ENTRY,POST_ENTRY=0x528000,0x528800
PATH,DATA,STAT,SAVED=(NAMED[name] for name in ('PATH','DATA','STAT','SAVED'))
MAX=(1<<64)-1
LIMIT=(1<<63)-1

class PositionalWitness(NAMED['NamedWitness']):
    def __init__(self,entry=ENTRY):super().__init__(entry)

def fork(w):
    w.open('/tmp/pos-fork');w.open('/tmp/pos-fork',2,5);w.call(32,(4,),6)
    w.store(DATA,int.from_bytes(b'parent!!','little'));w.call(18,(4,DATA,8,0),8)
    w.call(8,(4,5,0),5);w.call(8,(5,7,0),7)
    w.imm('49bc',0x51ab);w.imm('48b8',57);w.raw('0f05')
    child=dict(returned=ENTRY+len(w.code),number=57,arguments=[],expected=0,tag='positional-fork')
    w.calls.append(child);w.raw('4885c00f85');jump=len(w.code);w.code.extend(bytes(4))
    w.zero(DATA,32);w.call(17,(6,DATA,8,0),8);w.memory(DATA,int.from_bytes(b'parent!!','little'))
    w.store(DATA,int.from_bytes(b'child!!!','little'));w.call(18,(6,DATA,8,1),8)
    w.call(8,(4,0,1),5);w.call(8,(5,0,1),7)
    w.unlink('/tmp/pos-fork');w.call(3,(4,),0)
    w.imm('48bf',9);w.imm('48b8',60);w.raw('0f050f0b')
    struct.pack_into('<i',w.code,jump,len(w.code)-jump-4)
    w.calls.append(dict(child,expected=11));w.expect(11)
    w.call(61,(11,STAT,0,0),11);w.memory(STAT,9<<8)
    w.call(8,(6,0,1),5);w.call(8,(5,0,1),7)
    w.zero(DATA,32);w.call(17,(5,DATA,9,0),9)
    w.memory(DATA,int.from_bytes(b'pchild!!','little'));w.memory(DATA+8,ord('!'))
    w.call(5,(6,STAT),0);w.memory(STAT+16,0)
    for fd in (4,5,6):w.call(3,(fd,),0)

def exec_lifetime():
    pre,post=PositionalWitness(PRE_ENTRY),PositionalWitness(POST_ENTRY)
    pre.imm('48b8',0x4c4000);pre.raw('ffd0')
    p,d=0x608200,0x608300
    pre.text(p,'/tmp/pos-exec');pre.call(257,(-100,p,0x42,0o666),5)
    pre.call(33,(5,15),15);pre.call(3,(5,),0)
    pre.call(257,(-100,p,0,0),5);pre.call(33,(5,13),13);pre.call(3,(5,),0)
    pre.call(72,(15,1030,14),14)
    pre.store(d,int.from_bytes(b'execdata','little'));pre.call(18,(15,d,8,0),8)
    pre.call(8,(15,3,0),3);pre.call(59,(0,0,MAX),-14);pre.raw('c3')
    environment=runpy.run_path(str(LAB/'static_environment_checks.py'))['ENVIRONMENT']
    post.raw('0fb60425');post.code.extend(struct.pack('<I',environment+1022));post.raw('3c780f85')
    skip=len(post.code);post.code.extend(bytes(4))
    post.call(72,(14,1,0),-9);post.call(8,(15,0,1),3);post.call(8,(13,0,1),0)
    post.call(17,(15,d,8,0),8);post.memory(d,int.from_bytes(b'execdata','little'))
    post.zero(d,16);post.call(17,(13,d,6,2),6);post.memory(d,int.from_bytes(b'ecdata'+bytes(2),'little'))
    post.call(8,(15,0,1),3);post.call(8,(13,0,1),0)
    post.text(p,'/tmp/pos-exec');post.call(87,(p,),0);post.call(3,(15,),0)
    post.call(17,(13,d,8,0),8);post.memory(d,int.from_bytes(b'execdata','little'));post.call(3,(13,),0)
    struct.pack_into('<i',post.code,skip,len(post.code)-skip-4)
    post.imm('48b8',0x4c8000);post.raw('ffd0c3')
    assert len(pre.code)<POST_ENTRY-PRE_ENTRY and len(post.code)<0x800
    return pre,post

def code(lifetime=False):
    w=PositionalWitness()
    w.imm('48b8',NAMED['ENTRY']);w.raw('ffd0')
    w.open('/tmp/position')
    w.store(DATA,int.from_bytes(b'abcdefgh','little'));w.call(1,(4,DATA,8),8)
    w.call(32,(4,),5);w.open('/tmp/position',2,6);w.call(8,(6,2,0),2)
    w.open('/tmp/position',0x401,7);w.open('/tmp/position',0,8);w.open('/tmp/position',1,9)
    w.open('/tmp',0x10000,10)
    w.call(293,(DATA+256,0),0);w.memory(DATA+256,11|(12<<32))
    for number,args,result in [
        (17,(99,MAX,0,MAX),-22),(18,(99,MAX,0,MAX),-22),
        (17,(99,MAX,0,0),-9),
        (17,(11,MAX,0,0),-29),(18,(12,MAX,0,0),-29),
        (17,(11,MAX,1,0),-29),(18,(11,MAX,1,0),-29),
        (17,(11,MAX,1,MAX),-22),
        (17,(9,MAX,0,0),-9),(18,(8,MAX,0,0),-9),
        (17,(10,DATA,0,0),-21),(17,(10,MAX,1,0),-14),
        (17,(10,DATA,1,LIMIT),-22),(18,(10,MAX,0,0),-9),
        (17,(4,MAX,0,0),-14),(18,(4,MAX,0,0),-14),
        (17,(4,0,0,0),0),(18,(4,0,0,0),0),
        (17,(4,DATA,1,LIMIT),-22),(18,(4,DATA,1,LIMIT),-22),
        (17,(4,DATA,0,LIMIT),0),(18,(4,DATA,0,LIMIT),0),
        (18,(7,DATA,2,LIMIT-1),-22),(18,(7,DATA,0,LIMIT),0),
        (17,(4,0x400000,2,0),-14),
        (17,(4,0x7fffff,2,0),-14),(18,(4,0x7fffff,2,0),-14),
    ]:w.call(number,args,result)
    for fd,at in [(4,8),(5,8),(6,2),(7,0)]:w.call(8,(fd,0,1),at)
    w.zero(DATA,32);w.call(17,(4,DATA,3,2),3);w.memory(DATA,int.from_bytes(b'cde'+bytes(5),'little'))
    w.store(DATA,int.from_bytes(b'XY'+bytes(6),'little'));w.call(18,(5,DATA,2,1),2)
    w.calls[-1]['tag']='positional-content-write'
    w.zero(DATA,32);w.call(17,(6,DATA,8,0),8);w.calls[-1]['tag']='positional-data-read';w.memory(DATA,int.from_bytes(b'aXYdefgh','little'))
    w.store(DATA,int.from_bytes(b'APP'+bytes(5),'little'));w.call(18,(7,DATA,3,0),3)
    w.zero(DATA,32);w.call(17,(4,DATA,11,0),11)
    w.memory(DATA,int.from_bytes(b'aXYdefgh','little'));w.memory(DATA+8,int.from_bytes(b'APP'+bytes(5),'little'))
    for fd,at in [(4,8),(5,8),(6,2),(7,0)]:w.call(8,(fd,0,1),at)
    w.call(8,(3,37,0),37);w.zero(DATA,32);w.call(17,(3,DATA,8,0),8)
    w.memory(DATA,int.from_bytes(bytes.fromhex('7f454c4602010103'),'little'))
    w.call(8,(3,0,1),37);w.call(18,(3,MAX,0,0),-9)
    w.call(17,(3,0x400000,2,0),-14)
    w.unlink('/tmp/position');w.call(5,(4,STAT),0);w.memory(STAT+16,0)
    sparse=(1<<48)+13
    w.store(DATA,int.from_bytes(b'AB'+bytes(6),'little'));w.call(18,(6,DATA,2,sparse),2)
    w.zero(DATA,32);w.call(17,(5,DATA,8,sparse-6),8);w.memory(DATA,int.from_bytes(bytes(6)+b'AB','little'))
    w.call(5,(4,STAT),0);w.memory(STAT+48,sparse+2);w.memory(STAT+64,16)
    for fd,at in [(4,8),(5,8),(6,2),(7,0)]:w.call(8,(fd,0,1),at)
    w.call(8,(4,LIMIT,0),LIMIT);w.call(17,(4,DATA,8,0),8)
    w.call(8,(5,0,1),LIMIT)
    for fd in range(4,13):w.call(3,(fd,),0)
    if lifetime:
        fork(w)
        w.open('/tmp/pos-persist');w.store(DATA,int.from_bytes(b'linked!!','little'))
        w.call(18,(4,DATA,8,3),8);w.call(8,(4,0,1),0);w.call(3,(4,),0)
        w.open('/tmp/pos-unlinked');w.store(DATA,int.from_bytes(b'unlinked','little'))
        w.call(18,(4,DATA,8,sparse),8);w.call(8,(4,0,1),0);w.unlink('/tmp/pos-unlinked')
    w.zero(PATH,0x2000);w.raw('c3')
    assert len(w.code)<0x8000
    return bytes(w.code),w.calls

def fixture(original,lifetime=False):
    assert original[:8]==bytes.fromhex('7f454c4602010103')
    image,previous=NAMED['fixture'](original,False);image=bytearray(image)
    start=ENTRY-0x400000;assert not any(image[start:start+0x10000])
    payload,calls=code(lifetime);image[start:start+len(payload)]=payload
    runtime=bytes(image[0xaa000:0xab000]);pattern=b'\xb8'+struct.pack('<I',NAMED['ENTRY'])+b'\xff\xd0'
    assert runtime.count(pattern)==1
    at=0xaa000+runtime.index(pattern);image[at:at+7]=b'\xb8'+struct.pack('<I',ENTRY)+b'\xff\xd0'
    if lifetime:
        pattern=bytes.fromhex('48a1')+struct.pack('<Q',STAT+8)+bytes.fromhex('49bb')+struct.pack('<Q',7)+bytes.fromhex('4c39d875fe')
        begin=NAMED['VOL']['ENTRY']-0x400000;anonymous=bytes(image[begin:begin+0x4000])
        assert anonymous.count(pattern)==1
        at=begin+anonymous.index(pattern)+12;image[at:at+8]=struct.pack('<Q',8)
        pre,post=exec_lifetime()
        for entry,value in [(PRE_ENTRY,pre.code),(POST_ENTRY,post.code)]:image[entry-0x400000:entry-0x400000+len(value)]=value
        before=bytes(image[0xb2000:0xb4000]);pattern=bytes.fromhex('48b800404c0000000000ffd0')
        assert before.count(pattern)==1;at=0xb2000+before.index(pattern)+2;image[at:at+8]=struct.pack('<Q',PRE_ENTRY)
        pattern=bytes.fromhex('b800804c00ffd0');assert runtime.count(pattern)==1
        at=0xaa000+runtime.index(pattern);image[at:at+7]=b'\xb8'+struct.pack('<I',POST_ENTRY)+b'\xff\xd0'
        previous=pre.calls+post.calls+previous
    return bytes(image),previous+calls
