#!/usr/bin/env python3
"""Cross-check ordinary interface records and actual process-MM boundaries.

These saved-record checks complement the bound native source observers. They
do not independently reconstruct every observer assertion or prove continuous
exclusion, every instruction, arbitrary applications or public activation.
"""
import argparse
from collections import Counter
from copy import deepcopy
import json
import importlib.util
from pathlib import Path
import struct

from process_native import require, snapshot
from readback_process import digest, event

MODES=['transfers','exec','signals','poll','pipes','storage','metadata','serial','stat']
GROUPS=['executions','startup_executions','file_executions','shell_executions','command_executions','poll_executions']


def apply_deltas(before,changes):
    after=bytearray(before);end=0
    for row in changes:
        start=row['offset'];old,new=bytes.fromhex(row['before']),bytes.fromhex(row['after'])
        require(len(old)==len(new)>0 and end<=start<start+len(old)<=len(before), 'complete nonoverlapping metadata delta')
        require(before[start:start+len(old)]==old and all(a!=b for a,b in zip(old,new)), 'metadata before-byte binding')
        after[start:start+len(new)]=new;end=start+len(old)
    return bytes(after)


def metadata_transitions(row):
    path=Path(__file__).resolve().parents[3]/'linux-kernel/tools/user-memory/metadata_model.py'
    spec=importlib.util.spec_from_file_location('interface_metadata_model',path)
    model=importlib.util.module_from_spec(spec);spec.loader.exec_module(model)
    fs=bytes.fromhex(row['metadata_initial_filesystem']);table=bytes.fromhex(row['metadata_initial_table'])
    initial_fs,initial_table=fs,table
    require(len(fs)==0x9000 and len(table)==8192, 'whole metadata starting state')
    last=None
    for call in [c for c in row['calls'] if c['number']!=12]:
        clocks=call['metadata_clocks']
        mutation=call['result']>=0 and call['number'] in [257,18,77,91,82,86,87]
        require(len(clocks)==int(mutation) and all(c['carry']==0 and 0<c['seconds']<1<<63 for c in clocks), 'actual metadata clock count/result')
        seconds=clocks[0]['seconds'] if clocks else None
        paths=[bytes.fromhex(p) if p is not None else None for p in call['metadata_paths']]
        try:expected=model.transition(fs,table,call['number'],call['args'],call['result'],paths,seconds,bytes.fromhex(call['metadata_payload']))
        except (AssertionError,StopIteration,IndexError) as error:raise ValueError('metadata transition model refused record') from error
        actual_fs=apply_deltas(fs,call['metadata_filesystem_delta'])
        actual_table=apply_deltas(table,call['metadata_table_delta'])
        require((actual_fs,actual_table)==expected, 'complete metadata transition differs')
        require(digest(actual_fs)['sha256']==call['metadata_filesystem_sha256']
                and digest(actual_table)['sha256']==call['metadata_table_sha256'], 'whole metadata state digest')
        fs,table=actual_fs,actual_table
        if call['number']==87 and call['result']==0:last=seconds
    reclaimed=bytearray(initial_fs)
    struct.pack_into('<Q',reclaimed,0,struct.unpack_from('<Q',initial_fs,0)[0]+1)
    require(last is not None, 'final unlink clock')
    struct.pack_into('<QQ',reclaimed,48,last,last)
    require(fs==bytes(reclaimed) and table==initial_table, 'complete named-object/page/table reclamation')


def interface_case(row,mode):
    calls=row['calls'];numbers=[c['number'] for c in calls]
    require(len(calls)>=3 and numbers[:2]==[12,12] and numbers[-1]==12, 'ordinary break wrapper')
    require([calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000], 'source break results')
    require(calls[0]['args'][0]==0 and calls[1]['args'][0]==0x63a000 and calls[-1]['args'][0]==0x619000,
            'source break arguments')
    actions=calls[2:-1]
    if mode in ['transfers','poll']:
        expected=[12,12]+[63]*7+[1]*4
        results=[0x619000,0x63a000,0,0]+[-14]*5+[0,-9,-22,-14]
        if mode=='transfers':
            expected += [1,1,1,79]
            results += [-14,24,24,len(row['cwd_expected'])//2]
            require(calls[-2]['output_hex']==row['cwd_expected'], 'actual cwd output')
        require(numbers==expected+[12] and [c['result'] for c in calls]==results+[0x619000], 'ordinary transfer results')
    elif mode=='exec':
        require(numbers==[12,12]+[59]*29+[12], 'exec refusal request coverage')
        require([c['result'] for c in actions]==[-14]*5+[-2,-36]+[-14]*10+[-22,-7,-7]+[-14]*6+[-7]*3, 'exec error order')
        require(all(c['private_staging_zero'] and c['protected_state_unchanged'] for c in actions), 'exec protected state')
    elif mode=='signals':
        require(numbers==[12,12]+[13]*23+[14]*23+[12], 'signal request coverage')
        require(Counter(c['result'] for c in actions)=={-14:22,-22:6,0:18}, 'signal results')
        require(all(c['complete_table_verified'] and c['private_staging_zero'] for c in actions)
                and sum(c['committed'] for c in actions)==10
                and sum(c['committed'] and c['result']==-14 for c in actions)==2, 'signal mutation/error order')
    elif mode=='pipes':
        require(len(calls)==44 and Counter(numbers)=={12:3,293:6,16:10,1:10,0:13,3:2}
                and len(actions)==41 and row['pipe_retirement_verified'], 'FIFO request/retirement coverage')
        require(sum(c['result']==-14 for c in actions)==19 and sum(c['result']==-11 for c in actions)==1
                and sum(c['number'] in [0,1] and c['result']>0 for c in actions)==10, 'FIFO ordinary results')
        require(all(c['complete_fifo_state_verified'] and c['complete_table_verified'] for c in actions)
                and all(c['copy_window_verified'] for c in actions if c['number']!=3), 'FIFO state/output windows')
        # Reconstruct application-visible byte order independently of the
        # native ring cursor/layout model, including both wrap directions.
        queue=b''
        for call in actions:
            n,result=call['number'],call['result']
            if n==1 and result>0:
                data=bytes.fromhex(call['buffer_before'])
                require(result<=len(data) and result<=call['args'][2] and len(queue)+result<=4096, 'FIFO write extent')
                queue+=data[:result]
            elif n==0 and result>0:
                data=bytes.fromhex(call['output_hex'])
                require(result<=call['args'][2] and len(data)==result and data==queue[:result], 'FIFO application byte order')
                queue=queue[result:]
            elif n==0 and result==0 and call['args'][2]:require(not queue, 'FIFO premature EOF')
        require(not queue, 'FIFO payload retained at close')
    elif mode=='storage':
        require(len(calls)==62 and Counter(numbers)=={12:3,257:3,1:7,8:8,0:7,17:18,18:7,3:3,217:6}, 'storage request coverage')
        require(sum(c['result']==-14 for c in actions)==28 and sum(c['result']==-22 for c in actions)==1, 'storage ordinary errors')
        require(row['storage_retirement_verified'] and all(c['complete_storage_state_verified'] and c['complete_table_verified'] for c in actions),
                'storage state/retirement')
    elif mode=='metadata':
        require(numbers==[12,12,90,257,5,5,18,8,5,77,5,91,91,5,86,5,82,87,5,257,5,18,8,5,91,86,82,87,5,3,5,3,87,12],
                'metadata request coverage')
        require([c['result'] for c in actions if c['result']<0]==[-14,-9,-17,-14,-2], 'metadata ordinary errors')
        require(row['metadata_retirement_verified'] and all(c['complete_metadata_delta_verified'] for c in actions)
                and sum(len(c['metadata_clocks']) for c in actions)==11, 'metadata complete deltas/clocks')
        metadata_transitions(row)
    elif mode=='serial':
        require(16<=len(calls)<=29 and Counter(numbers)[12]==3 and Counter(numbers)[1]==2 and set(numbers)<={0,1,12},
                'serial ordinary requests')
        require(sum(c['result']==-14 for c in calls)==5 and sum(c['result']==-9 for c in calls)==2
                and row['serial_markers']==2 and bytes(row['serial_received'])==b'Uabcdefg12345678'
                and row['serial_input_verified'], 'serial actual input/error coverage')
        require(all(c['copy_window_verified'] and c['complete_table_verified'] and c['native_input_verified'] for c in actions), 'serial input/windows')
        large=[c for c in actions if c['number']==0 and c['args'][2]==512]
        require(len(large)==1 and large[0]['extent_call']==dict(candidate=0x639f00,length=256), 'serial admitted prefix')
    elif mode=='stat':
        require(len(calls)==38 and Counter(numbers)=={12:3,5:13,262:9,257:4,293:1,1:1,263:1,3:6}, 'stat request coverage')
        queries=[c for c in calls if c['number'] in [5,262]]
        require(Counter(c['result'] for c in queries)=={0:15,-14:5,-9:1,-2:1}
                and all(c['stat_private_state_preserved'] and c['stat_output_window_verified'] for c in queries), 'stat results/windows')
        require({c['stat_variant'] for c in queries if c['result']==0}=={
            'terminal','immutable-file','directory-root','directory-bin','directory-tmp','pipe-reader','pipe-writer','volatile-file'}, 'stat variant coverage')
    else:require(False,'unknown interface cohort')


def check(user,mm,kernel_sha256):
    require(user['passed'] and 'observer_failure' not in user and 'invariant_failure' not in user, 'ordinary run incomplete')
    require(mm['passed'] and 'failure' not in mm and mm['pending'] is mm['active_case'] is mm['refusal_pending'] is None,
            'MM run incomplete')
    mode=mm['mode'];require(mode in MODES, 'interface cohort')
    require(user['process_memory_regression']==dict(passed=True,mode=mode,cases=4,boundaries=len(mm['events']),kernel_sha256=kernel_sha256),
            'ordinary/MM/kernel observation binding')
    require(len(user['executions'])==len(mm['cases'])==4, 'four source invocation boundaries')
    for ordinary,owned in zip(user['executions'],mm['cases']):
        before,after=owned['before'],owned['after'];snapshot(before);snapshot(after)
        require(before['cs']&3==after['cs']&3==3 and before['ids']==after['ids']
                and before['ids'][0]==1 and before['ids'][3:6]==[0]*3, 'retained ordinary process identity')
        require(ordinary['before']==before['state'] and ordinary['after']==after['state']
                and before['state'][:3]==after['state'][:3]==[1,0x619000,34], 'source/MM byte-break binding')
        require(before['active']['bytes']==after['active']['bytes']
                and digest(bytes.fromhex(before['active']['bytes']))['sha256']==ordinary['prefix_before'], 'source/MM complete heap prefix')
        interface_case(ordinary,mode)
    require([r['before']['ids'][2] for r in mm['cases']]==[0,1,2,3], 'actual successive image generations')
    events=mm['events'];counts=Counter(r['operation'] for r in events)
    for row in events:require(event(row)==row['comparison'], 'recorded MM comparison differs')
    # This ordinary ABI witness creates children that exit directly. Actual
    # cooperative handoffs remain covered by the retained process cohorts.
    require([counts[n] for n in [6,7,9,10,11,12,13]]==[1,3,4,4,0,4,1], 'retained ABI child/exec/terminal transitions')
    require([r['before']['state'][2] for r in events if r['operation']==9]==[34]*4, 'retained child heap geometry')
    require(set(mm['final'])=={'ids','inactive_token','inactive_state','prepared_token','prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in mm['final'].values()), 'final MM authority retained')
    return mode,counts


def readback(user,mm,kernel_sha256):
    mode,counts=check(user,mm,kernel_sha256);controls=[]
    def reject(name,ordinary,owned):
        try:check(ordinary,owned,kernel_sha256)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved interface control: '+name)
    for name,path,replacement in [
        ('source-prefix-not-retained',['executions',0,'prefix_before'],'0'*64),
        ('source-return-break-changed',['executions',0,'after',1],0x63a000),
        ('source-query-result-changed',['executions',0,'calls',0,'result'],0),
        ('wrong-kernel-observation',['process_memory_regression','kernel_sha256'],'0'*64),
    ]:
        changed=deepcopy(user);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,changed,mm)
    for name,path,replacement in [
        ('lost-source-owner',['cases',0,'after','ids',0],0),
        ('wrong-source-image-generation',['cases',0,'after','ids',2],99),
        ('truncated-source-heap',['cases',0,'after','active','bytes'],''),
        ('retained-final-owner',['final','ids',0],1),
    ]:
        changed=deepcopy(mm);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,user,changed)
    for name,op,path,replacement in [
        ('missing-copy-pin',9,['after','prepared','record',6],0),
        ('stale-preparation-entry',9,['after','prepared_state',6],0),
        ('child-adopts-parent-MM',10,['after','ids',1],1),
        ('commit-loses-retained-parent',10,['after','ids',3],0),
        ('exit-missing-retirement',12,['retired'],[]),
        ('exit-truncated-erasure',12,['retired',0,'bytes'],''),
        ('parent-missing-retirement',6,['retired'],[]),
    ]:
        changed=deepcopy(mm);target=next(r for r in changed['events'] if r['operation']==op)
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,user,changed)
    if mode=='pipes':
        for name,size in [('wrapped-read-byte-order',18),('full-ring-final-byte',4096)]:
            changed=deepcopy(user)
            call=next(c for c in changed['executions'][0]['calls'] if c['number']==0 and c['result']==size)
            data=bytearray.fromhex(call['output_hex']);data[-1]^=1;call['output_hex']=data.hex()
            reject(name,changed,mm)
    if mode=='metadata':
        changed=deepcopy(user)
        call=next(c for c in changed['executions'][0]['calls'] if c.get('metadata_clocks'))
        call['metadata_clocks'][0]['seconds']+=1
        reject('metadata-clock-delta-disagreement',changed,mm)
        changed=deepcopy(user)
        call=next(c for c in changed['executions'][0]['calls'] if c['number']==18 and c['result']>0)
        payload=bytearray.fromhex(call['metadata_payload']);payload[0]^=1;call['metadata_payload']=payload.hex()
        reject('metadata-write-payload-disagreement',changed,mm)
    retired=[r for row in mm['events'] for r in row['retired']]
    return dict(passed=True,mode=mode,source_cases=4,source_case_syscalls=sum(len(r['calls']) for r in user['executions']),
        observed_interface_syscalls=sum(len(r['calls']) for group in GROUPS for r in user[group]),
        operations=dict(sorted(counts.items())),boundaries=len(mm['events']),retired_extents=len(retired),
        erased_released_bytes=sum(r['count']*4096 for r in retired),controls=controls,
        scope='Cross-bound source interface and MM boundary records plus selected result checks; not an independent replay of every native observer assertion, continuous exclusion or public qualification')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path)
    args=parser.parse_args();build=args.build.resolve()
    root=Path(__file__).resolve().parents[3]
    require(build.is_relative_to(root/'linux-kernel'), 'Linux build directory')
    user=json.loads((build/'source-user-extents/observations.json').read_text())
    mm=json.loads((build/'source-user-extents/process-mm-observations.json').read_text())
    result=readback(user,mm,digest((build/'kernel.elf').read_bytes())['sha256'])
    (build/'interface-readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='controls'}))


if __name__=='__main__':main()
