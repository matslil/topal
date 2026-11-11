"""Observe source-built ordinary user requests and whole-process retirement.

TOPAL-SYSTEMS-USER-EXTENT-001. The debugger observes actual calls and returns.
It never assigns guest registers, mappings, ownership records or user data.
"""
import hashlib
import json
from pathlib import Path
import sys

import frame_root_inspect as harness
from user_extent_application_inspect import invariant_halt, members


PROGRAM = r'''
import gdb
import hashlib
import json
import struct
import traceback

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(pointer, size): return bytes(gdb.selected_inferior().read_memory(pointer, size))
def words(pointer, count): return list(struct.unpack('<'+'Q'*count, memory(pointer, count*8)))
def word(pointer): return words(pointer, 1)[0]
def signed(value): return value if value < (1<<63) else value-(1<<64)
def state(): return words(address('topal_x86_frame_mm_state_v1'), 4)
def prefix(pages): return hashlib.sha256(memory(0x5f7000, pages*4096)).hexdigest()
def credentials_pointer():
    return word(context+920)+96 if memory(context+896,1)[0] == 1 else 0x7fff18

context = address('topal_bootstrap_storage') + 512
current = pending = startup_current = bridge_current = None
component_returns = {}
syscall_returns = {}
observed = dict(passed=False, executions=[], startup_executions=[], bridge_returns=[])
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
        if pending['result'] in [-14,-22] or pending['number'] == 12:
            assert pending['phase_after'] == pending['phase_before'], pending
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
        assert 0x4a6100 <= site < 0x4a6580
        bridge_current = dict(stack=stack,return_site=site,credentials_pointer=credentials_pointer(),
            registers={name:reg(name) for name in callee+['rdi','rsi','rdx','r10','r8','r9']})
        if site not in component_returns:
            component_returns[site] = BridgeReturned('*'+hex(site), internal=True)
        component_returns[site].enabled = True
        return False

class Complete(Checked):
    def observe(self):
        assert current is startup_current is pending is bridge_current is None and observed['executions']
        numbers = [row['number'] for row in observed['startup_executions']]
        assert numbers and len(numbers) % 5 == 0
        assert numbers == [302,318,318,157,262]*(len(numbers)//5)
        assert len(observed['bridge_returns']) == len(numbers)
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
'''


if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    component = json.loads((build / 'source-component.json').read_text())
    image = (build / 'static-elf-abi-fixture.elf').read_bytes()
    assert hashlib.sha256(image).hexdigest() == component['image_sha256']
    archive = members((build / 'initramfs.cpio').read_bytes())
    assert archive['static-user.elf'] == image
    harness.GDB_PROGRAM = PROGRAM
    harness.main(
        work_name='source-user-extents',
        program_prefix=(f'component={component!r}\n'
                        f'native_sites={invariant_halt(build / "kernel.elf")!r}\n'),
        input_scope='source-built ordinary user-buffer boundary cases',
        guest_timeout_seconds=300,
    )
