#!/usr/bin/env python3
"""Bind a focused pre-import syscall/owned-helper prefix to its native inputs.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001. This
does not accept whole application/VM completion or continuous lifetime proof.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from process_native import require, snapshot
from readback_process import event
from policy_failure import tables

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
sys.path.insert(0,str(ROOT/'linux-kernel/tools/user-memory'))
from process_mm_prelude_stat_inspect import PROGRAM, layout
from inspect_call_inventory import Elf
from readback_publication import call


def sha(raw):return hashlib.sha256(raw).hexdigest()


def stable(first,last):
    snapshot(first);snapshot(last)
    for key in ['allocator','ids','state','tokens','inactive_state','prepared_state','prepared_owner',
                'transition','active','candidate','inactive','prepared','root_owners','kernel_spans']:
        require(first[key]==last[key],'prelude retained MM '+key)
    for key in ['address','state','root','low','heap','self','bootstrap','table']:
        require(first['context'][key]==last['context'][key],'prelude context '+key)
    require(first['cr3']==last['cr3']==last['context']['root']
            and first['cr0']==last['cr0'] and first['cr4']==last['cr4']
            and last['cr0']&(1<<16),'prelude paging mode/WP')
    tables(first['tables'],last['tables'])


def checked(value,component,profile,kernel,user):
    require(value['passed'] and 'failure' not in value and value['pending'] is None
            and value['full_ordinary_cohort_complete'] is False,'prelude focused outcome')
    require(value['events'] and all(event(row)==row['comparison'] for row in value['events']),
            'prelude retained MM event prefix')
    v=value['prelude_stat'];request=v['user_request'];returned=v['user_return']
    source=v['source_call'];done=v['source_return'];gateway=v['gateway_entry'];gate_return=v['gateway_return']
    marker=v['marker'];before=request['snapshot']
    require(request['rip']==component['syscall4'] and request['arguments']==dict(
        rdi=262,rsi=(1<<64)-100,rdx=profile['path'],rcx=(1<<64)-1,r8=0),
        'prelude actual syscall arguments')
    call(user,request['return_pc'],component['syscall4'],'topal_user_extent_command_cases')
    require(before['cs']&3==3 and before['context']['phase']==29 and before['table_live']==0,
            'prelude pre-import entry state')
    require(before['state'][1:3]==[0x619000,34] and before['ids'][0]==1
            and before['ids'][3:6]==[0]*3,'prelude retained parent')
    require(source['rip']==profile['call'] and done['rip']==profile['returned']
            and source['arguments']==dict(r8=profile['path'],r9=9),'prelude selected native helper')
    require(source['snapshot']['cs']&3==done['snapshot']['cs']&3==0
            and source['flags']&0x600==done['flags']&0x601==0,'prelude helper critical/result state')
    require(done['sp']==source['sp'] and done['callee']==source['callee'],'prelude helper caller ABI')
    require(gateway['rip']==kernel.symbols['topal_x86_frame_gateway_v1']['value']
            and gateway['arguments']==dict(rdi=8,rsi=profile['path'],rdx=9,rcx=0),
            'prelude actual owned-gateway arguments')
    call(kernel,gateway['return_pc'],kernel.symbols['topal_x86_frame_gateway_v1']['value'],
         'topal_x86_user_extent_call_v1')
    require(gate_return['rip']==kernel.symbols['topal_x86_frame_gateway_return_reclaimed_v1']['value']
            and gate_return['sp']==gateway['sp'] and gate_return['result']==0,
            'prelude actual gateway return')
    outer=source['sp']-gateway['sp'];require(outer in [80,88],'prelude outer adapter geometry')
    low=gateway['sp']-4000
    require(gate_return['inner_caller']==dict(address=low,bytes=4000,sha256=sha(bytes(4000))),
            'prelude inner caller retirement')
    require(done['caller']==dict(address=low,bytes=4000+outer,sha256=sha(bytes(4000+outer))),
            'prelude complete actual caller retirement')
    sizes=dict(private=65528,stack=4096,facts=24,output=56,details=32,
        topal_x86_frame_gateway_recovery_v1=48,topal_frame_allocator_active_v1=8,
        topal_x86_tail_boundary_active_v1=8)
    for row in [gate_return,done]:
        require(set(row['domains'])==set(sizes),'prelude retirement domain closure')
        for name,size in sizes.items():
            symbol='topal_x86_frame_'+name+'_v1' if name in ['private','stack','facts','output','details'] else name
            require(row['domains'][name]==dict(address=kernel.symbols[symbol]['value'],bytes=size,
                sha256=sha(bytes(size))),'prelude private retirement '+name)
    for row in [source['snapshot'],done['snapshot'],returned['snapshot'],marker['snapshot']]:
        stable(before,row)
        require(row['epoch']==[before['epoch'][0]+1,262] and row['table_live']==0,
                'prelude fresh entry/retained pre-import owner')
    require(source['snapshot']['context']['phase']==done['snapshot']['context']['phase']==29,
            'prelude helper precedes missing observation')
    require(returned['rip']==request['return_pc'] and returned['sp']==request['sp']+8
            and returned['result']==(1<<64)-2 and returned['callee']==request['callee']
            and returned['snapshot']['cs']&3==3 and returned['snapshot']['context']['phase']==30,
            'prelude actual negative syscall return')
    require(marker['rip']==profile['entry'] and marker['result']==returned['result']
            and marker['snapshot']['cs']&3==3 and marker['snapshot']['context']['phase']==30,
            'prelude actual userspace completion marker')
    return dict(passed=True,helper_offset=profile['offset'],syscall=262,result=-2,
        inner_caller_bytes=4000,outer_adapter_bytes=outer,physical_caller_bytes=4000+outer,
        prefix_MM_events=len(value['events']),held_heap_pages=34,
        full_ordinary_cohort_complete=False,
        scope='Selected native pre-import missing-stat prefix, whole retained heap and observed helper/private retirement; no full ABI/application/terminal VM, every caller/path, continuous lifetime or public qualification')


def readback(build):
    component,profile=layout(build);kernel=Elf(build/'kernel.elf');user=Elf(build/'source-user.elf')
    work=build/'process-mm-prelude-stat'
    require((work/'inspect.gdb').read_text().count(PROGRAM)==1,'prelude complete observer program')
    require(profile==json.loads((build/'prelude-stat-layout.json').read_text()),'prelude native layout binding')
    require(sha((build/'static-elf-abi-fixture.elf').read_bytes())==component['image_sha256']
            and sha(user.data)==component['linked_sha256'],'prelude source image binding')
    require(json.loads((build/'guest.command.json').read_text())['exit_code']==0
            and (work/'gdb.stdout.log').read_bytes().count(b'PRELUDE_STAT_PREFIX_PASS')==1,
            'prelude native prefix completion')
    qemu=json.loads((work/'qemu.command.json').read_text())
    require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
            and qemu[qemu.index('-accel')+1].startswith('tcg,'),'prelude VM geometry')
    raw=(work/'observations.json').read_bytes();value=json.loads(raw)
    result=checked(value,component,profile,kernel,user);controls=[]
    def reject(name,change):
        altered=deepcopy(value);change(altered)
        try:checked(altered,component,profile,kernel,user)
        except (AssertionError,ValueError) as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted prelude control: '+name)
    def v(row):return row['prelude_stat']
    reject('false-whole-cohort-completion',lambda r:r.update(full_ordinary_cohort_complete=True))
    reject('wrong-syscall',lambda r:v(r)['user_request']['arguments'].update(rdi=5))
    reject('wrong-native-helper-call',lambda r:v(r)['source_call'].update(rip=profile['call']+1))
    reject('failed-helper-admission',lambda r:v(r)['source_return'].update(flags=v(r)['source_return']['flags']|1))
    reject('lost-outer-retirement',lambda r:v(r)['source_return']['caller'].update(bytes=4000))
    reject('dirty-inner-retirement',lambda r:v(r)['gateway_return']['inner_caller'].update(sha256='0'*64))
    reject('wrong-private-retirement-address',lambda r:v(r)['source_return']['domains']['private'].update(address=0))
    reject('stale-syscall-entry',lambda r:v(r)['user_return']['snapshot']['epoch'].__setitem__(0,0))
    reject('lost-parent-handle',lambda r:v(r)['user_return']['snapshot']['active'].update(token=[0]*3))
    reject('premature-descriptor-import',lambda r:v(r)['user_request']['snapshot'].update(table_live=1))
    reject('wrong-result',lambda r:v(r)['user_return'].update(result=0))
    reject('skipped-startup-transition',lambda r:v(r)['user_return']['snapshot']['context'].update(phase=29))
    result.update(kernel_sha256=sha(kernel.data),observation_sha256=sha(raw),controls=controls)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path);args=parser.parse_args()
    build=args.build.resolve();result=readback(build)
    (build/'process-mm-prelude-stat/readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result))
