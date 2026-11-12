"""Read-only observations of ordinary syscall buffers using the shared adapter."""
from pathlib import Path
import json
import sys

import user_extent_inspect

harness = user_extent_inspect.harness
common, separator, ending = harness.GDB_PROGRAM.partition('\ntry:\n    Gate(')
assert separator
start = common.index('class NativeTrace(')
end = common.index('MemoryGate=Gate', start)
common = common[:start] + common[end:]
common = common.split('MemoryComplete=Complete', 1)[0]
assert common.count('epoch[1]==12') == 1
common = common.replace('epoch[1]==12', '0<=epoch[1]<(1<<32)')
assert common.count('-4080,4080') == 2
common = common.replace('-4080,4080', '-4000,4000')
# Kernel ELF symbol locations are immutable throughout this VM. Resolve those
# locations once; every register, table, token, owner and byte observation at
# the resulting locations remains fresh, including across user exec entries.
old_address = "def address(s):return value('(unsigned long long)&'+s)"
assert common.count(old_address) == 1
common = common.replace(old_address, """symbol_addresses={}
def address(s):
    if s not in symbol_addresses:
        symbol_addresses[s]=value('(unsigned long long)&'+s)
    return symbol_addresses[s]""")
# Keep fresh complete owner/table/mode observations on every call. For the
# numerous single-byte pathname checks, fingerprint the covered user bytes;
# also retain whole-backing samples for each ABI/direction and the new buffers.
# The predecessor observer retains its original whole-backing behavior.
for name, expression, base, size, user_base in [
    ('image', "memory(roles[2]['physical'],503*4096)", "roles[2]['physical']", '503*4096', '0x400000'),
    ('stack', "memory(roles[3]['physical'],8192)", "roles[3]['physical']", '8192', '0x7fe000'),
    ('heap', "memory(st[3],st[2]*4096) if st[2] else b''", 'st[3]', 'st[2]*4096', '0x5f7000'),
]:
    old=f'{name}_sha256=hashlib.sha256({expression}).hexdigest()'
    assert common.count(old)==1, name
    common=common.replace(old, f'{name}_fingerprint=backing_fingerprint({base},{size},{user_base})')
harness.GDB_PROGRAM = common + r'''
import gzip
helper_pending=None
caller_return_points={}
sample_whole_backing=False
whole_backing_keys=set()
extent_records=gzip.open(observation_path+'.extent-records.jsonl.gz','wt',encoding='utf-8')
observed['extent_records_file']=observation_path+'.extent-records.jsonl.gz'
observed['whole_backing_sample_count']=0
observed['helper_calls']=[]
observed['process_cleanup_checked']=False
observed['startup_outputs_checked']=False
observed['shell_outputs_checked']=False
observed['stat_outputs_checked']=False
observed['stat_staging_samples']=[]
observed['statat_paths_checked']=False
observed['statat_staging_samples']=[]
observed['statat_user_results']=[]
observed['write_staging_samples']=[]
observed['write_user_results']=[]
observed['file_staging_samples']=[]
observed['file_user_results']=[]
observed['command_user_results']=[]
command_case_by_page={row['page']:i for i,row in enumerate(command_cases)}

observed['shell_io_user_results']=[]
shell_io_case_by_page={row['page']:i for i,row in enumerate(shell_io_cases)}
shell_io_argument_registers={2:'rdx',6:'rsi',7:'rdi'}
file_case_by_page={row['page']:i for i,row in enumerate(file_cases)}
observed['symbol_resolution_scope']='Only immutable linked kernel ELF symbol locations are memoized; all runtime contents and registers are freshly read'
observed['scope']='Private shared-caller migration with ordinary ABI buffers; direct callers and canonical publication remain unqualified'
observed['byte_observation_scope']='Fresh complete ownership/translation state for every call; covered bytes on every call and whole backing for each ABI/direction plus every new split-page/cross-table buffer'
helper_registers=preserved+['rdx','rsi','rdi','r8','r9']

def backing_fingerprint(base,size,user_base):
    scope='whole-backing' if sample_whole_backing else 'covered-bytes'
    start=helper_pending['start'];length=helper_pending['length']
    if sample_whole_backing:offset,count=0,size
    elif start+length>1<<64:offset,count=0,0
    else:
        lower=max(start,user_base);upper=min(start+length,user_base+size)
        offset,count=(lower-user_base,upper-lower) if lower<upper else (0,0)
    return dict(scope=scope,physical_start=base+offset,byte_count=count,
                sha256=hashlib.sha256(memory(base+offset,count) if count else b'').hexdigest())

base_fail=fail
def fail():
    observed['helper_pending']=helper_pending
    if not extent_records.closed:extent_records.flush()
    base_fail()

user_scope_active=None
user_scope_entry=None
user_scope_points={}
user_scope_exits={}
observed['assertion_scopes_entered']=0
observed['assertion_scopes_completed']=0
observed['user_assertions_checked']=0
observed['repeated_entry_notifications']=[]

def close_user_scope(page):
    global user_scope_active,user_scope_entry
    if user_scope_active is None:return
    assert user_scope_active==page
    for point in user_scope_points[page]:point.enabled=False
    user_scope_exits[page].enabled=False
    user_scope_active=None
    user_scope_entry=None
    observed['assertion_scopes_completed']+=1

class UserAssertion(gdb.Breakpoint):
    def stop(self):
        try:
            assert user_scope_active==value('$rip')&-4096
            observed['user_assertions_checked']+=1
            if user_scope_active in command_case_by_page:
                case=command_case_by_page[user_scope_active]
                site=value('$rip')
                number=words(address('topal_x86_frame_mm_epoch_v1'),2)[1]
                if number==0 and value('$rcx')==site-6:
                    assert value('$cs')&3==3 and memory(site-6,2)==bytes.fromhex('483d')
                    c=context()
                    assert memory(c+296,120)==bytes(120)
                    state=word(c+904)+0x9e00
                    cursor,length,match=words(state+256,3)
                    assert length<=256 and memory(state+length,256-length)==bytes(256-length)
                    result=value('$rax')&((1<<64)-1)
                    if result>>63:result-=1<<64
                    observed['command_user_results'].append(dict(case=case,site=site,
                        pointer=value('$rsi'),count=value('$rdx'),result=result,cursor=cursor,
                        pending=memory(state,length).hex() if length else '',prefix=match,
                        phase=memory(c+104,1)[0],owner=memory(c+418,1)[0]))
                    print('COMMAND_READ_RESULT',case,hex(value('$rsi')),value('$rdx'),result,flush=True)
            if user_scope_active in stat_pages or user_scope_active in statat_pages:
                assert value('$cs')&3==3
                c=context();console=memory(c+472,144);other=memory(c+744,144)
                assert console==other==bytes(144)
                is_stat=user_scope_active in stat_pages
                cases=stat_pages if is_stat else statat_pages
                field='stat_staging_samples' if is_stat else 'statat_staging_samples'
                observed[field].append(dict(
                    case=cases.index(user_scope_active),site=value('$rip'),
                    console_sha256=hashlib.sha256(console).hexdigest(),
                    other_sha256=hashlib.sha256(other).hexdigest()))
                site=value('$rip')
                if not is_stat and words(address('topal_x86_frame_mm_epoch_v1'),2)[1]==262 and memory(site-6,2)==bytes.fromhex('483d'):
                    result=value('$rax')&((1<<64)-1)
                    if result>>63:result-=1<<64
                    observed['statat_user_results'].append(dict(case=cases.index(user_scope_active),site=site,result=result))
            if user_scope_active in write_pages:
                assert value('$cs')&3==3
                case=write_pages.index(user_scope_active)
                c=context()
                banner=memory(c+624,112)
                transfer=memory(c+744,144)
                assert banner==bytes(112) and transfer==bytes(144),case
                site=value('$rip')
                observed['write_staging_samples'].append(dict(case=case,site=site,
                    banner_sha256=hashlib.sha256(banner).hexdigest(),
                    transfer_sha256=hashlib.sha256(transfer).hexdigest()))
                if words(address('topal_x86_frame_mm_epoch_v1'),2)[1]==1 and memory(site-6,2)==bytes.fromhex('483d'):
                    result=value('$rax')&((1<<64)-1)
                    if result>>63:result-=1<<64
                    observed['write_user_results'].append(dict(case=case,site=site,result=result))
            if user_scope_active in shell_io_case_by_page:
                case=shell_io_case_by_page[user_scope_active]
                config=shell_io_cases[case]
                site=value('$rip')
                number=words(address('topal_x86_frame_mm_epoch_v1'),2)[1]
                if number==config['number'] and value('$rcx')==site-6:
                    assert value('$cs')&3==3
                    assert memory(site-6,2)==bytes.fromhex('483d')
                    c=context()
                    assert memory(c+296,120)==bytes(120)
                    assert memory(c+744,144)==bytes(144)
                    result=value('$rax')&((1<<64)-1)
                    if result>>63:result-=1<<64
                    pointer=value('$'+shell_io_argument_registers[config['argument']])&((1<<64)-1)
                    state_after=dict(phase=memory(c+104,1)[0],
                        action=memory(c+417,1)[0],header=list(memory(c+888,5)),
                        child=[word(c+offset) for offset in [168,176,224,232]])
                    observed['shell_io_user_results'].append(dict(case=case,site=site,
                        number=number,pointer=pointer,result=result,state=state_after))
                    print('SHELL_IO_SYSCALL_RESULT',case,number,hex(pointer),result,flush=True)
            if user_scope_active in file_case_by_page:
                assert value('$cs')&3==3
                case=file_case_by_page[user_scope_active]
                c=context()
                first,second=memory(c+472,144),memory(c+744,144)
                assert first==second==bytes(144),case
                site=value('$rip')
                observed['file_staging_samples'].append(dict(case=case,site=site,
                    first_sha256=hashlib.sha256(first).hexdigest(),
                    second_sha256=hashlib.sha256(second).hexdigest()))
                number=words(address('topal_x86_frame_mm_epoch_v1'),2)[1]
                # SYSRET restores the original arguments and clears the
                # kernel capture slots. Observe the restored user registers.
                if number==file_cases[case]['number'] and value('$rcx')==site-6:
                    assert memory(site-6,2)==bytes.fromhex('483d')
                    assert memory(c+296,120)==bytes(120)
                    result=value('$rax')&((1<<64)-1)
                    if result>>63:result-=1<<64
                    arguments=[value('$'+name)&((1<<64)-1) for name in ['rdi','rsi','rdx']]
                    observed['file_user_results'].append(dict(case=case,site=site,result=result,
                        number=number,arguments=arguments))
                    print('FILE_SYSCALL_RESULT',case,number,hex(arguments[1]),result,flush=True)
        except Exception:fail()
        if value('$eflags')&64:return False
        observed['ordinary_user_assertion_failure']=dict(
            registers={name:value('$'+name) for name in ['rip','rax','rdi','rsi','rdx','eflags']},
            entry=words(address('topal_x86_frame_mm_epoch_v1'),2),
            phase=memory(context()+104,1)[0],helper=helper_pending)
        try:raise AssertionError(observed['ordinary_user_assertion_failure'])
        except Exception:fail()
        return True
for site in user_assertions:
    point=UserAssertion('*'+hex(site),internal=True);point.enabled=False
    user_scope_points.setdefault(site&-4096,[]).append(point)

class UserScopeExit(gdb.Breakpoint):
    def __init__(self,page,site):
        super().__init__('*'+hex(site),internal=True);self.page=page;self.enabled=False
    def stop(self):
        try:close_user_scope(self.page)
        except Exception:fail()
        return False

class UserScopeEnter(gdb.Breakpoint):
    def __init__(self,page):super().__init__('*'+hex(page),internal=True);self.page=page
    def stop(self):
        global user_scope_active,user_scope_entry
        try:
            cs=value('$cs')
            entry=dict(page=self.page,sp=value('$rsp'),cs=cs,
                       epoch=words(address('topal_x86_frame_mm_epoch_v1'),2),
                       assertions=observed['user_assertions_checked'])
            assert cs&3==3,entry
            if user_scope_active==self.page:
                # Resuming a breakpoint while arming its assertion sites can
                # produce another stop at the same unadvanced entry.
                assert entry==user_scope_entry,(entry,user_scope_entry)
                assert not observed['repeated_entry_notifications'] or observed['repeated_entry_notifications'][-1]!=entry,entry
                observed['repeated_entry_notifications'].append(entry)
                return False
            assert user_scope_active is None,(entry,user_scope_entry)
            # These are the fixture's actual two-byte assertion instructions.
            # A block entry rearms them on every invocation/generation; its
            # final assertion continuation disarms them only after execution.
            for point in user_scope_points[self.page]:
                site=int(point.location[1:],16)
                assert memory(site,2)==bytes.fromhex('75fe')
                point.enabled=True
            user_scope_exits[self.page].enabled=True
            user_scope_active=self.page
            user_scope_entry=entry
            observed['assertion_scopes_entered']+=1
        except Exception:fail()
        return False

for page in user_scope_points:
    last=max(site for site in user_assertions if site&-4096==page)
    user_scope_exits[page]=UserScopeExit(page,last+2)
    UserScopeEnter(page)

class CallerReturned(gdb.Breakpoint):
    def stop(self):
        global helper_pending
        # A Python stop callback that resumes execution does not consume a
        # temporary breakpoint. Disable this observation explicitly, then
        # rearm the same object at its next call instead of accumulating
        # thousands of disabled observers for byte-by-byte pathname checks.
        self.enabled=False
        try:
            assert helper_pending and helper_pending.get('returned')
            assert value('$rsp')==helper_pending['sp']+8
            assert word(helper_pending['sp'])==0
            assert bool(value('$eflags')&1)==bool(helper_pending['result'])
            for name,number in helper_pending['registers'].items():assert value('$'+name)==number,name
            helper_pending['caller_return_erased']=True
            observed['helper_calls'].append(helper_pending);helper_pending=None
        except Exception:fail()
        return False

class Helper(gdb.Breakpoint):
    def __init__(self,symbol,direction,returning):
        super().__init__('*'+symbol);self.direction=direction;self.returning=returning
    def stop(self):
        global helper_pending
        try:
            privileged()
            if not self.returning:
                assert helper_pending is None
                sp=value('$rsp');site=word(sp)
                assert memory(site,9)==bytes.fromhex('48c74424f800000000')
                helper_pending=dict(sp=sp,site=site,direction=self.direction,
                    start=value('$r8')&((1<<64)-1),length=value('$r9')&((1<<64)-1),
                    registers={r:value('$'+r) for r in helper_registers})
                if site not in caller_return_points:
                    caller_return_points[site]=CallerReturned('*'+hex(site+9),internal=True)
                else:caller_return_points[site].enabled=True
            else:
                assert helper_pending and pending is None and self.direction==helper_pending['direction']
                assert value('$rsp')==helper_pending['sp']
                for name,number in helper_pending['registers'].items():assert value('$'+name)==number,name
                result=value('$rax');assert result==helper_pending['native_result']
                assert bool(value('$eflags')&1)==bool(result)
                gate=helper_pending['gateway_sp']
                zeros(gate-4000,4000)
                zeros(gate,helper_pending['sp']-gate)
                assert word(helper_pending['sp'])==helper_pending['site']
                helper_pending.update(returned=True,result=result)
        except Exception:fail()
        return False

ExtentGate=Gate
class Gate(ExtentGate):
    def stop(self):
        global sample_whole_backing
        is_extent=(self.role=='entry' and value('$rdi')==8) or (self.role!='entry' and pending and pending['op']==8)
        if is_extent and self.role=='entry':
            try:
                assert helper_pending and 'gateway_sp' not in helper_pending
                assert value('$rsi')&((1<<64)-1)==helper_pending['start']
                assert value('$rdx')&((1<<64)-1)==helper_pending['length']
                assert value('$rcx')==helper_pending['direction']
                helper_pending['gateway_sp']=value('$rsp')
                helper_pending['epoch']=words(address('topal_x86_frame_mm_epoch_v1'),2)
                key=(helper_pending['epoch'][1],helper_pending['direction'])
                sample_whole_backing=key not in whole_backing_keys or (
                    (helper_pending['start'],helper_pending['length']) in [
                        (0x5ffff8,16),(0x638ff8,16),(0x638ffc,8),(0x638ff8,390),(0x638fff,2),(0x638ff8,144),
                        (0x637ff8,144),(0x638ffc,5),(0x638ffc,13),(0x638ffc,18),(0x638ffc,21)]
                    or (helper_pending['epoch'][1]==1 and helper_pending['start']==0x638fff)
                    or (helper_pending['epoch'][1] in [217,257] and
                        helper_pending['start'] in [0x638fff,0x638ff8,0x639f80]))
            except Exception:fail()
        result=super().stop()
        if is_extent and self.role=='return':
            call=observed['user_extents'][-1]
            if call['epoch'][1] in [63,79]:call['process_phase']=call['before']['phase']
            if call['epoch'][1]==5 and call['before']['captured'][0]&-4096 in stat_pages:
                call['stat_fixture_case']=stat_pages.index(call['before']['captured'][0]&-4096)
            if call['epoch'][1]==262 and call['before']['captured'][0]&-4096 in statat_pages:
                call['statat_fixture_case']=statat_pages.index(call['before']['captured'][0]&-4096)
            if call['epoch'][1]==1 and call['before']['captured'][0]&-4096 in write_pages:
                call['write_fixture_case']=write_pages.index(call['before']['captured'][0]&-4096)
            if call['before']['captured'][0]&-4096 in file_case_by_page:
                case=file_case_by_page[call['before']['captured'][0]&-4096]
                assert call['epoch'][1]==file_cases[case]['number']
                call['file_fixture_case']=case
            if call['before']['captured'][0]&-4096 in shell_io_case_by_page:
                case=shell_io_case_by_page[call['before']['captured'][0]&-4096]
                assert call['epoch'][1]==shell_io_cases[case]['number']
                call['shell_io_fixture_case']=case
            if call['before']['captured'][0]&-4096 in command_case_by_page:
                case=command_case_by_page[call['before']['captured'][0]&-4096]
                assert call['epoch'][1]==0
                call['command_fixture_case']=case
            helper_pending['native_result']=call['result']
            # Archive every full observation after its independent before/after
            # comparison; keep only its identity in the in-memory history.
            # Otherwise thousands of byte checks retain duplicate page tables.
            record=json.dumps(call,sort_keys=True,separators=(',',':'))
            extent_records.write(record+'\n')
            call['before']=dict(archive_record=len(observed['user_extents'])-1,
                record_sha256=hashlib.sha256(record.encode()).hexdigest(),
                byte_scope='whole-backing' if sample_whole_backing else 'covered-bytes')
            if sample_whole_backing:observed['whole_backing_sample_count']+=1
            whole_backing_keys.add((helper_pending['epoch'][1],helper_pending['direction']))
            if len(observed['user_extents'])%64==0:
                print('CALLER_PROGRESS',len(observed['user_extents']),flush=True)
            sample_whole_backing=False
        return result

class NativeTrace(gdb.Breakpoint):
    def stop(self):
        try:
            assert pending and pending['op']==8 and helper_pending
            trace=gdb.execute('bt',to_string=True)
            for name in ['topal_x86_owned_user_extent_v1','topal_frame_kernel_dispatch_v1',
                         'topal_x86_frame_gateway_v1','topal_x86_user_extent_call_v1',
                         'topal_x86_systems_static_image_process_enter']:
                assert name in trace,(name,trace)
            frame=gdb.newest_frame()
            while frame and frame.name() not in ['topal_x86_user_extent_readable_v1','topal_x86_user_extent_writable_v1']:
                frame=frame.older()
            assert frame and frame.older(),trace
            for name in preserved:assert int(frame.older().read_register(name))==helper_pending['registers'][name],name
            sp=helper_pending['sp'];r=helper_pending['registers']
            assert words(sp-48,5)==[r[n] for n in ['r9','r8','rdi','rsi','rdx']]
            assert word(sp-8)==r['rbp']
            observed['native_extent_trace']=trace;observed['native_extent_trace_samples']=1
            self.enabled=False
        except Exception:fail()
        return False

BaseComplete=Complete

def caller_history_checked():
    assert user_scope_active is None
    assert observed['assertion_scopes_entered']==observed['assertion_scopes_completed']
    assert observed['user_assertions_checked']>0
    assert helper_pending is None and pending is None and observed['native_extent_trace']
    assert len(observed['helper_calls'])==len(observed['user_extents'])>=4
    actual_return_observers=sum(isinstance(b,CallerReturned) for b in gdb.breakpoints())
    assert actual_return_observers==len({h['site'] for h in observed['helper_calls']})
    observed['return_observer_sites']=actual_return_observers
    assert observed['wp_transitions'] and value('$cr0')&(1<<16)
    observed['shared_caller_registers_and_retirement_checked']=True

def startup_outputs_checked():
    pairs={(c['start'],c['length'],c['write'],c['epoch'][1],c['result']) for c in observed['user_extents']}
    for number,length,valid in [(302,16,0x638ff8),(318,8,0x638ffc),(157,16,0x638ff8)]:
        for pointer,status in [(valid,0),(0x63a000,2),(0x401000,3)]:
            expected=(pointer,length,1,number,status)
            assert expected in pairs,(expected,pairs)
    observed['startup_outputs_checked']=True

def shell_outputs_checked(full):
    calls=observed['user_extents']
    phases=[19,20,26,30] if full else [19,20]
    for phase in phases:
        number,length,valid,partial=(63,390,0x638ff8,0x639ff8) if phase==19 else (79,2,0x638fff,0x639fff)
        for pointer,status in [(valid,0),(0x63a000,2),(0x401000,3),(partial,2)]:
            assert any(c.get('process_phase')==phase and c['epoch'][1]==number and
                       (c['start'],c['length'],c['write'],c['result'])==(pointer,length,1,status)
                       for c in calls),(phase,number,hex(pointer),status)
    observed['shell_output_phases_checked']=phases
    observed['shell_outputs_checked']=True

def stat_outputs_checked(full):
    cases=[0,1,2,3] if full else [0]
    for case in cases:
        calls=[c for c in observed['user_extents'] if c.get('stat_fixture_case')==case]
        assert len(calls)==4,(case,len(calls))
        for pointer,status in [(0x638ff8,0),(0x63a000,2),(0x401000,3),(0x639ff8,2)]:
            assert any((c['start'],c['length'],c['write'],c['result'])==(pointer,144,1,status)
                       for c in calls),(case,hex(pointer),status)
        assert sum(s['case']==case for s in observed['stat_staging_samples'])>=11
    observed['stat_fixture_cases_checked']=cases
    observed['stat_outputs_checked']=True
    observed['stat_output_scope']='Four ordinary fixture descriptor paths; external-shell pwd dispatch guards and newfstatat remain separately unqualified'

def statat_paths_checked(full):
    cases=list(range(6)) if full else [0]
    for case in cases:
        calls=[c for c in observed['user_extents'] if c.get('statat_fixture_case')==case]
        length=[18,21,5,13,13,13][case]
        for pointer,status in [(0x63a000,2),(0x639ffe,2),(0x638ffc,0)]:
            assert any((c['start'],c['length'],c['write'],c['result'])==(pointer,length,0,status) for c in calls),(case,hex(pointer),status)
        if case<2:
            assert all(c['write']==0 for c in calls),case
            expected=-2
        else:
            for pointer,status in [(0x637ff8,0),(0x63a000,2),(0x401000,3),(0x639ff8,2)]:
                assert any((c['start'],c['length'],c['write'],c['result'])==(pointer,144,1,status) for c in calls),(case,hex(pointer),status)
            expected=0
        results={r['result'] for r in observed['statat_user_results'] if r['case']==case}
        assert {-14,expected}<=results,(case,results)
        assert sum(s['case']==case for s in observed['statat_staging_samples'])>=7
    observed['statat_fixture_cases_checked']=cases
    observed['statat_paths_checked']=True
    observed['statat_scope']='Six ordinary fixture paths; external-shell missing-root dispatch remains separately unqualified'

def write_inputs_checked(full):
    cases=list(range(len(write_pages))) if full else [0]
    for case in cases:
        count=write_lengths[case]
        calls=[c for c in observed['user_extents'] if c.get('write_fixture_case')==case]
        outcomes={(c['start'],c['length'],c['result']) for c in calls}
        assert calls and all(c['write']==0 for c in calls),case
        assert {(0x63a000,count,2),(0x638fff,count,0)}<=outcomes,(case,outcomes)
        if count>1:assert (0x639fff,count,2) in outcomes,(case,outcomes)
        assert all(c['length']==count for c in calls),case
        results={row['result'] for row in observed['write_user_results'] if row['case']==case}
        assert {-14,count}<=results,(case,results)
        assert sum(s['case']==case for s in observed['write_staging_samples'])>=7,case
    observed['write_fixture_cases_checked']=cases

def file_transfers_checked():
    assert len(file_cases)==11
    for case,config in enumerate(file_cases):
        number,count,result=(config[k] for k in ('number','count','result'))
        calls=[c for c in observed['user_extents'] if c.get('file_fixture_case')==case]
        actual=[(c['start'],c['length'],c['write'],c['result']) for c in calls]
        results=[r for r in observed['file_user_results'] if r['case']==case]
        if number==257:
            expected=[(0x63a000,count,0,2),(0x639fff,count,0,2),(0x638fff,count,0,0)]
            returns=[(0x63a000,-14),(0x639fff,-14),(0x638fff,result)]
            assert all(r['arguments'][0]&0xffffffff==0xffffff9c for r in results)
        elif count:
            assert number==217 and count in [72,80]
            start=0x638ff8 if count==72 else 0x639f80
            expected=[(0x63a000,count,1,2),(0x401000,count,1,3),(0x639ff8,count,1,2),(start,count,1,0)]
            returns=[(0x63a000,-14),(0x401000,-14),(0x639ff8,-14),(start,result)]
        else:
            expected=[]
            returns=[(0x63a000,0)]
            if number==0:returns.insert(0,((1<<47)-4096-127,-14))
        if number in [0,217]:
            assert all(r['arguments'][0]==3 and r['arguments'][2]==(128 if number==0 else 512) for r in results)
        assert actual==expected,(case,actual,expected)
        assert [(r['arguments'][1],r['result']) for r in results]==returns,(case,results,returns)
        assert sum(s['case']==case for s in observed['file_staging_samples'])>=len(returns)+1
    observed['file_fixture_cases_checked']=list(range(len(file_cases)))

def shell_io_checked():
    assert len(shell_io_cases)==15
    for case,config in enumerate(shell_io_cases):
        number,count,direction,result=(config[k] for k in ('number','count','direction','result'))
        calls=[c for c in observed['user_extents'] if c.get('shell_io_fixture_case')==case]
        actual=[(c['start'],c['length'],c['write'],c['result']) for c in calls]
        results=[r for r in observed['shell_io_user_results'] if r['case']==case]
        expected=[]
        if direction is None:
            assert count==0 and number==16 and result==-25
            returns=[(0x63a000,-25)]
        else:
            assert direction in [0,1] and count in [1,2,32]
            invalid=[(0x63a000,2)]
            if direction==1:invalid.append((0x401000,3))
            if count>1:invalid.append((0x639fff,2))
            expected=[(start,count,direction,status) for start,status in invalid]
            expected.append((0x638fff,count,direction,0))
            returns=[(start,-14) for start,status in invalid]+[(0x638fff,result)]
            assert len({json.dumps(r['state'],sort_keys=True) for r in results[:-1]})==1
            if number==0:
                assert results[-1]['state']['header'][1]==results[0]['state']['header'][1]+1
        assert actual==expected,(case,actual,expected)
        assert [(r['pointer'],r['result']) for r in results]==returns,(case,results,returns)
    observed['shell_io_fixture_cases_checked']=list(range(len(shell_io_cases)))

def command_reads_checked():
    assert [r['length'] for r in command_cases]==[11,5,294,24,len(serial_command)]
    for case,spec in enumerate(command_cases):
        length,capacity=spec['length'],spec['capacity']
        suffix=length-2
        calls=[c for c in observed['user_extents'] if c.get('command_fixture_case')==case]
        actual=[(c['start'],c['length'],c['write'],c['result']) for c in calls]
        expected=[(0x63a000,length,1,2),(0x639fff,length,1,2),(0x638fff,2,1,0),
                  (0x63a000,suffix,1,2),(0x63a000-suffix,suffix,1,0)]
        assert actual==expected,(case,actual,expected)
        results=[r for r in observed['command_user_results'] if r['case']==case]
        returns=[(0x63a000,0,0),(0x63a000,capacity,-14),(0x639fff,length,-14),
                 (0x638fff,2,2),(0x63a000,capacity,-14),(0x63a000-suffix,capacity,suffix)]
        assert [(r['pointer'],r['count'],r['result']) for r in results]==returns,(case,results,returns)
        phase=[26,28,29,30,31][case]
        assert [(r['phase'],r['owner']) for r in results]==[(phase,case+1)]*5+[
            ([27,28,29,30,32][case],case+2)]
        if case==4:
            assert [r['cursor'] for r in results]==[0]*6
            expected_pending=[b'',serial_command,serial_command,serial_command[2:],serial_command[2:],b'']
            assert [bytes.fromhex(r['pending']) for r in results]==expected_pending
            assert [r['prefix'] for r in results]==[0,6,6,6,6,6]
        else:
            assert [r['cursor'] for r in results]==[0,0,0,2,2,0]
            assert all(not r['pending'] and not r['prefix'] for r in results)
    observed['command_fixture_cases_checked']=list(range(len(command_cases)))

class Complete(BaseComplete):
    def stop(self):
        try:
            assert not focused,'focused fixture did not reach its user continuation'
            caller_history_checked()
            startup_outputs_checked()
            shell_outputs_checked(True)
            stat_outputs_checked(True)
            statat_paths_checked(True)
            write_inputs_checked(True)
            file_transfers_checked()
            shell_io_checked()
            command_reads_checked()
            pairs={(c['start'],c['length'],c['write'],c['epoch'][1],c['result']) for c in observed['user_extents']}
            for expected in [(0x5ffff8,16,1,229,0),(0x638ff8,16,1,229,0),
                             (0x638ff8,16,0,35,0),(0x638ff8,16,1,228,0)]:
                assert expected in pairs,(expected,pairs)
            extent_records.close()
        except Exception:fail()
        result=super().stop()
        observed['process_cleanup_checked']=True
        with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
        return result

class StartupComplete(gdb.Breakpoint):
    def stop(self):
        try:
            assert startup_only and value('$cs')&3==3
            # This focused endpoint is also the final assertion continuation;
            # close idempotently, independent of breakpoint callback order.
            close_user_scope(startup_completion_site&-4096)
            caller_history_checked()
            startup_outputs_checked()
            c=context();heap=state()
            assert heap[:3]==[1,0x619000,34],heap
            assert memory(c+104,1)[0]==10
            # Refused output requests did not consume the initial seed or a
            # generator result, and the two valid requests consumed two only.
            assert word(c+264)==2 and memory(c+184,1)[0]==3
            for at,want in [(0x7fff18,11),(0x7fff20,0),(0x7fff28,12),(0x7fff30,0),
                            (0x7fff38,13),(0x7fff40,0),(0x7fff48,14),(0x7fff50,0)]:
                assert word(at)==want,('startup metadata',hex(at),word(at),want)
            destination=value('$r12')
            assert words(destination,2)==[int.from_bytes(b'busybox\0','little'),0]
            observed['startup_context']=dict(heap=heap,phase=10,random_requests=2)
            observed['scope']='Focused normal startup output/mapping/adapter component; whole-process cleanup and publication are not checked'
            observed['passed']=True
            extent_records.close()
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

if startup_only:StartupComplete('*'+hex(startup_completion_site),internal=True)

class ShellComplete(gdb.Breakpoint):
    def stop(self):
        try:
            assert shell_only and value('$cs')&3==3
            close_user_scope(shell_completion_site&-4096)
            caller_history_checked()
            startup_outputs_checked()
            shell_outputs_checked(False)
            write_inputs_checked(False)
            assert state()[:3]==[1,0x619000,34]
            assert memory(context()+104,1)[0]==21
            assert memory(value('$r12'),2)==b'/\0'
            observed['scope']='Focused normal startup/uname/initial-getcwd component; later getcwd branches, whole-process cleanup and publication are not checked'
            observed['passed']=True
            extent_records.close()
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

if shell_only:ShellComplete('*'+hex(shell_completion_site),internal=True)

class StatComplete(gdb.Breakpoint):
    def stop(self):
        try:
            assert stat_only and value('$cs')&3==3
            close_user_scope(stat_completion_site&-4096)
            caller_history_checked()
            startup_outputs_checked()
            shell_outputs_checked(False)
            stat_outputs_checked(False)
            assert state()[:3]==[1,0x619000,34]
            c=context();assert memory(c+104,1)[0]==26 and memory(c+616,1)[0]==1
            expected=[0,1,1,0x2180,0,0x4400,0,4096]+[0]*10
            assert words(value('$r12'),18)==expected
            observed['scope']='Focused normal console-fstat/startup component; remaining descriptor cases, whole-process cleanup and publication are not checked'
            observed['passed']=True
            extent_records.close()
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

if stat_only:StatComplete('*'+hex(stat_completion_site),internal=True)

class StatAtComplete(gdb.Breakpoint):
    def stop(self):
        try:
            assert statat_only and value('$cs')&3==3
            close_user_scope(statat_completion_site&-4096)
            caller_history_checked()
            startup_outputs_checked()
            statat_paths_checked(False)
            assert state()[:3]==[1,0x619000,34]
            assert memory(context()+104,1)[0]==12
            assert memory(value('$r12'),144)==bytes([0xa5])*144
            observed['scope']='Focused normal optional-config pathname/absence/error-order component; later stat paths, whole-process cleanup and publication are not checked'
            observed['passed']=True
            extent_records.close()
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

if statat_only:StatAtComplete('*'+hex(statat_completion_site),internal=True)

class ShellIoComplete(gdb.Breakpoint):
    def stop(self):
        try:
            assert shell_io_only and value('$cs')&3==3
            assert memory(command_completion_site-2,2)==bytes.fromhex('75fe')
            close_user_scope(command_completion_site&-4096)
            caller_history_checked()
            startup_outputs_checked()
            shell_outputs_checked(True)
            stat_outputs_checked(True)
            statat_paths_checked(True)
            write_inputs_checked(True)
            file_transfers_checked()
            shell_io_checked()
            command_reads_checked()
            assert state()[:3]==[1,0x619000,34]
            c=context()
            assert memory(c+104,1)[0]==32
            assert memory(c+418,1)[0]==6 and memory(c+743,1)[0]==10
            assert memory(c+296,120)==bytes(120)
            observed['scope']='Ordinary first-image shell calls through serial input'
            observed['focused_command_continuation']=command_completion_site
            observed['passed']=True
            extent_records.close()
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

if shell_io_only:ShellIoComplete('*'+hex(command_completion_site),internal=True)

for direction,name in [(0,'readable'),(1,'writable')]:
    Helper('topal_x86_user_extent_'+name+'_v1',direction,False)
    Helper('topal_x86_user_extent_'+name+'_return_v1',direction,True)
NativeTrace("*'u.owner'")
Unexpected('*topal_x86_user_extent_caller_abort_v1')
''' + separator + ending

if __name__ == '__main__':
    build=Path(sys.argv[1]).resolve()
    bindings=json.loads((build/'user-owners.json').read_text())
    sites=json.loads((build/'caller-fixture-sites.json').read_text())
    selection=json.loads((build/'caller-observation.json').read_text())
    model_dir=Path(__file__).resolve().parents[3]/'tools/user-memory'
    prefix=f'import sys\nsys.path.insert(0,{str(model_dir)!r})\nowner_bindings={bindings!r}\nuser_assertions={sites!r}\nstartup_only={selection["startup_only"]!r}\nstartup_completion_site={selection["startup_completion"]!r}\nshell_only={selection["shell_only"]!r}\nshell_completion_site={selection["shell_completion"]!r}\nstat_only={selection["stat_only"]!r}\nstat_completion_site={selection["stat_completion"]!r}\nstat_pages={selection["stat_pages"]!r}\n'
    prefix+=f'statat_only={selection["statat_only"]!r}\nstatat_completion_site={selection["statat_completion"]!r}\nstatat_pages={selection["statat_pages"]!r}\n'
    prefix+=f'write_pages={selection["write_pages"]!r}\nwrite_lengths={selection["write_lengths"]!r}\n'
    prefix+=f'file_cases={selection["file_cases"]!r}\n'
    prefix+=f'shell_io_cases={selection["shell_io_cases"]!r}\n'
    prefix+=f'command_cases={selection["command_cases"]!r}\nserial_command={harness.topal_gate.PROCESS_COMMAND!r}\n'
    focused=any(selection[name] for name in ['startup_only','shell_only','stat_only','statat_only','shell_io_only'])
    prefix+=f'focused={focused!r}\nshell_io_only={selection["shell_io_only"]!r}\n'
    if selection['shell_io_only']:
        command_page=selection['command_cases'][-1]['page']
        completion=max(site for site in sites if site&-4096==command_page)+2
        prefix+=f'command_completion_site={completion!r}\n'
    harness.main(work_name='user-extent-callers',program_prefix=prefix,
                 input_scope='ordinary split-page and cross-table syscall buffers',
                 expect_backing=not focused,
                 # Four images retain the focused per-image correctness budget.
                 # A separate output-inactivity guard detects a quiet guest;
                 # final native/model assertions still decide correctness.
                 guest_timeout_seconds=1200 if selection['shell_io_only'] else 900 if focused else 4*900,
                 output_activity_seconds=None if focused else 300)
