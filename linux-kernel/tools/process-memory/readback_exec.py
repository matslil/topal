#!/usr/bin/env python3
"""Check saved ordinary child-exec observations, retaining native coverage gaps."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from process_native import require, snapshot
from readback_process import digest, event, readback as process_readback


def child_entry(value, executions):
    row=value['snapshot']
    snapshot(row)
    require(row['cs']&3==3 and row['cr3']==row['context']['root'], 'child entry actual root/mode')
    require(value['initial_sp']==0x7ff800, 'child exec initial stack')
    require(row['state']==[0]*4 and row['tokens']==[0]*6 and row['context']['heap']==0, 'fresh child heap')
    require(row['fs_base']==row['context']['tls']==0, 'child exec FS reset')
    require(row['context']['registered_tid']==row['context']['robust']==0, 'child exec registration reset')
    require(row['ids'][0]>=2 and row['ids'][3]==1 and row['inactive_state'][2]==67, 'retained parent owner')
    match=[r for r in executions if r['after']['ids']==row['ids']]
    require(len(match)==1, 'entry lacks exact child exec')
    transition=match[0]
    require(transition['before']['inactive']==transition['after']['inactive']==row['inactive'], 'child exec changed parent bytes')
    require(transition['after']['inactive_state']==row['inactive_state'], 'child exec changed parent break')
    require(transition['after']['epoch']==row['epoch'], 'child entry changed before startup')
    return dict(pid=row['ids'][0], mm=row['ids'][1], generation=row['ids'][2],
                parent_mm=row['ids'][4],parent_pages=row['inactive_state'][2])


def readback(value):
    result=process_readback(value,child_exec=True)
    executions=[r for r in value['events'] if r['operation']==7 and r['before']['ids'][0]>=2]
    entries=value['child_exec_entries']
    require(len(executions)==len(entries)==4, 'missing child exec executions/entries')
    require(all(r['before']['context']['tls']==0x5f7500
                and r['before']['context']['registered_tid']==r['before']['context']['robust']==(1<<64)-1
                for r in executions), 'nonvacuous exec reset inputs')
    actual=[child_entry(row,executions) for row in entries]
    require([row['generation'] for row in actual]==[1,2,3,4], 'child image generations')
    require(len({row['mm'] for row in actual})==4, 'child MM reuse')
    controls=[]
    for name,path,replacement in [
        ('child-exec-changes-parent-byte',['after','inactive','bytes'],
         '01'+executions[0]['after']['inactive']['bytes'][2:]),
        ('child-exec-restores-old-MM',['after','ids',1],executions[0]['before']['ids'][1]),
        ('child-exec-restores-old-generation',['after','ids',2],executions[0]['before']['ids'][2]),
        ('child-exec-retires-parent',['after','inactive_state',0],0),
    ]:
        row=deepcopy(executions[0]);target=row
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:event(row)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved child-exec control: '+name)
    for name,path,replacement in [
        ('child-entry-old-stack',['initial_sp'],0x7fffa0),
        ('child-entry-lost-parent',['snapshot','ids',3],0),
        ('child-entry-retained-old-heap',['snapshot','context','heap'],executions[0]['before']['state'][3]),
        ('child-entry-retained-FS',['snapshot','fs_base'],0x5f7500),
        ('child-entry-retained-TID',['snapshot','context','registered_tid'],(1<<64)-1),
        ('child-entry-retained-robust-list',['snapshot','context','robust'],(1<<64)-1),
    ]:
        row=deepcopy(entries[0]);target=row
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement
        try:child_entry(row,executions)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved child-entry control: '+name)
    result.update(child_exec_entries=actual,child_exec_controls=controls,
        scope='Experimental child exec, retained parent and early ordinary services; not public ELF-loader, continuous lifetime, cancellation or fatal-path qualification')
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observations',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[3]
    output=args.output.resolve()
    require(output.is_relative_to(root/'linux-kernel'), 'Linux output directory')
    raw=args.observations.read_bytes();result=readback(json.loads(raw))
    result['observations']=digest(raw)
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('x') as stream:stream.write(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:result[k] for k in ['passed','cases','operations','child_exec_entries','retired_extents']}))


if __name__=='__main__':main()
