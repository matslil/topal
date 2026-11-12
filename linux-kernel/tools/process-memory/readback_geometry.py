#!/usr/bin/env python3
"""Check saved maximum-copy fork/clone and empty/one-page handoff records."""
from collections import Counter
from copy import deepcopy

from process_native import require, snapshot
from readback_process import event

PAGE=4096
START=0x5f7000
MAXIMUM=0x7fe000
PATTERN=bytes(i^0x5a for i in range(256))


def patterned(value,count,inverted=False):
    block=bytes(b^0xff for b in PATTERN) if inverted else PATTERN
    require(bytes.fromhex(value['bytes'])==block*(count*PAGE//256), 'complete application pattern')


def check(value):
    require(value['passed'] and value.get('failure') is None and value['pending'] is value['refusal_pending'] is None,
            'native geometry run incomplete')
    config=value['geometry_profile']
    require(config['syscall'] in [56,57] and config['process_storage_bytes']==0x4c000
            and config['identity_storage_bytes']==4096, 'geometry configuration')
    require(value['allocation_profile'] is value['cancellation'] is None and value['refusals']==[], 'mixed cohorts')
    require(value['cases']==[] and value['active_case'] is not None, 'terminal source case returned')
    events=value['events'];counts=Counter(r['operation'] for r in events)
    require([counts[n] for n in [6,7,9,10,11,12,13]]==[1,1,1,1,4,1,1], 'geometry transition coverage')
    for row in events:require(event(row)==row['comparison'], 'recorded comparison differs')
    prep=next(r for r in events if r['operation']==9)
    require(prep['result']==1 and prep['before']['epoch'][1]==config['syscall']
            and prep['before']['ids']==[1,2,1,0,0,0,3]
            and prep['before']['state'][:3]==[1,MAXIMUM-13,519], 'actual maximum source owner')
    patterned(prep['before']['active'],519);patterned(prep['after']['prepared'],519)
    commit=next(r for r in events if r['operation']==10)
    require(commit['request']==2 and commit['before']['prepared']==prep['after']['prepared']
            and commit['before']['epoch']==prep['after']['epoch']
            and commit['after']['ids']==[2,3,1,1,2,1,4], 'maximum child publication')
    patterned(commit['after']['active'],519);patterned(commit['after']['inactive'],519)
    within=[r for r in events if r['operation']==5 and r['before']['ids'][0]==2
            and r['before']['state'][2]==r['after']['state'][2]==519 and r['request'] in [MAXIMUM,MAXIMUM-13]]
    require([r['request'] for r in within]==[MAXIMUM,MAXIMUM-13], 'maximum partial/full-page changes')
    for row in within:
        require(row['before']['allocator'][2]==row['after']['allocator'][2], 'within-page allocation')
        patterned(row['after']['active'],519)
    empty=next(r for r in events if r['operation']==5 and r['before']['ids'][0]==2 and r['request']==START)
    require(empty['before']['state'][2]==519 and empty['after']['state']==[1,START,0,0], 'child shrink to empty')
    patterned(empty['before']['active'],519,True);patterned(empty['after']['inactive'],519)
    grow=next(r for r in events if r['operation']==5 and r['before']['ids'][0]==2 and r['request']==START+1)
    require(grow['before']['state']==[1,START,0,0] and grow['after']['state'][:3]==[1,START+1,1], 'child empty-to-one growth')
    require(bytes.fromhex(grow['after']['active']['bytes'])==bytes(PAGE), 'whole new child page zero')
    patterned(grow['after']['inactive'],518)
    switches=[r for r in events if r['operation']==11]
    require([(r['before']['ids'][0],r['before']['state'][2],r['after']['state'][2]) for r in switches]
            ==[(2,0,519),(1,518,0),(2,1,518),(1,518,1)], 'actual empty/one-page handoff sequence')
    marked=bytearray(PAGE);marked[0]=0xc1;marked[-1]=0xc7
    for i,row in enumerate(switches):
        require(row['before']['epoch'][1]==24 and row['after']['ids'][6]==4, 'ordinary yield with retained issuance')
        parent=row['after']['active'] if i%2==0 else row['before']['active']
        child=row['before']['active'] if i%2==0 else row['after']['active']
        patterned(parent,519 if i==0 else 518)
        require(bytes.fromhex(child['bytes'])==(b'' if i<2 else bytes(marked)), 'child empty/one-page content across handoff')
    child_exit=next(r for r in events if r['operation']==12)
    require(child_exit['before']['ids']==[2,3,1,1,2,1,4]
            and child_exit['before']['state'][:3]==[1,START+1,1]
            and bytes.fromhex(child_exit['before']['active']['bytes'])==bytes(marked), 'child final owner')
    patterned(child_exit['after']['active'],518)
    entries=value['replacement_entries']
    require(len(entries)==1 and entries[0]['initial_sp']==0x7ff800 and value['child_exec_entries']==[], 'replacement entry coverage')
    entry=entries[0]['snapshot'];snapshot(entry)
    require(entry['cs']&3==3 and entry['ids']==[1,2,1,0,0,0,3]
            and entry['context']['phase']==0 and entry['state']==[0]*4, 'actual empty parent replacement entry')
    terminal=value['terminal']
    require(len(terminal)==2 and [r['retired'] for r in terminal]==[False,True], 'terminal boundaries')
    for row in terminal:
        actual=row['snapshot'];snapshot(actual)
        require(actual['cs']&3==0 and actual['epoch'][1]==231 and actual['context']['user_arg']==0
                and actual['ids']==[1,2,1,0,0,0,4]
                and actual['state'][:3]==[1,START+518*PAGE-7,518], 'parent terminal owner')
        patterned(actual['active'],518)
    require(terminal[0]['snapshot']['active']==terminal[1]['snapshot']['active'], 'premature active destruction')
    raw=bytes.fromhex(terminal[1]['storage'])
    require(len(raw)==config['process_storage_bytes'] and not any(raw), 'whole terminal descriptor readback')
    raw=bytes.fromhex(value['final_identity_store'])
    require(len(raw)==config['identity_storage_bytes'] and not any(raw), 'whole final PID-policy readback')
    require(set(value['final'])=={'ids','inactive_token','inactive_state','prepared_token','prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in value['final'].values()), 'final MM authority retained')
    return dict(syscall=config['syscall'],operations=dict(sorted(counts.items())))


def readback(value):
    result=check(value);controls=[]
    events=value['events']
    selected={op:next(r for r in events if r['operation']==op) for op in [9,10,11,12,6]}
    def reject(name,changed,checker):
        try:checker(changed)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved geometry control: '+name)
    for name,op,path,replacement in [
        ('prepared-truncated-to-34-pages',9,['after','prepared','bytes'],selected[9]['after']['prepared']['bytes'][:34*PAGE*2]),
        ('prepared-pin-missing',9,['after','prepared','record',6],0),
        ('prepared-stale-entry',9,['after','prepared_state',6],0),
        ('child-adopts-parent-MM',10,['after','ids',1],2),
        ('child-issuance-rolled-back',10,['after','ids',6],3),
        ('empty-handoff-loses-child-owner',11,['after','ids',3],0),
        ('empty-handoff-constructs-backing',11,['after','inactive','token'],selected[11]['after']['active']['token']),
        ('handoff-reuses-empty-mapping',11,['after','tables',str(selected[11]['after']['context']['low']),503],0),
        ('child-exit-missing-retirement',12,['retired'],[]),
        ('child-exit-truncated-erasure',12,['retired',0,'bytes'],''),
        ('parent-exit-missing-retirement',6,['retired'],[]),
        ('parent-exit-truncated-erasure',6,['retired',0,'bytes'],''),
        ('write-protection-disabled',9,['after','cr0'],selected[9]['after']['cr0']&~(1<<16)),
    ]:
        row=deepcopy(selected[op]);target=row
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,row,event)
    for name,offset in [('copy-wrong-after-page-34',34*PAGE+128),('copy-wrong-final-mapped-byte',519*PAGE-1)]:
        row=deepcopy(selected[9]);raw=bytearray.fromhex(row['after']['prepared']['bytes']);raw[offset]^=1
        row['after']['prepared']['bytes']=raw.hex();reject(name,row,event)
    for name,select,offset in [
        ('one-page-growth-not-zero',lambda r:r['operation']==5 and r['request']==START+1,17),
        ('parent-shrink-loses-prefix',lambda r:r['operation']==5 and r['before']['ids'][0]==1 and r['after']['state'][2]==518,517*PAGE+7),
    ]:
        row=deepcopy(next(r for r in events if select(r)));raw=bytearray.fromhex(row['after']['active']['bytes']);raw[offset]^=1
        row['after']['active']['bytes']=raw.hex();reject(name,row,event)
    for name,path,replacement in [
        ('wrong-creation-syscall',['geometry_profile','syscall'],56 if result['syscall']==57 else 57),
        ('dirty-terminal-descriptor',['terminal',1,'storage'],'01'+value['terminal'][1]['storage'][2:]),
        ('dirty-final-PID-policy',['final_identity_store'],'01'+value['final_identity_store'][2:]),
        ('wrong-replacement-stage',['replacement_entries',0,'snapshot','context','phase'],1),
        ('wrong-actual-terminal-status',['terminal',0,'snapshot','context','user_arg'],1),
    ]:
        changed=deepcopy(value);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,changed,check)
    retired=[r for row in events for r in row['retired']]
    result.update(passed=True,boundaries=len(events),copied_heap_bytes=519*PAGE,
        erased_released_bytes=sum(r['count']*PAGE for r in retired),retired_extents=len(retired),controls=controls,
        scope='Saved successful 519-page copy and empty/one-page handoff boundaries; not all callers, continuous exclusion, identity exhaustion, fatal paths or public activation')
    return result
