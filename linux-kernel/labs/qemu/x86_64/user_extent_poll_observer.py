"""Observe source-issued bounded descriptor readiness and byte consumption."""

PROGRAM = r'''
def poll_controls():
    return dict(registrations=memory(context+152,104).hex(),header=memory(context+416,8).hex(),
        stat_staging=memory(context+472,152).hex(),file=memory(context+736,160).hex(),queue=command_state())

def poll_bytes(pointer,size):
    if component['base']<=pointer<component['base']+component['text_bytes']:
        return memory(pointer,min(size,component['base']+component['text_bytes']-pointer))
    if 0x5f7000<=pointer<0x63a000:
        return memory(pointer,min(size,0x63a000-pointer))
    if pointer==0x400000: return memory(pointer,size)
    return b''

def poll_entered(n,args,stack,return_site):
    global startup_current
    assert n==0 and args[0]==3 and args[2]==1
    index=len(observed['poll_executions'])
    assert memory(context+416,1)==b'\x01'
    assert memory(context+888,4)==bytes([22+index,index,0,0])
    assert word(word(context+928))==0
    before=state()
    assert before[1:3]==[0x619000,34]
    startup_current=dict(kind='poll',number=n,args=args,stack=stack,before=before,
        index=index,callee={name:reg(name) for name in callee},calls=[],
        prefix_before=prefix(before[2]),credentials_pointer=credentials_pointer(),
        credentials=words(credentials_pointer(),8),controls_before=poll_controls())
    if return_site not in component_returns:
        component_returns[return_site]=StartupReturned('*'+hex(return_site),internal=True)
    component_returns[return_site].enabled=True
    return False

def observe_poll_request(call):
    call['controls_before']=poll_controls()
    n,args=call['number'],call['args']
    if n==7: call['input_hex']=poll_bytes(args[0],8).hex()
    if n==0: call['output_before']=poll_bytes(args[1],1).hex()

def observe_poll_result(call):
    expected=dict(call['controls_before'])
    fields=bytearray.fromhex(expected['file'])
    n,args,result=call['number'],call['args'],call['result']
    if n==7:
        assert args[1:3]==[1,(1<<64)-1]
        if result==1:
            assert struct.unpack('<iH',bytes.fromhex(call['input_hex'])[:6])==(3,1)
            after=memory(args[0],8)
            assert after[:6]==bytes.fromhex(call['input_hex'])[:6] and after[6:]==b'\x01\0'
            call['output_hex']=after.hex()
            fields[891-736]=1
        else:
            assert result==-14
            assert poll_bytes(args[0],8).hex()==call['input_hex']
    elif n==0:
        assert args[0]==3 and args[2]==1
        if result==1:
            assert fields[891-736]==1
            value=bytes([b'\x7fELF'[fields[889-736]]])
            assert memory(args[1],1)==value
            call['output_hex']=value.hex()
            fields[888-736]+=1
            fields[889-736]+=1
            fields[891-736]=0
        else:
            assert result==-14 and poll_bytes(args[1],1).hex()==call['output_before']
    else: assert n==12
    expected['file']=fields.hex()
    actual=poll_controls()
    assert actual==expected,call
    call['controls_after']=actual

def poll_returned(row):
    global startup_current
    assert current is pending is None and signed(reg('rax'))==1
    assert reg('rsp')==row['stack']+8 and reg('cs')&3==3
    assert {name:reg(name) for name in callee}==row['callee']
    assert words(row['credentials_pointer'],8)==row['credentials']
    assert state()[1:3]==row['before'][1:3] and prefix(34)==row['prefix_before']
    calls=row['calls']
    assert [c['number'] for c in calls]==[12,12]+[7]*8+[0]*5+[12]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    assert [c['result'] for c in calls[2:-1]]==[-14]*5+[1,-14,1]+[-14]*4+[1]
    assert memory(row['args'][1],1)==bytes([b'\x7fELF'[row['index']]])
    assert memory(context+888,4)==bytes([23+row['index'],row['index']+1,0,0])
    row['after']=state()
    observed['poll_executions'].append(row)
    print('SOURCE_POLL_CASES_PASS',len(observed['poll_executions']),len(calls),flush=True)
    startup_current=None
    return False
'''
