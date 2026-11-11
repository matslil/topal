"""Read-only file copy/position and sparse-page transition observations."""

PROGRAM = r'''
STORAGE_BASE=0x43000
STORAGE_BYTES=0x9000

def storage_word(data,offset): return struct.unpack_from('<Q',data,offset)[0]
def storage_put(data,offset,value): struct.pack_into('<Q',data,offset,value)

def storage_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['_table_before']=memory(table,8192).hex()
    call['_storage_before']=memory(table+STORAGE_BASE,STORAGE_BYTES).hex()
    call['context_before']=pipe_context()
    n,args=call['number'],call['args']
    if n==257:
        raw=pipe_visible(args[1],64)
        path=raw[:raw.index(0)+1]
        call['path_hex']=path.hex()
        pointer,size=args[1],len(path)
    elif n in [0,1,17,18,217]:
        pointer,size=args[1],min(args[2],48)
    else:
        pointer=size=None
    if pointer is not None:
        call['buffer_before']=pipe_visible(pointer,size).hex()
        call['buffer_pointer']=pointer
        call['window_pointer']=pointer-1
        call['window_before']=pipe_visible(pointer-1,size+2).hex()
    if n!=257:
        index=word(table+64+args[0]*16)
        assert 1<=index<=16
        call['description_index']=index
        call['description_before']=words(table+320+(index-1)*64,8)

def storage_result(call):
    table=word(context+928)
    before_table=bytearray.fromhex(call.pop('_table_before'))
    before=bytearray.fromhex(call.pop('_storage_before'))
    actual=memory(table+STORAGE_BASE,STORAGE_BYTES)
    expected=bytearray(before)
    expected_table=bytearray(before_table)
    n,args,result=call['number'],call['args'],call['result']
    output=b''
    if result<0:
        assert result in [-14,-22]
        if 'buffer_pointer' in call:
            raw=bytes.fromhex(call['buffer_before'])
            assert pipe_visible(call['buffer_pointer'],len(raw))==raw
    elif n==257:
        free=[i for i in range(16) if storage_word(before_table,64+i*16)==0]
        assert result==free[0]
        index=word(table+64+result*16)
        assert 1<=index<=16
        at=320+(index-1)*64
        assert before_table[at:at+64]==bytes(64)
        path=bytes.fromhex(call['path_hex'])
        flags=args[2]
        if flags==0x410002:
            assert path==b'/tmp\0'
            identity=storage_word(before,0)
            slot=next(i for i in range(4) if before[256+i*128:384+i*128]==bytes(128))
            object_at=256+slot*128
            record=list(struct.unpack_from('<16Q',actual,object_at))
            mode=0o100000|(args[3]&0o7777&~storage_word(before_table,1664))
            assert record[:4]==[1,identity,0,mode]
            assert record[4]==record[5]==record[6]>0
            assert record[7:]==[0,1]+[0]*7
            expected[object_at:object_at+128]=actual[object_at:object_at+128]
            storage_put(expected,0,identity+1)
            kind=7
            call['object_identity']=identity
            call['object_slot']=slot
        else:
            assert (path,flags) in [(b'/bin/busybox\0',0),(b'/tmp\0',0x10000)]
            kind,identity=(3,5) if flags==0 else (4,6)
        status=(flags&~(0x80000|0x100|0x40|0x80|0x200))|0x8000
        struct.pack_into('<8Q',expected_table,at,kind,1,status,0,identity,0,0,0)
        struct.pack_into('<QQ',expected_table,64+result*16,index,0)
        call['fresh_description_verified']=True
    else:
        desc=call['description_before']
        desc_at=320+(call['description_index']-1)*64
        kind,_,_,position,identity,*_=desc
        if n==8:
            assert args[2] in [0,1]
            expected_position=args[1]+(position if args[2]==1 else 0)
            assert result==expected_position
            storage_put(expected_table,desc_at+24,result)
        elif n==217:
            assert kind==4 and identity==6 and position==0 and result==48
            records=bytearray(48)
            for i,(inode,name) in enumerate([(6,b'.\0'),(3,b'..\0')]):
                struct.pack_into('<QQHB',records,i*24,inode,i+1,24,4)
                records[i*24+19:i*24+19+len(name)]=name
            output=bytes(records)
            cursor=next((i+2 for i in range(4) if storage_word(before,64+i*16)),6)
            storage_put(expected_table,desc_at+24,cursor)
        elif n==3:
            assert result==0 and desc[1]==1
            expected_table[64+args[0]*16:80+args[0]*16]=bytes(16)
            expected_table[desc_at:desc_at+64]=bytes(64)
            if kind==7:
                object_at=next(256+i*128 for i in range(4)
                    if storage_word(before,256+i*128)==1 and storage_word(before,264+i*128)==identity)
                assert storage_word(before,object_at+56)==0
                for slot in range(8):
                    page=1024+slot*32
                    if storage_word(before,page)==identity:
                        assert before[24+slot]==1
                        expected[24+slot]=0
                        expected[page:page+32]=bytes(32)
                        expected[4096+slot*4096:8192+slot*4096]=bytes(4096)
                expected[object_at:object_at+128]=bytes(128)
        elif n in [0,1,17,18]:
            offset=args[3] if n in [17,18] else position
            if kind==3:
                assert n==17 and result==min(args[2],256,storage_word(before_table,24)-offset)
                output=bytes.fromhex(immutable_head)[offset:offset+result]
                assert len(output)==result
            else:
                assert kind==7
                object_at=next(256+i*128 for i in range(4)
                    if storage_word(before,256+i*128)==1 and storage_word(before,264+i*128)==identity)
                size=storage_word(before,object_at+16)
                writing=n in [1,18]
                if writing:
                    assert result==min(args[2],256,4096-offset%4096)
                else:
                    assert result==min(args[2],256,4096-offset%4096,max(size-offset,0))
                if result:
                    pages=[i for i in range(8) if storage_word(before,1024+i*32)==identity
                           and storage_word(before,1032+i*32)==offset//4096]
                    if writing and not pages:
                        pages=[i for i in range(8) if storage_word(actual,1024+i*32)==identity
                               and storage_word(actual,1032+i*32)==offset//4096]
                        assert len(pages)==1
                        slot=pages[0]
                        assert before[24+slot]==0 and before[1024+slot*32:1056+slot*32]==bytes(32)
                        assert before[4096+slot*4096:8192+slot*4096]==bytes(4096)
                        expected[24+slot]=1
                        struct.pack_into('<4Q',expected,1024+slot*32,identity,offset//4096,0,0)
                    if pages:
                        assert len(pages)==1
                        start=4096+pages[0]*4096+offset%4096
                    if writing:
                        data=bytes.fromhex(call['buffer_before'])[:result]
                        assert len(data)==result
                        expected[start:start+result]=data
                        storage_put(expected,object_at+16,max(size,offset+result))
                        for timestamp in [40,48]:
                            assert storage_word(actual,object_at+timestamp)>0
                            expected[object_at+timestamp:object_at+timestamp+8]=actual[object_at+timestamp:object_at+timestamp+8]
                    else:
                        output=bytes(before[start:start+result]) if pages else bytes(result)
                    if n in [0,1]: storage_put(expected_table,desc_at+24,offset+result)
            if n in [17,18]:
                assert word(table+desc_at+24)==position
                call['positional_position_preserved']=True
        else:
            raise AssertionError(('unexpected storage call',n,result))
    assert actual==bytes(expected), (n,result,'filesystem mutation outside selected contract')
    assert memory(table,8192)==bytes(expected_table), (n,result,'description/staging mutation')
    assert pipe_context()==call['context_before']
    if 'window_before' in call:
        window=bytearray.fromhex(call['window_before'])
        if output:
            assert 1+len(output)<=len(window)
            window[1:1+len(output)]=output
            assert memory(call['buffer_pointer'],len(output))==output
            call['output_hex']=output.hex()
        assert pipe_visible(call['window_pointer'],len(window))==bytes(window), (n,result,'copy window')
        call['copy_window_verified']=True
    call['complete_storage_state_verified']=True
    call['complete_table_verified']=True

def storage_returned(row,after):
    global current
    calls=row['calls']
    assert len(calls)==52
    assert {n:sum(c['number']==n for c in calls) for n in [12,257,1,8,0,17,18,3,217]}=={
        12:3,257:3,1:5,8:5,0:7,17:15,18:5,3:3,217:6}
    actions=[c for c in calls if c['number']!=12]
    assert sum(c['result']==-14 for c in actions)==28
    assert sum(c['result']==-22 for c in actions)==1
    assert all(c['complete_storage_state_verified'] and c['complete_table_verified'] for c in actions)
    table=word(context+928)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==row['storage_table_before_sha256']
    expected=bytearray.fromhex(row.pop('_storage_initial'))
    storage_put(expected,0,storage_word(expected,0)+1)
    assert memory(table+STORAGE_BASE,STORAGE_BYTES)==bytes(expected), 'filesystem reclamation/history'
    row['after']=after
    row['storage_retirement_verified']=True
    observed['executions'].append(row)
    print('SOURCE_STORAGE_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False
'''
