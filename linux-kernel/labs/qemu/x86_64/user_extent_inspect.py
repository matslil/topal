"""Independent observation of actual owned-extent calls in the full MM guest."""
from pathlib import Path
import json
import sys

import frame_memory_inspect

harness = frame_memory_inspect.harness
common, separator, ending = harness.GDB_PROGRAM.partition("\ntry:\n    Gate(")
assert separator, 'missing original MM observer installation'
harness.GDB_PROGRAM = common + r'''
import model as extent_model

observed['user_extents']=[]
observed['native_extent_trace']=None
observed['wp_transitions']=[]
observed['scope']='Unpublished full-root native extent/owner component; no all-caller or publication qualification'
extent_snapshot=None

class WriteProtection(gdb.Breakpoint):
    def __init__(self,symbol,ready):super().__init__('*'+symbol);self.ready=ready
    def stop(self):
        try:
            privileged();actual=value('$cr0')
            if not self.ready:
                assert not observed['wp_transitions'] or 'after' in observed['wp_transitions'][-1]
                assert value('$rax')==actual
                observed['wp_transitions'].append(dict(before=actual))
            else:
                assert observed['wp_transitions'] and 'after' not in observed['wp_transitions'][-1]
                assert actual==observed['wp_transitions'][-1]['before'] | (1<<16)
                observed['wp_transitions'][-1]['after']=actual
        except Exception:fail()
        return False
WriteProtection('topal_x86_user_extent_wp_before_v1',False)
WriteProtection('topal_x86_user_extent_wp_ready_v1',True)

def physical_owner(token_address,count,pins):
    t=words(token_address,3);issuer,identity,index=t
    n=word(address('topal_x86_frame_context_v1')+24)
    assert issuer==1 and identity>0 and index<n
    r=words(address('topal_x86_frame_registry_v1')+index*64,8)
    assert r[:2]==t[:2] and r[3:5]==[count,1] and r[5]>0 and r[6:]==[pins,1]
    assert 0<=r[2] and r[2]+count<=n
    cells=words(address('topal_x86_frame_table_v1')+128+r[2]*24,count*3)
    assert cells[0::3]==[identity]*count and cells[1::3]==[r[5]]*count
    assert cells[2::3]==list(range(cells[2],cells[2]+count))
    assert cells[2]+count<=1<<18
    return dict(token=t,record=r,cells=cells,physical=cells[2]*4096)

def snapshot():
    c=context();root=word(c+24);st=state()
    roles=[physical_owner(address('topal_x86_frame_root_tokens_v1')+r['slot']*24,
                          r['count'],r['pins']) for r in owner_bindings['roles']]
    assert roles[0]['physical']==root and roles[1]['physical']==word(c+112)
    assert roles[3]['physical']+4096==word(c+912)
    # Distinct full static allocations, even though the stack has two pins.
    spans=[(r['physical'],r['physical']+len(r['cells'])//3*4096) for r in roles]
    for i,(a,b) in enumerate(spans):
        assert all(b<=x or y<=a for x,y in spans[:i])
    table_roles={root:4,root+4096:3,root+8192:2,root+12288:2,root+16384:1,roles[1]['physical']:1}
    tables={p:extent_model.Table(level,tuple(words(p,512))) for p,level in table_roles.items()}
    phase=memory(c+104,1)[0]
    writable=493 if phase>=9 else 485
    backing={0x400000+i*4096:extent_model.Backing(roles[2]['physical']+i*4096,writable=i>=writable)
             for i in range(503)}
    backing.update({0x7fe000+i*4096:extent_model.Backing(roles[3]['physical']+i*4096) for i in range(2)})
    heap=None
    if st[2]:
        assert st[0]==1 and st[2]<=519
        heap=physical_owner(address('topal_x86_frame_mm_tokens_v1'),st[2],1)
        assert heap['physical']==st[3]
        for a,b in spans:assert st[3]+st[2]*4096<=a or b<=st[3]
        backing.update({0x5f7000+i*4096:extent_model.Backing(st[3]+i*4096) for i in range(st[2])})
    epoch=words(address('topal_x86_frame_mm_epoch_v1'),2)
    cr0=value('$cr0');cr4=value('$cr4')
    # The target separately observes EFER and BSP; record raw monitor evidence
    # on the first call as well as these independent paging-control checks.
    if 'paging' not in observed:
        observed['paging']=dict(cr0=cr0,cr4=cr4,registers=gdb.execute('monitor info registers',to_string=True))
    qualified=all(cr0&(1<<bit) for bit in [0,16,31]) and not any(cr4&(1<<bit) for bit in [12,17,21,22,23,24])
    obs=extent_model.Observation(root,value('$cr3'),epoch[0],epoch[0],tables,backing,
        frozenset(table_roles),privileged=value('$cs')&3==0,critical=value('$eflags')&0x600==0,
        stable=True,mode='la48-small' if qualified else 'unqualified-mode')
    fingerprints=dict(roles=roles,heap=heap,phase=phase,state=st,
        tables={str(p):dict(level=t.level,words=list(t.words)) for p,t in tables.items()},
        allocator=words(address('topal_x86_frame_context_v1'),16),
        tokens=words(address('topal_x86_frame_mm_tokens_v1'),6),
        epoch=epoch,captured=words(c+296,15),
        image_sha256=hashlib.sha256(memory(roles[2]['physical'],503*4096)).hexdigest(),
        stack_sha256=hashlib.sha256(memory(roles[3]['physical'],8192)).hexdigest(),
        heap_sha256=hashlib.sha256(memory(st[3],st[2]*4096) if st[2] else b'').hexdigest())
    return obs,fingerprints

class NativeTrace(gdb.Breakpoint):
    def stop(self):
        try:
            assert pending and pending['op']==8
            assert word(address('topal_x86_frame_gateway_recovery_v1')+40)==pending['epoch'][0]
            if observed['native_extent_trace'] is None:
                trace=gdb.execute('bt',to_string=True)
                for name in ['topal_x86_owned_user_extent_v1','topal_frame_kernel_dispatch_v1',
                             'topal_x86_frame_gateway_v1','topal_x86_user_extent_call_v1',
                             'topal_x86_user_extent_probe_v1','topal_x86_systems_static_image_process_enter']:
                    assert name in trace,(name,trace)
                frame=gdb.newest_frame()
                while frame and frame.name()!='topal_x86_frame_gateway_v1':frame=frame.older()
                assert frame and frame.older(),trace
                for r,n in pending['registers'].items():assert int(frame.older().read_register(r))==n,(r,trace)
                observed['native_extent_trace']=trace
                observed['native_extent_trace_samples']=1
                self.enabled=False
        except Exception:fail()
        return False
NativeTrace("*'u.owner'")

MemoryGate=Gate
class Gate(MemoryGate):
    def stop(self):
        global pending,extent_snapshot
        if self.role=='entry' and value('$rdi')!=8 or self.role!='entry' and pending and pending['op']!=8:
            return super().stop()
        try:
            privileged()
            if self.role=='entry':
                assert pending is None
                c=context();sp=value('$rsp');epoch=words(address('topal_x86_frame_mm_epoch_v1'),2)
                assert sp&15==8 and sp+16<=word(c+8)<=0x180000
                assert word(c)==1 and word(c+256)==c and value('$gs_base')==c
                assert value('$cr3')==word(c+24) and epoch[0]>0 and epoch[1]==12
                assert word(sp)==address('topal_x86_user_extent_return_v1')
                start=value('$rsi')&((1<<64)-1);length=value('$rdx')&((1<<64)-1);write=value('$rcx')
                assert write in [0,1] and value('$r8')==0
                obs,extent_snapshot=snapshot()
                expected=extent_model.admit(obs,start,length,write=bool(write))
                status={extent_model.Status.EMPTY:0,extent_model.Status.ADMITTED:0,
                    extent_model.Status.RANGE:1,extent_model.Status.ABSENT:2,
                    extent_model.Status.PERMISSION:3,extent_model.Status.UNSUPPORTED:4}[expected.status]
                pending=dict(op=8,start=start,length=length,write=write,sp=sp,epoch=epoch,
                    expected=status,checked_pages=expected.checked_pages,
                    registers={r:value('$'+r) for r in preserved},before=extent_snapshot)
                return False
            assert pending and pending['op']==8
            assert value('$rsp')==pending['sp']
            for r,n in pending['registers'].items():assert value('$'+r)==n,r
            for r in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:assert value('$'+r)==0,r
            zeros(pending['sp']-4080,4080)
            for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),
                              ('gateway_recovery',48),('details',32)]:
                zeros(address('topal_x86_frame_'+name+'_v1'),size)
            assert word(address('topal_frame_allocator_active_v1'))==0
            assert word(address('topal_x86_tail_boundary_active_v1'))==0
            assert snapshot()[1]==extent_snapshot
            result=value('$rax');assert result==pending['expected'],(result,pending['expected'],observed['paging'])
            pending['result']=result
            observed['user_extents'].append(pending);observed['calls'].append(pending)
            pending=None;extent_snapshot=None
        except Exception:fail()
        return False

MemoryComplete=Complete
class Complete(MemoryComplete):
    def stop(self):
        try:
            assert observed['native_extent_trace']
            assert observed['wp_transitions'] and all(t['after']==t['before']|(1<<16) for t in observed['wp_transitions'])
            assert value('$cr0') & (1<<16)
            counts=words(address('topal_x86_user_extent_probe_counts_v1'),4)
            assert counts[0]>=5 and counts[1]==len(observed['user_extents']) and counts[2]>0 and counts[3]>0,counts
            assert any(c['result']==0 and c['start']>=0x619000 and c['start']<0x7fe000 for c in observed['user_extents'])
            assert any(c['checked_pages']==2 for c in observed['user_extents'])
            assert {0,1,2,3}<=set(c['result'] for c in observed['user_extents'])
            observed['probe_counts']=counts
        except Exception:fail()
        return super().stop()

Unexpected('*topal_x86_user_extent_probe_fail_v1')
''' + separator + ending

if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    bindings = json.loads((build / 'user-owners.json').read_text())
    model_dir = Path(__file__).resolve().parents[3] / 'tools/user-memory'
    prefix = f'import sys\nsys.path.insert(0,{str(model_dir)!r})\nowner_bindings={bindings!r}\n'
    harness.main(work_name='user-extents', program_prefix=prefix,
                 input_scope='actual full-root heap and owned user-extent component')
