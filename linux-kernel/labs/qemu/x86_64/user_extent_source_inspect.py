"""Observe source-built ordinary user requests and whole-process retirement.

TOPAL-SYSTEMS-USER-EXTENT-001. The debugger observes actual calls and returns.
It never assigns guest registers, mappings, ownership records or user data.
"""
import hashlib
import json
import os
from pathlib import Path
import struct
import sys

import frame_root_inspect as harness
from user_extent_shell_observer import PROGRAM as SHELL_PROGRAM
from user_extent_exec_observer import PROGRAM as EXEC_PROGRAM
from user_extent_signal_observer import PROGRAM as SIGNAL_PROGRAM
from user_extent_poll_observer import PROGRAM as POLL_PROGRAM
from user_extent_call_coverage import PROGRAM as COVERAGE_PROGRAM, linked_inventory
from user_extent_pipe_observer import PROGRAM as PIPE_PROGRAM
from user_extent_storage_observer import PROGRAM as STORAGE_PROGRAM
from user_extent_serial_observer import PROGRAM as SERIAL_PROGRAM, DIALOGUE, native_sites as serial_native_sites
from user_extent_stat_observer import PROGRAM as STAT_PROGRAM
from user_extent_metadata_observer import PROGRAM as METADATA_PROGRAM
from user_extent_owner_observer import PROGRAM as OWNER_PROGRAM, MODEL as OWNER_MODEL
from user_extent_interval_observer import PROGRAM as INTERVAL_PROGRAM, MODEL as INTERVAL_MODEL
from user_extent_translation_observer import PROGRAM as TRANSLATION_PROGRAM, MODEL as TRANSLATION_MODEL, TRACE_MODEL as TRANSLATION_TRACE_MODEL
from user_extent_image_observer import PROGRAM as IMAGE_PROGRAM, MODEL as IMAGE_MODEL
from process_mm_preparation_observer import PROGRAM as PROCESS_PREPARATION_PROGRAM, MODEL as PROCESS_PREPARATION_MODEL
from user_extent_application_inspect import invariant_halt, members


PROGRAM = r'''
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
observed = dict(passed=False, executions=[], startup_executions=[], file_executions=[], shell_executions=[], command_executions=[], poll_executions=[], bridge_returns=[])
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
            if translation_profile is not None:
                observed['active_translation_publication']=translation_active
                observed['active_translation_request']=translation_request
                observed['pending_translation_root_switch']=translation_root_switch_pending
            if image_publication_profile is not None:
                observed['active_image_publication']=image_publication_active
            if process_preparation_profile is not None:
                observed['active_process_preparation']=process_preparation_active
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
        if component.get('mode') == 'signals':
            return signal_returned(current,after)
        if component.get('mode') == 'pipes':
            return pipe_returned(current,after)
        if component.get('mode') == 'storage':
            return storage_returned(current,after)
        if component.get('mode') == 'metadata':
            interval_case_completed()
            return metadata_returned(current,after)
        if component.get('mode') == 'serial':
            return serial_returned(current,after)
        if component.get('mode') == 'stat':
            return stat_returned(current,after)
        calls = current['calls']
        expected_numbers = [12,12,63,63,63,63,63,63,63,1,1,1,1]
        original = current['before'][1]
        expected_results = [original,0x63a000,0,0]+[-14]*5+[0,-9,-22,-14]
        if component.get('mode') == 'transfers':
            expected_numbers += [1,1,1,79]
            expected_results += [-14,24,24,len(current['cwd_expected'])//2]
            assert [row['args'] for row in calls[13:17]] == [
                [1,0x639fff,2],[1,0x5ffff8,24],[1,0x638ff8,24],[0x638fff,5,0]]
            assert word(word(context+928)+40) == current['cwd_before']
            assert calls[16]['output_hex'] == current['cwd_expected']
        assert [row['number'] for row in calls] == expected_numbers+[12], calls
        expected_results += [original]
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
        if component.get('mode') == 'transfers':
            current['cwd_before'] = word(word(context+928)+40)
            current['cwd_expected'] = {3:b'/\0',4:b'/bin\0',6:b'/tmp\0'}[current['cwd_before']].hex()
        if component.get('mode') == 'signals':
            current['table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
        if component.get('mode') == 'pipes':
            current['pipe_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['_pipe_initial'] = memory(word(context+928)+PIPE_BASE,PIPE_BYTES).hex()
        if component.get('mode') == 'storage':
            current['storage_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['_storage_initial'] = memory(word(context+928)+STORAGE_BASE,STORAGE_BYTES).hex()
        if component.get('mode') == 'metadata':
            metadata_begin(current)
        if component.get('mode') == 'serial':
            current['serial_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['serial_received']=[]
            current['serial_markers']=0
        if component.get('mode') == 'stat':
            current['stat_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
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
        copy_source_syscall_returned(pending)
        staged_source_syscall_returned(pending)
        pipe_fixed_syscall_returned(pending)
        pipe_loop_syscall_returned(pending)
        interval_syscall_returned(pending)
        if current and component.get('mode') == 'transfers' and pending['number'] == 79:
            expected = bytes.fromhex(current['cwd_expected'])
            assert pending['result'] == len(expected), pending
            pending['output_hex'] = memory(pending['args'][0],len(expected)).hex()
            assert pending['output_hex'] == current['cwd_expected'], pending
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_result(pending)
        if current and component.get('mode') == 'signals' and pending['number'] in [13,14]:
            observe_signal_result(pending)
        if current and component.get('mode') == 'pipes' and pending['number'] != 12:
            pipe_result(pending)
        if current and component.get('mode') == 'storage' and pending['number'] != 12:
            storage_result(pending)
        if current and component.get('mode') == 'metadata' and pending['number'] != 12:
            metadata_result(pending)
        if current and component.get('mode') == 'serial' and pending['number'] != 12:
            serial_result(pending)
        if current and component.get('mode') == 'stat' and pending['number'] in [5,262]:
            stat_result(pending)
        if startup_current and startup_current.get('kind') == 'poll':
            observe_poll_result(pending)
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
        copy_source_syscall_entered(pending)
        staged_source_syscall_entered(pending)
        pipe_fixed_syscall_entered(pending)
        pipe_loop_syscall_entered(pending)
        interval_syscall_entered(pending)
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_request(pending)
        if current and component.get('mode') == 'signals' and pending['number'] in [13,14]:
            observe_signal_request(pending)
        if current and component.get('mode') == 'pipes' and pending['number'] != 12:
            pipe_request(pending)
        if current and component.get('mode') == 'storage' and pending['number'] != 12:
            storage_request(pending)
        if current and component.get('mode') == 'metadata' and pending['number'] != 12:
            metadata_request(pending)
        if current and component.get('mode') == 'serial' and pending['number'] != 12:
            serial_request(pending)
        if current and component.get('mode') == 'stat' and pending['number'] in [5,262]:
            stat_request(pending)
        if startup_current and startup_current.get('kind') == 'poll':
            observe_poll_request(pending)
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
        if row and row.get('kind') == 'poll':
            return poll_returned(row)
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
        if component.get('mode') == 'poll':
            return poll_entered(n,args,stack,return_site)
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

# SHELL_OBSERVER

# EXEC_OBSERVER

# SIGNAL_OBSERVER

# POLL_OBSERVER

# EXTENT_COVERAGE

# PIPE_OBSERVER

# STORAGE_OBSERVER

# SERIAL_OBSERVER

# STAT_OBSERVER

# METADATA_OBSERVER

# OWNER_OBSERVER

# INTERVAL_OBSERVER
# TRANSLATION_OBSERVER
# IMAGE_PUBLICATION_OBSERVER
# PROCESS_PREPARATION_OBSERVER

class Complete(Checked):
    def observe(self):
        assert current is startup_current is pending is bridge_current is None and observed['executions']
        numbers = [row['number'] for row in observed['startup_executions']]
        if component.get('mode') in ['exec','signals','poll','pipes','storage','serial','stat','metadata']:
            assert len(observed['executions']) == 4
            assert all(not observed[key] for key in [
                'startup_executions','file_executions','shell_executions','command_executions'])
        else:
            assert numbers and len(numbers) % 5 == 0
        assert numbers == [302,318,318,157,262]*(len(numbers)//5)
        assert [row['number'] for row in observed['file_executions']] == ([] if component.get('mode') in ['exec','signals','poll','pipes','storage','serial','stat','metadata'] else [
            5,257,262,257,5,0,257,5,217,217,262,257,5,217,217,262,262,262,257,257]
        )
        assert len(observed['command_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes','storage','serial','stat','metadata'] else 5)
        assert len(observed['shell_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes','storage','serial','stat','metadata'] else 38)
        assert len(observed['poll_executions']) == (4 if component.get('mode') == 'poll' else 0)
        assert len(observed['bridge_returns']) == len(numbers)+len(observed['file_executions'])+len(observed['shell_executions'])+len(observed['command_executions'])+len(observed['poll_executions'])
        assert state() == [0,0,0,0]
        assert memory(address('topal_x86_frame_mm_tokens_v1'), 48) == bytes(48)
        assert word(context+904) == 0 and memory(context+152,16) == bytes(16)
        assert memory(context+416,8) == bytes(8)
        for snapshot in {row['snapshot'] for row in observed['executions']}:
            assert memory(snapshot+0x9e00,512) == bytes(512)
        extent_coverage_complete()
        owner_observation_complete()
        interval_observation_complete()
        translation_observation_complete()
        image_publication_observation_complete()
        process_preparation_observation_complete()
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
    install_extent_coverage()
    install_serial_observation()
    install_storage_input_observation()
    install_metadata_observation()
    install_owner_observation()
    install_interval_observation()
    install_translation_observation()
    install_image_publication_observation()
    install_process_preparation_observation()
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
'''

assert PROGRAM.count('# SHELL_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# SHELL_OBSERVER', SHELL_PROGRAM)
assert PROGRAM.count('# EXEC_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# EXEC_OBSERVER', EXEC_PROGRAM)
assert PROGRAM.count('# SIGNAL_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# SIGNAL_OBSERVER', SIGNAL_PROGRAM)
assert PROGRAM.count('# POLL_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# POLL_OBSERVER', POLL_PROGRAM)
assert PROGRAM.count('# EXTENT_COVERAGE') == 1
PROGRAM = PROGRAM.replace('# EXTENT_COVERAGE', COVERAGE_PROGRAM)
assert PROGRAM.count('# PIPE_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# PIPE_OBSERVER', PIPE_PROGRAM)
assert PROGRAM.count('# STORAGE_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# STORAGE_OBSERVER', STORAGE_PROGRAM)
assert PROGRAM.count('# SERIAL_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# SERIAL_OBSERVER', SERIAL_PROGRAM)
assert PROGRAM.count('# STAT_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# STAT_OBSERVER', STAT_PROGRAM)
assert PROGRAM.count('# METADATA_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# METADATA_OBSERVER', METADATA_PROGRAM)
assert PROGRAM.count('# OWNER_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# OWNER_OBSERVER', OWNER_PROGRAM)
assert PROGRAM.count('# INTERVAL_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# INTERVAL_OBSERVER', INTERVAL_PROGRAM)
assert PROGRAM.count('# TRANSLATION_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# TRANSLATION_OBSERVER', TRANSLATION_PROGRAM)
assert PROGRAM.count('# IMAGE_PUBLICATION_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# IMAGE_PUBLICATION_OBSERVER', IMAGE_PROGRAM)
assert PROGRAM.count('# PROCESS_PREPARATION_OBSERVER') == 1
PROGRAM = PROGRAM.replace('# PROCESS_PREPARATION_OBSERVER', PROCESS_PREPARATION_PROGRAM)


if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    component = json.loads((build / 'source-component.json').read_text())
    process_preparation_profile = None
    preparation_setting = os.environ.get('TOPAL_TEST_PROCESS_MM_PREPARATION')
    assert preparation_setting in [None,'metadata']
    if preparation_setting is not None:
        assert component['mode'] == preparation_setting
        kernel = (build / 'kernel.elf').read_bytes()
        phoff = struct.unpack_from('<Q',kernel,32)[0]
        phsize,phcount = struct.unpack_from('<HH',kernel,54)
        spans=[]
        for index in range(phcount):
            kind,flags,offset,virtual,physical,filesize,memsize,alignment=struct.unpack_from('<IIQQQQQQ',kernel,phoff+index*phsize)
            if kind==1:
                assert virtual==physical and physical+memsize<=1<<30
                spans.append([physical,physical+memsize])
        assert spans
        process_preparation_profile=dict(reserved_spans=spans,
            bindings=json.loads((build/'user-owners.json').read_text()))
    owner_profile = None
    image_publication_profile = None
    image_setting = os.environ.get('TOPAL_TEST_IMAGE_PUBLICATION')
    assert image_setting in [None,'metadata']
    assert image_setting is None or os.environ.get('TOPAL_TEST_EXTENT_OWNERS')=='metadata'
    if image_setting is not None:
        assert component['mode'] == image_setting
        sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'tools/user-memory'))
        from inspect_image_publication import inspect as inspect_image
        image_publication_profile = inspect_image(build)
    translation_profile = None
    translation_setting = os.environ.get('TOPAL_TEST_EXTENT_TRANSLATIONS')
    assert translation_setting in [None,'metadata']
    if translation_setting is not None:
        assert component['mode'] == translation_setting
        sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'tools/user-memory'))
        from inspect_translation_publication import inspect as inspect_translation
        translation_profile = inspect_translation(build)
    interval_profile = os.environ.get('TOPAL_TEST_EXTENT_INTERVALS')
    assert interval_profile in [None,'metadata']
    assert interval_profile is None or os.environ.get('TOPAL_TEST_EXTENT_OWNERS')=='metadata'
    if os.environ.get('TOPAL_TEST_EXTENT_OWNERS'):
        assert os.environ['TOPAL_TEST_EXTENT_OWNERS'] == component['mode'] == 'metadata'
        kernel = (build / 'kernel.elf').read_bytes()
        phoff = struct.unpack_from('<Q', kernel, 32)[0]
        phsize, phcount = struct.unpack_from('<HH', kernel, 54)
        assert kernel[:6] == b'\x7fELF\x02\x01' and phsize == 56
        spans = []
        for i in range(phcount):
            kind, flags, offset, virtual, physical, size, memsize, align = struct.unpack_from('<II6Q',kernel,phoff+i*phsize)
            if kind == 1:
                assert virtual == physical and memsize and physical+memsize <= 1<<30
                spans.append([physical,physical+memsize])
        assert spans
        owner_profile = dict(bindings=json.loads((build / 'user-owners.json').read_text()),
            kernel_spans=spans,model=OWNER_MODEL,kernel_sha256=hashlib.sha256(kernel).hexdigest())
    image = (build / 'static-elf-abi-fixture.elf').read_bytes()
    assert hashlib.sha256(image).hexdigest() == component['image_sha256']
    archive = members((build / 'initramfs.cpio').read_bytes())
    assert archive['static-user.elf'] == image
    extent_inventory = linked_inventory(build) if component.get('mode') in ['transfers','poll','pipes','storage','serial','stat','metadata'] else None
    if component.get('mode') == 'transfers':
        extent_inventory['native_observers'] = ['staged','copies']
        extent_inventory['source_scoped_staged'] = True
        extent_inventory['source_scoped_copies'] = True
    if component.get('mode') == 'stat':
        assert extent_inventory.get('stat_output_audit') is not None
        extent_inventory['source_scoped_copies'] = True
    if component.get('mode') == 'pipes' and extent_inventory.get('pipe_fixed_output_audit') is not None:
        extent_inventory['native_observers'] = ['callbacks','copies','staged','pipe-fixed']
        if extent_inventory.get('pipe_loop_audit') is not None:
            extent_inventory['native_observers'].append('pipe-loops')
    serial_sites=serial_native_sites(build) if component.get('mode')=='serial' else None
    harness.GDB_PROGRAM = PROGRAM
    harness.main(
        work_name='source-user-extents',
        program_prefix=(f'component={component!r}\n'
                        f'process_preparation_profile={process_preparation_profile!r}\n'
                        f'process_preparation_model_source={PROCESS_PREPARATION_MODEL!r}\n'
                        f'owner_profile={owner_profile!r}\n'
                        f'image_publication_profile={image_publication_profile!r}\n'
                        f'image_publication_model_source={IMAGE_MODEL!r}\n'
                        f'interval_profile={interval_profile!r}\n'
                        f'translation_profile={translation_profile!r}\n'
                        f'translation_model_source={TRANSLATION_MODEL!r}\n'
                        f'translation_trace_model_source={TRANSLATION_TRACE_MODEL!r}\n'
                        f'interval_model_sha256={hashlib.sha256(INTERVAL_MODEL.encode()).hexdigest()!r}\n'
                        f'immutable_head={image[:256].hex()!r}\n'
                        f'serial_sites={serial_sites!r}\n'
                        f'serial_patterns={[(m.hex(),p.hex()) for m,p in DIALOGUE]!r}\n'
                        f'extent_inventory={extent_inventory!r}\n'
                        f'native_sites={invariant_halt(build / "kernel.elf")!r}\n'),
        input_scope='source-built ordinary user-buffer boundary cases',
        # The additional read-only stack breakpoints consume debugger wall time.
        # Keep a finite deadline and the same guest/host resource boundaries.
        guest_timeout_seconds=(600 if component.get('mode')=='metadata' else
                               480 if component.get('mode')=='transfers' and extent_inventory is not None and
                               extent_inventory.get('stack_retirement_audit') is not None else 300),
        serial_dialogue=DIALOGUE*4 if component.get('mode')=='serial' else (),
        cpu_trace_sites=tuple(sorted(set(
            ([translation_profile['sites']['invalidated']] if translation_profile is not None else [])+
            ([p['sites']['invalidated'] for p in image_publication_profile['profiles'].values()] if image_publication_profile is not None else [])))),
    )
