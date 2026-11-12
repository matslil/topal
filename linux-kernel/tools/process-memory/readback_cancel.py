#!/usr/bin/env python3
"""Check saved live-child cancellation boundaries; not continuous native proof."""
from collections import Counter
from copy import deepcopy
import struct

from process_native import require, snapshot
from readback_process import event


def terminal(rows, config):
    require(len(rows)==2 and [r['retired'] for r in rows]==[False,True], 'terminal boundary order')
    before,after=[r['snapshot'] for r in rows]
    for value in [before,after]:
        snapshot(value)
        require(value['cs']&3==0 and value['ids'][0]==1,
                'actual parent terminal owner')
        require(value['epoch'][1]==config['syscall'] and value['context']['user_arg']==config['status'],
                'captured terminal syscall/status')
        require(value['state'][2]==35, 'retained parent heap')
        require(value['context']['phase']==(1 if config['early'] else 32), 'actual early/late stage')
    require(before['ids'][3]>=2 and before['inactive_state'][2]==101, 'live inactive child')
    require(before['context']['pid']==1 and after['context']['pid']==0, 'descriptor owner destruction')
    require(before['inactive_wait'][0]==2 and before['inactive_wait'][1]>0, 'actual blocked pipe read')
    child=bytes.fromhex(before['inactive']['bytes'])
    require([child[i] for i in [34*4096+128,66*4096+64,101*4096-1]]==[0xc1,0xc2,0xc7],
            'complete nontrivial child heap')
    require(bytes.fromhex(before['active']['bytes'])[34*4096+128]==0xb1, 'independent parent bytes')
    raw=bytes.fromhex(rows[0]['storage'])
    require(len(raw)==config['process_storage_bytes']==0x4c000, 'complete descriptor storage')
    def words(offset,count):return list(struct.unpack_from('<'+'Q'*count,raw,offset))
    require(words(0,1)==[1] and words(56,1)==[1] and words(0x2000,1)==[1], 'imported parent table')
    require(words(0x2010,1)==[before['ids'][3]] and words(0x3038,1)==[before['ids'][3]], 'retained child relationship')
    require([words(o,1)[0] for o in [0x2060,0x2078]]==before['inactive_wait'], 'wait storage binding')
    pipes=[words(0x3e000+i*128,16) for i in range(4)]
    matches=[r for r in pipes if r[0]==1 and r[1]==before['inactive_wait'][1]]
    require(len(matches)==1, 'exact FIFO generation')
    pipe=matches[0]
    parent=words(64,32)[0::2];child_slots=words(0x3040,32)[0::2]
    require(pipe[4:6]==[0,0] and pipe[2] in child_slots and pipe[2] not in parent,
            'empty FIFO with inactive last reader')
    require(parent.count(pipe[3])==2 and pipe[3] not in child_slots, 'parent writer aliases')
    descriptions=([words(320+i*64,8) for i in range(16)]
                  +[words(0x3d000+i*64,8) for i in range(16)])
    require(descriptions[pipe[2]-1][:2]==[5,1] and descriptions[pipe[3]-1][:2]==[6,2],
            'live endpoint description references')
    files=[(i+1,r) for i,r in enumerate(descriptions) if r[0]==7]
    require(len(files)==1, 'one retained anonymous file')
    index,file=files[0]
    require(file[1]==1 and file[3]==256 and index in child_slots and index not in parent,
            'child-only anonymous description')
    objects=[words(0x43100+i*128,16) for i in range(4)]
    objects=[r for r in objects if r[0]==1 and r[1]==file[4]]
    require(len(objects)==1 and objects[0][2]==256 and objects[0][7]==0, 'unlinked file object')
    pages=[i for i in range(8) if words(0x43400+i*32,1)==[file[4]]]
    require(len(pages)==1, 'actual resident file page')
    payload=raw[0x44000+pages[0]*4096:0x44000+(pages[0]+1)*4096]
    require(payload[:256]==bytes(i^0x5a for i in range(256)) and payload[256:]==bytes(4096-256),
            'nontrivial owned file payload')
    retired=bytes.fromhex(rows[1]['storage'])
    require(len(retired)==len(raw) and not any(retired), 'whole destructor storage not zero')
    require(after['ids'][3:6]==[0]*3 and after['inactive_state']==[0]*4
            and after['inactive_wait']==[0,0], 'child authority/wait not retired')
    require(before['active']==after['active'] and before['state']==after['state'], 'parent changed during cancellation')


def check(value):
    require(value['passed'] and value['pending'] is None, 'incomplete native observation')
    config=value['cancellation']
    require(config['syscall'] in [60,231] and config['status'] in [0x13a,0x200,0x2c5,0x301], 'case configuration')
    require(value['cases']==[] and value['active_case'] is not None, 'terminal application returned')
    events=value['events'];counts=Counter(r['operation'] for r in events)
    require([counts[n] for n in [6,7,9,10,11,12,13]]==[1,int(config['early']),1,1,1,0,1], 'cancellation transition coverage')
    for row in events:event(row)
    entries=value['replacement_entries']
    require(len(entries)==int(config['early']), 'replacement entry coverage')
    if config['early']:
        entry=entries[0]['snapshot'];snapshot(entry)
        require(entry['cs']&3==3 and entry['ids'][:3]==[1,2,1]
                and entry['context']['phase']==0 and entry['state']==[0]*4
                and entries[0]['initial_sp']==0x7ff800,'actual phase-zero replacement entry')
    terminal(value['terminal'],config)
    cancellation=next(r for r in events if r['operation']==13)
    cleanup=next(r for r in events if r['operation']==6)
    require(cancellation['before']['inactive_wait']==cancellation['after']['inactive_wait']==[0,0], 'wait not cancelled before MM retirement')
    require(cancellation['before']['inactive']==value['terminal'][0]['snapshot']['inactive'], 'cancelled wrong child backing')
    require(cancellation['after']['active']==value['terminal'][1]['snapshot']['active']==cleanup['before']['active'], 'lost active owner before detach')
    require(cancellation['after']['inactive_state']==[0]*4, 'inactive state retained')
    preparation=next(r for r in events if r['operation']==9)
    require(preparation['before']['state'][2]==(0 if config['early'] else 67), 'empty/nonempty fork coverage')
    require([r['operation'] for r in events][-2:]==[13,6], 'terminal transition order')
    require(set(value['final'])=={'ids','inactive_token','inactive_state','prepared_token','prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in value['final'].values()), 'final authority retained')
    return counts


def readback(value):
    counts=check(value)
    controls=[]
    for name,path,replacement in [
        ('missing-cancelled-child',['terminal',0,'snapshot','ids',3],0),
        ('no-real-wait',['terminal',0,'snapshot','inactive_wait',0],0),
        ('wrong-wait-generation',['terminal',0,'snapshot','inactive_wait',1],999),
        ('wrong-exit-status',['terminal',0,'snapshot','context','user_arg'],0xdead),
        ('wrong-startup-stage',['terminal',0,'snapshot','context','phase'],16),
        ('truncated-destructor-readback',['terminal',1,'storage'],''),
        ('dirty-destructor-storage',['terminal',1,'storage'],'01'+value['terminal'][1]['storage'][2:]),
        ('wrong-child-retirement',['terminal',0,'snapshot','inactive','bytes'],''),
    ]:
        altered=deepcopy(value);target=altered
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:check(altered)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved cancellation control: '+name)
    for name,path,replacement in [
        ('missing-child-retirement',['retired'],[]),
        ('truncated-child-erasure',['retired',0,'bytes'],''),
        ('child-record-still-live',['retired',0,'record',7],1),
        ('wait-survives-to-retirement',['before','inactive_wait',0],2),
    ]:
        altered=deepcopy(value)
        row=next(r for r in altered['events'] if r['operation']==13);target=row
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:check(altered)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved retirement control: '+name)
    retired=[r for e in value['events'] for r in e['retired']]
    return dict(passed=True,case=value['cancellation'],operations=dict(sorted(counts.items())),
        boundaries=len(value['events']),retired_extents=len(retired),
        erased_released_bytes=sum(r['count']*4096 for r in retired),controls=controls,
        scope='Saved cancellation boundaries and destructor readback; no continuous exclusion, every release-order, general orphan/init or public-kernel qualification')
