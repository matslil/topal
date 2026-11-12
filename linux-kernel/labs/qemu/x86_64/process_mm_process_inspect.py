"""Observe ordinary fork/yield/resize/wait with actual independent heap owners."""
import hashlib
import json
from pathlib import Path
import struct
import sys

import frame_root_inspect as harness
from user_extent_application_inspect import invariant_halt

MODEL = (Path(__file__).resolve().parents[3] / 'tools/process-memory/process_native.py').read_text()
PROGRAM = 'model={}\nexec(' + repr(MODEL) + ',model)\n' + r'''
import gdb,hashlib,json,struct,traceback
def reg(name):return int(gdb.parse_and_eval('$'+name))&((1<<64)-1)
def address(name):return int(gdb.parse_and_eval("(unsigned long long)&'"+name+"'"))
def memory(p,n):return bytes(gdb.selected_inferior().read_memory(p,n)) if n else b''
def words(p,n):return list(struct.unpack('<'+'Q'*n,memory(p,n*8)))
def word(p):return words(p,1)[0]
context=address('topal_bootstrap_storage')+512
observed=dict(passed=False,cases=[],events=[],child_exec_entries=[],replacement_entries=[],terminal=[])
child_exec=component['mode']=='process-mm-exec'
cancel_config=component.get('cancellation')
observed['cancellation']=cancel_config
pending=case=None
returns={}
callee=['rbx','rbp','r12','r13','r14','r15']

def allocation(pointer,data=True):
    token=words(pointer,3)
    if token==[0]*3:return dict(token=token,record=[],cells=[],bytes='')
    allocator=words(address('topal_x86_frame_context_v1'),16)
    assert token[0]==1 and 0<token[1]<allocator[2] and token[2]<allocator[3]
    record=words(allocator[6]+token[2]*64,8)
    assert record[:2]==token[:2] and record[7]==1 and 0<record[3]<=allocator[3]
    assert 0<=record[2]<record[2]+record[3]<=allocator[3]
    cells=words(allocator[4]+128+record[2]*24,record[3]*3)
    return dict(token=token,record=record,cells=cells,
        bytes=memory(cells[2]*4096,record[3]*4096).hex() if data else '')

def capture():
    root,low,table=word(context+24),word(context+112),word(context+928)
    tokens=address('topal_x86_frame_mm_tokens_v1')
    root_tokens=address('topal_x86_frame_root_tokens_v1')
    return dict(allocator=words(address('topal_x86_frame_context_v1'),16),
        ids=words(address('topal_x86_process_mm_ids_v1'),7),
        state=words(address('topal_x86_frame_mm_state_v1'),4),tokens=words(tokens,6),
        inactive_state=words(address('topal_x86_process_mm_inactive_state_v1'),4),
        prepared_state=words(address('topal_x86_process_mm_prepared_state_v1'),7),
        prepared_owner=words(address('topal_x86_process_mm_prepared_owner_v1'),3),
        transition=words(address('topal_x86_process_mm_transition_v1'),3),
        active=allocation(tokens),candidate=allocation(tokens+24),
        inactive=allocation(address('topal_x86_process_mm_inactive_token_v1')),
        prepared=allocation(address('topal_x86_process_mm_prepared_token_v1')),
        root_owners=[allocation(root_tokens+i*24,False) for i in range(token_bytes//24) if words(root_tokens+i*24,3)!=[0]*3],
        details=words(address('topal_x86_frame_details_v1'),4),
        epoch=words(address('topal_x86_frame_mm_epoch_v1'),2),
        context=dict(address=context,state=word(context),root=root,low=low,heap=word(context+120),
            self=word(context+256),bootstrap=word(context+16),user_arg=word(context+312),phase=word(context+104),
            tls=word(context+128),registered_tid=word(context+152),robust=word(context+160),
            table=table,pid=word(table+56),child=word(table+0x2010),relationship=word(table+0x2000),
            saved_pid=word(table+0x3038),saved_heap=word(table+0x2178)),
        inactive_wait=[word(table+0x2060),word(table+0x2078)],
        tables={str(a):words(a,512) for a in [root,root+4096,root+8192,root+12288,root+16384,low]},
        kernel_spans=kernel_spans,cr3=reg('cr3'),cr0=reg('cr0'),cr4=reg('cr4'),cs=reg('cs'),flags=reg('eflags'),gs_base=reg('gs_base'),fs_base=reg('fs_base'),
        thread=list(gdb.selected_thread().ptid))

def save():
    observed['pending']=pending
    observed['active_case']=case
    with open(observation_path,'w') as f:json.dump(observed,f)

class Checked(gdb.Breakpoint):
    def stop(self):
        try:return self.observe()
        except Exception:
            traceback.print_exc()
            observed['failure']=traceback.format_exc()
            save()
            return True

class Gateway(Checked):
    def __init__(self,symbol,enter):
        self.enter=enter
        super().__init__("*'"+symbol+"'",internal=True)
    def observe(self):
        global pending
        if self.enter:
            op=reg('rdi')
            if op not in [5,6,7,9,10,11,12,13]:return False
            assert pending is None
            pending=dict(operation=op,request=reg('rsi'),before=capture(),sp=reg('rsp'),
                callee={r:reg(r) for r in callee},retired=[])
            model['snapshot'](pending['before'])
        elif pending is not None:
            row=pending
            assert reg('rsp')==row['sp'] and {r:reg(r) for r in callee}==row['callee']
            assert all(reg(r)==0 for r in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11'])
            after=capture()
            retained=[after[k]['token'] for k in ['active','inactive','prepared','candidate']]
            for bank in ['active','inactive','prepared','candidate']:
                old=row['before'][bank]
                if old['token']!=[0]*3 and old['token'] not in retained:
                    a=after['allocator'];r=old['record'];count=r[3]
                    row['retired'].append(dict(token=old['token'],count=count,
                        record=words(a[6]+old['token'][2]*64,8),cells=words(a[4]+128+r[2]*24,count*3),
                        bytes=memory(old['cells'][2]*4096,count*4096).hex()))
            row.update(after=after,result=reg('rax'))
            row['comparison']=model['event'](row)
            observed['events'].append(row)
            pending=None
            save()
        return False

class CaseReturned(Checked):
    def observe(self):
        global case
        self.enabled=False
        assert case is not None and pending is None and reg('cs')&3==3
        assert cancel_config is None, ('cancellation case unexpectedly returned',reg('rax'))
        if reg('rax')!=0:
            value=capture()
            raise AssertionError(('source case failed',reg('rax'),value['context'],value['ids']))
        assert reg('rsp')==case['sp']+8 and {r:reg(r) for r in callee}==case['callee']
        after=capture();model['snapshot'](after)
        case['returned']=dict(result=reg('rax'),sp=reg('rsp'),callee={r:reg(r) for r in callee},cs=reg('cs'))
        assert after['ids'][:3]==case['before']['ids'][:3] and after['ids'][3:6]==[0]*3
        assert after['ids'][6]==case['before']['ids'][6]+(2 if child_exec else 1)
        assert after['state'][1:3]==case['before']['state'][1:3]
        assert hashlib.sha256(bytes.fromhex(after['active']['bytes'])).hexdigest()==case['prefix']
        case['after']=after
        observed['cases'].append(case);case=None
        print('PROCESS_MM_CASE_PASS',len(observed['cases']),flush=True)
        save()
        return False

class CaseEntered(Checked):
    def observe(self):
        global case
        assert case is None and pending is None and reg('cs')&3==3
        before=capture();model['snapshot'](before)
        assert before['ids'][0]==1 and before['state'][1:3]==[0x619000,34]
        case=dict(before=before,sp=reg('rsp'),callee={r:reg(r) for r in callee},
            prefix=hashlib.sha256(bytes.fromhex(before['active']['bytes'])).hexdigest())
        site=word(reg('rsp'))
        if site not in returns:returns[site]=CaseReturned('*'+hex(site),internal=True)
        returns[site].enabled=True
        return False

class ImageEntered(Checked):
    def observe(self):
        assert reg('cs')&3==3 and pending is None
        value=capture()
        if cancel_config is not None and cancel_config['early'] and value['ids'][0]==1:
            assert value['ids'][2]==1 and value['context']['phase']==0 and value['state']==[0]*4
            assert value['fs_base']==value['context']['tls']==0
            observed['replacement_entries'].append(dict(snapshot=value,initial_sp=reg('rsp')))
            save()
        if value['ids'][0]<2:return False
        assert child_exec and case is not None
        model['snapshot'](value)
        assert value['state']==[0]*4 and value['tokens']==[0]*6
        assert value['fs_base']==value['context']['tls']==0
        assert value['context']['registered_tid']==value['context']['robust']==0
        assert value['ids'][3:6]==case['before']['ids'][:3]
        assert value['ids'][2]==case['before']['ids'][2]+1
        assert value['inactive_state'][2]==67
        last=observed['events'][-1]
        assert last['operation']==7 and last['before']['ids'][0]==value['ids'][0]
        assert last['after']['ids']==value['ids']
        assert last['before']['inactive']==last['after']['inactive']==value['inactive']
        assert word(reg('rsp'))==2
        observed['child_exec_entries'].append(dict(snapshot=value,initial_sp=reg('rsp')))
        save()
        return False

class Terminal(Checked):
    def __init__(self,symbol,retired):
        self.retired=retired
        super().__init__("*'"+symbol+"'",internal=True)
    def observe(self):
        assert cancel_config is not None and pending is None and case is not None
        value=capture();model['snapshot'](value)
        assert reg('cs')&3==0 and value['ids'][0]==1
        assert value['epoch'][1]==cancel_config['syscall']
        assert value['context']['user_arg']==cancel_config['status']
        assert value['state'][2]==35
        raw=memory(value['context']['table'],cancel_config['process_storage_bytes'])
        if self.retired:
            assert value['ids'][3:6]==[0]*3 and value['inactive_state']==[0]*4
            assert raw==bytes(len(raw)) and value['inactive_wait']==[0,0]
            assert len(observed['terminal'])==1
            assert value['active']==observed['terminal'][0]['snapshot']['active']
        else:
            assert observed['terminal']==[]
            assert value['ids'][3]>=2 and value['inactive_state'][2]==101
            assert value['inactive_wait'][0]==2 and value['inactive_wait'][1]>0
            assert value['context']['phase']==(1 if cancel_config['early'] else 32)
            child=bytes.fromhex(value['inactive']['bytes'])
            assert [child[i] for i in [34*4096+128,66*4096+64,101*4096-1]]==[0xc1,0xc2,0xc7]
        observed['terminal'].append(dict(retired=self.retired,snapshot=value,storage=raw.hex()))
        save()
        return False

class Halt(Checked):
    def observe(self):
        value=capture()
        observed['halt']=dict(snapshot=value,registers={r:reg(r) for r in
            ['rax','rcx','rdx','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15','rsp','rip']},
            stack=words(reg('rsp'),16),context_words=words(context,120))
        raise AssertionError(('provider invariant halt',value['context'],value['ids']))

class Complete(Checked):
    def observe(self):
        assert pending is None
        ops=[r['operation'] for r in observed['events']]
        # Four new large-heap cases plus the unchanged four child/clone cases
        # in the surrounding full ABI witness all use actual MM owners.
        if cancel_config is not None:
            assert case is not None and observed['cases']==[] and len(observed['terminal'])==2
            assert [ops.count(n) for n in [6,9,10,11,12,13]]==[1,1,1,1,0,1]
            assert ops.count(7)==int(cancel_config['early'])
            assert len(observed['replacement_entries'])==int(cancel_config['early'])
        else:
            assert case is None and len(observed['cases'])==4
            assert [r['before']['ids'][2] for r in observed['cases']]==[0,1,2,3]
            assert [ops.count(n) for n in [9,10,11,12]]==[8,8,8,8]
            preparations=[r['before']['state'][2] for r in observed['events'] if r['operation']==9]
            assert sorted(preparations)==[34]*4+[67]*4
            assert ops.count(7)==(7 if child_exec else 3)
            assert len(observed['child_exec_entries'])==(4 if child_exec else 0)
        assert words(address('topal_x86_frame_mm_tokens_v1'),6)==[0]*6
        assert words(address('topal_x86_frame_mm_state_v1'),4)==[0]*4
        observed['final']={}
        for suffix,count in [('ids',7),('inactive_token',3),('inactive_state',4),('prepared_token',3),
                             ('prepared_state',7),('prepared_owner',3),('transition',3)]:
            actual=words(address('topal_x86_process_mm_'+suffix+'_v1'),count)
            assert actual==[0]*count
            observed['final'][suffix]=actual
        observed['passed']=True
        save()
        return True

try:
    Gateway('topal_x86_frame_gateway_v1',True)
    Gateway('topal_x86_frame_gateway_return_reclaimed_v1',False)
    CaseEntered('*'+hex(component['entry']),internal=True)
    if child_exec or (cancel_config is not None and cancel_config['early']):
        ImageEntered('*'+hex(component['image_entry']),internal=True)
    if cancel_config is not None:
        Terminal('topal_x86_process_mm_parent_exit_v1',False)
        Terminal('topal_x86_process_mm_parent_resources_retired_v1',True)
    Halt('*'+hex(native_halt),internal=True)
    Complete('*topal_x86_systems_fatal',internal=True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    save()
    gdb.execute('quit 1')
'''


if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    component = json.loads((build / 'source-component.json').read_text())
    assert component['mode'] in ['process-mm','process-mm-exec','process-mm-cancel']
    suffix={'process-mm':'','process-mm-exec':'_exec','process-mm-cancel':'_cancel'}[component['mode']]
    source = Path(__file__).with_name('user_extent_process_mm'+suffix+'_cases.c').read_bytes()
    assert hashlib.sha256(source).hexdigest() == component['process_mm'+suffix+'_source_sha256']
    kernel = (build / 'kernel.elf').read_bytes()
    phoff = struct.unpack_from('<Q', kernel, 32)[0]
    phsize, phcount = struct.unpack_from('<HH', kernel, 54)
    spans = []
    for i in range(phcount):
        p = struct.unpack_from('<IIQQQQQQ', kernel, phoff + i * phsize)
        if p[0] == 1:
            assert p[3] == p[4] and p[4] + p[6] <= 1 << 30
            spans.append([p[4], p[4] + p[6]])
    work_name = sys.argv[2] if len(sys.argv) == 3 else 'process-mm-users'
    assert Path(work_name).name == work_name and work_name.startswith('process-mm-users')
    # The shared harness reads only its normal build argument. Separate names
    # preserve prior attempts instead of overwriting their observations.
    sys.argv = sys.argv[:2]
    harness.GDB_PROGRAM = PROGRAM
    harness.main(work_name=work_name,
        program_prefix=f'component={component!r}\nkernel_spans={spans!r}\nnative_halt={invariant_halt(build / "kernel.elf")["halt"]!r}\n',
        input_scope='source-built ordinary fork/yield/resize/wait and real MM owners',
        guest_timeout_seconds=600)
