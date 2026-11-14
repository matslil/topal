#!/usr/bin/env python3
"""Check saved runnable-child and unconsumed-zombie termination boundaries."""
from collections import Counter
from copy import deepcopy
import struct

from process_native import require, snapshot
from readback_process import event

PAGE=4096
START=0x5f7000
PATTERN=bytes(i^0x5a for i in range(256))


def storage(row,config):
    raw=bytes.fromhex(row['storage'])
    require(len(raw)==config['process_storage_bytes']==0x4c000, 'whole process storage')
    return raw,lambda offset,count=1:list(struct.unpack_from('<'+'Q'*count,raw,offset))


def live_resources(row,config,active_child):
    raw,words=storage(row,config)
    child=words(64 if active_child else 0x3040,32)[0::2]
    parent=words(0x3040 if active_child else 64,32)[0::2]
    pipes=[(i,words(0x3e000+i*128,16)) for i in range(4)]
    pipes=[r for r in pipes if r[1][0]==1]
    require(len(pipes)==1, 'one retained FIFO')
    index,pipe=pipes[0];reader,writer=pipe[2:4]
    require(reader>0 and writer>0 and pipe[4:6]==[0,0], 'live empty FIFO endpoints')
    require(child.count(reader)==1 and child.count(writer)==0 and parent.count(writer)==2
            and parent.count(reader)==int(active_child), 'actual active/inactive endpoint references')
    descriptions=([words(320+i*64,8) for i in range(16)]
                  +[words(0x3d000+i*64,8) for i in range(16)])
    require(descriptions[reader-1][:2]==[5,1+int(active_child)]
            and descriptions[writer-1][:2]==[6,2]
            and descriptions[reader-1][4]==descriptions[writer-1][4]==pipe[1], 'live FIFO description binding')
    require(raw[0x3f000+index*PAGE:0x3f000+(index+1)*PAGE]==bytes(PAGE), 'empty FIFO payload')
    files=[(i+1,r) for i,r in enumerate(descriptions) if r[0]==7]
    require(len(files)==1, 'one live anonymous file')
    file_index,file=files[0]
    require(file_index in child and file_index not in parent and file[1]==1 and file[3]==256,
            'child-only file reference')
    objects=[(i,words(0x43100+i*128,16)) for i in range(4)]
    objects=[r for r in objects if r[1][0]==1 and r[1][1]==file[4]]
    require(len(objects)==1 and objects[0][1][2]==256 and objects[0][1][7]==0, 'unlinked file object')
    pages=[i for i in range(8) if words(0x43400+i*32)[0]==file[4]]
    require(len(pages)==1, 'one resident anonymous page')
    page=raw[0x44000+pages[0]*PAGE:0x44000+(pages[0]+1)*PAGE]
    require(page==PATTERN+bytes(PAGE-256), 'whole dirty file page')
    return dict(pipe=pipe,reader=reader,writer=writer,file_index=file_index,file=file,page=page)


def check(value):
    require(value['passed'] and value.get('failure') is None and value['pending'] is value['refusal_pending'] is None,
            'native relationship run incomplete')
    config=value['relationship_profile'];zombie=config['zombie']
    expected_status={(False,60):0x13a,(False,231):0x200,(True,60):0x2c5,(True,231):0x301}
    require(config['status']==expected_status.get((config['early'],config['syscall']))
            and config['identity_storage_bytes']==4096, 'relationship configuration')
    require(value['cancellation'] is value['allocation_profile'] is value['geometry_profile'] is None
            and value['refusals']==[], 'mixed native cohorts')
    require(value['cases']==[] and value['active_case'] is not None and value['child_exec_entries']==[], 'terminal source case')
    events=value['events'];counts=Counter(r['operation'] for r in events)
    require([counts[n] for n in [6,7,9,10,11,12,13]]==[1,int(config['early']),1,1,int(not zombie),int(zombie),1],
            'relationship transition coverage')
    for row in events:require(event(row)==row['comparison'], 'recorded comparison differs')
    require(len(value['child_departures'])==1, 'actual child departure')
    departure=value['child_departures'][0];child=departure['snapshot'];snapshot(child)
    require(child['cs']&3==3 and child['ids'][0]==child['context']['pid']==2 and child['ids'][3]==1
            and child['context']['relationship']==1 and child['state'][:3]==[1,START+101*PAGE-11,101], 'live departing child owner')
    require(departure['syscall']==(60 if zombie else 24) and departure['argument']==(0x217 if zombie else 0),
            'actual ordinary departure syscall')
    heap=bytes.fromhex(child['active']['bytes'])
    require([heap[i] for i in [34*PAGE+128,66*PAGE+64,101*PAGE-1]]==[0xc1,0xc2,0xc7]
            and heap[35*PAGE:35*PAGE+256]==PATTERN, 'complete nontrivial child heap')
    original=live_resources(departure,config,True)
    _,child_words=storage(departure,config)
    require(child_words(0)==[1] and child_words(56)==[2] and child_words(0x2000)==[1]
            and child_words(0x3038)==[1], 'departing child storage identity')
    rows=value['terminal']
    require(len(rows)==2 and [r['retired'] for r in rows]==[False,True], 'terminal boundary order')
    before,after=[r['snapshot'] for r in rows]
    for actual in [before,after]:
        snapshot(actual)
        require(actual['cs']&3==0 and actual['ids'][0]==1 and actual['state'][1:3]==[START+35*PAGE,35]
                and actual['epoch'][1]==config['syscall'] and actual['context']['user_arg']==config['status']
                and actual['context']['phase']==(1 if config['early'] else 32)
                and actual['inactive_wait']==[0,0], 'actual parent terminal state')
        require(bytes.fromhex(actual['active']['bytes'])[34*PAGE+128]==0xb1, 'independent parent content')
    raw,words=storage(rows[0],config)
    require(words(0x2000)==[before['context']['relationship']]
            and [words(o)[0] for o in [0x2060,0x2078]]==before['inactive_wait'], 'snapshot/storage relationship and wait binding')
    require(words(0)==words(56)==[1] and words(0x2000)==[2 if zombie else 1] and words(0x2010)==[2],
            'retained actual child relationship')
    if zombie:
        require(words(0x2018)==[0x17] and not any(raw[0x2020:0x3d000]), 'unreaped status and erased private parking')
        require(before['ids'][3:6]==[0]*3 and before['inactive_state']==[0]*4 and before['inactive']['token']==[0]*3,
                'zombie retained MM authority')
        require(words(0x3e000+16)==[0] and words(0x3e000+24)==[original['writer']], 'last reader release before zombie termination')
        require(not any(raw[0x43100:0x43300]) and not any(raw[0x43400:0x43500]) and not any(raw[0x44000:]),
                'zombie file object/page not retired')
        handoff=next(r for r in events if r['operation']==12)
        require(handoff['before']['active']==child['active'] and len(handoff['retired'])==1,
                'child exit did not retire its actual heap')
    else:
        require(before['ids'][3]==2 and before['inactive']==child['active'] and before['inactive_state']==child['state']
                and words(0x3038)==[2], 'retained runnable child owner')
        retained=live_resources(rows[0],config,False)
        require(retained['file']==original['file'] and retained['page']==original['page']
                and retained['pipe'][1]==original['pipe'][1], 'runnable child resource generation/content')
        handoff=next(r for r in events if r['operation']==11)
        require(handoff['before']['active']==child['active'] and handoff['after']['inactive']==child['active'],
                'yield lost departing child owner')
    require(before['context']['pid']==1 and after['context']['pid']==0 and before['active']==after['active'],
            'ordered descriptor/active-MM destruction')
    retired,_=storage(rows[1],config)
    require(not any(retired) and after['ids'][3:6]==[0]*3 and after['inactive_state']==[0]*4, 'whole terminal destructor readback')
    cancellation=next(r for r in events if r['operation']==13)
    cleanup=next(r for r in events if r['operation']==6)
    require(cancellation['before']['inactive_wait']==cancellation['after']['inactive_wait']==[0,0]
            and cancellation['before']['inactive']==before['inactive'], 'terminal child owner/wait binding')
    require(len(cancellation['retired'])==int(not zombie), 'duplicate or omitted child retirement')
    require(cancellation['after']['active']==after['active']==cleanup['before']['active']
            and [r['operation'] for r in events][-2:]==[13,6], 'active owner before detached cleanup')
    prep=next(r for r in events if r['operation']==9)
    require(prep['before']['state'][2]==(0 if config['early'] else 67), 'initial/replacement heap coverage')
    entries=value['replacement_entries']
    require(len(entries)==int(config['early']), 'replacement entry coverage')
    if entries:
        entry=entries[0]['snapshot'];snapshot(entry)
        require(entry['cs']&3==3 and entry['ids'][:3]==[1,2,1] and entry['context']['phase']==0
                and entry['state']==[0]*4 and entries[0]['initial_sp']==0x7ff800, 'actual empty replacement entry')
    raw=bytes.fromhex(value['final_identity_store'])
    require(len(raw)==4096 and not any(raw), 'whole final PID-policy readback')
    require(set(value['final'])=={'ids','inactive_token','inactive_state','prepared_token','prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in value['final'].values()), 'final MM authority retained')
    return counts


def readback(value):
    counts=check(value);controls=[]
    def reject(name,changed):
        try:check(changed)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved relationship control: '+name)
    for name,path,replacement in [
        ('missing-departing-child',['child_departures',0,'snapshot','ids',0],0),
        ('wrong-child-departure',['child_departures',0,'syscall'],231),
        ('wrong-child-full-exit-status',['child_departures',0,'argument'],0x17 if value['relationship_profile']['zombie'] else 1),
        ('truncated-child-storage',['child_departures',0,'storage'],''),
        ('lost-child-heap',['child_departures',0,'snapshot','active','bytes'],''),
        ('wrong-parent-relationship',['terminal',0,'snapshot','context','relationship'],0),
        ('wrong-parent-status',['terminal',0,'snapshot','context','user_arg'],0xdead),
        ('wrong-parent-startup-stage',['terminal',0,'snapshot','context','phase'],16),
        ('dirty-terminal-storage',['terminal',1,'storage'],'01'+value['terminal'][1]['storage'][2:]),
        ('truncated-terminal-storage',['terminal',1,'storage'],''),
        ('dirty-final-PID-store',['final_identity_store'],'01'+value['final_identity_store'][2:]),
    ]:
        changed=deepcopy(value);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,changed)
    for name,op,path,replacement in [
        ('missing-prepared-owner',9,['after','prepared_owner',0],0),
        ('prepared-entry-stale',9,['after','prepared_state',6],0),
        ('missing-child-retirement',12 if value['relationship_profile']['zombie'] else 13,['retired'],[]),
        ('child-erasure-truncated',12 if value['relationship_profile']['zombie'] else 13,['retired',0,'bytes'],''),
        ('child-record-retained',12 if value['relationship_profile']['zombie'] else 13,['retired',0,'record',7],1),
        ('missing-parent-retirement',6,['retired'],[]),
    ]:
        changed=deepcopy(value);target=next(r for r in changed['events'] if r['operation']==op)
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,changed)
    raw=bytes.fromhex(value['terminal'][0]['storage'])
    changes=[('lost-relationship-header',0x2000,0),('lost-child-PID',0x2010,0)]
    if value['relationship_profile']['zombie']:
        changes += [('lost-unreaped-status',0x2018,0),('zombie-private-slot-retained',0x3040,1),
                    ('zombie-file-payload-retained',0x44000,1)]
    else:
        file_page=next(i for i in range(8) if any(struct.unpack_from('<QQQQ',raw,0x43400+i*32)))
        changes += [('runnable-child-writer-alias',0x3040+16,struct.unpack_from('<Q',raw,0x3e000+24)[0]),
                    ('runnable-wait-invented',0x2060,2),('runnable-file-payload-lost',0x44000+file_page*PAGE+248,0)]
    for name,offset,word in changes:
        changed=deepcopy(value);altered=bytearray(raw);struct.pack_into('<Q',altered,offset,word)
        changed['terminal'][0]['storage']=altered.hex();reject(name,changed)
    retired=[r for row in value['events'] for r in row['retired']]
    return dict(passed=True,case=value['relationship_profile'],operations=dict(sorted(counts.items())),
        boundaries=len(value['events']),retired_extents=len(retired),erased_released_bytes=sum(r['count']*PAGE for r in retired),
        controls=controls,scope='Saved runnable/zombie boundaries and whole-resource readback; not continuous exclusion, every release/invalidation instruction, general orphan/init or public qualification')
