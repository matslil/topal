"""Faults at actual syscall admission and during an owned heap replacement."""
import sys
import frame_root_inspect as harness


harness.GDB_PROGRAM = r'''
import gdb,json,traceback,hashlib

observed=dict(passed=False,mode=mode)
pending=None
injected=False
watchers=[]
preserved=['rbx','rbp','r12','r13','r14','r15']
refusals={'return-site','gs-context','address-space','entry-identity','abi-number',
          'captured-stack','request','busy','direction-flag'}
def value(s):return int(gdb.parse_and_eval(s))
def address(s):return value('(unsigned long long)&'+s)
def memory(p,n):return bytes(gdb.selected_inferior().read_memory(p,n))
def words(p,n):
    b=memory(p,n*8)
    return [int.from_bytes(b[i:i+8],'little') for i in range(0,len(b),8)]
def word(p):return words(p,1)[0]
def put(p,n):gdb.selected_inferior().write_memory(p,int(n).to_bytes(8,'little'))
def digest(p,n):return hashlib.sha256(memory(p,n)).hexdigest()
def zeros(p,n):assert memory(p,n)==bytes(n),(hex(p),n)
def context():return address('topal_bootstrap_storage')+512
def ptes():
    c=context()
    root=observed.get('static_root',word(c+24))
    return words(word(c+112)+0xfb8,9)+words(root+0x4000,510)
def snapshot():
    n=word(address('topal_x86_frame_context_v1')+24)
    state=words(address('topal_x86_frame_mm_state_v1'),4)
    return dict(state=state,tokens=words(address('topal_x86_frame_mm_tokens_v1'),6),
        next_id=word(address('topal_x86_frame_context_v1')+16),ptes=ptes(),
        query_header=words(address('topal_x86_frame_table_v1'),16),
        table=digest(address('topal_x86_frame_table_v1')+128,n*24),
        registry=digest(address('topal_x86_frame_registry_v1'),n*64),
        old_bytes=digest(state[3],state[2]*4096) if state[2] else None)
def fail():
    traceback.print_exc()
    observed.update(pending=pending,trace=gdb.execute('bt',to_string=True))
    with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
    gdb.execute('quit 1')
def retired():
    assert value('$rsp')==pending['sp']
    for r,n in pending['registers'].items():assert value('$'+r)==n,r
    for r in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:assert value('$'+r)==0,r
    zeros(pending['sp']-4080,4080)
    for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),
                      ('details',32),('gateway_recovery',48)]:
        zeros(address('topal_x86_frame_'+name+'_v1'),size)
    assert word(address('topal_frame_allocator_active_v1'))==0
    assert word(address('topal_x86_tail_boundary_active_v1'))==0

def complete_map(boot):
    claims=[]
    for i in range(memory(boot+488,1)[0]):
        a,length=words(boot+720+20*i,2)
        kind=int.from_bytes(memory(boot+736+20*i,4),'little')
        claims.append((a,a+length,kind))
    resource=int.from_bytes(memory(boot+536,4),'little')
    length=int.from_bytes(memory(boot+540,4),'little')
    reserves=[(0,address('topal_x86_kernel_reservation_end_v1')),(resource,resource+length)]
    cuts=sorted({v for a,b,_ in claims for v in (a,b)}|{v for a,b in reserves for v in (a,b)})
    frames=[]
    for a,b in zip(cuts,cuts[1:]):
        ram=any(k==1 and start<=a and b<=end for start,end,k in claims)
        blocked=any(k!=1 and start<b and a<end for start,end,k in claims)
        reserved=any(start<b and a<end for start,end in reserves)
        if ram and not blocked and not reserved:frames.extend(range((a+4095)//4096,b//4096))
    n=word(address('topal_x86_frame_context_v1')+24)
    cells=words(address('topal_x86_frame_table_v1')+128,n*3)
    assert cells[2::3]==frames
    run=largest=0;previous=None
    for i in range(n):
        identity,provenance,physical=cells[i*3:i*3+3]
        if identity==0 and provenance>0:
            run=run+1 if previous==(provenance,physical-1) else 1
            largest=max(largest,run);previous=(provenance,physical)
        else:run=0;previous=None
    assert largest<519,largest
    return dict(claims=claims,reservations=reserves,frames=frames,complete=True,
                largest_free_extent_pages=largest)

class Gate(gdb.Breakpoint):
    def __init__(self,name,role):super().__init__('*'+name);self.role=role
    def stop(self):
        global pending,injected
        try:
            if self.role=='entry':
                assert pending is None
                pending=dict(op=value('$rdi'),request=value('$rsi')&((1<<64)-1),sp=value('$rsp'),
                             registers={r:value('$'+r) for r in preserved})
                if pending['op']==0:observed['boot_params']=pending['request']
                if pending['op']==5 and not injected:
                    c=context();epoch=address('topal_x86_frame_mm_epoch_v1')
                    if mode in refusals:
                        if mode=='return-site':put(pending['sp'],word(pending['sp'])+1)
                        elif mode=='gs-context':gdb.execute('set $gs_base = 0')
                        elif mode=='address-space':
                            observed['static_root']=word(c+24)
                            assert value('$cr3')==observed['static_root']
                            put(c+24,word(c+16)) # a different address space's context
                        elif mode=='entry-identity':put(epoch,0)
                        elif mode=='abi-number':put(epoch+8,59)
                        elif mode=='captured-stack':put(c+8,word(c+8)+16)
                        elif mode=='request':gdb.execute('set $rsi = 1')
                        elif mode=='busy':put(address('topal_x86_frame_gateway_recovery_v1')+8,85)
                        elif mode=='direction-flag':gdb.execute('set $eflags = $eflags | 1024')
                        observed['before']=snapshot()
                        observed['recovery']=words(address('topal_x86_frame_gateway_recovery_v1'),6)
                        injected=True
                    elif mode in {'partial-source','published-unpin'} and pending['request']==0x63a000:
                        assert words(address('topal_x86_frame_mm_state_v1'),4)[2]==34
                        observed['before']=snapshot()
                        if mode=='partial-source':prepare_source_fault()
                        pin.enabled=mode=='published-unpin'
                    elif mode=='no-fit' and pending['request']==0x7fe000:
                        observed['before']=snapshot()
                        assert observed['before']['state'][2]==67
                        observed['map']=complete_map(observed['boot_params'])
                        injected=True # observation only; no memory/register/map edits
                return False
            if self.role=='return':
                if mode=='no-fit' and injected:
                    retired()
                    assert value('$rax')==observed['before']['state'][1]
                    after=snapshot();observed['after']=after.copy()
                    header=after['query_header']
                    assert header[1]==519 and header[5]==4 and header[11]==0
                    after['query_header']=observed['before']['query_header']
                    assert after==observed['before']
                    observed.update(passed=True,ordinary_no_fit=True,guest_state_modified=False)
                    return True
                assert not injected,'faulted request returned to its caller'
                pending=None
                return False
            assert pending is not None and injected
            if self.role=='refused':
                assert mode in refusals
                assert snapshot()==observed['before']
                assert words(address('topal_x86_frame_gateway_recovery_v1'),6)==observed['recovery']
                assert value('$rsp')==pending['sp']
                for r,n in pending['registers'].items():assert value('$'+r)==n,r
                assert word(address('topal_x86_frame_context_v1'))==1
                assert word(address('topal_frame_allocator_active_v1'))==0
                observed.update(passed=True,refused_without_retirement_claim=True)
            else:
                assert self.role=='fatal' and mode in {'partial-source','published-unpin'}
                assert word(address('topal_x86_frame_context_v1'))==3
                assert word(address('topal_x86_frame_mm_state_v1'))==3
                retired()
                actual=snapshot();expected=observed['effects']
                actual['state'][0]=expected['state'][0] # only the owner poison is added
                assert actual==expected,(actual,expected)
                observed.update(passed=True,performed_effects_retained=True,
                                physical_caller_retired=True,process_not_resumed=True)
            return True
        except Exception:fail()
        return False

class SourceFault(gdb.Breakpoint):
    def stop(self):
        global injected
        try:
            table=address('topal_x86_frame_table_v1')
            n=word(address('topal_x86_frame_context_v1')+24)
            identity=observed['before']['next_id']
            ids=words(table+128,n*3)[0::3]
            cells=[i for i,v in enumerate(ids) if v==identity]
            if cells:
                assert 0<len(cells)<67
                observed.update(partial_cells=cells,effects=snapshot())
                assert observed['effects']['tokens'][3:]==[0,0,0]
                assert observed['effects']['ptes']==observed['before']['ptes']
                gdb.execute('set $rdx = 1') # malformed staged source value
                injected=True;self.enabled=False
        except Exception:fail()
        return False
source=SourceFault('*topal_tail_stage_number_v1');source.enabled=False

class FirstStore(gdb.Breakpoint):
    def stop(self):
        source.enabled=True;self.enabled=False
        observed['actual_source_store_observed']=True
        return False

def prepare_source_fault():
    table=address('topal_x86_frame_table_v1')
    n=word(table);data=words(table+128,n*3)
    start=0;length=0;previous=None
    for i in range(n):
        identity,provenance,physical=data[i*3:i*3+3]
        if identity==0 and provenance>0:
            if previous!=(provenance,physical-1):start=i;length=0
            length+=1;previous=(provenance,physical)
            if length==67:
                at=table+128+start*24
                observed['watched_first_fit_cell']=start
                watchers.append(FirstStore(f'*(unsigned long long*){hex(at)}',
                    gdb.BP_WATCHPOINT,wp_class=gdb.WP_WRITE,internal=True))
                return
        else:length=0;previous=None
    raise AssertionError('no real 67-page extent for the partial-store case')

class PublishedFault(gdb.Breakpoint):
    def stop(self):
        global injected
        try:
            if value('$rdx')&1:return False # the new extent's valid pin
            assert value('$rsi')==address('topal_x86_frame_mm_tokens_v1')
            observed['effects']=snapshot()
            assert observed['effects']['tokens'][:3]==observed['before']['tokens'][:3]
            assert observed['effects']['tokens'][4]>0 # candidate remains owned
            assert observed['effects']['ptes']!=observed['before']['ptes']
            assert sum(bool(p) for p in observed['effects']['ptes'])==67
            gdb.execute('set $rsi = '+str(address('topal_x86_frame_mm_tokens_v1')+8))
            injected=True;self.enabled=False
        except Exception:fail()
        return False
pin=PublishedFault('*topal_frame_allocator_pin_v1');pin.enabled=False

try:
    Gate('topal_x86_frame_gateway_v1','entry')
    Gate('topal_x86_frame_gateway_return_reclaimed_v1','return')
    Gate('topal_x86_frame_gateway_refused_v1','refused')
    Gate('topal_x86_frame_gateway_fatal_reclaimed_v1','fatal')
    gdb.execute('continue')
    assert observed['passed']
    with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
    print('FRAME_ROOT_GDB_PASS')
    print('FRAME_MEMORY_FAULT_GDB_PASS',mode)
    gdb.execute('detach')
except Exception:fail()
'''

if __name__ == '__main__':
    mode = sys.argv.pop()
    assert mode in {'return-site', 'gs-context', 'address-space', 'entry-identity',
                    'abi-number', 'captured-stack', 'request', 'busy',
                    'direction-flag', 'partial-source', 'published-unpin', 'no-fit'}
    harness.main(work_name='memory-' + mode, program_prefix=f'mode={mode!r}\n',
                 expect_backing=False, ram_mib=50 if mode=='no-fit' else None)
