"""Independent caller-issued named-file checks, never linked into the kernel."""
from pathlib import Path
import runpy
import struct
LAB=Path(__file__).resolve().parent
VOL=runpy.run_path(str(LAB/'static_volatile_file_checks.py'))
ENTRY,PRE_ENTRY,POST_ENTRY=0x510000,0x518000,0x518800
PATH,DATA,STAT,SAVED=0x607000,0x608000,0x608800,0x608c00
MAX=(1<<64)-1
class NamedWitness(VOL['FileWitness']):
    def __init__(self,entry=ENTRY):super().__init__(entry)
    def bytes(self,address,value):
        value+=b'\0';value+=bytes((-len(value))%8)
        for offset in range(0,len(value),8):self.store(address+offset,int.from_bytes(value[offset:offset+8],'little'))
    def zero(self,address,size):
        self.imm('48bf',address);self.imm('48b9',size//8);self.raw('31c0fcf348ab')
    def same(self,actual,expected):
        self.imm('48a1',actual);self.imm('48b9',expected);self.raw('4c8b194c39d875fe')
    def save(self,source,destination):self.imm('48a1',source);self.imm('48a3',destination)
    def open(self,name,flags=0x42,fd=4,mode=0o666):
        self.text(PATH,name);self.call(257,(-100,PATH,flags,mode),fd)
    def unlink(self,name,expected=0):self.text(PATH,name);self.call(87,(PATH,),expected)

def basic(w):
    w.open('/tmp/a');w.call(5,(4,STAT),0)
    for offset,value in [(0,2),(16,1),(24,0o100644),(48,0),(64,0)]:w.memory(STAT+offset,value)
    w.save(STAT+8,SAVED)
    w.store(DATA,int.from_bytes(b'abcdefgh','little'));w.call(1,(4,DATA,8),8)
    w.calls[-1]['tag']='named-content-write'
    w.open('/tmp/a',0,5);w.call(32,(5,),6)
    w.call(8,(4,0,1),8);w.call(8,(5,0,1),0)
    w.zero(DATA,32);w.call(0,(5,DATA,2),2);w.memory(DATA,int.from_bytes(b'ab\0\0\0\0\0\0','little'))
    w.call(0,(6,DATA,2),2);w.memory(DATA,int.from_bytes(b'cd\0\0\0\0\0\0','little'))
    w.call(8,(5,0,1),4);w.call(8,(4,0,1),8)
    w.open('/tmp/a',0xc0,-17);w.open('/tmp/a',0x80,7)
    w.call(3,(7,),0)
    w.open('/tmp/a',0x200,7);w.call(5,(7,STAT),0)
    w.memory(STAT+16,1);w.memory(STAT+24,0o100644);w.memory(STAT+48,0);w.memory(STAT+64,0)
    w.call(8,(4,0,1),8);w.call(8,(5,0,1),4)
    w.call(1,(7,DATA,1),-9)
    w.store(DATA,int.from_bytes(b'x1234567','little'));w.call(1,(4,DATA,1),1)
    w.zero(DATA,16);w.call(0,(7,DATA,9),9);w.memory(DATA,0);w.memory(DATA+8,ord('x'))
    w.call(8,(5,0,0),0);w.call(0,(6,DATA,9),9);w.memory(DATA,0);w.memory(DATA+8,ord('x'))
    w.unlink('/tmp/a');w.call(5,(4,STAT),0);w.memory(STAT+16,0)
    w.open('/tmp/a',0,-2)
    w.open('/tmp/a',0xc2,8,0o600);w.call(5,(8,STAT),0)
    w.imm('48a1',STAT+8);w.imm('48b9',SAVED);w.raw('4c8b194c39d874fe')
    w.memory(STAT+16,1);w.memory(STAT+24,0o100600);w.memory(STAT+48,0)
    w.store(DATA,int.from_bytes(b'new-data','little'));w.call(1,(8,DATA,8),8)
    for fd in (4,5,6):w.call(3,(fd,),0)
    w.call(8,(7,0,0),0);w.zero(DATA,16);w.call(0,(7,DATA,9),9);w.memory(DATA,0);w.memory(DATA+8,ord('x'))
    w.call(3,(7,),0);w.call(3,(8,),0)
    w.open('/tmp/a',0,4);w.call(0,(4,DATA,8),8);w.memory(DATA,int.from_bytes(b'new-data','little'))
    w.call(3,(4,),0)
    w.open('/tmp/a',0x401,4);w.call(8,(4,0,0),0);w.store(DATA,int.from_bytes(b'append!!','little'))
    w.call(1,(4,DATA,8),8);w.call(8,(4,0,1),16);w.call(3,(4,),0)
    w.open('/tmp/a',0,4);w.call(0,(4,DATA,16),16)
    w.memory(DATA,int.from_bytes(b'new-data','little'));w.memory(DATA+8,int.from_bytes(b'append!!','little'))
    w.call(3,(4,),0);w.unlink('/tmp/a')

def errors(w):
    for name,flags,result in [('/tmp/missing/',0x42,-21),('/tmp/missing/',0,-2),('/tmp/.',0x40,-21),
        ('/tmp/.',0xc0,-17),('/bin/busybox',0x40,4),('/bin/busybox',0xc0,-17),
        ('/bin/missing',0x42,-30),('/tmp/missing/child',0x42,-2)]:
        w.open(name,flags,result)
        if result>=0:w.call(3,(result,),0)
    w.call(257,(-100,MAX,0x10040,0),-22)
    w.call(263,(-100,MAX,1),-22);w.call(263,(-100,MAX,0x200),-38)
    w.unlink('/tmp/missing/',-2);w.unlink('/tmp/.',-21);w.unlink('/tmp/..',-21)
    w.unlink('/bin/missing',-30);w.unlink('/bin/missing/',-30)
    w.text(PATH,'/bin/'+'x'*256);w.call(87,(PATH,),-30)
    w.open('/tmp/file');w.open('/tmp/file/',0x42,-21);w.unlink('/tmp/file/',-20)
    w.open('/tmp/file/child',0x42,-20)
    w.text(PATH,'/tmp/file');w.call(80,(PATH,),-20);w.call(263,(4,PATH,0),0)
    w.call(5,(4,STAT),0);w.memory(STAT+16,0);w.call(3,(4,),0)
    w.open('/tmp',0x10000,4)
    w.text(PATH,'relative');w.call(257,(4,PATH,0x42,0o666),5)
    w.call(262,(4,PATH,STAT,0),0);w.memory(STAT+16,1)
    w.call(263,(5,PATH,0),-20)
    w.call(263,(4,PATH,0),0);w.call(3,(5,),0);w.call(3,(4,),0)

def long_record(w):
    raw=bytearray(b'\xff'*255);raw[10]=0x80
    w.open('/tmp',0x10000,4)
    w.bytes(PATH,bytes(raw));w.call(257,(4,PATH,0x42,0o600),5)
    w.call(5,(5,STAT),0);w.save(STAT+8,SAVED);w.call(3,(5,),0)
    w.zero(DATA,512);w.call(217,(4,DATA,48),48)
    w.memory(DATA,6);w.memory(DATA+8,1);w.memory(DATA+24,3);w.memory(DATA+32,2)
    w.call(217,(4,DATA,279),-22);w.call(8,(4,0,1),2)
    w.call(217,(4,0x400000,280),-14);w.call(8,(4,0,1),2)
    w.zero(DATA,512);w.call(217,(4,DATA,280),280)
    w.same(DATA,SAVED);w.memory(DATA+8,3)
    w.memory(DATA+16,int.from_bytes(bytes([24,1,8])+bytes(raw[:5]),'little'))
    for offset in range(0,256,8):
        value=bytes(raw[offset:offset+8]);value+=bytes(8-len(value))
        w.memory(DATA+19+offset,int.from_bytes(value,'little'))
    w.memory(DATA+275,0)
    w.call(217,(4,MAX,512),0);w.call(8,(4,0,1),6)
    w.call(8,(4,0,0),0);w.call(217,(4,DATA,48),48)
    w.bytes(PATH,bytes(raw));w.call(263,(4,PATH,0),0);w.call(3,(4,),0)
    w.bytes(PATH,b'/tmp/'+b'x'*256);w.call(257,(-100,PATH,0x42,0),-36)

def capacity(w):
    w.open('/tmp/slots');w.store(DATA,int.from_bytes(b'reopened','little'));w.call(1,(4,DATA,8),8)
    for fd in range(5,16):w.open('/tmp/slots',0,fd)
    w.open('/tmp/missing',0,-24)
    for fd in range(4,16):w.call(3,(fd,),0)
    w.open('/tmp/slots',0,4);w.call(0,(4,DATA,8),8);w.memory(DATA,int.from_bytes(b'reopened','little'))
    w.call(3,(4,),0);w.unlink('/tmp/slots')
    for index in range(4):w.open('/tmp/f'+str(index));w.call(3,(4,),0)
    w.open('/tmp/f4',0x42,-28)
    w.unlink('/tmp/f1');w.open('/tmp/f4');w.call(3,(4,),0)
    w.open('/tmp/f0',2,4);w.open('/tmp/f0',2,5);w.unlink('/tmp/f0')
    w.open('/tmp/f5',0x42,-28);w.call(3,(4,),0);w.open('/tmp/f5',0x42,-28);w.call(3,(5,),0)
    w.open('/tmp/f5');w.call(5,(4,STAT),0);w.memory(STAT+48,0);w.memory(STAT+64,0);w.call(3,(4,),0)
    for index in (2,3,4,5):w.unlink('/tmp/f'+str(index))

def fork(w):
    w.open('/tmp/fork');w.open('/tmp/fork',2,5);w.call(32,(4,),6)
    w.store(DATA,int.from_bytes(b'parent!!','little'));w.call(1,(4,DATA,8),8)
    w.imm('49bc',0x51ab);w.imm('48b8',57);w.raw('0f05')
    child=dict(returned=ENTRY+len(w.code),number=57,arguments=[],expected=0,tag='named-fork')
    w.calls.append(child);w.raw('4885c00f85');jump=len(w.code);w.code.extend(bytes(4))
    w.call(95,(0o077,),0o022);w.store(DATA,int.from_bytes(b'child!!!','little'));w.call(1,(6,DATA,8),8)
    w.unlink('/tmp/fork');w.call(8,(5,0,0),0);w.call(0,(5,DATA,8),8)
    w.memory(DATA,int.from_bytes(b'parent!!','little'));w.call(3,(4,),0)
    w.imm('48bf',9);w.imm('48b8',60);w.raw('0f050f0b')
    struct.pack_into('<i',w.code,jump,len(w.code)-jump-4)
    w.calls.append(dict(child,expected=11));w.expect(11)
    w.call(95,(0o022,),0o022);w.call(61,(11,STAT,0,0),11);w.memory(STAT,9<<8)
    w.call(8,(4,0,1),16);w.call(8,(5,0,1),8);w.call(5,(4,STAT),0);w.memory(STAT+16,0)
    w.open('/tmp/fork',0,-2);w.call(0,(5,DATA,8),8);w.memory(DATA,int.from_bytes(b'child!!!','little'))
    w.call(8,(6,0,0),0);w.call(8,(5,0,1),16);w.call(3,(4,),0);w.call(3,(6,),0)
    w.call(8,(5,0,0),0);w.call(0,(5,DATA,16),16)
    w.memory(DATA,int.from_bytes(b'parent!!','little'));w.memory(DATA+8,int.from_bytes(b'child!!!','little'));w.call(3,(5,),0)

def code(lifetime=False):
    w=NamedWitness();w.imm('48b8',VOL['ENTRY']);w.raw('ffd0')
    basic(w);errors(w);long_record(w);capacity(w)
    if lifetime:
        fork(w)
        w.open('/tmp/persist');w.store(DATA,int.from_bytes(b'linked!!','little'));w.call(1,(4,DATA,8),8);w.call(3,(4,),0)
        w.open('/tmp/unlinked');w.store(DATA,int.from_bytes(b'unlinked','little'));w.call(1,(4,DATA,8),8);w.unlink('/tmp/unlinked')
    w.zero(PATH,0x2000);w.raw('c3')
    assert len(w.code)<PRE_ENTRY-ENTRY
    return bytes(w.code),w.calls

def exec_lifetime():
    pre,post=NamedWitness(PRE_ENTRY),NamedWitness(POST_ENTRY)
    pre.imm('48b8',0x4c4000);pre.raw('ffd0')
    p,d=0x608200,0x608300
    pre.text(p,'/tmp/exec');pre.call(257,(-100,p,0x42,0o666),5);pre.call(33,(5,15),15);pre.call(3,(5,),0)
    pre.call(257,(-100,p,0,0),5);pre.call(33,(5,13),13);pre.call(3,(5,),0);pre.call(72,(15,1030,14),14)
    pre.store(d,int.from_bytes(b'execdata','little'));pre.call(1,(15,d,8),8);pre.call(59,(0,0,MAX),-14);pre.raw('c3')
    environment=runpy.run_path(str(LAB/'static_environment_checks.py'))['ENVIRONMENT']
    post.raw('0fb60425');post.code.extend(struct.pack('<I',environment+1022));post.raw('3c780f85')
    skip=len(post.code);post.code.extend(bytes(4))
    post.call(72,(14,1,0),-9);post.call(8,(15,0,1),8);post.call(8,(13,0,1),0)
    post.text(p,'/tmp/exec');post.call(87,(p,),0);post.call(5,(15,STAT),0);post.memory(STAT+16,0)
    post.call(3,(15,),0);post.call(0,(13,d,8),8);post.memory(d,int.from_bytes(b'execdata','little'));post.call(3,(13,),0)
    struct.pack_into('<i',post.code,skip,len(post.code)-skip-4)
    post.imm('48b8',0x4c8000);post.raw('ffd0c3')
    assert len(pre.code)<POST_ENTRY-PRE_ENTRY and len(post.code)<0x800
    return pre,post

def fixture(original,lifetime=False):
    image,previous=VOL['fixture'](original,False);image=bytearray(image)
    start=ENTRY-0x400000;assert not any(image[start:start+0x10000])
    payload,calls=code(lifetime);image[start:start+len(payload)]=payload
    runtime=bytes(image[0xaa000:0xab000]);pattern=b'\xb8'+struct.pack('<I',VOL['ENTRY'])+b'\xff\xd0'
    assert runtime.count(pattern)==1;position=0xaa000+runtime.index(pattern)
    image[position:position+7]=b'\xb8'+struct.pack('<I',ENTRY)+b'\xff\xd0'
    if lifetime:
        # The real pre-exec named creation consumes inode seven. Preserve the
        # anonymous caller's exact first-inode check against the next identity.
        pattern=bytes.fromhex('48a1')+struct.pack('<Q',STAT+8)+bytes.fromhex('49bb')+struct.pack('<Q',7)+bytes.fromhex('4c39d875fe')
        begin=VOL['ENTRY']-0x400000;anonymous=bytes(image[begin:begin+0x4000])
        assert anonymous.count(pattern)==1
        at=begin+anonymous.index(pattern)+12
        image[at:at+8]=struct.pack('<Q',8)
        pre,post=exec_lifetime()
        for entry,value in [(PRE_ENTRY,pre.code),(POST_ENTRY,post.code)]:image[entry-0x400000:entry-0x400000+len(value)]=value
        before=bytes(image[0xb2000:0xb4000]);pattern=bytes.fromhex('48b800404c0000000000ffd0')
        assert before.count(pattern)==1;position=0xb2000+before.index(pattern)+2;image[position:position+8]=struct.pack('<Q',PRE_ENTRY)
        pattern=bytes.fromhex('b800804c00ffd0');assert runtime.count(pattern)==1
        position=0xaa000+runtime.index(pattern);image[position:position+7]=b'\xb8'+struct.pack('<I',POST_ENTRY)+b'\xff\xd0'
        calls=pre.calls+post.calls+previous+calls
    else:calls=previous+calls
    return bytes(image),calls
