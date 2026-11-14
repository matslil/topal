#!/usr/bin/env python3
"""Bind normal-return samples to complete ordinary interface/MM observations.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
Saved-data checks do not prove every path or continuous lifetime.
"""
import argparse
import ast
import json
from pathlib import Path
import sys

from caller_retirement import sha
from process_native import require
from readback_caller_retirement import sample_readback
from readback_interfaces import readback as interface_readback

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
from inspect_call_inventory import Elf
from process_mm_caller_retirement_observer import boundary_program,layout
from process_mm_regression_observer import BOUNDARY_PROGRAM
from user_extent_source_inspect import PROGRAM


def readback(build):
    component=json.loads((build/'source-component.json').read_bytes())
    require(component['mode'] in ['poll','pipes','storage','serial'],'caller interface mode')
    profile=layout(build);kernel=Elf(build/'kernel.elf')
    require(profile==json.loads((build/'caller-retirement-layout.json').read_bytes()),'caller interface layout')
    require(sha((build/'source-user.elf').read_bytes())==component['linked_sha256']
            and sha((build/'static-elf-abi-fixture.elf').read_bytes())==component['image_sha256'],
            'caller interface source/image identity')
    sources=dict(source='source',startup='startup',file='file',shell='shell',command='command',
        poll='poll',pipe='pipe',storage='storage',serial='serial')
    facts={}
    for field,stem in sources.items():
        key='source_sha256' if field=='source' else field+'_source_sha256'
        if component.get(key) is None:continue
        raw=(build/(stem+'-cases.c')).read_bytes()
        require(sha(raw)==component[key],'caller interface C source '+stem)
        facts[stem+'-cases.c']=dict(bytes=len(raw),sha256=sha(raw))
    require(set(facts)=={p.name for p in build.glob('*.c')},'caller interface C closure')
    work=build/'source-user-extents';script=(work/'inspect.gdb').read_text()
    require(script.count(PROGRAM)==1,'caller interface complete observer')
    boundary=BOUNDARY_PROGRAM+boundary_program()
    require(script.count('process_regression_boundary_program='+repr(boundary)+'\n')==1,
            'caller interface complete MM/caller boundary source')
    prefix='process_regression_profile='
    lines=[line[len(prefix):] for line in script.splitlines() if line.startswith(prefix)]
    require(len(lines)==1,'caller interface profile definition')
    actual=ast.literal_eval(lines[0])
    require(actual['process_memory'] is True and actual['kernel_sha256']==profile['kernel_sha256']
            and actual['caller_retirement_profile']==profile,'caller interface profile identity')
    user_raw=(work/'observations.json').read_bytes()
    mm_raw=(work/'process-mm-observations.json').read_bytes()
    user,mm=json.loads(user_raw),json.loads(mm_raw)
    ordinary=interface_readback(user,mm,profile['kernel_sha256'])
    require(ordinary['mode']==component['mode'],'caller interface actual mode')
    samples=sample_readback(mm['caller_retirement'],profile,kernel)
    require(json.loads((build/'guest.command.json').read_bytes())['exit_code']==0,'caller interface guest completion')
    require((work/'gdb.stdout.log').read_bytes().count(b'FRAME_ROOT_GDB_PASS')==1,
            'caller interface native completion marker')
    qemu=json.loads((work/'qemu.command.json').read_bytes())
    require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
            and qemu[qemu.index('-accel')+1].startswith('tcg,'),'caller interface VM geometry')
    require(component['conservative_user_stack_bytes']<=2048,'caller interface user stack')
    return dict(passed=True,mode=component['mode'],kernel_sha256=profile['kernel_sha256'],
        inventory_sha256=profile['inventory_sha256'],user_observation_sha256=sha(user_raw),
        mm_observation_sha256=sha(mm_raw),source_files=facts,interface=ordinary,callers=samples,
        scope='Complete selected ordinary interface/MM observations plus first normal return per sampled helper. No every argument/path, continuous/private-loan lifetime, fatal/exception or public activation qualification.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('build',type=Path)
    args=parser.parse_args();build=args.build.resolve();result=readback(build)
    (build/'interface-callers-readback.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result))
