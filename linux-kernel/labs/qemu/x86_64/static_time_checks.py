"""Independent executable monotonic-time and relative-interval witnesses."""
ENTRY,TEMPLATE=0x4E0000,0x4E8000
BUF=0x7FE400
INTERVAL,REMAIN=BUF+64,BUF+96
MAX=(1<<64)-1


def copy(w,source,destination,count):
    for prefix,value in (('48be',source),('48bf',destination),('48b9',count)):
        w.imm(prefix,value)
    w.raw('fcf3a4')


def normalized(w,address):
    w.imm('48a1',address)
    w.raw('4885c078fe')
    w.imm('48a1',address+8)
    w.raw('483d00ca9a3b73fe')


def nanoseconds(w,address):
    w.imm('48a1',address)
    w.raw('4869c000ca9a3b')
    w.imm('48be',address+8)
    w.raw('480306')


def code(witness_type):
    w=witness_type()
    w.raw('53554154415541564157')
    copy(w,0x7FFE00,0x7FEB00,512)
    w.call(229,(0x100000001,BUF),0)
    w.memory(BUF,0)
    w.memory(BUF+8,10)
    w.call(229,(1,0),0)
    for number in (228,229):
        w.call(number,(0,MAX),-22)
        for address in (0x619000,MAX,ENTRY,0x7FFFF1):
            w.store(0x7FFFF8,0xDECAFBAD)
            w.call(number,(1,address),-14)
            w.memory(0x7FFFF8,0xDECAFBAD)
        w.call(number,(1,0x7FFFF0),0)
        normalized(w,0x7FFFF0)
    w.call(228,(1,0),-14)
    w.store(REMAIN,0xDECAFBAD)
    w.store(REMAIN+8,0x51AB)
    for address in (0,MAX,0x619000,0x7FFFF1):
        w.call(35,(address,REMAIN),-14)
        w.call(230,(1,0,address,REMAIN),-14)
    for seconds,nanos in ((MAX,0),(0,MAX),(0,1000000000)):
        w.store(INTERVAL,seconds)
        w.store(INTERVAL+8,nanos)
        w.call(35,(INTERVAL,REMAIN),-22)
        w.call(230,(0,0,INTERVAL,REMAIN),-22)
    w.call(35,(TEMPLATE,MAX),0)
    w.store(0x7FFFF0,0)
    w.store(0x7FFFF8,1)
    w.call(230,(1,0,0x7FFFF0,MAX),0)
    for index,nanos in enumerate((0,1,1000001,2000000)):
        w.store(INTERVAL,0)
        w.store(INTERVAL+8,nanos)
        w.call(228,(0x100000001,BUF),0)
        normalized(w,BUF)
        nanoseconds(w,BUF)
        w.raw('4889c3')
        if index%2==0:
            w.call(35,(INTERVAL,REMAIN),0)
        else:
            w.call(230,(0x100000001,0x100000000,INTERVAL,REMAIN),0)
        w.call(228,(1,BUF+16),0)
        normalized(w,BUF+16)
        nanoseconds(w,BUF+16)
        w.raw('4829d872fe')
        w.imm('49bb',nanos)
        w.raw('4c39d872fe')
        w.memory(REMAIN,0xDECAFBAD)
        w.memory(REMAIN+8,0x51AB)
    copy(w,0x7FEB00,0x7FFE00,512)
    w.imm('48bf',0x7FEB00)
    w.imm('48b9',512//8)
    w.raw('31c0fcf348ab')
    w.raw('415f415e415d415c5d5b')
    return w.finish()
