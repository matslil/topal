#!/usr/bin/env python3
"""Archive native physical refusals, exact inputs and retained same-kernel runs."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_cancel import INPUTS as CANCEL_INPUTS
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_allocation import readback
from readback_cancel import readback as cancel_readback
from readback_exec import readback as exec_readback
from readback_process import digest

INPUTS=sorted(set(CANCEL_INPUTS+[
    'src/topal-compiler/src/systems_process_retained.rs',
    'src/topal-compiler/src/systems_process_identity.rs',
    'src/topal-compiler/src/systems_retained_layout.rs',
    'linux-kernel/kernel/policy/structured-process-identity.t',
    'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_allocation_cases.c',
    'linux-kernel/labs/qemu/x86_64/topal_gate.py',
    'linux-kernel/labs/qemu/x86_64/manifest.json',
    'linux-kernel/tools/process-memory/model.py',
    'linux-kernel/tools/process-memory/test_model.py',
    'linux-kernel/tools/process-memory/trace_check.py',
    'linux-kernel/tools/process-memory/test_physical_catalog.py',
    'linux-kernel/tools/process-memory/readback_allocation.py',
    'linux-kernel/tools/process-memory/archive_allocation.py',
    'linux-kernel/tools/process-memory/replay_allocation.py',
] ))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--regression',action='append',default=[],type=Path)
    parser.add_argument('--attempt',action='append',default=[],type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    build,reference,output=args.build.resolve(),args.reference.resolve(),args.output.resolve()
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in [build,reference,output]), 'Linux directories')
    sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
    from process_mm_process_inspect import PROGRAM
    raw=(build/'process-mm-users/observations.json').read_bytes()
    observed=json.loads(raw);report=readback(observed)
    component=json.loads((build/'source-component.json').read_text())
    require(component['mode']=='process-mm-allocation' and component['allocation_profile']==observed['allocation_profile'], 'application profile binding')
    source=(ROOT/'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_allocation_cases.c').read_bytes()
    require((build/'process-mm-allocation-cases.c').read_bytes()==source
            and component['process_mm_allocation_source_sha256']==digest(source)['sha256'], 'application source binding')
    require((build/'process-mm-users/inspect.gdb').read_text().count(PROGRAM)==1, 'exact debugger source binding')
    qemu=json.loads((build/'process-mm-users/qemu.command.json').read_text())
    require(qemu[qemu.index('-m')+1]=='52M' and qemu[qemu.index('-accel')+1].startswith('tcg,'), 'actual guest RAM/acceleration')
    require(qemu[qemu.index('-smp')+1].startswith('cpus=2,'), 'retained two-CPU guest')
    serial=(build/'process-mm-users/serial.log').read_bytes()
    require([r for r in serial.splitlines() if r.startswith(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=')]
            ==[b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=00'], 'actual parent terminal status')
    artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
               if p.suffix in ['.ll','.o','.elf'] or p.name in ['bzImage','initramfs.cpio']}
    require(component['image_sha256']==artifacts['static-elf-abi-fixture.elf']['sha256'], 'application image binding')
    frame=json.loads((build/'frame-inspection.json').read_text())
    require(all(frame['user_extent_callers'][k]<=4096 for k in ['complete_stack_bytes','complete_physical_bytes']), 'complete native bounds')
    require({'p.prepare','p.commit','p.child_exit','p.cancel_inactive'}<=set(frame['calls']), 'native process caller closure')
    preserved={}
    for actual,old in [(n,n) for n in PRESERVED]+[('retained-owned-provider.o','provider.o'),('retained-owned-gateway.o','gateway.o')]:
        payload=(build/actual).read_bytes();require(payload==(reference/old).read_bytes(),'changed predecessor '+actual)
        preserved[actual]=digest(payload)
    verification=json.loads((build/'llvm-verification.json').read_text())
    require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'},'LLVM module closure')
    require(all(v['passed'] and v['sha256']==artifacts[k+'.ll']['sha256'] for k,v in verification.items()),'LLVM verification binding')
    validation=json.loads((build/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(v['exit_code']==0 for v in validation.values()), 'fresh validation coverage')
    require('Ran 196 tests' in (build/'user-memory-models.stderr.log').read_text()
            and 'Ran 27 tests' in (build/'process-memory-models.stderr.log').read_text(), 'model coverage')
    require('test result: ok. 1 passed' in (build/'default-ABI-fixture.stdout.log').read_text(), 'default ABI fixture coverage')
    guest=json.loads((build/'guest.command.json').read_text());require(guest['exit_code']==0,'native failure')
    resources=json.loads((build/'resource-observation.json').read_text())
    require(resources['swap_max_bytes']==0 and all(v==0 for v in resources['events'].values()), 'resource failure')
    report.update(schema='topal-process-mm-native-allocation-refusal-v1',observation=digest(raw),
        component=component,artifacts=artifacts,preserved=preserved,guest=guest,
        native_bounds=frame['user_extent_callers'],validation=validation,last_resource_sample=resources,
        selected_inputs={name:digest((ROOT/name).read_bytes()) for name in INPUTS})
    payloads={'native-observations.json':raw}
    for name in INPUTS:payloads['inputs/'+name]=(ROOT/name).read_bytes()
    for p in sorted(build.iterdir()):
        if p.is_file() and (p.suffix in ['.ll','.c','.py','.t'] or p.name.endswith(('.command.json','.stdout.log','.stderr.log'))
            or p.name in ['source-component.json','frame-inspection.json','llvm-verification.json','validation.json',
                          'resource-observation.json','readback.json','process-byte-origins.json','extent-call-inventory.json']):
            payloads['build/'+p.name]=p.read_bytes()
    for p in sorted((build/'process-mm-users').iterdir()):
        if p.is_file() and p.name!='observations.json':payloads['native/'+p.name]=p.read_bytes()
    report['retained_regressions']={}
    for path in args.regression:
        regression=path.resolve();require(regression.is_relative_to(ROOT/'linux-kernel'), 'Linux regression directory')
        comp=json.loads((regression/'source-component.json').read_text())
        if comp['mode']=='process-mm-exec':kind='child-exec';reader=exec_readback
        else:
            require(comp['mode']=='process-mm-cancel','regression cohort')
            kind='writer-cancellation' if comp['cancellation'].get('writer',False) else 'reader-cancellation'
            reader=cancel_readback
        require(kind not in report['retained_regressions'], 'duplicate regression cohort')
        require((regression/'kernel.elf').read_bytes()==(build/'kernel.elf').read_bytes(), 'retained regression kernel differs')
        require(comp['image_sha256']==digest((regression/'static-elf-abi-fixture.elf').read_bytes())['sha256'], 'retained application image')
        prior=(regression/'process-mm-users/observations.json').read_bytes()
        summary=reader(json.loads(prior))
        report['retained_regressions'][kind]=dict(observation=digest(prior),summary=summary,
            component=comp,kernel=artifacts['kernel.elf'],scope='Retained native run on the byte-identical kernel, replayed with current readers; no new VM execution')
        prefix='regressions/'+kind+'/'
        payloads[prefix+'observations.json']=prior
        for p in sorted((regression/'process-mm-users').iterdir()):
            if p.is_file() and p.name!='observations.json':payloads[prefix+p.name]=p.read_bytes()
        for p in sorted(regression.iterdir()):
            if p.is_file() and (p.suffix=='.c' or p.name in ['source-component.json','guest.command.json']):
                payloads[prefix+'build/'+p.name]=p.read_bytes()
    require(set(report['retained_regressions'])=={'child-exec','reader-cancellation','writer-cancellation'}, 'regression cohort closure')
    report['attempts']=[]
    for i,path in enumerate(args.attempt):
        attempt=path.resolve();require(attempt.is_relative_to(ROOT/'linux-kernel'),'Linux attempt directory')
        report['attempts'].append(dict(kernel=digest((attempt/'kernel.elf').read_bytes()),
            guest=json.loads((attempt/'guest.command.json').read_text()),
            scope='Earlier narrower run: retained source/LLVM, commands and debugger observations; not the final expanded scope or a complete historical rebuild'))
        for p in sorted(attempt.iterdir()):
            if p.is_file() and (p.suffix in ['.ll','.c'] or p.name.endswith('.command.json')):payloads[f'history/{i}/build/'+p.name]=p.read_bytes()
        for p in sorted((attempt/'process-mm-users').iterdir()):
            if p.is_file():payloads[f'history/{i}/native/'+p.name]=p.read_bytes()
    report['remaining']=['other allocation misses and identity-width exhaustion',
        'native fatal partial effects and every loan/caller/mapping producer',
        'continuous exclusion, every invalidation/pre-release instruction, full application corpus and public activation',
        'original full Linux/distribution/container/VM-host objectives']
    output.mkdir(parents=True,exist_ok=False)
    for name,data in payloads.items():
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(data,mtime=0))
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,entries=len(manifest),inputs=len(INPUTS),controls=len(report['controls']),
                         capacity=report['capacity'],fragmentation=report['fragmentation'])))


if __name__=='__main__':main()
