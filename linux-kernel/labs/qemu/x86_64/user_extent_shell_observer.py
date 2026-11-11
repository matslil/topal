"""Read-only observations for ordinary shell and command source calls."""

PROGRAM = r'''
def protected_controls():
    return (memory(context+152,104)+memory(context+416,8)).hex()+file_controls()
def command_state():
    snapshot = word(context+904)
    assert snapshot
    return memory(snapshot+0x9e00,512).hex()
def masked_prefix(pages,pointer,size):
    data = bytearray(memory(0x5f7000,pages*4096))
    left,right=max(pointer,0x5f7000),min(pointer+size,0x5f7000+len(data))
    if right>left: data[left-0x5f7000:right-0x5f7000] = bytes(right-left)
    return hashlib.sha256(data).hexdigest()
def shell_shape(n,args):
    a,b,c,d=args
    if n==16: return False,0,c
    input = n in [1,80] or (n==13 and b!=0)
    size = 390 if n==63 else (32 if n==13 else (2 if n in [79,80] else c))
    pointer = (b if b else c) if n==13 else (a if n in [63,79,80] else b)
    return input,size,pointer

def shell_entered(n,args,stack,return_site):
    global startup_current
    command = n==0 and args[0]==0
    input,size,pointer = (False,args[2],args[1]) if command else shell_shape(n,args)
    before = state()
    assert before[1:3]==[0x619000,34]
    write_pointer,write_size = (0,0) if input else (pointer,size)
    startup_current = dict(kind='command' if command else 'shell',number=n,args=args,
        stack=stack,before=before,callee={name:reg(name) for name in callee},calls=[],
        input=bool(input),size=size,pointer=pointer,
        original_bytes=memory(pointer,size if size else 1).hex(),
        write_pointer=write_pointer,write_size=write_size,
        prefix_before=masked_prefix(before[2],write_pointer,write_size),
        credentials_pointer=credentials_pointer(),credentials=words(credentials_pointer(),8),
        phase_before=memory(context+104,1)[0],queue_before=command_state())
    assert startup_current['credentials']==[11,0,12,0,13,0,14,0]
    if return_site not in component_returns:
        component_returns[return_site]=StartupReturned('*'+hex(return_site),internal=True)
    component_returns[return_site].enabled=True
    return False

def observe_shell_request(call):
    call['protected_before']=protected_controls()
    call['queue_before']=command_state()
    row=startup_current
    if row['kind']=='shell' and row['input'] and call['number']==row['number']:
        index=1 if call['number'] in [1,13] else 0
        pointer=call['args'][index]
        if pointer in [0x638ff8,0x638fff] or component['base']<=pointer<component['base']+component['text_bytes']:
            call['input_hex']=memory(pointer,row['size']).hex()

def observe_shell_result(call):
    row=startup_current
    call['queue_after']=command_state()
    if call['result']==-14:
        assert protected_controls()==call['protected_before'],call
    if row['kind']=='shell' and not row['input'] and call['number']==row['number'] and call['result']>=0:
        n=call['number']
        if n!=16:
            index=2 if n==13 else (0 if n in [63,79] else 1)
            call['output_hex']=memory(call['args'][index],row['size']).hex()

def shell_returned(row):
    global startup_current
    assert current is None and pending is None and reg('cs')&3==3
    assert reg('rsp')==row['stack']+8
    assert {name:reg(name) for name in callee}==row['callee']
    assert credentials_pointer()==row['credentials_pointer']
    assert words(row['credentials_pointer'],8)==row['credentials']
    after=state()
    assert after[1:3]==row['before'][1:3]
    assert masked_prefix(after[2],row['write_pointer'],row['write_size'])==row['prefix_before']
    calls=row['calls']
    assert [calls[i]['number'] for i in [0,1,-1]]==[12,12,12]
    assert [calls[i]['args'][0] for i in [0,1,-1]]==[0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    if row['kind']=='command':
        command_returned(row)
    else:
        n,args,size,input=row['number'],row['args'],row['size'],row['input']
        actions=calls[2:-1]
        assert all(call['number']==n for call in actions)
        if n==16:
            expected=-25
            assert len(actions)==1 and actions[0]['args'][2]==(1<<64)-1
            assert memory(row['pointer'],1).hex()==row['original_bytes']
        else:
            index=1 if n in [1,13,0] and (n!=13 or input) else (2 if n==13 else 0)
            bad=[0x63a000,(1<<64)-1,0x800000,0x63b000 if size==1 else 0x639fff]
            if not input: bad.append(0x400000)
            assert [call['args'][index] for call in actions[:-1]]==bad
            assert [call['result'] for call in actions[:-1]]==[-14]*len(bad)
            expected=size if n in [0,1,79] else 0
            good=actions[-1]
            assert good['result']==expected
            if input:
                assert good['input_hex']==row['original_bytes']
                if n==13: assert component['base']<=good['args'][index]<component['base']+component['text_bytes']
                assert memory(row['pointer'],size).hex()==row['original_bytes']
            else:
                assert memory(row['pointer'],size).hex()==good['output_hex']
            row['output_after']=memory(row['pointer'],size).hex()
        assert signed(reg('rax'))==expected,('shell case failed',signed(reg('rax')),row)
        observed['shell_executions'].append(row)
        print('SOURCE_SHELL_CASES_PASS',len(observed['shell_executions']),n,len(calls),flush=True)
    row['after']=after
    row['phase_after']=memory(context+104,1)[0]
    startup_current=None
    return False

def command_returned(row):
    actions=row['calls'][2:-1]
    assert [call['number'] for call in actions]==[0]*10
    count=row['args'][2]
    assert [call['args'][1] for call in actions]==[
        (1<<64)-1,0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff,
        0x638ff8,(1<<64)-1,0x639fff,0x638ff9]
    assert [call['args'][2] for call in actions]==[0]+[count]*5+[1,0,count,count]
    assert [call['result'] for call in actions[:9]]==[0]+[-14]*5+[1,0,-14]
    rest=actions[-1]['result']
    assert rest>0 and signed(reg('rax'))==rest+1
    assert all(call['phase_after']==call['phase_before'] for call in actions[:-1])
    assert actions[0]['queue_before']==actions[0]['queue_after']
    assert actions[1]['queue_after']==actions[2]['queue_after']==actions[3]['queue_after']==actions[4]['queue_after']==actions[5]['queue_after']
    assert actions[6]['queue_after']==actions[7]['queue_after']==actions[8]['queue_after']
    serial=count==23
    def queue(call):
        data=bytes.fromhex(call['queue_after'])
        return data[:256],struct.unpack_from('<QQQ',data,256)
    if serial:
        queued=queue(actions[1]); short=queue(actions[6]); complete=queue(actions[-1])
        assert queued[1][1]==23 and short[1][1]==22 and complete[1][1]==0
        assert short[0][:22]==queued[0][1:23] and short[0][22:]==bytes(234)
        assert complete[0]==bytes(256)
    else:
        assert queue(actions[1])[1][0]==0 and queue(actions[6])[1][0]==1
        assert queue(actions[-1])[1][0]==0
    before=bytes.fromhex(row['original_bytes'])
    total=rest+1
    assert memory(row['pointer']+total,len(before)-total)==before[total:]
    row['delivered_hex']=memory(row['pointer'],total).hex()
    observed['command_executions'].append(row)
    print('SOURCE_COMMAND_CASES_PASS',len(observed['command_executions']),total,len(row['calls']),flush=True)
'''
