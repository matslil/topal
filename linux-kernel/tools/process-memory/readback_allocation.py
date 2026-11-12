#!/usr/bin/env python3
"""Replay actual physical refusals and ordinary retries from saved boundaries."""
from collections import Counter
from copy import deepcopy
import struct

from process_native import firmware_frames, physical_catalog, preparation_refusal, require, resize_refusal, snapshot
from readback_process import event


def returned_refusal(row, gateway, config):
    before,after=row['before'],row['after']
    for value in [before,after]:
        snapshot(value);physical_catalog(value)
        require(value['cs']&3==3 and value['cr3']==value['context']['root'], 'actual user refusal return')
        require(value['ids']==[1,2,1,0,0,0,3] and value['state'][1:3]==[0x7fe000,519], 'failed fork retained owner')
        raw=bytes.fromhex(value['active']['bytes'])
        require(raw==bytes(i^0x5a for i in range(256))*(519*4096//256), 'complete retained parent pattern')
        require(len(bytes.fromhex(value['identity_store']))==config['identity_storage_bytes']==4096, 'complete PID-policy store')
    require(row['result']==(1<<64)-12 and row['returned_sp']==row['sp']+8, 'ordinary ENOMEM return')
    require(row['returned_pc']==row['return_pc'] and row['returned_callee']==row['callee'], 'source ABI refusal return')
    require(after['epoch']==[before['epoch'][0]+1,57], 'actual failed fork entry')
    for key in ['allocator','ids','state','active','inactive','inactive_state','catalog','registry','tables','identity_store']:
        require(before[key]==after[key], 'user refusal changed '+key)
    require(before['context']['policy_store']==after['context']['policy_store']>0, 'PID-policy store binding')
    original,returned=[bytes.fromhex(row[k]) for k in ['storage','after_storage']]
    require(len(original)==len(returned)==config['process_storage_bytes']==0x4c000
            and original==returned, 'whole fork frontend storage changed')
    word=lambda offset:int.from_bytes(returned[offset:offset+8],'little')
    require(word(0)==word(56)==1 and all(word(o)==0 for o in [0x2000,0x2010,0x2020,0x2028]),
            'fork transient state not cleared')
    require(gateway['before']['active']==before['active']==gateway['after']['active']==after['active'], 'refused backing binding')
    require(gateway['before']['epoch']==gateway['after']['epoch']==after['epoch'], 'refused gateway entry binding')
    return preparation_refusal(gateway)


def check(value):
    require(value['passed'] and 'failure' not in value and value['pending'] is value['refusal_pending'] is None,
            'incomplete native allocation run')
    config=value['allocation_profile']
    require(config['guest_ram_mib']==52, 'small-RAM cohort')
    require(value['cases']==[] and value['active_case'] is not None, 'terminal source case returned')
    events=value['events'];counts=Counter(r['operation'] for r in events)
    expected_frames=firmware_frames(value['firmware'])
    require(b''.join(struct.pack('<QQI',*row) for row in value['firmware']['e820']).hex()
            ==value['firmware']['raw_map'], 'decoded original firmware bytes')
    require(len(expected_frames)==1480, 'complete firmware frame count')
    for row in events:
        for side in ['before','after']:
            require(row[side]['catalog'][2::3]==expected_frames, 'catalog differs from original firmware')
    require([counts[n] for n in [6,7,9,10,11,12,13]]==[1,1,3,2,0,2,1], 'allocation transition coverage')
    for row in events:event(row)
    preparations=[r for r in events if r['operation']==9]
    require([(r['before']['state'][2],r['result']) for r in preparations]==[(519,0),(1,1),(0,1)], 'refusal and successful retries')
    require([r['request'] for r in events if r['operation']==10]==[2,3], 'failed creation consumed a PID')
    require(len(value['refusals'])==1, 'missing userspace refusal return')
    capacity=returned_refusal(value['refusals'][0],preparations[0],config)
    require(capacity['frames']==1480 and capacity['free']<519, 'actual capacity shortage')
    failed_resizes=[r for r in events if r['operation']==5 and r['request']==0x7fe000
                    and r['result']==r['before']['state'][1] and r['before']['state'][2]==1]
    require(len(failed_resizes)==1, 'missing fragmented resize refusal')
    fragmentation=resize_refusal(failed_resizes[0])
    require(fragmentation['free']>=519 and fragmentation['largest_run']<519, 'actual fragmented free space')
    commits=[r for r in events if r['operation']==10]
    require(commits[0]['after']['state'][2]==1 and commits[1]['after']['state'][2]==0, 'one/zero-page creation')
    empty=preparations[2]
    require(empty['before']['allocator'][2]==empty['after']['allocator'][2]
            and empty['after']['prepared']['token']==[0]*3, 'empty fork issued backing')
    exits=[r for r in events if r['operation']==12]
    require([r['before']['ids'][0] for r in exits]==[2,3]
            and [r['before']['state'][2] for r in exits]==[518,0], 'child heap retirement coverage')
    child=bytes.fromhex(exits[0]['before']['active']['bytes'])
    require(len(child)==518*4096 and child[0]==0xc1 and child[-1]==0xc7, 'complete dirty child extent')
    require(exits[0]['after']['active']==exits[0]['before']['inactive'], 'parent after child exit')
    entries=value['replacement_entries']
    require(len(entries)==1 and entries[0]['initial_sp']==0x7ff800, 'replacement entry')
    entry=entries[0]['snapshot'];snapshot(entry)
    require(entry['cs']&3==3 and entry['ids'][:3]==[1,2,1] and entry['context']['phase']==0
            and entry['state']==[0]*4, 'actual empty replacement startup')
    terminal=value['terminal']
    require(len(terminal)==2 and [r['retired'] for r in terminal]==[False,True], 'terminal boundaries')
    for row in terminal:
        actual=row['snapshot'];snapshot(actual)
        require(actual['cs']&3==0 and actual['epoch'][1]==231 and actual['context']['user_arg']==0
                and actual['ids']==[1,2,1,0,0,0,5] and actual['state']==[1,0x5f7000,0,0], 'terminal actual owner')
    raw=bytes.fromhex(terminal[1]['storage'])
    require(len(raw)==config['process_storage_bytes'] and not any(raw), 'terminal whole descriptor readback')
    raw=bytes.fromhex(value['final_identity_store'])
    require(len(raw)==config['identity_storage_bytes'] and not any(raw), 'terminal whole PID-policy readback')
    require(set(value['final'])=={'ids','inactive_token','inactive_state','prepared_token','prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in value['final'].values()), 'final memory authority retained')
    return dict(capacity=capacity,fragmentation=fragmentation,operations=dict(sorted(counts.items())))


def readback(value):
    result=check(value);controls=[]
    for name,path,replacement in [
        ('returned-success-instead-of-ENOMEM',['refusals',0,'result'],0),
        ('truncated-frontend-readback',['refusals',0,'after_storage'],''),
        ('retained-PID-store-changed',['refusals',0,'after','identity_store'],
            '01'+value['refusals'][0]['after']['identity_store'][2:]),
        ('truncated-parent-extent',['refusals',0,'after','active','bytes'],''),
        ('wrong-user-return-mode',['refusals',0,'after','cs'],0),
        ('wrong-return-stack',['refusals',0,'returned_sp'],0),
        ('wrong-return-PC',['refusals',0,'returned_pc'],0),
        ('lost-callee-register',['refusals',0,'returned_callee','rbx'],value['refusals'][0]['callee']['rbx']^1),
        ('omitted-firmware-reservation',['firmware','reservations',1,1],value['firmware']['reservations'][1][0]+1),
        ('stale-refusal-entry',['refusals',0,'after','epoch',0],1),
        ('dirty-terminal-descriptors',['terminal',1,'storage'],'01'+value['terminal'][1]['storage'][2:]),
        ('dirty-final-PID-store',['final_identity_store'],'01'+value['final_identity_store'][2:]),
        ('wrong-replacement-entry',['replacement_entries',0,'snapshot','context','phase'],1),
    ]:
        changed=deepcopy(value);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:check(changed)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved allocation control: '+name)
    for name,op,path,replacement in [
        ('refusal-issued-frame-identity',9,['after','allocator',2],99999),
        ('refusal-retained-prepared-receipt',9,['after','prepared_state',0],1),
        ('refusal-changed-MM-generation',9,['after','ids',2],99),
        ('refusal-truncated-catalog',9,['after','catalog'],[]),
        ('refusal-truncated-registry',9,['after','registry'],[]),
        ('refusal-retained-PID-store',9,['after','identity_store'],''),
        ('fragmentation-reported-success',5,['result'],0x7fe000),
        ('fragmentation-issued-identity',5,['after','allocator',2],99999),
    ]:
        changed=deepcopy(value)
        if op==9:target=next(r for r in changed['events'] if r['operation']==9 and r['result']==0)
        else:target=next(r for r in changed['events'] if r['operation']==5 and r['request']==0x7fe000 and r['before']['state'][2]==1)
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:check(changed)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved physical-refusal control: '+name)
    retired=[r for row in value['events'] for r in row['retired']]
    result.update(passed=True,boundaries=len(value['events']),retired_extents=len(retired),
        erased_released_bytes=sum(r['count']*4096 for r in retired),controls=controls,
        scope='Saved real capacity/fragmentation refusals, ordinary retries and retirement boundaries; not continuous exclusion, all resource failures, identity exhaustion, fatal paths or public activation')
    return result
