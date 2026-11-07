"""Directory cookies and inode metadata after entry/object slots diverge."""
import struct

from static_namespace_transaction_checks import TransactionWitness,ENTRY,PATH,NEW,DATA,STAT,SAVED,names


def stat(w,fd,identity,mode,links=1):
    w.call(5,(fd,STAT),0)
    w.same(STAT+8,SAVED+identity*8)
    w.memory(STAT+16,links)
    w.memory(STAT+24,0o100000|mode)


def record(w,address,identity,cookie,name):
    w.same(address,SAVED+identity*8)
    w.memory(address+8,cookie)
    payload=bytes((24,0,8))+name.encode()+b'\0'
    payload+=bytes(8-len(payload))
    w.memory(address+16,int.from_bytes(payload,'little'))


def listing(w,associations):
    w.call(8,(5,0,0),0)
    w.zero(DATA,256)
    w.call(217,(5,DATA,256),48+24*len(associations))
    w.memory(DATA,6)
    w.memory(DATA+8,1)
    w.memory(DATA+24,3)
    w.memory(DATA+32,2)
    for index,(identity,cookie,name) in enumerate(associations):
        record(w,DATA+48+24*index,identity,cookie,name)
    w.call(8,(5,0,1),6)
    w.call(217,(5,(1<<64)-1,256),0)


def fixture(original):
    w=TransactionWitness(ENTRY)
    w.imm('48b8',0x4ac000)
    w.raw('ffd0')
    w.open('/tmp/a',0x42,4,0o644)
    w.call(5,(4,STAT),0)
    w.save(STAT+8,SAVED)
    w.store(DATA,int.from_bytes(b'a-inode!','little'))
    w.call(1,(4,DATA,8),8)
    names(w,'/tmp/a','/tmp/b')
    w.call(86,(PATH,NEW),0)
    stat(w,4,0,0o644,2)
    w.open('/tmp',0x10000,5)
    w.open('/tmp/d',0x42,6,0o600)
    w.call(5,(6,STAT),0)
    w.save(STAT+8,SAVED+8)
    w.store(DATA,int.from_bytes(b'd-inode!','little'))
    w.call(1,(6,DATA,8),8)
    w.unlink('/tmp/a')
    # Entry0 is reused by object2; entry1 still names object0 and entry2 object1.
    w.open('/tmp/e',0x42,7,0o641)
    w.call(5,(7,STAT),0)
    w.save(STAT+8,SAVED+16)
    w.store(DATA,int.from_bytes(b'e-inode!','little'))
    w.call(1,(7,DATA,8),8)
    listing(w,[(2,3,'e'),(0,4,'b'),(1,5,'d')])
    w.call(8,(5,0,0),0)
    w.call(217,(5,DATA,48),48)
    w.call(217,(5,DATA,24),24)
    record(w,DATA,2,3,'e')
    w.call(217,(5,DATA,23),-22)
    w.call(8,(5,0,1),3)
    w.call(217,(5,DATA,24),24)
    record(w,DATA,0,4,'b')
    w.call(217,(5,0x400000,24),-14)
    w.call(8,(5,0,1),4)
    w.call(217,(5,DATA,24),24)
    record(w,DATA,1,5,'d')
    w.call(8,(5,0,1),6)
    w.text(PATH,'/tmp/b')
    w.call(90,(PATH,0o640),0)
    stat(w,4,0,0o640)
    stat(w,6,1,0o600)
    stat(w,7,2,0o641)
    names(w,'/tmp/b','/tmp/d')
    w.call(316,(-100,PATH,-100,NEW,2),0)
    listing(w,[(2,3,'e'),(1,4,'b'),(0,5,'d')])
    w.text(PATH,'/tmp/b')
    w.call(90,(PATH,0o644),0)
    stat(w,4,0,0o640)
    stat(w,6,1,0o644)
    w.call(91,(4,0o643),0)
    w.open('/tmp/d',0,8)
    stat(w,8,0,0o643)
    w.text(PATH,'/tmp/d')
    w.call(262,(-100,PATH,STAT,0),0)
    w.same(STAT+8,SAVED)
    w.memory(STAT+24,0o100643)
    for fd,payload in ((8,b'a-inode!'),(6,b'd-inode!'),(7,b'e-inode!')):
        w.call(8,(fd,0,0),0)
        w.zero(DATA,16)
        w.call(0,(fd,DATA,8),8)
        w.memory(DATA,int.from_bytes(payload,'little'))
    w.unlink('/tmp/b')
    w.unlink('/tmp/e')
    stat(w,6,1,0o644,0)
    stat(w,7,2,0o641,0)
    listing(w,[(0,5,'d')])
    w.unlink('/tmp/d')
    stat(w,4,0,0o643,0)
    for fd in (8,7,6,5,4):
        w.call(3,(fd,),0)
    w.zero(PATH,0x2000)
    w.raw('c3')
    assert len(w.code)<0x8000
    image=bytearray(original)
    at=ENTRY-0x400000
    assert not any(image[at:at+0x8000])
    image[at:at+len(w.code)]=w.code
    runtime=bytes(image[0xaa000:0xab000])
    pattern=bytes.fromhex('b800c04a00ffd0')
    assert runtime.count(pattern)==1
    at=0xaa000+runtime.index(pattern)
    image[at:at+7]=b'\xb8'+struct.pack('<I',ENTRY)+b'\xff\xd0'
    return bytes(image),w.calls
