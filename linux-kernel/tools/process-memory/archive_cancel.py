#!/usr/bin/env python3
"""Archive four completed live-child cancellations and their exact inputs."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_process import PRESERVED, ROOT, SELECTED
from process_native import require
from readback_cancel import readback
from readback_exec import readback as exec_readback
from readback_process import digest

INPUTS=SELECTED+[
    'src/topal-compiler/src/systems_process_runtime.rs',
    'src/topal-compiler/src/systems_process_terminal.rs',
    'src/topal-compiler/src/systems_process_entry_start.rs',
    'src/topal-compiler/src/systems_process_entry_completion.rs',
    'src/topal-compiler/src/systems_process_pipes.rs',
    'src/topal-compiler/src/systems_pipe_wait.rs',
    'src/topal-compiler/src/systems_file_descriptions.rs',
    'src/topal-compiler/src/systems_volatile_files.rs',
    'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_cancel_cases.c',
    'linux-kernel/tools/process-memory/readback_cancel.py',
    'linux-kernel/tools/process-memory/archive_cancel.py',
    'linux-kernel/tools/process-memory/replay_cancel.py',
]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('builds',nargs=4,type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--regression',required=True,type=Path)
    parser.add_argument('--resource',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--attempt',action='append',default=[],type=Path)
    args=parser.parse_args()
    output=args.output.resolve();reference=args.reference.resolve();regression=args.regression.resolve()
    require(output.is_relative_to(ROOT/'linux-kernel'), 'Linux output directory')
    sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
    from process_mm_process_inspect import PROGRAM
    payloads={};report=dict(passed=True,schema='topal-process-mm-native-cancellation-v1',cohorts=[])
    components={}
    for i,path in enumerate(args.builds):
        build=path.resolve();require(build.is_relative_to(ROOT/'linux-kernel'),'Linux build directory')
        prefix=f'cohorts/{i}/'
        component=json.loads((build/'source-component.json').read_text())
        require(component['mode']=='process-mm-cancel', 'wrong cohort')
        raw=(build/'process-mm-users/observations.json').read_bytes()
        observed=json.loads(raw);result=readback(observed)
        require(component['cancellation']==result['case'],'application configuration binding')
        source=(ROOT/'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_cancel_cases.c').read_bytes()
        require((build/'process-mm-cancel-cases.c').read_bytes()==source and component['process_mm_cancel_source_sha256']==digest(source)['sha256'], 'application source binding')
        require((build/'process-mm-users/inspect.gdb').read_text().count(PROGRAM)==1,'debugger source differs')
        command=json.loads((build/'process-mm-cancel-cases-compile.command.json').read_text())
        config=result['case'];argv=command['argv']
        require(all(s in argv for s in [f'-DTOPAL_CANCEL_EARLY={int(config["early"])}',
                f'-DTOPAL_CANCEL_EXIT_SYSCALL={config["syscall"]}',f'-DTOPAL_CANCEL_STATUS={config["status"]}']), 'compile configuration')
        serial=(build/'process-mm-users/serial.log').read_bytes()
        markers=[line for line in serial.splitlines() if line.startswith(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=')]
        require(markers==[f'TOPAL_KERNEL_PROCESS_EXIT_STATUS={config["status"]&255:02x}'.encode()], 'actual terminal status witness')
        frame=json.loads((build/'frame-inspection.json').read_text())
        require(all(frame['user_extent_callers'][k]<=4096 for k in ['complete_stack_bytes','complete_physical_bytes']), 'native bound')
        preserved={}
        for actual,old in [(n,n) for n in PRESERVED]+[('retained-owned-provider.o','provider.o'),('retained-owned-gateway.o','gateway.o')]:
            data=(build/actual).read_bytes()
            require(data==(reference/old).read_bytes(),'changed predecessor '+actual)
            preserved[actual]=digest(data)
        for name in ['provider.o','gateway.o','target.o','dispatch.o']:
            data=(build/name).read_bytes()
            if name in components:require(data==components[name], 'case-dependent kernel '+name)
            else:components[name]=data
            require(data==(regression/name).read_bytes(),'regression kernel component '+name)
        artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
                   if p.suffix in ['.o','.ll','.elf'] or p.name in ['bzImage','initramfs.cpio']}
        require((build/'kernel.elf').read_bytes()==(regression/'kernel.elf').read_bytes(), 'case-dependent linked kernel')
        require(component['image_sha256']==artifacts['static-elf-abi-fixture.elf']['sha256'],'source image binding')
        verification=json.loads((build/'llvm-verification.json').read_text())
        require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'},'LLVM module closure')
        require(all(v['passed'] and v['sha256']==artifacts[k+'.ll']['sha256'] for k,v in verification.items()),'LLVM verification')
        guest=json.loads((build/'guest.command.json').read_text())
        require(guest['exit_code']==0,'native test failure')
        result.update(observation=digest(raw),component=component,artifacts=artifacts,preserved=preserved,
                      native_bounds=frame['user_extent_callers'],guest=guest)
        report['cohorts'].append(result)
        payloads[prefix+'observations.json']=raw
        for p in sorted(build.iterdir()):
            if p.is_file() and (p.suffix in ['.ll','.c'] or p.name.endswith(('.command.json','.stdout.log','.stderr.log'))
                               or p.name in ['source-component.json','frame-inspection.json','llvm-verification.json']):
                payloads[prefix+'build/'+p.name]=p.read_bytes()
        for p in sorted((build/'process-mm-users').iterdir()):
            if p.is_file() and p.name!='observations.json':payloads[prefix+'native/'+p.name]=p.read_bytes()
    require([(r['case']['early'],r['case']['syscall'],r['case']['status']) for r in report['cohorts']]
            ==[(False,60,0x13a),(False,231,0x200),(True,60,0x2c5),(True,231,0x301)],'four-case closure')
    regression_raw=(regression/'process-mm-users/observations.json').read_bytes()
    regression_result=exec_readback(json.loads(regression_raw))
    report['regression']=dict(observation=digest(regression_raw),summary=regression_result)
    payloads['regression/observations.json']=regression_raw
    for p in sorted((regression/'process-mm-users').iterdir()):
        if p.is_file() and p.name!='observations.json':payloads['regression/'+p.name]=p.read_bytes()
    validation=json.loads((regression/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(v['exit_code']==0 for v in validation.values()),'regression validation')
    require('Ran 196 tests' in (regression/'user-memory-models.stderr.log').read_text()
            and 'Ran 17 tests' in (regression/'process-memory-models.stderr.log').read_text(),'model coverage')
    require('test result: ok. 1 passed' in (regression/'default-ABI-fixture.stdout.log').read_text(),'default fixture coverage')
    report['validation']=validation
    for p in sorted(regression.iterdir()):
        if p.is_file() and (p.name.endswith(('.command.json','.stdout.log','.stderr.log')) or p.name in ['source-component.json','validation.json']):
            payloads['regression/'+p.name]=p.read_bytes()
    resource=json.loads(args.resource.read_text())
    require(resource['swap_max_bytes']==0 and all(v==0 for v in resource['events'].values()),'resource failure')
    report['last_resource_sample']=resource
    report['selected_inputs']={name:digest((ROOT/name).read_bytes()) for name in INPUTS}
    for name in INPUTS:payloads['inputs/'+name]=(ROOT/name).read_bytes()
    report['history']=[]
    for i,path in enumerate(args.attempt):
        build=path.resolve();require(build.is_relative_to(ROOT/'linux-kernel'),'Linux history directory')
        report['history'].append(dict(kernel=digest((build/'kernel.elf').read_bytes()),
            guest=json.loads((build/'guest.command.json').read_text()),
            scope='Earlier failure: retained source/LLVM, commands and debugger records; not current qualification or a complete historical rebuild'))
        for p in sorted(build.iterdir()):
            if p.is_file() and (p.suffix in ['.ll','.c'] or p.name.endswith('.command.json')):payloads[f'history/{i}/build/'+p.name]=p.read_bytes()
        for p in sorted((build/'process-mm-users').iterdir()):
            if p.is_file():payloads[f'history/{i}/native/'+p.name]=p.read_bytes()
    report['remaining']=['every native invalidation/pre-release and continuous loan/exclusion obligation',
        'identity exhaustion, allocation misses, partial-effect fatal cleanup, all mapping writers',
        'general orphan/init semantics, full 51-workload regressions and public reconstruction',
        'original full Linux/distribution/container/VM-host objectives']
    output.mkdir(parents=True,exist_ok=False)
    for name,raw in payloads.items():
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(raw,mtime=0))
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,cases=len(report['cohorts']),entries=len(manifest))))


if __name__=='__main__':main()
