"""Read-only syscall observations checked against the retained-signal contract."""

PROGRAM = r'''
def signal_read(pointer,size):
    if component['base'] <= pointer < component['base']+component['text_bytes']:
        return memory(pointer,min(size,component['base']+component['text_bytes']-pointer))
    if 0x5f7000 <= pointer < 0x63a000:
        return memory(pointer,min(size,0x63a000-pointer))
    if pointer == 0x400000:
        return memory(pointer,size)
    return b''

def signal_writable(pointer,size):
    return 0x5f7000 <= pointer and pointer+size <= 0x63a000

def signal_context():
    return (memory(context+152,104)+memory(context+416,8)+memory(context+736,160)+
            memory(word(context+904)+0x9e00,512)).hex()

def signal_state(table):
    return dict(blocked=struct.unpack_from('<Q',table,32)[0],
        action10=list(struct.unpack_from('<4Q',table,1792+9*32)))

def observe_signal_request(call):
    table=memory(word(context+928),8192)
    assert word(word(context+928)) == 1
    assert table[1408:1664] == bytes(256)
    call['_table_before'] = table.hex()
    call['state_before'] = signal_state(table)
    call['context_before'] = signal_context()
    size=32 if call['number']==13 else 8
    _,new,old,_=call['args']
    call['input_hex'] = signal_read(new,size).hex() if new else ''
    call['old_output_prefix_before'] = signal_read(old,size).hex() if old else ''

def observe_signal_result(call):
    table=bytearray.fromhex(call.pop('_table_before'))
    number=call['number']
    selector,new,old,sigsetsize=call['args']
    size=32 if number==13 else 8
    input_bytes=bytes.fromhex(call['input_hex'])
    expected=0
    output=None
    committed=False
    if sigsetsize!=8: expected=-22
    elif new and len(input_bytes)!=size: expected=-14
    elif number==13:
        if not 1<=selector<=64 or (new and selector in [9,19]): expected=-22
        else:
            at=1792+(selector-1)*32
            output=bytes(table[at:at+32])
            if new:
                handler,flags,restorer,mask=struct.unpack('<4Q',input_bytes)
                table[at:at+32]=struct.pack('<4Q',handler,flags&0xdc000807,restorer,mask&~0x40100)
                committed=True
    else:
        output=bytes(table[32:40])
        if new:
            if selector not in [0,1,2]: expected=-22
            else:
                requested=struct.unpack('<Q',input_bytes)[0]&~0x40100
                previous=struct.unpack('<Q',output)[0]
                value=[previous|requested,previous&~requested,requested][selector]
                table[32:40]=struct.pack('<Q',value)
                committed=True
    if expected==0 and old:
        if signal_writable(old,size):
            assert memory(old,size)==output,call
            call['output_hex']=output.hex()
        else: expected=-14
    assert call['result']==expected,call
    actual=memory(word(context+928),8192)
    assert actual==bytes(table),call
    assert signal_context()==call['context_before'],call
    if expected<0 and old:
        assert signal_read(old,size).hex()==call['old_output_prefix_before'],call
    assert actual[1408:1664]==bytes(256)
    call['state_after']=signal_state(actual)
    call['committed']=committed
    call['complete_table_verified']=True
    call['private_staging_zero']=True

def signal_returned(row,after):
    global current
    calls=row['calls']
    assert [c['number'] for c in calls]==[12,12]+[13]*23+[14]*23+[12]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    actions=calls[2:-1]
    assert all(c['complete_table_verified'] and c['private_staging_zero'] for c in actions)
    assert sum(c['result']==-14 for c in actions)==22
    assert sum(c['result']==-22 for c in actions)==6
    assert sum(c['result']==0 for c in actions)==18
    assert sum(c['committed'] for c in actions)==10
    assert sum(c['committed'] and c['result']==-14 for c in actions)==2
    assert hashlib.sha256(memory(word(context+928),8192)).hexdigest()==row['table_before_sha256']
    row['after']=after
    observed['executions'].append(row)
    print('SOURCE_SIGNAL_CASES_PASS',len(observed['executions']),len(actions),flush=True)
    current=None
    return False
'''
