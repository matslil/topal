"""Independent user-mode process and system observation witnesses."""
ENTRY,TEMPLATE=0x4D0000,0x4D8000
BUFFER=0x7FE400
MAX=(1<<64)-1


def system_bytes():
    return b''.join(field+bytes(65-len(field)) for field in
                    (b'Linux',b'topal',b'7.2.9-topal',b'#1 Topal',b'x86_64',b'(none)'))


def copy(w,source,destination,count):
    for prefix,value in (('48be',source),('48bf',destination),('48b9',count)):
        w.imm(prefix,value)
    w.raw('fcf3a4')


def compare(w,address):
    for prefix,value in (('48be',address),('48bf',TEMPLATE),('48b9',390)):
        w.imm(prefix,value)
    w.raw('fcf3a675fe')


def code(witness_type):
    w=witness_type()
    copy(w,0x7FFE00,0x7FE800,512)
    w.store(BUFFER,0xDECAFBAD)
    for number in (MAX,0x7FFFFFFF,0x2000,0x100002000,0x40000027):
        w.imm('49b8',MAX-7)
        w.imm('49b9',0x619000)
        w.call(number,(MAX,0x619000,0x800000,MAX-1),-38)
        for instruction,value in (('4889f8',MAX),('4889f0',0x619000),
                                  ('4889d0',0x800000),('4c89d0',MAX-1),
                                  ('4c89c0',MAX-7),('4c89c8',0x619000)):
            w.raw(instruction)
            w.expect(value)
        w.memory(BUFFER,0xDECAFBAD)
        w.call(0x100000027,(),1)
    for number,expected in ((186,1),(110,0),(39,1),(39,1),(110,0),(186,1)):
        w.call(number,(MAX,0x619000,0,17),expected)
    for address in (BUFFER,0x7FF000,0x800000-390):
        w.call(63,(address,MAX),0)
        compare(w,address)
    w.store(0x7FFFF8,0xDECAFBAD)
    for address in (0,MAX-100,0x7FFFF8,0x619000,ENTRY,0x800000-389):
        w.call(63,(address,),-14)
        w.memory(0x7FFFF8,0xDECAFBAD)
    w.call(63,(BUFFER,),0)
    compare(w,BUFFER)
    copy(w,0x7FE800,0x7FFE00,512)
    return w.finish()
