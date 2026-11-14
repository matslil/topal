"""Actual unpublished full-root syscall heap ownership and retirement."""
import frame_root_inspect as harness


harness.GDB_PROGRAM = r'''
import gdb,json,traceback,hashlib,struct

observed=dict(passed=False,calls=[],memory=[],source_trace=None)
pending=None
preserved=['rbx','rbp','r12','r13','r14','r15']
def value(s):return int(gdb.parse_and_eval(s))
def address(s):return value('(unsigned long long)&'+s)
def memory(p,n):return bytes(gdb.selected_inferior().read_memory(p,n))
def words(p,n):
    b=memory(p,n*8)
    return list(struct.unpack('<%dQ'%n,b))
def word(p):return words(p,1)[0]
def zeros(p,n):assert memory(p,n)==bytes(n),(hex(p),n)
def context():return address('topal_bootstrap_storage')+512
def state():return words(address('topal_x86_frame_mm_state_v1'),4)
def ptes():
    c=context();low=word(c+112);root=word(c+24)
    return words(low+0xfb8,9)+words(root+0x4000,510)
def token():return words(address('topal_x86_frame_mm_tokens_v1'),3)
def fail():
    traceback.print_exc()
    if pending and isinstance(pending.get('data_before'),bytes):
        pending['data_before_sha256']=hashlib.sha256(pending.pop('data_before')).hexdigest()
    observed['pending']=pending
    observed['backtrace']=gdb.execute('bt',to_string=True)
    with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
    gdb.execute('quit 1')
def privileged():assert value('$cs')&3==0 and value('$eflags')&0x600==0

class Source(gdb.Breakpoint):
    def stop(self):
        try:
            assert pending and pending['op']>=5
            trace=gdb.execute('bt',to_string=True)
            assert 'topal_frame_memory_request_v1' in trace and 'topal_x86_frame_gateway_v1' in trace,trace
            assert 'topal_x86_systems_static_image_process_enter' in trace,trace
            frame=gdb.newest_frame()
            while frame and frame.name()!='topal_x86_frame_gateway_v1':frame=frame.older()
            assert frame and frame.older(),trace
            for r,n in pending['registers'].items():
                assert int(frame.older().read_register(r))==n,(r,trace)
            observed['source_trace']=trace
            observed['syscall_caller_cfi_registers_checked']=True
            self.enabled=False
        except Exception:fail()
        return False
source=Source('*topal_tail_step_component_v1');source.enabled=False

class Gate(gdb.Breakpoint):
    def __init__(self,name,role):super().__init__('*'+name);self.role=role
    def stop(self):
        global pending
        try:
            privileged()
            if self.role=='entry':
                assert pending is None
                pending=dict(op=value('$rdi'),request=value('$rsi')&((1<<64)-1),sp=value('$rsp'),
                    registers={r:value('$'+r) for r in preserved})
                if pending['op']>=5:
                    op=pending['op'];assert op in [5,6,7]
                    c=context();epoch=words(address('topal_x86_frame_mm_epoch_v1'),2)
                    assert word(c)==(2 if op==6 else 1)
                    assert pending['sp']&15==8 and pending['sp']+16==word(c+8)
                    assert value('$cr3')==word(c+(16 if op==6 else 24))
                    if op!=6:assert value('$gs_base')==c
                    name={5:'brk',6:'cleanup',7:'exec'}[op]
                    assert word(pending['sp'])==address('topal_x86_frame_mm_'+name+'_return_v1')
                    assert epoch[0]>0 and epoch[1] in ({12} if op==5 else {59} if op==7 else {60,231})
                    pending['epoch']=epoch
                    pending['captured']=words(c+296,15)
                    pending['before']=state();pending['token_before']=token()
                    n=pending['before'][2];base=pending['before'][3]
                    pending['data_before']=memory(base,n*4096) if n else b''
                    pending['ptes_before']=ptes()
                    if observed['source_trace'] is None and op==5 and pending['request']:
                        source.enabled=True
                print('FRAME_MEMORY_ENTER',len(observed['calls']),pending['op'],pending['request'],flush=True)
                return False
            assert pending is not None
            assert value('$rsp')==pending['sp']
            for r,n in pending['registers'].items():assert value('$'+r)==n,r
            for r in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:assert value('$'+r)==0,r
            zeros(pending['sp']-4080,4080)
            for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),('gateway_recovery',48)]:
                zeros(address('topal_x86_frame_'+name+'_v1'),size)
            if pending['op']>=5:
                c=context();op=pending['op'];result=value('$rax')
                before=pending['before'];after=state();now=token();pages=ptes()
                assert words(c+296,15)==pending['captured']
                zeros(address('topal_x86_frame_mm_tokens_v1')+24,24)
                zeros(address('topal_x86_frame_details_v1'),32)
                oldBreak=before[1] if before[0] else 0x5f7000
                if op==5:
                    request=pending['request']
                    if not request or not 0x5f7000<=request<=0x7fe000:
                        assert result==oldBreak
                        assert now==pending['token_before'] and pages==pending['ptes_before']
                    else:
                        assert result in [oldBreak,request]
                        if result!=request:
                            assert after==before and now==pending['token_before'] and pages==pending['ptes_before']
                    assert after[0]==1 and after[1]==result
                    assert after[2]==(result-0x5f7000+4095)//4096
                    kept=min(before[2],after[2])*4096
                    if kept:assert memory(after[3],kept)==pending['data_before'][:kept]
                    if after[2]>before[2]:zeros(after[3]+before[2]*4096,(after[2]-before[2])*4096)
                else:
                    assert result==1 and after==[0,0,0,0] and now==[0,0,0]
                for i,pte in enumerate(pages):
                    expected=(after[3]+i*4096)|0x8000000000000007 if i<after[2] else 0
                    assert pte & ~0x60==expected,(i,hex(pte),hex(expected))
                if after[2]:
                    issuer,identity,slot=now
                    assert issuer==1 and identity>0
                    reg=words(address('topal_x86_frame_registry_v1')+slot*64,8)
                    assert reg[:2]==[issuer,identity] and reg[3]==after[2] and reg[6:]==[1,1]
                    start=reg[2]
                    data=words(address('topal_x86_frame_table_v1')+128+start*24,after[2]*3)
                    assert data[0::3]==[identity]*after[2]
                    assert data[2::3]==list(range(after[3]//4096,after[3]//4096+after[2]))
                if before[2] and now!=pending['token_before']:
                    # The old physical extent is retired before it becomes free.
                    zeros(before[3],before[2]*4096)
                    old=pending['token_before'];n=word(address('topal_x86_frame_context_v1')+24)
                    registry=words(address('topal_x86_frame_registry_v1'),n*8)
                    assert all(registry[i+7]!=1 or registry[i+1]!=old[1] for i in range(0,len(registry),8))
                pending['after']=after;pending['token_after']=now;pending['result']=result
                pending['data_before_sha256']=hashlib.sha256(pending.pop('data_before')).hexdigest()
                observed['memory'].append(pending.copy())
            observed['calls'].append(pending);pending=None
        except Exception:
            if pending and isinstance(pending.get('data_before'),bytes):
                pending['data_before_sha256']=hashlib.sha256(pending.pop('data_before')).hexdigest()
            fail()
        return False

class Unexpected(gdb.Breakpoint):
    def stop(self):
        try:raise AssertionError('unexpected memory failure: '+self.location)
        except Exception:fail()
        return True

class Complete(gdb.Breakpoint):
    def stop(self):
        try:
            privileged();assert pending is None
            assert observed['source_trace'] is not None
            assert any(c['op']==7 for c in observed['memory']) and sum(c['op']==6 for c in observed['memory'])==1
            assert len([c for c in observed['memory'] if c['op']==5])>=10
            assert state()==[0,0,0,0]
            zeros(address('topal_x86_frame_mm_tokens_v1'),48)
            n=word(address('topal_x86_frame_context_v1')+24)
            registry=words(address('topal_x86_frame_registry_v1'),n*8)
            live=[registry[i:i+8] for i in range(0,len(registry),8) if registry[i+7]]
            assert len(live)==1 and live[0][3]==5 and live[0][6]==1,live
            start=live[0][2];identity=live[0][1]
            table=words(address('topal_x86_frame_table_v1')+128,n*3)
            assert table[0::3]==[identity if start<=i<start+5 else 0 for i in range(n)]
            assert value('$cr3')==table[start*3+2]*4096
            observed.update(passed=True,n=n,live=live)
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

try:
    Gate('topal_x86_frame_gateway_v1','entry')
    Gate('topal_x86_frame_gateway_return_reclaimed_v1','return')
    for symbol in ['topal_x86_frame_gateway_refused_v1','topal_x86_frame_gateway_fatal_reclaimed_v1','topal_x86_frame_gateway_late_reclaimed_v1']:
        Unexpected('*'+symbol)
    Complete('*topal_x86_systems_fatal')
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    print('FRAME_MEMORY_GDB_PASS',len(observed['memory']))
    gdb.execute('detach')
except Exception:fail()
'''

if __name__ == '__main__':
    harness.main()
