#!/usr/bin/env python3
"""Archive runnable/zombie termination and recheck identified retained records."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_geometry import INPUTS as GEOMETRY_INPUTS
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_allocation import readback as allocation_readback
from readback_cancel import readback as cancel_readback
from readback_exec import readback as exec_readback
from readback_geometry import readback as geometry_readback
from readback_process import digest
from readback_relationship import readback
from replay_process import checked

INPUTS=sorted(set(GEOMETRY_INPUTS+[
    'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_relationship_cases.c',
    'linux-kernel/tools/process-memory/readback_relationship.py',
    'linux-kernel/tools/process-memory/archive_relationship.py',
    'linux-kernel/tools/process-memory/replay_relationship.py',
]))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('builds',nargs=8,type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--regression-build',required=True,type=Path)
    parser.add_argument('--regression-archive',required=True,type=Path)
    parser.add_argument('--resource',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    reference,regression,prior,output=[p.resolve() for p in [args.reference,args.regression_build,args.regression_archive,args.output]]
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in [reference,regression,prior,output,args.resource.resolve()]), 'Linux archive paths')
    kernel=(regression/'kernel.elf').read_bytes();kernel_fact=digest(kernel)
    sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
    from process_mm_process_inspect import PROGRAM
    payloads={};report=dict(passed=True,schema='topal-process-mm-native-relationship-termination-v1',cohorts=[])
    source=(ROOT/'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_relationship_cases.c').read_bytes()
    for i,path in enumerate(args.builds):
        build=path.resolve();require(build.is_relative_to(ROOT/'linux-kernel'), 'Linux build path')
        prefix=f'cohorts/{i}/'
        raw=(build/'process-mm-users/observations.json').read_bytes();observed=json.loads(raw);result=readback(observed)
        component=json.loads((build/'source-component.json').read_text());config=result['case']
        require(component['mode']=='process-mm-relationship' and component['relationship_profile']==config, 'source profile binding')
        require((build/'process-mm-relationship-cases.c').read_bytes()==source
                and component['process_mm_relationship_source_sha256']==digest(source)['sha256'], 'source bytes binding')
        command=json.loads((build/'process-mm-relationship-cases-compile.command.json').read_text())
        require(command['exit_code']==0 and all(f'-DTOPAL_RELATIONSHIP_{name}={int(flag)}' in command['argv'] for name,flag in
                [('EARLY',config['early']),('GROUP',config['syscall']==231),('ZOMBIE',config['zombie'])]), 'compile configuration')
        require((build/'process-mm-users/inspect.gdb').read_text().count(PROGRAM)==1, 'exact observer source')
        qemu=json.loads((build/'process-mm-users/qemu.command.json').read_text())
        require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-accel')+1].startswith('tcg,')
                and qemu[qemu.index('-smp')+1].startswith('cpus=2,'), 'retained QEMU configuration')
        serial=(build/'process-mm-users/serial.log').read_bytes()
        require([r for r in serial.splitlines() if r.startswith(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=')]
                ==[f'TOPAL_KERNEL_PROCESS_EXIT_STATUS={config["status"]&255:02x}'.encode()], 'actual terminal status')
        artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
                   if p.suffix in ['.ll','.o','.elf'] or p.name in ['bzImage','initramfs.cpio']}
        require((build/'kernel.elf').read_bytes()==kernel, 'relationship-dependent linked kernel')
        require(component['image_sha256']==artifacts['static-elf-abi-fixture.elf']['sha256']
                and component['conservative_user_stack_bytes']<=2048, 'application binding/stack')
        frame=json.loads((build/'frame-inspection.json').read_text())
        require(all(frame['user_extent_callers'][k]<=4096 for k in ['complete_stack_bytes','complete_physical_bytes'])
                and {'p.prepare','p.commit','p.switch','p.child_exit','p.cancel_inactive'}<=set(frame['calls']), 'native caller bounds')
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
                or p.name in ['source-component.json','frame-inspection.json','llvm-verification.json','validation.json','readback.json',
                              'process-byte-origins.json','extent-call-inventory.json']):payloads[prefix+'build/'+p.name]=p.read_bytes()
        for p in sorted((build/'process-mm-users').iterdir()):
            if p.is_file() and p.name!='observations.json':payloads[prefix+'native/'+p.name]=p.read_bytes()
    require([(r['case']['zombie'],r['case']['early'],r['case']['syscall']) for r in report['cohorts']]
            ==[(z,e,s) for z in [False,True] for e in [False,True] for s in [60,231]], 'eight-case closure')
    validation_build=args.builds[0].resolve();validation=json.loads((validation_build/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(v['exit_code']==0 for v in validation.values()), 'fresh validation coverage')
    require('Ran 196 tests' in (validation_build/'user-memory-models.stderr.log').read_text()
            and 'Ran 27 tests' in (validation_build/'process-memory-models.stderr.log').read_text()
            and 'test result: ok. 1 passed' in (validation_build/'default-ABI-fixture.stdout.log').read_text(), 'model/default coverage')
    report['validation']=validation
    resources=json.loads(args.resource.read_text())
    require(resources['swap_max_bytes']==0 and all(v==0 for v in resources['events'].values()), 'resource failure')
    report['last_resource_sample']=resources;payloads['resource-observation.json']=args.resource.read_bytes()
    manifest_raw=(prior/'manifest.json').read_bytes();manifest=json.loads(manifest_raw)
    require({p.relative_to(prior).as_posix() for p in prior.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}, 'prior archive closure')
    for name,fact in manifest.items():
        path=(prior/name).resolve();require(path.is_relative_to(prior), 'prior archive path');checked(path.read_bytes(),fact)
    prior_report_raw=(prior/'report.json').read_bytes();old=json.loads(prior_report_raw)
    require(old['passed'] and old['schema']=='topal-process-mm-native-maximum-geometry-v1', 'prior archive schema')
    report['prior_archive']=dict(manifest=digest(manifest_raw),report=digest(prior_report_raw),
        path=prior.relative_to(ROOT).as_posix(),scope='Exact prior archive verified at collection; retained observations replay below, not a new VM or full historical rebuild')
    payloads['prior/manifest.json']=manifest_raw;payloads['prior/report.json']=prior_report_raw
    retained={}
    for i,row in enumerate(old['cohorts']):
        require(row['artifacts']['kernel.elf']==kernel_fact, 'prior maximum-copy kernel')
        retained['maximum-'+str(row['syscall'])]=(f'cohorts/{i}/observations.json.gz',row['observation'],geometry_readback)
    for kind,row in old['retained_regressions'].items():
        require(row['kernel']==kernel_fact, 'prior retained kernel')
        reader=allocation_readback if kind=='allocation-refusal' else exec_readback if kind=='child-exec' else cancel_readback
        retained[kind]=('regressions/'+kind+'/observations.json.gz',row['observation'],reader)
    require(len(retained)==12, 'retained cohort count')
    report['retained_regressions']={}
    for kind,(name,fact,reader) in retained.items():
        raw=checked(gzip.decompress((prior/name).read_bytes()),fact);summary=reader(json.loads(raw))
        report['retained_regressions'][kind]=dict(observation=digest(raw),summary=summary,kernel=kernel_fact,
            scope='Identified earlier native run on the byte-identical kernel; current saved-record readers, no new VM execution')
        payloads['regressions/'+kind+'/observations.json']=raw
    report['selected_inputs']={name:digest((ROOT/name).read_bytes()) for name in INPUTS}
    for name in INPUTS:payloads['inputs/'+name]=(ROOT/name).read_bytes()
    report['remaining']=['all caller/loan/mapping producers and broader process/exec/MM shapes',
        'other resource/identity-width failures, native fatal partial effects and continuous exclusion',
        'complete invalidation/pre-release order, full application corpus and public activation',
        'original full Linux/distribution/container/VM-host objectives']
    output.mkdir(parents=True,exist_ok=False)
    for name,data in payloads.items():
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(data,mtime=0))
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,entries=len(manifest),inputs=len(INPUTS),controls=sum(len(r['controls']) for r in report['cohorts']))))


if __name__=='__main__':main()
