"""Bind saved process publications, CPU logs and retirement to the linked kernel.

TOPAL-SYSTEMS-PROCESS-MM-001. No guest execution, TLB introspection, continuous
exclusion, all mapping-writer coverage or public activation is established here.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import struct
import sys

from process_native import require,mappings
from publication import check,retirement

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/tools/user-memory'))
sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
from inspect_call_inventory import Elf
from inspect_translation_publication import inspect
from process_mm_process_inspect import PROGRAM as BASE_PROGRAM
from process_mm_publication_observer import augment
from qemu_publication_trace import parse,completed
from translation_publication import check_switch


def digest(raw):return hashlib.sha256(raw).hexdigest()


def facts(build):
    elf=Elf(build/'kernel.elf');require(elf.kind==2,'linked publication ELF')
    profile=inspect(build)
    profile.update(pin_entry=elf.symbols['topal_frame_allocator_pin_v1']['value'],
        release_entry=elf.symbols['topal_frame_allocator_release_v1']['value'])
    phoff=struct.unpack_from('<Q',elf.data,32)[0]
    size,count=struct.unpack_from('<HH',elf.data,54)
    require(size==56 and phoff+size*count<=len(elf.data),'complete publication ELF headers')
    spans=[]
    for i in range(count):
        kind,flags,offset,virtual,physical,filesize,memsize,alignment=struct.unpack_from('<IIQQQQQQ',elf.data,phoff+i*size)
        if kind==1:
            require(virtual==physical and 0<=physical<physical+memsize<=1<<30
                    and filesize<=memsize and offset+filesize<=len(elf.data),'publication kernel load span')
            spans.append([physical,physical+memsize])
    require(spans,'publication kernel spans')
    return elf,dict(profile=profile,kernel_spans=spans,
        component=json.loads((build/'source-component.json').read_text()))


def call(elf,site,target,caller):
    symbol,body=elf.function(caller)
    offset=site-symbol['value']
    require(5<=offset<=len(body) and body[offset-5]==0xe8,'publication recorded direct call')
    require(site+struct.unpack_from('<i',body,offset-4)[0]==target,'publication call target')


def unchanged_tables(first,last):
    require(set(first)==set(last),'publication/MM table closure')
    for name,words in first.items():
        actual=last[name]
        require(len(words)==len(actual)==512 and all(a&~0x60==b&~0x60 and a|b==b for a,b in zip(words,actual)),
                'publication/MM table binding')


def cohort_complete(value,mode):
    require(value['passed'] and 'failure' not in value
            and value['pending'] is value['refusal_pending'] is None,'publication ordinary cohort incomplete')
    terminal=mode in ['process-mm-allocation','process-mm-cancel','process-mm-relationship'] or mode.startswith('process-mm-geometry-')
    require(terminal or mode in ['process-mm','process-mm-exec'],'publication source cohort')
    # These source-built cases terminate with exec/exit rather than returning
    # through the original case marker. Their original cohort reader checks
    # replacement startup, terminal retirement and final authority separately.
    require(value['active_case'] is not None if terminal else value['active_case'] is None,
            'publication source case completion')


def check_saved(value,binding,elf,work):
    cohort_complete(value,binding['component']['mode'])
    native=value['process_publication'];profile=binding['profile']
    require(native['passed'] and native['profile']==profile and 'failure' not in native,
            'publication profile/native completion')
    records=native['records'];switches=native['root_switches']
    require(records and len(switches)==1,'publication records and terminal root handoff')
    for row in switches:
        require(row['comparison']==check_switch(row)
                and row['before_site']==profile['root_switch_sites']['before']
                and row['after_site']==profile['root_switch_sites']['after']
                and row['source_register']==row['before']['context']['bootstrap'],'publication observed root switch')
    segments={};results=[]
    callers={5:'topal_frame_memory_request_v1',6:'topal_frame_memory_request_v1',7:'topal_frame_memory_request_v1',
             10:'p.commit',11:'p.switch',12:'p.child_exit'}
    for row in records:
        for key in ['before','stored','after']:
            require(row[key]['kernel_spans']==binding['kernel_spans'],'publication linked kernel spans')
        mappings(row['before'],row['old_count'],row['before']['state'][3])
        for key in ['stored','after']:mappings(row[key],row['new_count'],row['new_base'])
        require(row['operation'] in callers,'publication operation')
        call(elf,row['entry_return_address'],profile['sites']['entry'],callers[row['operation']])
        traced=[];cancelled=[]
        for part in row['trace_slices']:
            name=part['file']
            require(Path(name).name==name and name.startswith('cpu-') and name.endswith('.log'),'publication trace path')
            data=(work/name).read_bytes();start,size=part['start'],part['size']
            require(0<=start<=start+size<=len(data),'publication trace slice')
            require(not segments.get(name) or start>=segments[name][-1][1],'publication overlapping CPU slices')
            segments.setdefault(name,[]).append([start,start+size])
            raw=data[start:start+size]
            require(digest(raw)==part['sha256'],'publication CPU slice identity')
            events=parse(raw,profile['sites']['invalidated'])
            for event in events:
                require(event['cpu']+1==row['before']['thread'][1] and row['before']['thread'][0]==1
                        and (event['cr0'],event['cr3'],event['cr4'])==tuple(row['before'][key] for key in ['cr0','cr3','cr4'])
                        and event['efer']&0xd00==0xd00 and event['flags']&0x600==0,'publication actual CPU/mode')
            states,retries=completed(events)
            traced.extend(dict(event,thread=row['before']['thread']) for event in states);cancelled.extend(retries)
        require(traced==row['invalidations'] and cancelled==row['cancelled_post_states'],'publication CPU logs differ')
        result=check(row);require(result==row['comparison'],'publication comparison differs');results.append(result)
    for path in work.glob('cpu-*.log'):
        raw=path.read_bytes();actual=parse(raw,profile['sites']['invalidated'])
        saved=sum(len(parse(raw[a:b],profile['sites']['invalidated'])) for a,b in segments.get(path.name,[]))
        require(len(actual)==saved,'unattributed publication CPU headers')
    used=[];retired=[]
    for event in value['events']:
        indices=event['publication_indices'];operation=event['operation']
        old,new=event['before']['state'][2],event['after']['state'][2]
        expected=1 if operation in [10,11,12] else int(old!=new) if operation==5 else int(old>0) if operation in [6,7] else 0
        require(len(indices)==expected and indices==list(range(len(used),len(used)+expected)),
                'publication complete MM request sequence')
        for index in indices:
            row=records[index]
            require(row['operation']==operation and row['request_sp']==event['sp']
                    and row['old_count']==old and row['new_count']==new
                    and row['new_base']==event['after']['state'][3],'publication MM request binding')
            require({k:v for k,v in row['before']['active'].items() if k!='bytes'}==
                    {k:v for k,v in event['before']['active'].items() if k!='bytes'},'publication actual old MM owner')
            unchanged_tables(event['before']['tables'],row['before']['tables'])
            unchanged_tables(row['after']['tables'],event['after']['tables'])
        used.extend(indices)
        releases=event['publication_retirements']
        require(len(releases)==len(event['retired']) and sorted(r['owner']['token'] for r in releases)==
                sorted(r['token'] for r in event['retired']),'publication complete MM retirement sequence')
        for row in releases:
            require(row['operation']==operation and row['publication_count']==len(used)
                    and row['before']['kernel_spans']==binding['kernel_spans'],'retirement MM/kernel binding')
            role=row['bank']
            pointer=elf.symbols['topal_x86_frame_mm_tokens_v1' if role=='active' else 'topal_x86_process_mm_inactive_token_v1']['value']
            require(role in ['active','inactive'] and row['token_pointer']==pointer,'retirement canonical token bank')
            caller='topal_frame_memory_request_v1' if operation in [5,6,7] else 'p.retire'
            call(elf,row['unpin_return'],profile['pin_entry'],caller)
            call(elf,row['release_return'],profile['release_entry'],caller)
            result=retirement(row,records[:row['publication_count']])
            require(result==row['comparison'],'retirement comparison differs');retired.append(result)
    require(used==list(range(len(records))),'publication not bound to MM request')
    return dict(passed=True,publications=len(records),pages_invalidated=sum(r['pages_invalidated'] for r in results),
        equal_size_process_publications=sum(r['operation'] in [10,11,12] and r['old_count']==r['new_count'] for r in records),
        retirements=len(retired),erased_bytes=sum(r['erased_bytes'] for r in retired),
        root_switches=len(switches),cancelled_CPU_blocks=sum(len(r['cancelled_post_states']) for r in records))


def readback(build,work_name='process-mm-users'):
    work=build/work_name;raw=(work/'observations.json').read_bytes();value=json.loads(raw)
    elf,binding=facts(build)
    require((work/'inspect.gdb').read_text().count(augment(BASE_PROGRAM))==1,'bound complete process publication observer')
    mode=binding['component']['mode']
    if mode=='process-mm':from readback_process import readback as original
    elif mode=='process-mm-exec':from readback_exec import readback as original
    elif mode=='process-mm-cancel':from readback_cancel import readback as original
    elif mode=='process-mm-allocation':from readback_allocation import readback as original
    elif mode.startswith('process-mm-geometry-'):from readback_geometry import readback as original
    elif mode=='process-mm-relationship':from readback_relationship import readback as original
    else:raise ValueError('unsupported publication source cohort')
    legacy=original(value)
    result=check_saved(value,binding,elf,work);controls=[]
    def reject(name,change):
        changed=deepcopy(value);change(changed)
        try:check_saved(changed,binding,elf,work)
        except (AssertionError,ValueError) as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted saved publication control: '+name)
    first=next(i for i,row in enumerate(value['process_publication']['records']) if row['invalidations'])
    def publication(v):return v['process_publication']['records'][first]
    reject('lost-post-INVLPG-CPU-state',lambda v:publication(v)['invalidations'].pop())
    reject('wrong-published-owner-base',lambda v:publication(v).__setitem__('new_base',publication(v)['new_base']^4096))
    reject('wrong-linked-kernel-span',lambda v:publication(v)['before']['kernel_spans'].pop())
    reject('wrong-terminal-root-source',lambda v:v['process_publication']['root_switches'][0].update(source_register=0))
    reject('missing-publication-barrier',lambda v:publication(v).update(final_barrier_observed=False))
    def remove_hierarchy_user_permission(v):
        row=publication(v)
        for name in ['before','stored','after']:
            row[name]['tables'][str(row[name]['context']['root'])][0]&=~4
    reject('lost-hierarchy-user-permission',remove_hierarchy_user_permission)
    def foreign_unrelated_mapping(v):
        row=publication(v)
        for name in ['before','stored','after']:
            row[name]['tables'][str(row[name]['context']['root'])][8]=7
    reject('unbound-unrelated-mapping-between-MM-boundaries',foreign_unrelated_mapping)
    event_index=next(i for i,row in enumerate(value['events']) if row['publication_retirements'])
    def release(v):return v['events'][event_index]['publication_retirements'][0]
    reject('unpin-before-completed-publication',lambda v:release(v).update(publication_count=0))
    reject('failed-unpin-return',lambda v:release(v).update(unpin_succeeded=False))
    reject('dirty-old-backing-before-release',lambda v:release(v)['release'].update(erased_sha256='0'*64))
    reject('retained-released-owner',lambda v:release(v)['released']['record'].__setitem__(7,1))
    reject('wrong-native-unpin-call',lambda v:release(v).__setitem__('unpin_return',release(v)['unpin_return']+1))
    result.update(controls=controls,original_cohort=legacy,mode=mode,kernel_sha256=binding['profile']['kernel_sha256'],
        observation_sha256=digest(raw),scope='Bound local publisher/CPU traces and heap retirement plus original cohort replay; no VM replay, continuous exclusion, all writers, AP/DMA/async/fatal or public activation')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path)
    parser.add_argument('--work',default='process-mm-users');args=parser.parse_args()
    build=args.build.resolve();require(build.is_relative_to(ROOT/'linux-kernel'),'Linux publication build')
    require(Path(args.work).name==args.work and args.work.startswith('process-mm-users'),'publication work directory')
    result=readback(build,args.work)
    (build/args.work/'publication-readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['controls','original_cohort','scope']}))
