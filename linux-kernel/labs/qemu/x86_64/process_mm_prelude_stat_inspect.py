"""Read-only focused pre-import stat caller observation; not whole-VM success."""
import hashlib
import json
from pathlib import Path
import struct
import sys

import frame_root_inspect as harness
from process_mm_regression_observer import BOUNDARY_PROGRAM
from user_extent_application_inspect import invariant_halt


PROGRAM=BOUNDARY_PROGRAM+r'''
probe=None

def probe_snapshot():
    value=capture()
    value['table_live']=word(value['context']['table'])
    return value

def probe_domains():
    result={}
    for suffix,size in [('private',65528),('stack',4096),('facts',24),('output',56),('details',32)]:
        at=address('topal_x86_frame_'+suffix+'_v1')
        raw=memory(at,size);assert raw==bytes(size)
        result[suffix]=dict(address=at,bytes=size,sha256=hashlib.sha256(raw).hexdigest())
    for name,size in [('topal_x86_frame_gateway_recovery_v1',48),
                      ('topal_frame_allocator_active_v1',8),('topal_x86_tail_boundary_active_v1',8)]:
        at=address(name);raw=memory(at,size);assert raw==bytes(size)
        result[name]=dict(address=at,bytes=size,sha256=hashlib.sha256(raw).hexdigest())
    return result

class ProbeUserReturned(Checked):
    def observe(self):
        self.enabled=False
        assert probe is not None and pending is None and probe.get('source_return') is not None
        assert reg('cs')&3==3 and reg('rax')==(1<<64)-2
        assert reg('rsp')==probe['user_request']['sp']+8
        assert {r:reg(r) for r in callee}==probe['user_request']['callee']
        probe['user_return']=dict(rip=reg('rip'),sp=reg('rsp'),result=reg('rax'),
            callee={r:reg(r) for r in callee},snapshot=probe_snapshot())
        return False

class ProbeUserRequest(Checked):
    def observe(self):
        global probe
        if reg('rdi')!=262 or reg('rdx')!=prelude_profile['path']:return False
        assert probe is None and pending is None and reg('cs')&3==3
        assert reg('rsi')==(1<<64)-100 and reg('rcx')==(1<<64)-1 and reg('r8')==0
        value=probe_snapshot();model['snapshot'](value)
        assert value['context']['phase']==29 and value['table_live']==0
        probe=dict(user_request=dict(rip=reg('rip'),sp=reg('rsp'),return_pc=word(reg('rsp')),
            arguments={r:reg(r) for r in ['rdi','rsi','rdx','rcx','r8']},
            callee={r:reg(r) for r in callee},snapshot=value))
        ProbeUserReturned('*'+hex(word(reg('rsp'))),internal=True)
        return False

class ProbeCall(Checked):
    def __init__(self,site,returned):
        self.returned=returned
        super().__init__('*'+hex(site),internal=True)
    def observe(self):
        assert probe is not None and pending is None and reg('cs')&3==0
        assert reg('eflags')&0x600==0
        if not self.returned:
            assert 'source_call' not in probe and reg('r8')==prelude_profile['path'] and reg('r9')==9
            assert memory(reg('r8'),9)==b'/missing\0'
            probe['source_call']=dict(rip=reg('rip'),sp=reg('rsp'),
                arguments={r:reg(r) for r in ['r8','r9']},callee={r:reg(r) for r in callee},
                flags=reg('eflags'),snapshot=probe_snapshot())
        else:
            assert 'source_return' not in probe and 'gateway_return' in probe
            entered=probe['source_call'];gateway=probe['gateway_entry']
            assert reg('rsp')==entered['sp'] and reg('eflags')&1==0
            assert {r:reg(r) for r in callee}==entered['callee']
            assert reg('r8')==entered['arguments']['r8'] and reg('r9')==9
            low=gateway['sp']-4000;size=entered['sp']-low
            assert size in [4080,4088] and memory(low,size)==bytes(size)
            probe['source_return']=dict(rip=reg('rip'),sp=reg('rsp'),flags=reg('eflags'),
                callee={r:reg(r) for r in callee},snapshot=probe_snapshot(),domains=probe_domains(),
                caller=dict(address=low,bytes=size,sha256=hashlib.sha256(memory(low,size)).hexdigest()))
        return False

class ProbeGateway(Checked):
    def __init__(self,symbol,returned):
        self.returned=returned
        super().__init__("*'"+symbol+"'",internal=True)
    def observe(self):
        if probe is None or 'source_call' not in probe or 'source_return' in probe:return False
        if not self.returned:
            assert reg('rdi')==8 and reg('rsi')==prelude_profile['path'] and reg('rdx')==9 and reg('rcx')==0
            assert 'gateway_entry' not in probe
            probe['gateway_entry']=dict(rip=reg('rip'),sp=reg('rsp'),return_pc=word(reg('rsp')),
                arguments={r:reg(r) for r in ['rdi','rsi','rdx','rcx']},callee={r:reg(r) for r in callee})
        else:
            assert 'gateway_entry' in probe and 'gateway_return' not in probe
            assert reg('rsp')==probe['gateway_entry']['sp'] and reg('rax')==0
            assert {r:reg(r) for r in callee}==probe['gateway_entry']['callee']
            low=reg('rsp')-4000;assert memory(low,4000)==bytes(4000)
            probe['gateway_return']=dict(rip=reg('rip'),sp=reg('rsp'),result=reg('rax'),
                domains=probe_domains(),inner_caller=dict(address=low,bytes=4000,
                    sha256=hashlib.sha256(memory(low,4000)).hexdigest()))
        return False

class ProbeComplete(Checked):
    def observe(self):
        assert probe is not None and pending is None and 'user_return' in probe
        assert reg('cs')&3==3 and reg('rdi')==(1<<64)-2
        assert probe['user_return']['snapshot']['context']['phase']==30
        probe['marker']=dict(rip=reg('rip'),result=reg('rdi'),snapshot=probe_snapshot())
        observed.update(passed=True,prelude_stat=probe,full_ordinary_cohort_complete=False,
            scope='Selected source-built missing-stat prefix; kernel/helper return and cleanup only, not complete application/ABI/VM terminal retirement')
        save()
        print('PRELUDE_STAT_PREFIX_PASS',flush=True)
        return True

class ProbeHalt(Checked):
    def observe(self):
        raise AssertionError(('unexpected invariant halt',probe_snapshot()))

try:
    Gateway('topal_x86_frame_gateway_v1',True)
    Gateway('topal_x86_frame_gateway_return_reclaimed_v1',False)
    ProbeUserRequest('*'+hex(component['syscall4']),internal=True)
    ProbeCall(prelude_profile['call'],False)
    ProbeCall(prelude_profile['returned'],True)
    ProbeGateway('topal_x86_frame_gateway_v1',False)
    ProbeGateway('topal_x86_frame_gateway_return_reclaimed_v1',True)
    ProbeComplete('*'+hex(prelude_profile['entry']),internal=True)
    ProbeHalt('*'+hex(native_halt),internal=True)
    gdb.execute('continue')
    assert observed['passed'] and observed['full_ordinary_cohort_complete'] is False
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc();save();gdb.execute('quit 1')
'''


def layout(build):
    sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'tools/user-memory'))
    from inspect_call_inventory import ERASE_RETURN, Elf, verify_calls
    component=json.loads((build/'source-component.json').read_text())
    assert component['mode']=='prelude-stat' and component['prelude_stat'] is not None
    inventory=json.loads((build/'extent-call-inventory.json').read_text())
    kernel=Elf(build/'kernel.elf');provider=Elf(build/'provider.o')
    function=verify_calls(kernel,inventory);verify_calls(provider,inventory)
    selected=[row for row in inventory['calls'] if 'static-process-shell-root-directory-stat-profile-ready' in row['nearest_labels']]
    assert len(selected)==1 and selected[0]['target']=='runtime-readable-extent'
    offset=selected[0]['offset'];_,body=kernel.function(inventory['function'])
    assert body[offset+5:offset+5+len(ERASE_RETURN)]==ERASE_RETURN
    user=Elf(build/'source-user.elf');marker,_=user.function('topal_prelude_stat_completed')
    assert marker['value']==component['prelude_stat']['entry']
    path=user.symbols['topal_prelude_stat_path'];section=user.sections[path['section']]
    at=path['value']-section[3]
    assert path['value']==component['prelude_stat']['path'] and user.section_bytes(path['section'])[at:at+9]==b'/missing\0'
    assert hashlib.sha256((build/'command-cases.c').read_bytes()).hexdigest()==component['prelude_stat']['source_sha256']
    assert hashlib.sha256((build/'command-original.c').read_bytes()).hexdigest()==component['command_source_sha256']
    profile=dict(component['prelude_stat'],offset=offset,call=function['value']+offset,
        returned=function['value']+offset+5+len(ERASE_RETURN),kernel_sha256=hashlib.sha256(kernel.data).hexdigest(),
        inventory_sha256=hashlib.sha256((build/'extent-call-inventory.json').read_bytes()).hexdigest())
    return component,profile


def main():
    build=Path(sys.argv[1]).resolve();component,profile=layout(build)
    kernel=(build/'kernel.elf').read_bytes();phoff=struct.unpack_from('<Q',kernel,32)[0]
    width,count=struct.unpack_from('<HH',kernel,54);spans=[]
    for i in range(count):
        p=struct.unpack_from('<IIQQQQQQ',kernel,phoff+i*width)
        if p[0]==1:
            assert p[3]==p[4] and p[4]+p[6]<=1<<30;spans.append([p[4],p[4]+p[6]])
    (build/'prelude-stat-layout.json').write_text(json.dumps(profile,indent=2)+'\n')
    harness.GDB_PROGRAM=PROGRAM
    harness.main(work_name='process-mm-prelude-stat',expect_backing=False,
        program_prefix=f'component={component!r}\nkernel_spans={spans!r}\nprelude_profile={profile!r}\nnative_halt={invariant_halt(build/"kernel.elf")["halt"]!r}\n',
        input_scope='Source-built pre-import missing-stat prefix; no full ABI/VM completion claim',guest_timeout_seconds=600)


if __name__=='__main__':main()
