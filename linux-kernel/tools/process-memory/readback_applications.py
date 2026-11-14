#!/usr/bin/env python3
"""Join unchanged application consoles, legacy ownership and process-MM records.

TOPAL-SYSTEMS-PROCESS-MM-001: record replay complements the bound native
observers. It does not independently rerun every observer assertion, a VM,
continuous exclusion, every release/invalidation instruction or public loading.
"""
import argparse
from collections import Counter
from copy import deepcopy
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import struct
import sys

from process_native import firmware_frames, require
from readback_process import digest, event

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
import topal_gate

SIZES=dict(caller=4080,private=65528,stack=4096,facts=24,output=56,gateway_recovery=48)
FINAL=dict(ids=7,inactive_token=3,inactive_state=4,prepared_token=3,prepared_state=7,
           prepared_owner=3,transition=3)


@lru_cache(maxsize=4)
def catalog(e820,reservations):
    return tuple(firmware_frames(dict(e820=e820,reservations=reservations)))


def ptes(value):
    context=value['context'];tables=value['tables']
    return tables[str(context['low'])][503:512]+tables[str(context['root']+16384)][:510]


def final_physical(user,mm,facts):
    frames=user['physical_frames'];n=user['n']
    require(len(user['reservations'])==2 and user['reservations'][0]==[0,facts['reservation_end']]
            and user['reservations'][1][1]-user['reservations'][1][0]==facts['boot_resource_bytes'],
            'artifact-bound kernel and boot-resource reservations')
    expected=catalog(tuple(tuple(row) for row in user['e820']),tuple(tuple(row) for row in user['reservations']))
    require(n==len(frames) and tuple(frames)==expected, 'complete firmware/catalog identity')
    live=user['live'];saved=user['final_physical']
    require(len(live)==1 and len(live[0])==8, 'remaining bootstrap record')
    record=live[0];issuer,identity,start,count,alignment,provenance,pins,held=record
    require(issuer==provenance==pins==held==1 and identity>0 and count==5
            and 0<=start<start+count<=n and alignment>0 and alignment&(alignment-1)==0,
            'remaining pinned bootstrap owner')
    physical=frames[start]
    require(physical%alignment==0 and frames[start:start+5]==list(range(physical,physical+5)), 'bootstrap physical span')
    tokens=saved['root_tokens']
    require(len(tokens)*8==facts['root_token_bytes'] and len(tokens)%3==0, 'whole remaining root token bank')
    owned=[tokens[i:i+3] for i in range(0,len(tokens),3) if tokens[i:i+3]!=[0]*3]
    require(len(owned)==1 and owned[0][:2]==[issuer,identity] and 0<=owned[0][2]<n, 'remaining bootstrap token identity')
    require(saved['cr3']==mm['events'][-1]['after']['cr3']==physical*4096, 'actual remaining bootstrap CR3')
    registry=bytearray(n*64)
    struct.pack_into('<8Q',registry,owned[0][2]*64,*record)
    require(hashlib.sha256(registry).hexdigest()==saved['registry_sha256'], 'complete final registry differs')
    table=bytearray(n*24)
    for i,frame in enumerate(frames):
        struct.pack_into('<3Q',table,i*24,identity if start<=i<start+5 else 0,1,frame)
    require(hashlib.sha256(table).hexdigest()==saved['table_sha256'], 'complete final frame catalog differs')


def check(user,mm,serial,workload,facts):
    require(workload in topal_gate.PROCESS_WORKLOADS, 'unknown application workload')
    try:topal_gate.verify_process_serial(serial,external=True,process_workload=workload)
    except topal_gate.GateError as error:raise ValueError('application console contract: '+str(error)) from error
    require(user['passed'] and user['workload']==workload and 'observer_failure' not in user
            and user['syscall_caller_cfi_registers_checked'] and user['source_trace']
            and user['complete_topology_checked'], 'application/legacy observer incomplete')
    require(mm['passed'] and mm['mode']=='pinned-application' and mm['workload']==workload
            and mm['kernel_sha256']==facts['kernel_sha256'] and 'failure' not in mm
            and mm['pending'] is mm['active_case'] is mm['refusal_pending'] is None
            and not mm['cases'], 'application/MM observation binding')
    require(mm['final']=={name:[0]*size for name,size in FINAL.items()}, 'remaining process-MM authority')
    coverage=user['extent_call_coverage']
    require(coverage['observation_passed'] and coverage['reached']
            and coverage['kernel_sha256']==facts['kernel_sha256']
            and coverage['inventory_sha256']==facts['inventory_sha256']
            and user['callback_observation']['passed'], 'native extent/callback observation binding')
    events=mm['events'];counts=Counter(row['operation'] for row in events)
    require(events and events[-1]['operation']==6 and counts[6]==counts[13]==1
            and counts[7]==serial.count(topal_gate.EXEC_COMMIT_MARKER)
            and any(row['operation']==5 and row['request']>0 for row in events), 'application MM lifecycle')
    terminal=next(row for row in events if row['operation']==13)
    require(terminal['before']['context']['user_arg']&255==topal_gate.PROCESS_WORKLOADS[workload][2], 'captured terminal application status')
    for row in events:
        require(row['before']['kernel_spans']==row['after']['kernel_spans']==facts['kernel_spans'],
                'complete linked kernel load-span binding')
        require(event(row)==row['comparison'], 'application MM comparison differs')
        retired=row['gateway_retirement']
        require(set(retired)==set(SIZES), 'complete return-retirement domains')
        for name,size in SIZES.items():
            value=retired[name]
            pointer=row['sp']-4080 if name=='caller' else facts['retirement_symbols'][name]
            require(value==dict(address=pointer,bytes=size,sha256=hashlib.sha256(bytes(size)).hexdigest()),
                    'return-retirement address/extent/digest: '+name)
    legacy=user['memory'];ordinary=[row for row in events if row['operation'] in [5,6,7]]
    require(len(legacy)==len(ordinary), 'legacy/MM event count')
    for old,row in zip(legacy,ordinary):
        before,after=row['before'],row['after']
        require(old['op']==row['operation'] and old['request']==row['request'] and old['sp']==row['sp']
                and old['result']==row['result'] and old['before']==before['state'] and old['after']==after['state']
                and old['token_before']==before['tokens'][:3] and old['token_after']==after['tokens'][:3]
                and old['ptes_before']==ptes(before)
                and old['data_before_sha256']==digest(bytes.fromhex(before['active']['bytes']))['sha256']
                and {name:value&((1<<64)-1) for name,value in old['registers'].items()}==row['callee'],
                'independent legacy/MM transport, heap and translation binding')
    final_physical(user,mm,facts)
    return counts


def readback(user,mm,serial,workload,facts):
    counts=check(user,mm,serial,workload,facts);controls=[]
    def reject(name,ordinary,owned,console=serial):
        try:check(ordinary,owned,console,workload,facts)
        except ValueError as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved application control: '+name)
    for name,path,replacement in [
        ('wrong-application-workload',['workload'],'unknown'),
        ('lost-actual-source-CFI',['syscall_caller_cfi_registers_checked'],False),
        ('changed-legacy-MM-result',['memory',0,'result'],user['memory'][0]['result']^1),
        ('lost-final-registry-binding',['final_physical','registry_sha256'],'0'*64),
        ('lost-final-catalog-binding',['final_physical','table_sha256'],'0'*64),
        ('unpinned-bootstrap-root',['live',0,6],0),
    ]:
        changed=deepcopy(user);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,changed,mm)
    changed=deepcopy(user);changed['physical_frames'].pop();reject('truncated-firmware-catalog',changed,mm)
    changed=deepcopy(user);changed['reservations'][0][1]-=1;reject('changed-kernel-reservation-end',changed,mm)
    for name,path,replacement in [
        ('wrong-kernel-observation',['kernel_sha256'],'0'*64),
        ('retained-final-MM-owner',['final','ids',0],1),
        ('truncated-caller-retirement',['events',0,'gateway_retirement','caller','bytes'],4072),
        ('unbound-private-retirement',['events',0,'gateway_retirement','private','sha256'],'0'*64),
    ]:
        changed=deepcopy(mm);target=changed
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,user,changed)
    changed=deepcopy(mm)
    changed['events'][0]['before']['kernel_spans'].pop()
    reject('omitted-linked-kernel-load-span',user,changed)
    changed=deepcopy(mm)
    next(row for row in changed['events'] if row['operation']==7)['after']['ids'][2]=0
    reject('exec-lost-image-generation',user,changed)
    for name,operation,path,replacement in [
        ('creation-missing-copy-pin',9,['after','prepared','record',6],0),
        ('handoff-lost-inactive-owner',11,['after','ids',3],0),
        ('child-exit-missing-retirement',12,['retired'],[]),
    ]:
        candidates=[row for row in mm['events'] if row['operation']==operation
                    and (operation!=9 or row['after']['prepared']['record'])
                    and (operation!=12 or row['retired'])]
        if not candidates:continue
        changed=deepcopy(mm);target=changed['events'][mm['events'].index(candidates[0])]
        for key in path[:-1]:target=target[key]
        target[path[-1]]=replacement;reject(name,user,changed)
    status=topal_gate.PROCESS_WORKLOADS[workload][2]
    marker=f'TOPAL_KERNEL_PROCESS_EXIT_STATUS={status:02x}'.encode()
    require(marker in serial, 'actual status marker')
    changed=serial.replace(marker,f'TOPAL_KERNEL_PROCESS_EXIT_STATUS={status^1:02x}'.encode())
    reject('changed-terminal-console-status',user,mm,changed)
    reject('extra-successful-console-exec',user,mm,serial+topal_gate.EXEC_COMMIT_MARKER)
    retired=[record for row in mm['events'] for record in row['retired']]
    return dict(passed=True,workload=workload,boundaries=len(mm['events']),operations=dict(sorted(counts.items())),
        retired_extents=len(retired),erased_released_bytes=sum(row['count']*4096 for row in retired),controls=controls,
        reached_extent_sites=len(user['extent_call_coverage']['reached']),physical_frames=user['n'],
        scope='Joined unchanged application console, legacy/MM boundaries and final complete catalog/registry digests; not every observer assertion, continuous lifetime, a VM replay or public activation')


def facts(build):
    sys.path.insert(0,str(ROOT/'linux-kernel/tools/user-memory'))
    from inspect_call_inventory import Elf
    elf=Elf(build/'kernel.elf')
    require(elf.kind==2, 'linked kernel ELF')
    phoff=struct.unpack_from('<Q',elf.data,32)[0]
    size,count=struct.unpack_from('<HH',elf.data,54)
    require(size==56 and phoff+size*count<=len(elf.data), 'whole ELF program headers')
    spans=[]
    for i in range(count):
        kind,flags,offset,virtual,physical,filesize,memsize,alignment=struct.unpack_from('<IIQQQQQQ',elf.data,phoff+i*size)
        if kind==1:
            require(virtual==physical and 0<=physical<physical+memsize<=1<<30
                    and filesize<=memsize and offset+filesize<=len(elf.data), 'kernel physical load span')
            spans.append([physical,physical+memsize])
    reservation=elf.symbols['topal_x86_kernel_reservation_end_v1']['value']
    require(spans and max(end for _,end in spans)<=reservation<=1<<30, 'complete kernel reservation')
    return dict(kernel_sha256=digest(elf.data)['sha256'],inventory_sha256=digest((build/'extent-call-inventory.json').read_bytes())['sha256'],
        kernel_spans=spans,reservation_end=reservation,boot_resource_bytes=(build/'initramfs.cpio').stat().st_size,
        root_token_bytes=elf.symbols['topal_x86_frame_root_tokens_v1']['size'],
        retirement_symbols={name:elf.symbols['topal_x86_frame_'+name+'_v1']['value'] for name in SIZES if name!='caller'})


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path);parser.add_argument('workload')
    args=parser.parse_args();build=args.build.resolve()
    require(build.is_relative_to(ROOT/'linux-kernel'), 'Linux build path')
    native=build/'process-mm-applications'/args.workload
    require(native.resolve().is_relative_to(build), 'application record path')
    result=readback(json.loads((native/'observations.json').read_text()),
        json.loads((native/'process-mm-observations.json').read_text()),
        (native/'serial.log').read_bytes(),args.workload,facts(build))
    (native/'readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({key:value for key,value in result.items() if key not in ['controls','scope']}))


if __name__=='__main__':main()
