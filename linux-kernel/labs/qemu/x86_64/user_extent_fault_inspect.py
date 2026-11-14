"""Isolated negative native-extent boundaries; deliberately stop before user resume.

All mutations are explicit faults inside this disposable QEMU guest. Positive
observations never use them, and no host controls or native results are patched.
"""
from pathlib import Path
import json
import sys

import user_extent_inspect

harness = user_extent_inspect.harness
common, separator, ending = harness.GDB_PROGRAM.partition("\ntry:\n    Gate(")
assert separator
harness.GDB_PROGRAM = common + r'''
fault=None
fault_snapshot=None

def put(p,value):gdb.selected_inferior().write_memory(p,int(value).to_bytes(8,'little'))
def raw_domains():
    n=word(address('topal_x86_frame_context_v1')+24)
    c=context();st=state()
    roles=fault['baseline']['roles']
    return dict(allocator=words(address('topal_x86_frame_context_v1'),16),
        history=words(address('topal_x86_frame_history_v1'),5),
        state=st,tokens=words(address('topal_x86_frame_mm_tokens_v1'),6),
        root_tokens=words(address('topal_x86_frame_root_tokens_v1'),owner_bindings['token_bytes']//8),
        registry_sha256=hashlib.sha256(memory(address('topal_x86_frame_registry_v1'),n*64)).hexdigest(),
        cells_sha256=hashlib.sha256(memory(address('topal_x86_frame_table_v1'),128+n*24)).hexdigest(),
        image_sha256=hashlib.sha256(memory(roles[2]['physical'],503*4096)).hexdigest(),
        stack_sha256=hashlib.sha256(memory(roles[3]['physical'],8192)).hexdigest(),
        heap_sha256=hashlib.sha256(memory(st[3],st[2]*4096) if st[2] else b'').hexdigest(),
        epoch=words(address('topal_x86_frame_mm_epoch_v1'),2),
        process_context=words(c,(65536-512)//8))

def finish_fault(disposition):
    observed.update(passed=True,component_boundary=True,completed_guest=False,
        fault_mode=fault_mode,disposition=disposition,fault=fault,
        scope='Isolated disposable-guest fault at the native component boundary; no ordinary user resumption or complete application run')
    with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
    print('FRAME_ROOT_GDB_PASS')
    print('USER_EXTENT_NEGATIVE_PASS',fault_mode,disposition)
    gdb.execute('detach')
    gdb.execute('quit 0')

def pte_address(start,level):
    table=word(context()+24)
    for current in [4,3,2,1]:
        at=table+((start>>((current-1)*9+12))&511)*8
        if current==level:return at
        table=word(at)&0x000ffffffffff000
    raise AssertionError('invalid level')

PositiveGate=Gate
class Gate(PositiveGate):
    def stop(self):
        global fault,fault_snapshot,pending
        if self.role=='entry' and value('$rdi')==8 and fault is None:
            start=value('$rsi')&((1<<64)-1);write=value('$rcx')
            wants_write=fault_mode.startswith('write-')
            if (wants_write and (start!=0x7ffff0 or write!=1)) or (not wants_write and (start!=0x401000 or write!=0)):
                return super().stop()
            try:
                baseline=snapshot()[1]
                fault=dict(mode=fault_mode,baseline=baseline,sp=value('$rsp'),
                    registers={r:value('$'+r) for r in preserved},epoch=baseline['epoch'],
                    op=8,start=start,length=value('$rdx'),write=write)
                disposition='normal-rejection'
                if fault_mode.startswith(('user-','write-','absent-')):
                    kind,level=fault_mode.split('-');level=int(level)
                    # PML4/PDPT P or RW removal would also revoke the running
                    # kernel's shared lower-half translation, so actual-VM
                    # controls use US at every level and P/RW at user PD/PTE.
                    assert kind=='user' or level in [1,2]
                    at=pte_address(start,level);old=word(at)
                    bit={'user':4,'write':2,'absent':1}[kind]
                    new=old&~bit
                elif fault_mode in ['huge','leaf-format']:
                    at=pte_address(start,2 if fault_mode=='huge' else 1);old=word(at);new=old|128
                elif fault_mode=='foreign-table':
                    at=pte_address(start,2);old=word(at)
                    # An accessible image frame is nevertheless not a table owner.
                    new=baseline['roles'][2]['physical']|(old&0xfff)
                    disposition='fatal-retirement'
                elif fault_mode=='foreign-backing':
                    at=pte_address(start,1);old=word(at)
                    new=((old&0x8000000000000fff)|baseline['roles'][1]['physical'])
                    disposition='fatal-retirement'
                elif fault_mode=='foreign-issuer':
                    at=address('topal_x86_frame_root_tokens_v1')+owner_bindings['roles'][0]['slot']*24
                    old=word(at);new=old+1;disposition='fatal-retirement'
                elif fault_mode=='unpinned-table':
                    at=address('topal_x86_frame_registry_v1')+baseline['roles'][1]['token'][2]*64+48
                    old=word(at);new=0;disposition='fatal-retirement'
                elif fault_mode=='stale-cell':
                    at=address('topal_x86_frame_table_v1')+128+baseline['roles'][0]['record'][2]*24
                    old=word(at);new=old+1000;disposition='fatal-retirement'
                elif fault_mode=='return-site':
                    at=fault['sp'];old=word(at);new=old+1;disposition='pre-entry-refusal'
                elif fault_mode=='busy':
                    at=address('topal_x86_frame_gateway_recovery_v1')+8;old=word(at);new=1
                    disposition='pre-entry-refusal'
                elif fault_mode=='epoch-zero':
                    at=address('topal_x86_frame_mm_epoch_v1');old=word(at);new=0
                    disposition='pre-entry-refusal'
                elif fault_mode=='address-space':
                    at=context()+24;old=word(at);new=word(context()+16)
                    disposition='pre-entry-refusal'
                elif fault_mode=='context-self':
                    at=context()+256;old=word(at);new=old+8
                    disposition='pre-entry-refusal'
                elif fault_mode=='late-epoch':
                    fault.update(disposition='fatal-retirement',deferred=True)
                    pending=dict(fault)
                    LateEpoch('*topal_x86_user_extent_begin_v1')
                    return False
                else:raise AssertionError(fault_mode)
                fault.update(address=at,old=old,injected=new,disposition=disposition)
                put(at,new)
                fault_snapshot=raw_domains()
                if disposition=='normal-rejection':
                    return super().stop()
                pending=dict(fault)
                return False
            except Exception:fail()
        if self.role=='return' and fault and pending and pending.get('op')==8:
            try:
                assert fault['disposition']=='normal-rejection'
                result=super().stop()
                assert pending is None
                assert raw_domains()==fault_snapshot
                assert word(fault['address'])==fault['injected']
                put(fault['address'],fault['old'])
                finish_fault('normal-rejection')
                return result
            except Exception:fail()
        return super().stop()

class ExpectedHalt(gdb.Breakpoint):
    def __init__(self,symbol,kind):super().__init__('*'+symbol);self.kind=kind
    def stop(self):
        try:
            assert fault and fault['disposition']==self.kind,(fault,self.kind)
            assert word(fault['address'])==fault['injected'],'fault effects were rolled back'
            now=raw_domains()
            if self.kind=='pre-entry-refusal':
                assert now==fault_snapshot
            else:
                assert now['allocator'][0]==3 and now['allocator'][1:]==fault_snapshot['allocator'][1:]
                assert now['history'][:3]==fault_snapshot['history'][:3] and now['history'][3:]==[0,1]
                assert now['state'][0]==3 and now['state'][1:]==fault_snapshot['state'][1:]
                for k in set(now)-{'allocator','history','state'}:assert now[k]==fault_snapshot[k],k
                assert value('$rsp')==fault['sp']
                for r,n in fault['registers'].items():assert value('$'+r)==n,r
                zeros(fault['sp']-4080,4080)
                for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),
                                  ('gateway_recovery',48),('details',32)]:
                    zeros(address('topal_x86_frame_'+name+'_v1'),size)
                assert word(address('topal_frame_allocator_active_v1'))==0
                assert word(address('topal_x86_tail_boundary_active_v1'))==0
            finish_fault(self.kind)
        except Exception:fail()
        return True

class LateEpoch(gdb.Breakpoint):
    def stop(self):
        global fault_snapshot
        try:
            assert fault and fault.get('deferred') and pending['op']==8
            assert word(address('topal_x86_frame_gateway_recovery_v1')+8)==1
            assert word(address('topal_x86_frame_gateway_recovery_v1')+40)==fault['epoch'][0]
            at=address('topal_x86_frame_mm_epoch_v1');old=word(at)
            fault.update(address=at,old=old,injected=old+1)
            put(at,old+1);fault_snapshot=raw_domains();self.enabled=False
        except Exception:fail()
        return False

Complete=Unexpected
''' + r'''
try:
    Gate('topal_x86_frame_gateway_v1','entry')
    Gate('topal_x86_frame_gateway_return_reclaimed_v1','return')
    ExpectedHalt('topal_x86_frame_gateway_refused_v1','pre-entry-refusal')
    ExpectedHalt('topal_x86_frame_gateway_fatal_reclaimed_v1','fatal-retirement')
    Unexpected('*topal_x86_frame_gateway_late_reclaimed_v1')
    Unexpected('*topal_x86_systems_fatal')
    gdb.execute('continue')
    raise AssertionError('negative boundary did not terminate observation')
except Exception:fail()
'''

if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    mode = sys.argv.pop(2)
    assert mode in ['user-4','user-3','user-2','user-1','write-2','write-1','absent-2','absent-1',
                    'huge','leaf-format','foreign-table','foreign-backing','foreign-issuer',
                    'unpinned-table','stale-cell','return-site','busy','epoch-zero',
                    'address-space','context-self','late-epoch']
    bindings = json.loads((build / 'user-owners.json').read_text())
    model_dir = Path(__file__).resolve().parents[3] / 'tools/user-memory'
    prefix = f'import sys\nsys.path.insert(0,{str(model_dir)!r})\nowner_bindings={bindings!r}\nfault_mode={mode!r}\n'
    harness.main(work_name='user-extent-fault-'+mode,program_prefix=prefix,expect_backing=False,
                 input_scope=f'isolated negative native extent boundary ({mode}); incomplete guest')
