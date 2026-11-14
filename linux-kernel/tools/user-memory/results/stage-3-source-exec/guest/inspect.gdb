set pagination off
set confirm off
set remotetimeout 10
file "/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-exec-2259102-1794394119345857087/kernel.elf"
target remote 127.0.0.1:43705
python
token_bytes=336
observation_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-exec-2259102-1794394119345857087/source-user-extents/observations.json'
component={'base': 5701632, 'command_source_sha256': 'cac5afad81808c0123175a2de8b1e799c70579f9a5ef81445bb3a66433f9e3ef', 'conservative_user_stack_bytes': 1304, 'entry': 5708672, 'exec_source_sha256': '60605da3006e86b576892c157e83031061172466c121dde0a8a81a7977c6a634', 'file_source_sha256': '757a45600b38e0746659781ba6e14aa5a90ec9d378cf364fd555299890ef92fa', 'image_sha256': 'a09cd7d430d5671c5a269bb183e15d54d25e08b51a3c73a46ce51dd688f40a6c', 'linked_sha256': 'ce225f8b24d5f23fcf309ec0331daa1500128641556dba76124cc2be0947a946', 'mode': 'exec', 'scope': 'source-built ordinary syscall component in a freshly generated first-party ABI witness; not an arbitrary-program loader or production qualification', 'shell_source_sha256': '574462c620fb4e2db01bc4140e98377f52d4add8eddc3e4a9fa0428d2a3d8db4', 'source_sha256': '4350abed34fe56b3961d52ec3d73532a03de56320cf08f84d0cac5688538d70b', 'stack_frame_costs': {'command-cases': 104, 'exec-cases': 152, 'file-cases': 400, 'shell-cases': 168, 'source-cases': 496, 'startup-cases': 168}, 'startup': 5702592, 'startup_call': 5711328, 'startup_source_sha256': '2c5171f997c27d084883f6dce969747c191ab41649864e5c1d73f2367bb574aa', 'syscall': 5711280, 'syscall4': 5711296, 'text_bytes': 9893}
native_sites={'halt': 45795189, 'exits': [45689936, 45691580]}

import gdb
import hashlib
import json
import struct
import traceback

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(pointer, size): return bytes(gdb.selected_inferior().read_memory(pointer, size)) if size else b''
def words(pointer, count): return list(struct.unpack('<'+'Q'*count, memory(pointer, count*8)))
def word(pointer): return words(pointer, 1)[0]
def signed(value): return value if value < (1<<63) else value-(1<<64)
def state(): return words(address('topal_x86_frame_mm_state_v1'), 4)
def prefix(pages): return hashlib.sha256(memory(0x5f7000, pages*4096)).hexdigest()
def credentials_pointer():
    return word(context+920)+96 if memory(context+896,1)[0] == 1 else 0x7fff18
def file_controls():
    return (memory(context+472,152)+memory(context+736,160)).hex()

context = address('topal_bootstrap_storage') + 512
current = pending = startup_current = bridge_current = None
component_returns = {}
syscall_returns = {}
observed = dict(passed=False, executions=[], startup_executions=[], file_executions=[], shell_executions=[], command_executions=[], bridge_returns=[])
callee = ['rbx', 'rbp', 'r12', 'r13', 'r14', 'r15']

class Checked(gdb.Breakpoint):
    def stop(self):
        try:
            return self.observe()
        except Exception:
            traceback.print_exc()
            observed['observer_failure'] = traceback.format_exc()
            observed['active_startup'] = startup_current
            observed['active_bridge'] = bridge_current
            with open(observation_path, 'w') as handle:
                json.dump(observed, handle, indent=2)
            return True

class SourceReturned(Checked):
    def observe(self):
        global current
        self.enabled = False
        assert current and pending is None and reg('cs') & 3 == 3
        assert reg('rax') == 0, ('source case failed', reg('rax'), current)
        assert reg('rsp') == current['stack']+8
        assert {name:reg(name) for name in callee} == current['callee']
        after = state()
        assert after[1:3] == current['before'][1:3], (after, current['before'])
        assert prefix(after[2]) == current['prefix_before']
        assert word(context+904) == current['snapshot']
        assert hashlib.sha256(memory(current['snapshot']+0x9e00,512)).hexdigest() == current['command_before']
        if component.get('mode') == 'exec':
            return exec_returned(current,after)
        calls = current['calls']
        assert [row['number'] for row in calls] == [12,12,63,63,63,63,63,63,63,1,1,1,1,12], calls
        original = current['before'][1]
        expected_results = [original,0x63a000,0,0]+[-14]*5+[0,-9,-22,-14,original]
        assert [row['result'] for row in calls] == expected_results, calls
        assert [row['args'][0] for row in calls[2:9]] == [
            0x5ffff8,0x638ff8,0x639fff,0x63a000,(1<<64)-1,0x400000,0x800000]
        assert calls[0]['args'][0] == 0 and calls[1]['args'][0] == 0x63a000
        assert [row['args'] for row in calls[9:13]] == [
            [1,(1<<64)-1,0],[0,(1<<64)-1,0],
            [1,(1<<64)-1,257],[1,(1<<64)-1,1]]
        assert calls[-1]['args'][0] == original
        current['after'] = after
        observed['executions'].append(current)
        print('SOURCE_USER_CASES_PASS', len(observed['executions']), len(calls), flush=True)
        current = None
        return False

class SourceEntered(Checked):
    def observe(self):
        global current
        assert current is None and pending is None
        assert reg('cs') & 3 == 3 and reg('cr0') & (1<<16)
        stack = reg('rsp')
        assert 0x7fe000 <= stack < 0x800000 and stack % 16 == 8
        return_site = word(stack)
        assert 0x4e0000 <= return_site < 0x4e8000
        before = state()
        assert before[1] == 0x5f7000+before[2]*4096
        snapshot = word(context+904)
        assert snapshot
        current = dict(stack=stack, before=before, prefix_before=prefix(before[2]),
                       callee={name:reg(name) for name in callee}, calls=[],
                       snapshot=snapshot,
                       command_before=hashlib.sha256(memory(snapshot+0x9e00,512)).hexdigest())
        if return_site not in component_returns:
            component_returns[return_site] = SourceReturned('*'+hex(return_site), internal=True)
        component_returns[return_site].enabled = True
        return False

class SyscallReturned(Checked):
    def observe(self):
        global pending
        self.enabled = False
        assert (current or startup_current) and pending and reg('cs') & 3 == 3
        pending['result'] = signed(reg('rax'))
        pending['phase_after'] = memory(context+104,1)[0]
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_result(pending)
        if startup_current and startup_current.get('kind') in ['shell','command']:
            observe_shell_result(pending)
        if startup_current and startup_current.get('kind') == 'file':
            n = pending['number']
            result = pending['result']
            if (n in [5,262] and result == 0) or (n == 217 and result > 0):
                pointer = pending['args'][2 if n == 262 else 1]
                pending['output_hex'] = memory(pointer,result if n == 217 else 144).hex()
        if pending['result'] in [-14,-22] or pending['number'] == 12:
            assert pending['phase_after'] == pending['phase_before'], pending
        if startup_current and startup_current.get('kind') == 'file' and pending['result'] == -14:
            assert file_controls() == pending['controls_before'], pending
        (current or startup_current)['calls'].append(pending)
        pending = None
        return False

class SyscallEntered(Checked):
    def __init__(self, specification, count):
        super().__init__(specification, internal=True)
        self.count = count

    def observe(self):
        global pending
        assert (current or startup_current) and pending is None and reg('cs') & 3 == 3
        pending = dict(number=reg('rdi'),
                       args=[reg(name) for name in ['rsi','rdx','rcx','r8'][:self.count]],
                       phase_before=memory(context+104,1)[0])
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_request(pending)
        if startup_current and startup_current.get('kind') in ['shell','command']:
            observe_shell_request(pending)
        if startup_current and startup_current.get('kind') == 'file':
            pending['controls_before'] = file_controls()
            if pending['number'] in [257,262] and pending['args'][1] == 0x637ff8:
                data = memory(0x637ff8,64)
                pending['pathname_hex'] = data[:data.index(0)+1].hex()
        return_site = word(reg('rsp'))
        assert component['base'] <= return_site < component['base']+component['text_bytes']
        if return_site not in syscall_returns:
            syscall_returns[return_site] = SyscallReturned('*'+hex(return_site), internal=True)
        syscall_returns[return_site].enabled = True
        return False

class StartupReturned(Checked):
    def observe(self):
        global startup_current
        self.enabled = False
        row = startup_current
        if row and row.get('kind') in ['shell','command']:
            return shell_returned(row)
        if row and row.get('kind') == 'file':
            return file_returned(row)
        assert row and current is None and pending is None
        assert reg('cs') & 3 == 3 and reg('rsp') == row['stack']+8
        assert {name:reg(name) for name in callee} == row['callee']
        assert credentials_pointer() == row['credentials_pointer']
        assert words(row['credentials_pointer'],8) == row['credentials']
        n = row['number']
        expected = -2 if n == 262 else (8 if n == 318 else 0)
        assert signed(reg('rax')) == expected, ('startup source failed', signed(reg('rax')), row)
        after = state()
        assert after[1:3] == row['before'][1:3]
        assert prefix(after[2]) == row['prefix_before']
        calls = row['calls']
        assert calls[0]['number'] == calls[1]['number'] == calls[-1]['number'] == 12
        assert calls[0]['args'][0] == 0 and calls[1]['args'][0] == 0x63a000
        assert [calls[i]['result'] for i in [0,1,-1]] == [row['before'][1],0x63a000,row['before'][1]]
        assert calls[-1]['args'][0] == row['before'][1]
        assert all(call['number'] == n for call in calls[2:-1])
        if n == 262:
            assert [call['result'] for call in calls[2:-1]] == [-14]*4+[-2]
            assert [call['args'][1] for call in calls[2:-1]] == [
                0x63a000,(1<<64)-1,0x800000,0x639fff,0x638ff8]
            assert calls[-2]['args'][2] == (1<<64)-1
            assert memory(row['destination'],144).hex() == row['output_before']
        else:
            errors = calls[2:-2]
            if n == 318:
                assert [call['result'] for call in errors[:3]] == [0,-22,-22]
                assert [call['args'][:3] for call in errors[:3]] == [
                    [(1<<64)-1,0,1],[(1<<64)-1,8,8],[(1<<64)-1,17,1]]
                errors = errors[3:]
            assert [call['result'] for call in errors] == [-14]*5
            index = 3 if n == 302 else (1 if n == 157 else 0)
            assert [call['args'][index] for call in errors] == [
                0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff]
            assert calls[-2]['args'][index] == (0x5ffff8 if n == 302 else 0x638ff8)
            assert calls[-2]['result'] == expected
            output = memory(row['destination'],8 if n == 318 else 16)
            if n == 302: assert output == struct.pack('<QQ',0x800000,(1<<64)-1)
            if n == 157: assert output == b'busybox'+bytes(9)
            row['output_after'] = output.hex()
        row['after'] = after
        row['phase_after'] = memory(context+104,1)[0]
        expected_phase = {302:6,318:6,157:10,262:12}[n]
        # Exec retains an active descriptor/namespace owner. Its general
        # pathname service runs before the sealed startup fallback and does
        # not advance that fallback's phase.
        if n == 262 and row['table_active']:
            expected_phase = row['phase_before']
        assert row['phase_after'] == expected_phase, row
        observed['startup_executions'].append(row)
        print('SOURCE_STARTUP_CASES_PASS', len(observed['startup_executions']), n, len(calls), flush=True)
        startup_current = None
        return False

class StartupEntered(Checked):
    def observe(self):
        global startup_current
        assert current is startup_current is pending is None
        assert reg('cs') & 3 == 3 and reg('cr0') & (1<<16)
        stack = reg('rsp')
        assert 0x7fe000 <= stack < 0x800000 and stack % 16 == 8
        return_site = word(stack)
        assert component['base'] <= return_site < component['base']+component['text_bytes']
        args = [reg(name) for name in ['rsi','rdx','rcx','r8']]
        n = reg('rdi')
        if (n == 0 and (args[0] == 0 or args[2] == 1)) or n in [1,13,16,63,79,80]:
            return shell_entered(n,args,stack,return_site)
        if n in [5,257,217,0] or (n == 262 and args[1] != 0x4a6900):
            return file_entered(n,args,stack,return_site)
        assert n in [302,318,157,262]
        destination = args[{302:3,318:0,157:1,262:2}[n]]
        before = state()
        startup_current = dict(number=n,args=args,stack=stack,before=before,
            prefix_before=prefix(before[2]),callee={name:reg(name) for name in callee},
            calls=[],destination=destination,
            output_before=memory(destination,144 if n == 262 else (8 if n == 318 else 16)).hex(),
            phase_before=memory(context+104,1)[0],
            table_active=word(word(context+928)) == 1,
            credentials_pointer=credentials_pointer(),credentials=words(credentials_pointer(),8))
        assert startup_current['credentials'] == [11,0,12,0,13,0,14,0]
        assert startup_current['phase_before'] == ({302:5,318:6,157:9,262:11}[n])
        if return_site not in component_returns:
            component_returns[return_site] = StartupReturned('*'+hex(return_site), internal=True)
        component_returns[return_site].enabled = True
        return False

class BridgeReturned(Checked):
    def observe(self):
        global bridge_current
        self.enabled = False
        row = bridge_current
        assert row and current is startup_current is pending is None
        assert reg('rsp') == row['stack']+8 and reg('cs') & 3 == 3
        assert {name:reg(name) for name in row['registers']} == row['registers']
        assert credentials_pointer() == row['credentials_pointer']
        assert words(row['credentials_pointer'],8) == [11,0,12,0,13,0,14,0]
        observed['bridge_returns'].append(row)
        bridge_current = None
        return False

class BridgeEntered(Checked):
    def observe(self):
        global bridge_current
        assert current is startup_current is pending is bridge_current is None
        stack = reg('rsp')
        site = word(stack)
        assert 0x4a6100 <= site < 0x4b0000
        bridge_current = dict(stack=stack,return_site=site,credentials_pointer=credentials_pointer(),
            registers={name:reg(name) for name in callee+['rdi','rsi','rdx','r10','r8','r9']})
        if site not in component_returns:
            component_returns[site] = BridgeReturned('*'+hex(site), internal=True)
        component_returns[site].enabled = True
        return False

def file_entered(n,args,stack,return_site):
    global startup_current
    assert word(word(context+928)) == 0, 'bounded file fixture must precede table activation'
    before = state()
    assert before[1:3] == [0x619000,34]
    destination = args[2] if n == 262 else args[1]
    size = 0 if n == 257 else (512 if n == 217 else (128 if n == 0 else 144))
    path = None
    if n in [257,262]:
        data = memory(args[1],64)
        path = data[:data.index(0)+1].hex()
    startup_current = dict(kind='file',number=n,args=args,stack=stack,before=before,
        prefix_before=prefix(before[2]),callee={name:reg(name) for name in callee},calls=[],
        destination=destination,output_before=memory(destination,size).hex(),
        output_size=size,path=path,credentials_pointer=credentials_pointer(),
        credentials=words(credentials_pointer(),8),phase_before=memory(context+104,1)[0],
        controls_before=file_controls())
    assert startup_current['credentials'] == [11,0,12,0,13,0,14,0]
    if return_site not in component_returns:
        component_returns[return_site] = StartupReturned('*'+hex(return_site), internal=True)
    component_returns[return_site].enabled = True
    return False

def file_returned(row):
    global startup_current
    assert current is None and pending is None and reg('cs') & 3 == 3
    assert reg('rsp') == row['stack']+8
    assert {name:reg(name) for name in callee} == row['callee']
    assert credentials_pointer() == row['credentials_pointer']
    assert words(row['credentials_pointer'],8) == row['credentials']
    after = state()
    assert after[1:3] == row['before'][1:3]
    assert prefix(after[2]) == row['prefix_before']
    n = row['number']
    calls = row['calls']
    assert calls[0]['number'] == calls[1]['number'] == calls[-1]['number'] == 12
    assert [calls[i]['args'][0] for i in [0,1,-1]] == [0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]] == [0x619000,0x63a000,0x619000]
    actions = calls[2:-1]
    assert all(c['number'] == n for c in actions)
    path = bytes.fromhex(row['path']) if row['path'] else None
    if n in [257,262]:
        assert [c['args'][1] for c in actions[:4]] == [0x63a000,(1<<64)-1,0x800000,0x639fff]
        assert [c['result'] for c in actions[:4]] == [-14]*4
        actions = actions[4:]
        assert all(c['args'][1] == 0x637ff8 and c['pathname_hex'] == row['path'] for c in actions)
    if n == 257:
        expected = -2 if path == b'/dev/tty\0' else 3
        assert len(actions) == 1 and actions[0]['result'] == expected
    elif n == 0:
        expected = 0
        assert [c['args'][1] for c in actions] == [(1<<64)-1,(1<<47)-4096-127,0x63a000]
        assert [c['result'] for c in actions] == [-14,-14,0]
    elif n == 262 and path == b'/var/spool/mail/root\0':
        expected = -2
        assert len(actions) == 1 and actions[0]['result'] == -2 and actions[0]['args'][2] == 0x63a000
    elif n == 217 and len(actions) == 1:
        expected = 0
        assert actions[0]['args'][1] == 0x63a000 and actions[0]['result'] == 0
    else:
        expected = actions[-1]['result']
        assert expected in ([72,80] if n == 217 else [0])
        index = 2 if n == 262 else 1
        assert [c['args'][index] for c in actions[:-1]] == [0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff]
        assert [c['result'] for c in actions[:-1]] == [-14]*5
        actual = 0x639fb0 if n == 217 else (0x5ffff8 if n == 5 and row['args'][0] == 1 else 0x638ff8)
        assert actions[-1]['args'][index] == actual
        size = expected if n == 217 else 144
        assert memory(row['destination'],size).hex() == actions[-1]['output_hex']
        before = bytes.fromhex(row['output_before'])
        assert memory(row['destination']+size,len(before)-size) == before[size:]
        row['output_after'] = memory(row['destination'],size).hex()
    assert signed(reg('rax')) == expected, ('file source failed',signed(reg('rax')),row)
    if 'output_after' not in row:
        assert memory(row['destination'],row['output_size']).hex() == row['output_before']
    row['after'] = after
    row['phase_after'] = memory(context+104,1)[0]
    observed['file_executions'].append(row)
    print('SOURCE_FILE_CASES_PASS',len(observed['file_executions']),n,len(calls),flush=True)
    startup_current = None
    return False


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



def exec_protected():
    table = word(context+928)
    assert word(table) == 1
    return dict(table_sha256=hashlib.sha256(memory(table,8192)).hexdigest(),
        registrations=memory(context+152,104).hex(),
        shell=memory(context+416,8).hex(),files=memory(context+736,160).hex(),
        command=memory(word(context+904)+0x9e00,512).hex(),
        snapshot=word(context+904),auxiliary=word(context+920))

def observe_exec_request(call):
    call['protected_before'] = exec_protected()
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    name,argv,envp = call['args']
    def candidate(address,size):
        # Read only the candidate's actual mapped portion, including code data.
        if component['base'] <= address < component['base']+component['text_bytes']:
            return memory(address,min(size,component['base']+component['text_bytes']-address))
        if 0x5f7000 <= address < 0x63a000:
            return memory(address,min(size,0x63a000-address))
        return b''
    call['pathname_prefix_hex'] = candidate(name,256).hex()
    call['argv_prefix_hex'] = candidate(argv,72).hex()
    call['environment_prefix_hex'] = candidate(envp,136).hex()

def observe_exec_result(call):
    assert call['result'] in [-14,-2,-36,-22,-7],call
    assert exec_protected() == call['protected_before'],call
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    call['private_staging_zero'] = True
    call['protected_state_unchanged'] = True

def exec_returned(row,after):
    global current
    calls = row['calls']
    assert [c['number'] for c in calls] == [12,12]+[59]*29+[12]
    assert [calls[i]['args'][0] for i in [0,1,-1]] == [0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]] == [0x619000,0x63a000,0x619000]
    actions=calls[2:-1]
    expected = [-14]*5+[-2,-36]+[-14]*10+[-22,-7,-7]+[-14]*6+[-7]*3
    assert len(expected)==29
    assert [c['result'] for c in actions] == expected
    assert all(c['private_staging_zero'] and c['protected_state_unchanged'] for c in actions)
    assert [c['args'][0] for c in actions[:5]] == [0,0x63a000,(1<<64)-1,0x800000,0x639fff]
    assert [c['args'][1] for c in actions[7:12]] == [0,0x63a000,(1<<64)-4,0x800000,0x639ffc]
    assert actions[12]['args'][1] == 0x5ffffc
    assert actions[13]['args'][:2] == [0x638ff8,0x637ffc]
    assert component['base'] <= actions[14]['args'][1] < component['base']+component['text_bytes']
    assert actions[16]['args'][1] == 0x639ff0
    assert actions[23]['args'][2] == 0x639ffc
    assert actions[25]['args'][2] == 0x639ff0
    row['after'] = after
    observed['executions'].append(row)
    print('SOURCE_EXEC_CASES_PASS',len(observed['executions']),len(actions),flush=True)
    current = None
    return False


class Complete(Checked):
    def observe(self):
        assert current is startup_current is pending is bridge_current is None and observed['executions']
        numbers = [row['number'] for row in observed['startup_executions']]
        if component.get('mode') == 'exec':
            assert len(observed['executions']) == 4
            assert all(not observed[key] for key in [
                'startup_executions','file_executions','shell_executions','command_executions','bridge_returns'])
        else:
            assert numbers and len(numbers) % 5 == 0
        assert numbers == [302,318,318,157,262]*(len(numbers)//5)
        assert [row['number'] for row in observed['file_executions']] == ([] if component.get('mode') == 'exec' else [
            5,257,262,257,5,0,257,5,217,217,262,257,5,217,217,262,262,262,257,257]
        )
        assert len(observed['command_executions']) == (0 if component.get('mode') == 'exec' else 5)
        assert len(observed['shell_executions']) == (0 if component.get('mode') == 'exec' else 38)
        assert len(observed['bridge_returns']) == len(numbers)+len(observed['file_executions'])+len(observed['shell_executions'])+len(observed['command_executions'])
        assert state() == [0,0,0,0]
        assert memory(address('topal_x86_frame_mm_tokens_v1'), 48) == bytes(48)
        assert word(context+904) == 0 and memory(context+152,16) == bytes(16)
        assert memory(context+416,8) == bytes(8)
        for snapshot in {row['snapshot'] for row in observed['executions']}:
            assert memory(snapshot+0x9e00,512) == bytes(512)
        observed['passed'] = True
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

class InvariantHalt(Checked):
    def observe(self):
        observed['invariant_failure'] = dict(phase=memory(context+104,1)[0],
            heap=state(),current=current,pending=pending)
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        print('SOURCE_USER_INVARIANT_FAILURE', observed['invariant_failure'], flush=True)
        return True

try:
    SourceEntered('*'+hex(component['entry']), internal=True)
    StartupEntered('*'+hex(component['startup']), internal=True)
    BridgeEntered('*'+hex(component['startup_call']), internal=True)
    SyscallEntered('*'+hex(component['syscall']), 3)
    SyscallEntered('*'+hex(component['syscall4']), 4)
    InvariantHalt('*'+hex(native_sites['halt']), internal=True)
    Complete('*topal_x86_systems_fatal', internal=True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    gdb.execute('quit 1')

end
