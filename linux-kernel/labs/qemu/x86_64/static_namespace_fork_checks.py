"""Real fork/wait with shared aliases, descriptions and nonrewinding source epochs."""
import struct

from static_namespace_transaction_checks import TransactionWitness,ENTRY,PATH,NEW,DATA,STAT,SAVED,names,links


def checkpoint(w,epoch):
    w.calls[-1]['namespace_epoch']=epoch


def fixture(original):
    w=TransactionWitness(ENTRY)
    w.imm('48b8',0x4ac000)
    w.raw('ffd0')
    w.open('/tmp/a',0x42,4,0o644)
    w.store(DATA,int.from_bytes(b'parent!!','little'))
    w.call(1,(4,DATA,8),8)
    links(w,4,1)
    w.save(STAT+8,SAVED)
    w.open('/tmp/a',0,5)
    w.call(32,(4,),6)
    names(w,'/tmp/a','/tmp/b')
    w.call(86,(PATH,NEW),0)
    checkpoint(w,1)
    # Each branch has distinct observed syscall PCs. The raw fork return is
    # checked by branch selection and the parent's exact prepared child PID.
    w.imm('49bc',0x51ab)
    w.imm('48b8',57)
    w.raw('0f054885c00f85')
    parent=len(w.code)
    w.code.extend(bytes(4))
    names(w,'/tmp/b','/tmp/c')
    w.call(265,(-100,PATH,-100,NEW,0),0)
    checkpoint(w,2)
    names(w,'/tmp/b','/tmp/d')
    w.call(264,(-100,PATH,-100,NEW),0)
    checkpoint(w,3)
    w.unlink('/tmp/a')
    w.text(PATH,'/tmp/d')
    w.call(90,(PATH,0o640),0)
    w.store(DATA,int.from_bytes(b'child!!!','little'))
    w.call(1,(6,DATA,8),8)
    w.call(1,(5,DATA,1),-9)
    w.zero(DATA,16)
    w.call(0,(5,DATA,8),8)
    w.memory(DATA,int.from_bytes(b'parent!!','little'))
    links(w,4,2)
    w.same(STAT+8,SAVED)
    w.call(3,(4,),0)
    w.imm('48bf',9)
    w.imm('48b8',60)
    w.raw('0f050f0b')
    struct.pack_into('<i',w.code,parent,len(w.code)-parent-4)
    w.expect(11)
    w.call(61,(11,STAT,0,0),11)
    checkpoint(w,3)
    w.memory(STAT,9<<8)
    for fd,position in ((4,16),(6,16),(5,8)):
        w.call(8,(fd,0,1),position)
    links(w,4,2)
    w.same(STAT+8,SAVED)
    w.memory(STAT+24,0o100640)
    w.open('/tmp/a',0,-2)
    w.open('/tmp/b',0,-2)
    w.zero(DATA,16)
    w.call(0,(5,DATA,8),8)
    w.memory(DATA,int.from_bytes(b'child!!!','little'))
    names(w,'/tmp/c','/tmp/z')
    w.call(86,(PATH,NEW),0)
    checkpoint(w,4)
    names(w,'/tmp/c','/tmp/parent')
    w.call(82,(PATH,NEW),0)
    checkpoint(w,5)
    links(w,4,3)
    for name in ('d','z','parent'):
        w.unlink('/tmp/'+name)
    links(w,5,0)
    w.same(STAT+8,SAVED)
    names(w,'','/tmp/resurrect')
    w.call(265,(5,PATH,999,NEW,0x1000),-2)
    checkpoint(w,5)
    w.call(1,(5,DATA,1),-9)
    w.call(8,(4,0,0),0)
    w.zero(DATA,24)
    w.call(0,(4,DATA,16),16)
    w.memory(DATA,int.from_bytes(b'parent!!','little'))
    w.memory(DATA+8,int.from_bytes(b'child!!!','little'))
    for fd in (6,5,4):
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
