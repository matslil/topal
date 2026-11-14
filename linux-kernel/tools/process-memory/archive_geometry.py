#!/usr/bin/env python3
"""Archive both successful maximum-copy syscalls and same-kernel regressions."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_allocation import INPUTS as ALLOCATION_INPUTS
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_allocation import readback as allocation_readback
from readback_cancel import readback as cancel_readback
from readback_exec import readback as exec_readback
from readback_geometry import readback
from readback_process import digest

INPUTS=sorted(set(ALLOCATION_INPUTS+[
    'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_geometry_cases.c',
    'linux-kernel/tools/process-memory/readback_geometry.py',
    'linux-kernel/tools/process-memory/archive_geometry.py',
    'linux-kernel/tools/process-memory/replay_geometry.py',
]))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('builds',nargs=2,type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--regression',action='append',default=[],type=Path)
    parser.add_argument('--resource',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    output,reference=args.output.resolve(),args.reference.resolve()
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in [output,reference,args.resource.resolve()]), 'Linux archive directories')
    sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
    from process_mm_process_inspect import PROGRAM
    report=dict(passed=True,schema='topal-process-mm-native-maximum-geometry-v1',cohorts=[])
    payloads={};kernel=None
    for i,path in enumerate(args.builds):
        build=path.resolve();require(build.is_relative_to(ROOT/'linux-kernel'), 'Linux build directory')
        prefix=f'cohorts/{i}/'
        raw=(build/'process-mm-users/observations.json').read_bytes()
        observed=json.loads(raw);result=readback(observed)
        component=json.loads((build/'source-component.json').read_text())
        syscall=result['syscall']
        require(component['geometry_profile']==observed['geometry_profile']
                and component['mode']==('process-mm-geometry-fork' if syscall==57 else 'process-mm-geometry-clone'), 'application profile')
        source=(ROOT/'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_geometry_cases.c').read_bytes()
        require((build/'process-mm-geometry-cases.c').read_bytes()==source
                and component['process_mm_geometry_source_sha256']==digest(source)['sha256'], 'application source')
        command=json.loads((build/'process-mm-geometry-cases-compile.command.json').read_text())
        require(f'-DTOPAL_GEOMETRY_SYSCALL={syscall}' in command['argv'] and command['exit_code']==0, 'source syscall choice')
        require((build/'process-mm-users/inspect.gdb').read_text().count(PROGRAM)==1, 'exact observer source')
        qemu=json.loads((build/'process-mm-users/qemu.command.json').read_text())
        require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-accel')+1].startswith('tcg,')
                and qemu[qemu.index('-smp')+1].startswith('cpus=2,'), 'retained guest configuration')
        serial=(build/'process-mm-users/serial.log').read_bytes()
        require([r for r in serial.splitlines() if r.startswith(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=')]
                ==[b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=00'], 'actual source terminal status')
        artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
                   if p.suffix in ['.ll','.o','.elf'] or p.name in ['bzImage','initramfs.cpio']}
        if kernel is None:kernel=(build/'kernel.elf').read_bytes()
        else:require((build/'kernel.elf').read_bytes()==kernel, 'syscall-dependent linked kernel')
        require(component['image_sha256']==artifacts['static-elf-abi-fixture.elf']['sha256'], 'application image binding')
        require(component['conservative_user_stack_bytes']<=2048, 'complete userspace stack bound')
        frame=json.loads((build/'frame-inspection.json').read_text())
        require(all(frame['user_extent_callers'][k]<=4096 for k in ['complete_stack_bytes','complete_physical_bytes']), 'native bounds')
        require({'p.prepare','p.commit','p.switch','p.child_exit','p.cancel_inactive'}<=set(frame['calls']), 'process caller closure')
        preserved={}
        for actual,old in [(n,n) for n in PRESERVED]+[('retained-owned-provider.o','provider.o'),('retained-owned-gateway.o','gateway.o')]:
            data=(build/actual).read_bytes();require(data==(reference/old).read_bytes(), 'changed predecessor '+actual)
            preserved[actual]=digest(data)
        verification=json.loads((build/'llvm-verification.json').read_text())
        require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'}
                and all(v['passed'] and v['sha256']==artifacts[k+'.ll']['sha256'] for k,v in verification.items()), 'LLVM closure')
        guest=json.loads((build/'guest.command.json').read_text());require(guest['exit_code']==0, 'native failure')
        result.update(observation=digest(raw),component=component,artifacts=artifacts,preserved=preserved,
                      native_bounds=frame['user_extent_callers'],guest=guest)
        report['cohorts'].append(result);payloads[prefix+'observations.json']=raw
        for p in sorted(build.iterdir()):
            if p.is_file() and (p.suffix in ['.ll','.c','.py','.t'] or p.name.endswith(('.command.json','.stdout.log','.stderr.log'))
                or p.name in ['source-component.json','frame-inspection.json','llvm-verification.json','validation.json',
                              'readback.json','process-byte-origins.json','extent-call-inventory.json']):
                payloads[prefix+'build/'+p.name]=p.read_bytes()
        for p in sorted((build/'process-mm-users').iterdir()):
            if p.is_file() and p.name!='observations.json':payloads[prefix+'native/'+p.name]=p.read_bytes()
    require([r['syscall'] for r in report['cohorts']]==[57,56], 'fork and clone closure')
    validation_build=args.builds[0].resolve()
    validation=json.loads((validation_build/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(v['exit_code']==0 for v in validation.values()), 'fresh validation coverage')
    require('Ran 196 tests' in (validation_build/'user-memory-models.stderr.log').read_text()
            and 'Ran 27 tests' in (validation_build/'process-memory-models.stderr.log').read_text(), 'model coverage')
    require('test result: ok. 1 passed' in (validation_build/'default-ABI-fixture.stdout.log').read_text(), 'default fixture coverage')
    report['validation']=validation
    resources=json.loads(args.resource.read_text())
    require(resources['swap_max_bytes']==0 and all(v==0 for v in resources['events'].values()), 'resource failure')
    report['last_resource_sample']=resources;payloads['resource-observation.json']=args.resource.read_bytes()
    report['retained_regressions']={}
    for path in args.regression:
        regression=path.resolve();require(regression.is_relative_to(ROOT/'linux-kernel'), 'Linux regression directory')
        comp=json.loads((regression/'source-component.json').read_text())
        if comp['mode']=='process-mm-exec':kind='child-exec';reader=exec_readback
        elif comp['mode']=='process-mm-allocation':kind='allocation-refusal';reader=allocation_readback
        else:
            require(comp['mode']=='process-mm-cancel', 'regression mode')
            config=comp['cancellation']
            kind=('writer-cancellation' if config.get('writer',False) else 'reader-cancellation')
            kind+=('-early' if config['early'] else '-late')+('-exit-group' if config['syscall']==231 else '-exit')
            reader=cancel_readback
        require(kind not in report['retained_regressions'], 'duplicate regression')
        require((regression/'kernel.elf').read_bytes()==kernel, 'retained regression kernel differs')
        require(comp['image_sha256']==digest((regression/'static-elf-abi-fixture.elf').read_bytes())['sha256'], 'retained application image')
        prior=(regression/'process-mm-users/observations.json').read_bytes();summary=reader(json.loads(prior))
        report['retained_regressions'][kind]=dict(observation=digest(prior),summary=summary,component=comp,kernel=digest(kernel),
            scope='Retained native run on the byte-identical kernel, replayed with current readers; no new VM execution')
        prefix='regressions/'+kind+'/'
        payloads[prefix+'observations.json']=prior
        for p in sorted((regression/'process-mm-users').iterdir()):
            if p.is_file() and p.name!='observations.json':payloads[prefix+p.name]=p.read_bytes()
        for p in sorted(regression.iterdir()):
            if p.is_file() and (p.suffix=='.c' or p.name in ['source-component.json','guest.command.json']):
                payloads[prefix+'build/'+p.name]=p.read_bytes()
    expected={'allocation-refusal','child-exec'}|{
        f'{wait}-cancellation-{stage}-{exit_kind}' for wait in ['reader','writer']
        for stage in ['early','late'] for exit_kind in ['exit','exit-group']}
    require(set(report['retained_regressions'])==expected, 'retained cohort closure')
    report['selected_inputs']={name:digest((ROOT/name).read_bytes()) for name in INPUTS}
    for name in INPUTS:payloads['inputs/'+name]=(ROOT/name).read_bytes()
    report['remaining']=['all caller/loan/mapping producers and general process/MM geometry',
        'other resource failures, identity-width exhaustion and native fatal partial effects',
        'continuous exclusion, complete invalidation/pre-release order, full application corpus and public activation',
        'original full Linux/distribution/container/VM-host objectives']
    output.mkdir(parents=True,exist_ok=False)
    for name,data in payloads.items():
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(data,mtime=0))
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,entries=len(manifest),inputs=len(INPUTS),
                         controls=[len(r['controls']) for r in report['cohorts']])))


if __name__=='__main__':main()
