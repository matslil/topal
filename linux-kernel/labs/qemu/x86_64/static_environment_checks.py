"""Independent encoding of exec envp boundaries and above-entry payloads."""
PRE_ENTRY, POST_ENTRY, TEMPLATE = 0x4C4000, 0x4C8000, 0x4CC000
SOURCE, VECTOR, BAD, BAD_VECTOR = 0x607000, 0x607400, 0x608000, 0x608800
ENTRY, ENVIRONMENT = 0x7FF800, 0x7FFB20
MAX=(1<<64)-1


def entries():
    return [b'TOPAL_TOKEN=alpha',b'TOPAL_TOKEN=beta']+[b'']*13+[b'LONG='+b'x'*970]


def environment_bytes():
    value=b''.join(entry+b'\0' for entry in entries())
    assert len(value)==1024
    return value


def fill(w,address,count,value):
    w.imm('48bf',address)
    w.imm('48b9',count)
    w.imm('48b8',value)
    w.raw('fcf3aa')


def exec_error(w,environment,errno):
    w.call(59,(0x5F9000,0x5F9300,environment),-errno)
    w.call(8,(4,0,1),7)
    w.call(72,(10,1,0),1)


def pre_code(witness_type,mode):
    w=witness_type()
    w.imm('48b8',0x4D0000)
    w.raw('ffd0')
    for pointer in (MAX-3,0x7FFFFC):
        exec_error(w,pointer,14)
    first=int.from_bytes(b'A=one\0\0\0','little')
    w.store(BAD,first)
    w.store(BAD_VECTOR,BAD)
    w.store(BAD_VECTOR+8,0x619000)
    w.store(BAD_VECTOR+16,0)
    exec_error(w,BAD_VECTOR,14)
    w.memory(BAD,first)
    for index in range(16):
        w.store(BAD_VECTOR+8*index,BAD)
    w.store(BAD_VECTOR+128,MAX)
    exec_error(w,BAD_VECTOR,7)
    fill(w,BAD,1024,ord('x'))
    w.store(BAD+1024,0)
    w.store(BAD_VECTOR,BAD)
    w.store(BAD_VECTOR+8,0)
    exec_error(w,BAD_VECTOR,7)
    fill(w,BAD,512,ord('a'))
    w.store(BAD+512,0)
    fill(w,BAD+0x300,512,ord('b'))
    w.store(BAD+0x500,0)
    w.store(BAD_VECTOR,BAD)
    w.store(BAD_VECTOR+8,BAD+0x300)
    w.store(BAD_VECTOR+16,0)
    exec_error(w,BAD_VECTOR,7)
    w.memory(BAD,int.from_bytes(b'a'*8,'little'))
    if mode=='maximum':
        data=environment_bytes()
        for index in range(0,len(data),8):
            w.store(SOURCE+index,int.from_bytes(data[index:index+8],'little'))
        offset=0
        for index,value in enumerate(entries()):
            w.store(VECTOR+index*8,SOURCE+offset)
            offset+=len(value)+1
        w.store(VECTOR+128,0)
    else:
        assert mode in ('empty','null')
        w.store(VECTOR,0)
    return w.finish()


def payloads(w,count):
    auxiliary=ENTRY+80+(count+1)*8
    for address,value in (
        (ENTRY+80+count*8,0), (auxiliary,6),(auxiliary+8,4096),
        (auxiliary+80,31),(auxiliary+88,0x7FFF40),
        (auxiliary+12*16+8,0x7FFF20),(auxiliary+17*16+8,0x7FFF30),
        (auxiliary+18*16,0),(auxiliary+18*16+8,0),
        (0x7FFF40,int.from_bytes(b'/bin/bus','little')),
        (0x7FFF48,int.from_bytes(b'ybox\0\0\0\0','little')),
        (0x7FFF20,int.from_bytes(b'x86_64\0\0','little')),
    ):
        w.memory(address,value)
    w.imm('48be',0x7FFA20)
    w.imm('48bf',0x4AB000)
    w.imm('48b9',256)
    w.raw('fcf3a675fe')
    if count:
        w.imm('48be',ENVIRONMENT)
        w.imm('48bf',TEMPLATE)
        w.imm('48b9',1024)
        w.raw('fcf3a675fe')


def post_code(witness_type,mode):
    w=witness_type()
    w.imm('48b8',0x4D0000)
    w.raw('ffd0')
    count=16 if mode=='maximum' else 0
    if count:
        w.raw('440fb63c25')
        w.code.extend((ENVIRONMENT+1022).to_bytes(4,'little'))
        w.raw('4983ff787302ebfe4983ff7a7602ebfec60425')
        w.code.extend((ENVIRONMENT+1022).to_bytes(4,'little'))
        w.raw('78')
        offset=0
        for index,value in enumerate(entries()):
            w.memory(ENTRY+80+index*8,ENVIRONMENT+offset)
            offset+=len(value)+1
    payloads(w,count)
    w.raw('4881ec001000004889e7b900100000b05afcf3aa4881c400100000')
    payloads(w,count)
    if count:
        repeated_generation(w)
    w.imm("48b8",0x4F0000)
    w.raw("ffd0")
    w.imm("48b8",0x4F8000)
    w.raw("ffd0")
    return w.finish()


def repeated_generation(w):
    action,old=0x7FE900,0x7FE940
    w.call(72,(10,1,0),-9)
    w.call(8,(4,0,1),7)
    w.store(0x7FE400,0)
    w.call(79,(0x7FE400,64),5)
    w.memory(0x7FE400,int.from_bytes(b'/bin\0\0\0\0','little'))
    for signal,handler in ((10,0),(12,1)):
        w.call(13,(signal,0,old,8),0)
        for offset,value in ((0,handler),(8,0),(16,0),(24,0)):
            w.memory(old+offset,value)
    w.call(14,(99,0,old,8),0)
    w.memory(old,64)
    w.call(59,(0x7FFF40,ENTRY+8,MAX),-14)
    w.call(8,(4,0,1),7)
    w.raw('4983ff7a0f84')
    skip=len(w.code)
    w.raw('00000000')
    w.call(72,(4,1030,10),10)
    for offset,value in ((0,0x12345000),(8,MAX),(16,0xDECAFBAD),(24,MAX)):
        w.store(action+offset,value)
    w.call(13,(10,action,0,8),0)
    w.raw('41ffc744883c25')
    w.code.extend((ENVIRONMENT+1022).to_bytes(4,'little'))
    for prefix,value in (('48bf',0x7FFF40),('48be',ENTRY+8),('48ba',ENTRY+80),('48b8',59)):
        w.imm(prefix,value)
    w.raw('0f050f0b')
    w.code[skip:skip+4]=(len(w.code)-skip-4).to_bytes(4,'little')
