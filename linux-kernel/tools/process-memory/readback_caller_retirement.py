#!/usr/bin/env python3
"""Bind normal-return samples to the unchanged complete ordinary MM guest.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001. This is
saved-data replay, not another VM or a continuous lifetime qualification.
"""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys

from caller_retirement import checked,sha
from process_native import require
from readback_process import readback as process_readback

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
sys.path.insert(0,str(ROOT/'linux-kernel/tools/user-memory'))
from inspect_call_inventory import Elf
from process_mm_process_inspect import PROGRAM as BASE_PROGRAM
from process_mm_caller_retirement_observer import augment,layout


def readback(build):
    profile=layout(build);kernel=Elf(build/'kernel.elf')
    component=json.loads((build/'source-component.json').read_bytes())
    require(component['mode']=='process-mm','caller ordinary source component')
    require(profile==json.loads((build/'caller-retirement-layout.json').read_bytes()),'caller layout bytes')
    require(sha((build/'source-user.elf').read_bytes())==component['linked_sha256']
            and sha((build/'static-elf-abi-fixture.elf').read_bytes())==component['image_sha256'],
            'caller source/image identity')
    sources=dict(source='source-cases',startup='startup-cases',file='file-cases',shell='shell-cases',
        command='command-cases',process_mm='process-mm-cases')
    source_facts={}
    for key,stem in sources.items():
        raw=(build/(stem+'.c')).read_bytes()
        field='source_sha256' if key=='source' else key+'_source_sha256'
        require(sha(raw)==component[field],'caller native C source '+stem)
        source_facts[stem]=dict(bytes=len(raw),sha256=sha(raw))
    work=build/'process-mm-users';script=(work/'inspect.gdb').read_text()
    require(script.count(augment(BASE_PROGRAM))==1
            and script.count('caller_retirement_profile='+repr(profile)+'\n')==1,
            'caller complete observer/profile binding')
    require(json.loads((build/'guest.command.json').read_text())['exit_code']==0,'caller guest command')
    stdout=(work/'gdb.stdout.log').read_bytes()
    require(stdout.count(b'CALLER_RETIREMENT_PASS ')==1 and stdout.count(b'FRAME_ROOT_GDB_PASS')==1,
            'caller full native completion markers')
    qemu=json.loads((work/'qemu.command.json').read_bytes())
    require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
            and qemu[qemu.index('-accel')+1].startswith('tcg,'),'caller retained VM geometry')
    raw=(work/'observations.json').read_bytes();value=json.loads(raw)
    base=process_readback(value)
    data=value['caller_retirement'];result=checked(data,profile,kernel)
    controls=[]
    def reject(name,change):
        altered=deepcopy(data);change(altered)
        try:checked(altered,profile,kernel)
        except (AssertionError,ValueError) as error:controls.append(dict(name=name,rejected=True,reason=str(error)))
        else:raise AssertionError('accepted caller control: '+name)
    def first(data):return data['records'][0]
    reject('incomplete-normal-return',lambda r:r.update(complete=False))
    reject('retained-active-sample',lambda r:r.update(pending=deepcopy(first(r))))
    reject('duplicate-first-site',lambda r:r['records'].append(deepcopy(first(r))))
    reject('wrong-helper-address',lambda r:first(r).update(rip=first(r)['rip']+1))
    reject('wrong-gateway-direction',lambda r:first(r)['gateway_entry']['arguments'].update(
        rcx=1-first(r)['gateway_entry']['arguments']['rcx']))
    reject('lost-source-register',lambda r:first(r)['return']['registers'].update(
        r8=first(r)['return']['registers']['r8']^1))
    reject('wrong-result-carry',lambda r:first(r)['return'].update(flags=first(r)['return']['flags']^1))
    reject('gateway-helper-status-mismatch',lambda r:first(r)['gateway_return'].update(
        result=first(r)['gateway_return']['result']^1))
    def unknown_status(record):
        first(record)['gateway_return']['result']=5;first(record)['return'].update(result=5,flags=1)
    reject('unknown-status-vocabulary',unknown_status)
    reject('lost-outer-retirement',lambda r:first(r)['return']['caller'].update(bytes=4000))
    reject('dirty-inner-retirement',lambda r:first(r)['gateway_return']['inner_caller'].update(sha256='0'*64))
    reject('wrong-private-address',lambda r:first(r)['return']['domains']['topal_x86_frame_private_v1'].update(address=0))
    reject('stale-entry-epoch',lambda r:first(r)['return']['after']['epoch'].__setitem__(0,0))
    reject('disabled-write-protection',lambda r:first(r)['return']['after'].update(
        cr0=first(r)['return']['after']['cr0']&~(1<<16)))
    reject('lost-held-parent',lambda r:first(r)['return']['after']['ids'].__setitem__(0,
        first(r)['return']['after']['ids'][0]+1))
    reject('missing-live-heap-blob',lambda r:r['blobs'].pop(first(r)['before']['active']['bytes']['sha256']))
    reject('wrong-blob-width',lambda r:first(r)['before']['active']['bytes'].update(bytes=1))
    reject('wrong-blob-content',lambda r:r['blobs'].update(
        {first(r)['before']['active']['bytes']['sha256']:'01'}))
    reject('unused-heap-blob',lambda r:r['blobs'].update({sha(b'unused') : b'unused'.hex()}))
    result.update(kernel_sha256=profile['kernel_sha256'],inventory_sha256=profile['inventory_sha256'],
        observation_sha256=sha(raw),process_mm=base,controls=controls,source_files=source_facts)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path);args=parser.parse_args()
    build=args.build.resolve();result=readback(build)
    (build/'process-mm-users/caller-retirement-readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result))
