"""Observe ordinary FIFO requests, refusal state and consumed ring erasure."""

PROGRAM = r'''
PIPE_BASE=0x3e000
PIPE_BYTES=0x5000

def pipe_context():
    return (memory(context+152,104)+memory(context+416,8)+memory(context+736,160)+
            memory(word(context+904)+0x9e00,512)).hex()

def pipe_visible(pointer,size):
    for start,end in [(component['base'],component['base']+component['text_bytes']),
                      (0x5f7000,0x63a000),(0x400000,0x401000)]:
        if start<=pointer<end:
            return memory(pointer,min(size,end-pointer))
    return b''

def pipe_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['_table_before']=memory(table,8192).hex()
    call['_pipe_before']=memory(table+PIPE_BASE,PIPE_BYTES).hex()
    call['context_before']=pipe_context()
    n=call['number']
    args=call['args']
    if n in [22,293]: pointer,size=args[0],8
    elif n==16: pointer,size=args[2],4
    else: pointer,size=args[1],min(args[2],4096)
    if n!=3:
        call['buffer_before']=pipe_visible(pointer,size).hex()
        call['buffer_pointer']=pointer
        call['window_pointer']=pointer-1
        call['window_before']=pipe_visible(pointer-1,size+2).hex()
    if n in [0,1,3,16]:
        description=word(table+64+args[0]*16)
        assert 1<=description<=16
        call['description_index']=description
        call['description_before']=words(table+320+(description-1)*64,8)

def pipe_result(call):
    table=word(context+928)
    before_table=bytearray.fromhex(call.pop('_table_before'))
    before=bytearray.fromhex(call.pop('_pipe_before'))
    after=memory(table+PIPE_BASE,PIPE_BYTES)
    n,result=call['number'],call['result']
    args=call['args']
    expected_table=bytearray(before_table)
    expected=bytearray(before)
    output=b''
    if result<0:
        assert result in [-14,-11]
        if n!=3:
            assert pipe_visible(call['buffer_pointer'],len(bytes.fromhex(call['buffer_before']))).hex()==call['buffer_before']
    elif n in [22,293]:
        assert result==0
        free=[fd for fd in range(16) if struct.unpack_from('<Q',before_table,64+fd*16)[0]==0]
        pair=list(struct.unpack('<II',memory(args[0],8)))
        assert pair==free[:2]
        generation=struct.unpack_from('<Q',before,512)[0]+1
        index=next(i for i in range(4) if before[i*128:i*128+128]==bytes(128))
        current['pipe_index']=index
        call['pair']=pair
        call['generation']=generation
        call['object_index']=index
        descs=[struct.unpack_from('<Q',memory(table,8192),64+fd*16)[0] for fd in pair]
        assert descs[0]!=descs[1] and all(1<=d<=16 for d in descs)
        for endpoint,(fd,description) in enumerate(zip(pair,descs)):
            assert before_table[320+(description-1)*64:320+description*64]==bytes(64)
            struct.pack_into('<QQ',expected_table,64+fd*16,description,0)
            struct.pack_into('<8Q',expected_table,320+(description-1)*64,
                             5+endpoint,1,0x800|endpoint,0,generation,0,0,0)
        record=list(struct.unpack_from('<16Q',after,index*128))
        assert record[:9]==[1,generation,*descs,0,0,0,0,0o10600]
        assert record[9]==record[10]==record[11]==record[12]>0 and record[13:]==[0]*3
        expected[index*128:index*128+128]=after[index*128:index*128+128]
        struct.pack_into('<Q',expected,512,generation)
        output=struct.pack('<II',*pair)
    else:
        index=current['pipe_index']
        at=index*128
        ring=0x1000+index*4096
        cursor,count=struct.unpack_from('<QQ',before,at+32)
        if n==16:
            assert result==0 and args[1]==0x541b
            output=struct.pack('<I',count)
            assert memory(args[2],4)==output
        elif n in [0,1] and result>0:
            assert result<=args[2]
            if n==0:
                assert result==min(args[2],count)
                output=bytes(before[ring+(cursor+i)%4096] for i in range(result))
                assert memory(args[1],result)==output
                for i in range(result): expected[ring+(cursor+i)%4096]=0
                struct.pack_into('<QQ',expected,at+32,(cursor+result)%4096,count-result)
                timestamp_offsets=[72]
                call['output_hex']=output.hex()
            else:
                assert result==min(args[2],4096-count)
                data=bytes.fromhex(call['buffer_before'])
                assert len(data)>=result
                for i in range(result): expected[ring+(cursor+count+i)%4096]=data[i]
                struct.pack_into('<Q',expected,at+40,count+result)
                timestamp_offsets=[80,88]
            for offset in timestamp_offsets:
                assert struct.unpack_from('<Q',after,at+offset)[0]>0
                expected[at+offset:at+offset+8]=after[at+offset:at+offset+8]
        elif n==3:
            assert result==0 and call['description_before'][1]==1
            description=call['description_index']
            expected_table[64+args[0]*16:80+args[0]*16]=bytes(16)
            expected_table[320+(description-1)*64:320+description*64]=bytes(64)
            offset=16 if call['description_before'][0]==5 else 24
            struct.pack_into('<Q',expected,at+offset,0)
            if struct.unpack_from('<QQ',expected,at+16)==(0,0):
                expected[at:at+128]=bytes(128)
                expected[ring:ring+4096]=bytes(4096)
        else:
            assert result==0 and ((n in [0,1] and args[2]==0) or (n==0 and count==0))
    assert after==bytes(expected), (n,result,'FIFO bytes changed outside permitted transition')
    assert memory(table,8192)==bytes(expected_table), (n,result,'descriptor/staging change')
    assert pipe_context()==call['context_before']
    if n!=3:
        window=bytearray.fromhex(call['window_before'])
        if output:
            assert 1+len(output)<=len(window)
            window[1:1+len(output)]=output
        assert pipe_visible(call['window_pointer'],len(window))==bytes(window), (n,result,'user copy window')
        call['copy_window_verified']=True
    call['complete_fifo_state_verified']=True
    call['complete_table_verified']=True
    call['pipe_state_after_sha256']=hashlib.sha256(after).hexdigest()

def pipe_returned(row,after):
    global current
    calls=row['calls']
    assert len(calls)==44
    assert {n:sum(c['number']==n for c in calls) for n in [12,293,16,1,0,3]}=={
        12:3,293:6,16:10,1:10,0:13,3:2}
    actions=[c for c in calls if c['number']!=12]
    assert sum(c['result']==-14 for c in actions)==19
    assert sum(c['result']==-11 for c in actions)==1
    assert all(c['complete_fifo_state_verified'] and c['complete_table_verified'] for c in actions)
    table=word(context+928)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==row['pipe_table_before_sha256']
    shared=memory(table+PIPE_BASE,PIPE_BYTES)
    old=bytearray.fromhex(row.pop('_pipe_initial'))
    struct.pack_into('<Q',old,512,struct.unpack_from('<Q',old,512)[0]+1)
    assert shared==bytes(old), 'FIFO object/ring not retired exactly; only generation history may advance'
    row['after']=after
    row['pipe_retirement_verified']=True
    observed['executions'].append(row)
    print('SOURCE_PIPE_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False
'''
