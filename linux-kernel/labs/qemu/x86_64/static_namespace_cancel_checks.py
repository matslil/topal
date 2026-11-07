"""Actual child yield and parent exit with live shared namespace references."""
import struct

from static_namespace_transaction_checks import TransactionWitness,ENTRY,PATH,NEW,DATA,STAT,SAVED,names,links


def raw(w,number,args,event):
    for prefix,value in zip(('48bf','48be','48ba','49ba','49b8','49b9'),args):
        w.imm(prefix,value)
    w.imm('49bc',0x51ab)
    w.imm('48b8',number)
    entered=ENTRY+len(w.code)
    w.raw('0f05')
    w.calls.append(dict(returned=entered+2,number=number,arguments=list(args),event=event))


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
    w.calls[-1]['namespace_epoch']=1
    raw(w,57,(),'fork')
    w.raw('4885c00f85')
    parent=len(w.code)
    w.code.extend(bytes(4))
    names(w,'/tmp/b','/tmp/c')
    w.call(265,(-100,PATH,-100,NEW,0),0)
    w.calls[-1]['namespace_epoch']=2
    names(w,'/tmp/b','/tmp/d')
    w.call(264,(-100,PATH,-100,NEW),0)
    w.calls[-1]['namespace_epoch']=3
    w.unlink('/tmp/a')
    w.call(91,(4,0o600),0)
    w.store(DATA,int.from_bytes(b'child!!!','little'))
    w.call(1,(6,DATA,8),8)
    raw(w,24,(),'child-yield')
    w.raw('0f0b')  # Returning this parked child is a qualification failure.
    struct.pack_into('<i',w.code,parent,len(w.code)-parent-4)
    w.expect(11)
    for fd,position in ((4,16),(6,16),(5,0)):
        w.call(8,(fd,0,1),position)
        w.calls[-1]['namespace_epoch']=3
    links(w,4,2)
    w.same(STAT+8,SAVED)
    w.memory(STAT+24,0o100600)
    w.open('/tmp/a',0,-2)
    w.open('/tmp/b',0,-2)
    w.open('/tmp/c',0,7)
    links(w,7,2)
    w.same(STAT+8,SAVED)
    w.call(3,(7,),0)
    w.call(1,(5,DATA,1),-9)
    w.zero(DATA,24)
    w.call(0,(5,DATA,16),16)
    w.memory(DATA,int.from_bytes(b'parent!!','little'))
    w.memory(DATA+8,int.from_bytes(b'child!!!','little'))
    # Both descriptor banks and the two names remain owned at terminal exit.
    raw(w,60,(35,),'parent-exit')
    w.raw('0f0b')
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
