#!/usr/bin/env python3
"""Bind selected fatal outcomes to source-built VM artifacts and raw CPU logs.

TOPAL-SYSTEMS-PROCESS-MM-001. Saved-data controls are reader checks, never
additional guest executions or qualification of every fatal/retirement path.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from policy_failure import check, sha_words, tables
from process_native import mappings, require
from readback_process import event
from publication import retirement
from readback_publication import call, check_publications, facts
from process_mm_policy_failure_inspect import program


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def source_entry(fault):
    """Reconstruct the exact prefix before the deliberately invalid source delta."""
    before=fault['request']['before'];source=fault['source_before'];actual=source['snapshot']
    expected=deepcopy(before)
    if fault['kind'].startswith('release-'):
        candidate=actual['candidate']
        require(candidate['token'][1]==before['allocator'][2], 'diagnostic candidate issuance')
        expected['allocator'][2]+=1
        expected['candidate']=candidate
        expected['tokens'][3:]=candidate['token']
        expected['active']['record'][6]=0
        expected['active']['bytes']=bytes(67*4096).hex()
        root=before['context']['root'];low=before['context']['low'];base=candidate['cells'][2]*4096
        for i in range(101):
            table,index=(low,503+i) if i<9 else (root+4*4096,i-9)
            expected['tables'][str(table)][index]=(base+i*4096)|(1<<63)|7
    require(set(expected)==set(actual), 'diagnostic source-entry snapshot closure')
    for key in expected:
        if key=='flags':require(actual[key]&0x600==0, 'diagnostic source-entry critical state')
        elif key=='tables':tables(expected[key],actual[key])
        else:require(expected[key]==actual[key], 'diagnostic source-entry changed '+key)
    n=actual['allocator'][3];registry=[0]*(n*8);catalog=source['catalog'];occupied=set()
    owned=[actual[k] for k in ['active','inactive','prepared','candidate']]+actual['root_owners']
    for owner in owned:
        if owner['token']==[0]*3:continue
        index=owner['token'][2];record=owner['record'];start,count=record[2:4]
        require(index not in occupied, 'diagnostic registry owner alias');occupied.add(index)
        registry[index*8:(index+1)*8]=record
        require(owner['cells']==catalog[start*3:(start+count)*3], 'diagnostic owner/catalog binding')
    require(sha_words(registry)==source['registry_sha256'], 'diagnostic complete registry reconstruction')


def checked(value,binding,elf,work):
    require(value['passed'] and 'failure' not in value and value['refusal_pending'] is None,
            'diagnostic native outcome incomplete')
    fault=value['policy_failure'];request=fault['request'];source=fault['source_before']['snapshot']
    require(value['pending']==request and value['active_case'] is not None,
            'diagnostic lost retained terminal request/case')
    source_entry(fault)
    result=check(fault)
    require(result==fault['comparison'],'diagnostic comparison differs')
    allocating=fault['kind'].startswith('allocate-');partial=fault['kind'].endswith('-partial-status')
    symbols=elf.symbols
    entry_name='topal_frame_allocator_'+('allocate' if allocating else 'release')+'_v1'
    initial=fault['source_before'];arguments=initial['arguments']
    require(initial['entry_pc']==symbols[entry_name]['value']
            and arguments['rdi']==symbols['topal_x86_frame_context_v1']['value'],
            'diagnostic actual allocator call entry')
    if allocating:
        require(arguments['rsi']==101 and arguments['rdx']==1
                and arguments['rcx']==symbols['topal_x86_frame_mm_tokens_v1']['value']+24,
                'diagnostic actual allocation arguments')
    else:
        require(arguments['rsi']==symbols['topal_x86_frame_mm_tokens_v1']['value'],
                'diagnostic actual release argument')
    call(elf,initial['return_pc'],symbols[entry_name]['value'],'topal_frame_memory_request_v1')
    returned=fault['source_result']
    require(returned['entry_pc']==symbols['f.result']['value']
            and returned['output_pointer']==symbols['topal_x86_frame_output_v1']['value'],
            'diagnostic actual source decoder entry')
    call(elf,returned['return_pc'],symbols['f.result']['value'],'f.invoke')
    effects=fault['effects']
    require(effects['entry_pc']==symbols['f.fatal']['value'],'diagnostic actual controller fatal entry')
    call(elf,effects['return_pc'],symbols['f.fatal']['value'],'f.invoke' if partial else entry_name)
    native=value['process_publication'];records=native['records'];profile=binding['profile']
    require(not native['passed'] and native['root_switches']==[],
            'diagnostic claimed ordinary cohort completion')
    publications=check_publications(native,binding,elf,work)
    used=[];retired=0
    for row in value['events']:
        require(event(row)==row['comparison'],'diagnostic ordinary prefix comparison')
        indices=row['publication_indices'];op=row['operation']
        old,new=row['before']['state'][2],row['after']['state'][2]
        count=1 if op in [10,11,12] else int(old!=new) if op==5 else int(old>0) if op in [6,7] else 0
        require(indices==list(range(len(used),len(used)+count)), 'diagnostic prefix publication sequence')
        for index in indices:
            pub=records[index]
            require(pub['operation']==op and pub['request_sp']==row['sp']
                    and (pub['old_count'],pub['new_count'],pub['new_base'])==(old,new,row['after']['state'][3]),
                    'diagnostic prefix MM binding')
            for key in ['token','record','cells']:
                require(pub['before']['active'][key]==row['before']['active'][key], 'diagnostic prefix old owner')
            tables(row['before']['tables'],pub['before']['tables'])
            tables(pub['after']['tables'],row['after']['tables'])
        used.extend(indices)
        releases=row['publication_retirements']
        require(sorted(r['owner']['token'] for r in releases)==sorted(r['token'] for r in row['retired']),
                'diagnostic prefix retirement closure')
        for release in releases:
            require(retirement(release,records[:release['publication_count']])==release['comparison'],
                    'diagnostic prefix retirement differs')
            require(release['operation']==op and release['publication_count']==len(used),
                    'diagnostic prefix retirement binding')
            call(elf,release['unpin_return'],profile['pin_entry'],'topal_frame_memory_request_v1')
            call(elf,release['release_return'],profile['release_entry'],'topal_frame_memory_request_v1')
            require(release['token_pointer']==elf.symbols['topal_x86_frame_mm_tokens_v1']['value'],
                    'diagnostic prefix token address')
            retired+=1
    indices=request['publication_indices']
    require(indices==list(range(len(used),len(records))) and request['publication_retirements']==[],
            'diagnostic pending publication sequence')
    if not allocating:
        pub=records[indices[0]];partial=fault['open_retirement']
        require(pub['operation']==5 and pub['request_sp']==request['sp']
                and (pub['old_count'],pub['new_count'])==(67,101),'diagnostic failed resize binding')
        for key in ['token','record','cells']:
            require(pub['before']['active'][key]==request['before']['active'][key], 'diagnostic failed old owner')
            require(pub['after']['candidate'][key]==source['candidate'][key], 'diagnostic published candidate')
        tables(request['before']['tables'],pub['before']['tables'])
        tables(pub['after']['tables'],source['tables'])
        require(partial['publication_index']==indices[0] and partial['publication_count']==len(records)
                and partial['bank']=='active' and partial['operation']==5
                and partial['token_pointer']==elf.symbols['topal_x86_frame_mm_tokens_v1']['value'],
                'diagnostic incomplete release publication binding')
        require(partial['owner']['token']==source['active']['token']
                and partial['unpinned']['record']==source['active']['record'], 'diagnostic unpin identity')
        base=source['active']['cells'][2]*4096
        require(partial['erase']==dict(base=base,count=67)
                and partial['release']==dict(base=base,count=67,erased_bytes=67*4096,
                    erased_sha256=digest(bytes(67*4096))), 'diagnostic erasure before failed release')
        call(elf,partial['unpin_return'],profile['pin_entry'],'topal_frame_memory_request_v1')
        call(elf,partial['release_return'],profile['release_entry'],'topal_frame_memory_request_v1')
        mappings(source,101,source['candidate']['cells'][2]*4096)
    else:
        mappings(source,67,source['active']['cells'][2]*4096)
    for row in [request['before'],source,fault['effects']['snapshot'],fault['terminal']['snapshot']]:
        require(row['kernel_spans']==binding['kernel_spans'], 'diagnostic linked kernel spans')
    terminal=fault['terminal']
    require(terminal['rip']==symbols['topal_x86_frame_gateway_fatal_reclaimed_v1']['value'],
            'diagnostic terminal marker')
    abort,body=elf.function('topal_x86_frame_gateway_abort_v1')
    offset=terminal['rip']-abort['value']
    require(0<=offset and body[offset:offset+4]==bytes.fromhex('fa f4 eb fd'),
            'diagnostic linked nonreturning halt loop')
    frame=json.loads((work.parent/'frame-inspection.json').read_text())
    require(frame['syscall_adapter']['physical_retirement_bytes']==4000,
            'diagnostic actual caller profile')
    addresses=dict(caller=request['sp']-4000,
        **{name:symbols['topal_x86_frame_'+name+'_v1']['value'] for name in
           ['private','stack','facts','output','details']},
        recovery=symbols['topal_x86_frame_gateway_recovery_v1']['value'],
        allocator_active=symbols['topal_frame_allocator_active_v1']['value'],
        boundary_active=symbols['topal_x86_tail_boundary_active_v1']['value'])
    require(all(terminal['retired'][k]['address']==v for k,v in addresses.items()),
            'diagnostic physical retirement address binding')
    result.update(prefix_events=len(value['events']),completed_publications=len(publications),
                  pages_invalidated=sum(r['pages_invalidated'] for r in publications),
                  completed_prefix_retirements=retired)
    return result


def readback(build):
    work=build/'process-mm-policy-failure'
    elf,binding=facts(build)
    script=(work/'inspect.gdb').read_text()
    require(script.count(program())==1, 'diagnostic complete observer binding')
    diagnostic=json.loads((build/'process-mm-policy-failure.json').read_text())
    policy=(build/'diagnostic-policy.t').read_bytes()
    require(digest(policy)==diagnostic['source_sha256'], 'diagnostic policy identity')
    obligation=json.loads((build/'source-obligation.json').read_text())
    require(obligation['library']['text'].encode()==policy
            and obligation['library']['source_name']==diagnostic['source_name']
            and obligation['root_source']==(build/'root.t').read_text()
            and obligation['root_sha256']==digest((build/'root.t').read_bytes())
            and obligation['controller_sha256']==digest((build/'controller.ll').read_bytes()),
            'diagnostic checked source/controller input binding')
    require(diagnostic['source_name'].rsplit('/',1)[0] in (build/'source.ll').read_text(),
            'diagnostic compiled source attribution')
    source=(build/'process-mm-cases.c').read_bytes()
    require(digest(source)==binding['component']['process_mm_source_sha256'], 'diagnostic ordinary application source')
    require(digest((build/'static-elf-abi-fixture.elf').read_bytes())==binding['component']['image_sha256']
            and digest((build/'source-user.elf').read_bytes())==binding['component']['linked_sha256'],
            'diagnostic application image identity')
    require(json.loads((build/'guest.command.json').read_text())['exit_code']==0
            and (work/'gdb.stdout.log').read_bytes().count(b'FRAME_ROOT_GDB_PASS')==1, 'diagnostic guest completion')
    qemu=json.loads((work/'qemu.command.json').read_text())
    require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
            and qemu[qemu.index('-accel')+1].startswith('tcg,'), 'diagnostic VM geometry')
    raw=(work/'observations.json').read_bytes();value=json.loads(raw)
    require(value['policy_failure']['kind']==diagnostic['kind'], 'diagnostic kind binding')
    result=checked(value,binding,elf,work);controls=[]
    def reject(name,change):
        changed=deepcopy(value);change(changed)
        try:checked(changed,binding,elf,work)
        except (AssertionError,ValueError) as error:
            controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted diagnostic control: '+name)
    def fault(v):return v['policy_failure']
    reject('rolled-back-performed-source-effects',lambda v:fault(v)['terminal'].update(catalog_sha256=fault(v)['source_before']['catalog_sha256']))
    reject('lost-comparison-snapshot-identity',lambda v:fault(v)['effects'].update(snapshot_catalog_sha256='0'*64))
    reject('rewound-allocator-issuance',lambda v:fault(v)['terminal']['snapshot']['allocator'].__setitem__(2,1))
    reject('lost-inactive-parent-pin',lambda v:fault(v)['terminal']['snapshot']['inactive']['record'].__setitem__(6,0))
    reject('reclaimed-held-process-token',lambda v:fault(v)['terminal']['snapshot']['active'].__setitem__('token',[0]*3))
    reject('dirty-fatal-caller',lambda v:fault(v)['terminal']['retired']['caller'].update(sha256='0'*64))
    reject('wrong-retirement-address',lambda v:fault(v)['terminal']['retired']['private'].update(address=0))
    reject('unpoisoned-MM-owner',lambda v:fault(v)['terminal']['snapshot']['state'].__setitem__(0,1))
    reject('changed-held-registry',lambda v:fault(v)['terminal'].update(registry_sha256='0'*64))
    reject('false-ordinary-completion',lambda v:v['process_publication'].update(passed=True))
    reject('lost-post-INVLPG-state',lambda v:v['process_publication']['records'][0]['invalidations'].pop())
    reject('lost-terminal-request',lambda v:v.update(pending=None))
    reject('wrong-source-natural-result',lambda v:fault(v)['source_result']['output_words'].__setitem__(2,0))
    reject('wrong-native-source-decoder-call',lambda v:fault(v)['source_result'].__setitem__('return_pc',fault(v)['source_result']['return_pc']+1))
    reject('wrong-native-fatal-call',lambda v:fault(v)['effects'].__setitem__('return_pc',fault(v)['effects']['return_pc']+1))
    if diagnostic['kind'].endswith('-partial-status'):
        reject('pretended-source-completion',lambda v:fault(v)['effects']['header'].__setitem__(5,4))
        reject('lost-performed-prefix',lambda v:fault(v)['effects'].update(catalog_sha256=fault(v)['source_before']['catalog_sha256']))
    if diagnostic['kind'].startswith('release-'):
        reject('false-release-consumption',lambda v:fault(v)['open_retirement'].update(release_succeeded=True))
        reject('release-before-unpin',lambda v:fault(v)['open_retirement'].update(unpin_succeeded=False))
        reject('missing-replacement-publication',lambda v:fault(v)['request']['publication_indices'].clear())
    result.update(kernel_sha256=binding['profile']['kernel_sha256'],policy_sha256=digest(policy),
                  observation_sha256=digest(raw),controls=controls)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path)
    args=parser.parse_args();build=args.build.resolve()
    result=readback(build)
    (build/'process-mm-policy-failure/readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result))
