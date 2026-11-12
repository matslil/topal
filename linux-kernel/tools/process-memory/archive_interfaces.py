#!/usr/bin/env python3
"""Freeze the ordinary-interface MM matrix, exact artifacts and prior records."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_relationship import INPUTS as RELATIONSHIP_INPUTS
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_allocation import readback as allocation_readback
from readback_cancel import readback as cancel_readback
from readback_exec import readback as exec_readback
from readback_geometry import readback as geometry_readback
from readback_interfaces import MODES, readback
from readback_process import digest
from readback_relationship import readback as relationship_readback
from replay_process import checked


def selected_inputs():
    paths=set(RELATIONSHIP_INPUTS)|{
        'linux-kernel/labs/qemu/x86_64/user_extent_source_inspect.py',
        'linux-kernel/labs/qemu/x86_64/process_mm_regression_observer.py',
        'linux-kernel/tools/process-memory/readback_interfaces.py',
        'linux-kernel/tools/process-memory/archive_interfaces.py',
        'linux-kernel/tools/process-memory/replay_interfaces.py',
        'linux-kernel/tools/process-memory/test_interface_observer.py',
        'src/topal-compiler/Cargo.toml', 'Cargo.lock', 'Cargo.toml',
    }
    for base,patterns in [
        ('linux-kernel/tools/user-memory',['*.py']),
        ('linux-kernel/labs/qemu/x86_64',['user_extent*.py','user_extent*.c','process_mm*.py']),
        ('src/topal-compiler/src',['**/*.rs','**/*.ll']),
    ]:
        for pattern in patterns:
            paths.update(p.relative_to(ROOT).as_posix() for p in (ROOT/base).glob(pattern) if p.is_file())
    return sorted(paths)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('builds',nargs=9,type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--regression-build',required=True,type=Path)
    parser.add_argument('--regression-archive',required=True,type=Path)
    parser.add_argument('--resource',required=True,action='append',type=Path)
    parser.add_argument('--attempt',action='append',default=[],type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    reference,regression,prior,output=[p.resolve() for p in [args.reference,args.regression_build,args.regression_archive,args.output]]
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in [reference,regression,prior,output]+[p.resolve() for p in args.resource]), 'Linux archive paths')
    output.mkdir(parents=True,exist_ok=False)
    def save(name,raw):
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(raw,mtime=0))
    kernel=(regression/'kernel.elf').read_bytes();kernel_fact=digest(kernel)
    sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
    from user_extent_source_inspect import PROGRAM
    from process_mm_regression_observer import BOUNDARY_PROGRAM
    report=dict(passed=True,schema='topal-process-mm-native-interface-regressions-v1',cohorts=[])
    for i,path in enumerate(args.builds):
        build=path.resolve();require(build.is_relative_to(ROOT/'linux-kernel'), 'Linux build path')
        native=build/'source-user-extents';prefix=f'cohorts/{i}/'
        raw=(native/'observations.json').read_bytes();mm_raw=(native/'process-mm-observations.json').read_bytes()
        user,mm=json.loads(raw),json.loads(mm_raw)
        result=readback(user,mm,kernel_fact['sha256'])
        component=json.loads((build/'source-component.json').read_text())
        require(component['mode']==result['mode']==MODES[i], 'nine-mode order')
        require(json.loads((build/'process-mm-regression.json').read_text())['process_memory'] is True, 'selection metadata')
        require((build/'kernel.elf').read_bytes()==kernel, 'interface-dependent linked kernel')
        require(component['image_sha256']==digest((build/'static-elf-abi-fixture.elf').read_bytes())['sha256']
                and component['linked_sha256']==digest((build/'source-user.elf').read_bytes())['sha256']
                and component['conservative_user_stack_bytes']<=2048, 'application artifact/stack binding')
        source_names={'source':'source','startup':'startup','file':'file','shell':'shell','command':'command',
                      'exec':'exec','signal':'signal','poll':'poll','pipe':'pipe','storage':'storage','metadata':'metadata',
                      'serial':'serial','stat':'stat'}
        for field,stem in source_names.items():
            expected=component.get('source_sha256' if field=='source' else field+'_source_sha256')
            if expected is None:continue
            first=(ROOT/f'linux-kernel/labs/qemu/x86_64/user_extent_{stem}_cases.c').read_bytes()
            require(digest(first)['sha256']==expected and (build/f'{stem}-cases.c').read_bytes()==first, 'ordinary source binding '+field)
        gdb_source=(native/'inspect.gdb').read_text()
        require(gdb_source.count(PROGRAM)==1 and repr(BOUNDARY_PROGRAM) in gdb_source, 'exact ordinary and MM observer sources')
        qemu=json.loads((native/'qemu.command.json').read_text())
        require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-accel')+1].startswith('tcg,')
                and qemu[qemu.index('-smp')+1].startswith('cpus=2,'), 'retained guest configuration')
        guest=json.loads((build/'guest.command.json').read_text());require(guest['exit_code']==0, 'native matrix failure')
        require((native/'gdb.stdout.log').read_bytes().count(b'FRAME_ROOT_GDB_PASS')==1, 'actual debugger completion')
        terminal=next(r for r in mm['events'] if r['operation']==13)
        status=terminal['before']['context']['user_arg']&255
        require([r for r in (native/'serial.log').read_bytes().splitlines() if r.startswith(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=')]
                ==[f'TOPAL_KERNEL_PROCESS_EXIT_STATUS={status:02x}'.encode()], 'actual captured terminal result')
        frame=json.loads((build/'frame-inspection.json').read_text())
        require(all(frame['user_extent_callers'][k]<=4096 for k in ['complete_stack_bytes','complete_physical_bytes'])
                and {'p.prepare','p.commit','p.switch','p.child_exit','p.cancel_inactive'}<=set(frame['calls']), 'native caller bounds')
        artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
                   if p.suffix in ['.ll','.o','.elf'] or p.name in ['bzImage','initramfs.cpio']}
        preserved={}
        for actual,old in [(n,n) for n in PRESERVED]+[('retained-owned-provider.o','provider.o'),('retained-owned-gateway.o','gateway.o')]:
            data=(build/actual).read_bytes();require(data==(reference/old).read_bytes(), 'changed predecessor '+actual)
            preserved[actual]=digest(data)
        verification=json.loads((build/'llvm-verification.json').read_text())
        require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'}
                and all(v['passed'] and v['sha256']==artifacts[k+'.ll']['sha256'] for k,v in verification.items()), 'LLVM module closure')
        result.update(user_observation=digest(raw),mm_observation=digest(mm_raw),component=component,
                      artifacts=artifacts,preserved=preserved,native_bounds=frame['user_extent_callers'],guest=guest)
        report['cohorts'].append(result)
        save(prefix+'observations.json',raw);save(prefix+'mm-observations.json',mm_raw)
        for p in sorted(build.iterdir()):
            if p.is_file() and p.suffix in ['.ll','.o','.elf','.c','.py','.t','.json','.log','.su','.ld']:
                save(prefix+'build/'+p.name,p.read_bytes())
            elif p.is_file() and p.name in ['bzImage','initramfs.cpio']:save(prefix+'build/'+p.name,p.read_bytes())
        for p in sorted(native.iterdir()):
            if p.is_file() and p.name not in ['observations.json','process-mm-observations.json']:save(prefix+'native/'+p.name,p.read_bytes())
        del user,mm,raw,mm_raw
    totals=['source_cases','source_case_syscalls','observed_interface_syscalls','boundaries','retired_extents','erased_released_bytes']
    report['native_summary']={name:sum(row[name] for row in report['cohorts']) for name in totals}
    report['native_summary'].update(cohorts=len(report['cohorts']),controls=sum(len(row['controls']) for row in report['cohorts']))
    validation_build=args.builds[0].resolve();validation=json.loads((validation_build/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(v['exit_code']==0 for v in validation.values()), 'fresh validation closure')
    require('Ran 203 tests' in (validation_build/'user-memory-models.stderr.log').read_text()
            and 'Ran 31 tests' in (validation_build/'process-memory-models.stderr.log').read_text()
            and 'test result: ok. 1 passed' in (validation_build/'default-ABI-fixture.stdout.log').read_text(), 'model/default coverage')
    report['validation']=validation
    report['resource_samples']=[]
    for i,path in enumerate(args.resource):
        resources=json.loads(path.read_text())
        require(resources['swap_max_bytes']==0 and 0<resources['observed_peak_bytes']<=resources['memory_max_bytes']
                and all(v==0 for v in resources['events'].values()), 'resource failure')
        report['resource_samples'].append(resources)
        save(f'resources/{i}.json',path.read_bytes())
    manifest_raw=(prior/'manifest.json').read_bytes();manifest=json.loads(manifest_raw)
    require({p.relative_to(prior).as_posix() for p in prior.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}, 'prior archive closure')
    for name,fact in manifest.items():
        path=(prior/name).resolve();require(path.is_relative_to(prior), 'prior archive path');checked(path.read_bytes(),fact)
    old_raw=(prior/'report.json').read_bytes();old=json.loads(old_raw)
    require(old['passed'] and old['schema']=='topal-process-mm-native-relationship-termination-v1', 'prior archive schema')
    report['prior_archive']=dict(path=prior.relative_to(ROOT).as_posix(),manifest=digest(manifest_raw),report=digest(old_raw))
    save('prior/manifest.json',manifest_raw);save('prior/report.json',old_raw)
    retained={}
    for i,row in enumerate(old['cohorts']):
        require(row['artifacts']['kernel.elf']==kernel_fact, 'prior relationship kernel')
        c=row['case'];kind=f'relationship-{int(c["zombie"])}-{int(c["early"])}-{c["syscall"]}'
        retained[kind]=(f'cohorts/{i}/observations.json.gz',row['observation'],relationship_readback)
    for kind,row in old['retained_regressions'].items():
        require(row['kernel']==kernel_fact, 'prior retained kernel')
        reader=geometry_readback if kind.startswith('maximum-') else allocation_readback if kind=='allocation-refusal' else exec_readback if kind=='child-exec' else cancel_readback
        retained[kind]=('regressions/'+kind+'/observations.json.gz',row['observation'],reader)
    require(len(retained)==20, 'twenty retained cohorts')
    report['retained_regressions']={}
    for kind,(name,fact,reader) in retained.items():
        raw=checked(gzip.decompress((prior/name).read_bytes()),fact);summary=reader(json.loads(raw))
        report['retained_regressions'][kind]=dict(observation=digest(raw),summary=summary,kernel=kernel_fact,
            scope='Identified previous native execution on the byte-identical kernel, current record replay; no new VM or full historical rebuild')
        save('regressions/'+kind+'/observations.json',raw)
    report['attempts']=[]
    for i,path in enumerate(args.attempt):
        attempt=path.resolve();require(attempt.is_relative_to(ROOT/'linux-kernel'), 'Linux attempt path')
        row=dict(kernel=digest((attempt/'kernel.elf').read_bytes()),
            scope='Earlier failed matrix attempt; selected actual inputs, commands and partial observations, not complete historical reconstruction')
        report['attempts'].append(row)
        for p in sorted(attempt.iterdir()):
            if p.is_file() and p.suffix in ['.ll','.c','.py','.rs','.json','.log']:save(f'history/{i}/build/'+p.name,p.read_bytes())
        for directory in ['history-inputs','source-user-extents','source-entry-diagnostic']:
            if not (attempt/directory).exists():continue
            for p in sorted((attempt/directory).iterdir()):
                if p.is_file():save(f'history/{i}/{directory}/'+p.name,p.read_bytes())
    inputs=selected_inputs();report['selected_inputs']={name:digest((ROOT/name).read_bytes()) for name in inputs}
    for name in inputs:save('inputs/'+name,(ROOT/name).read_bytes())
    report['remaining']=['remaining native caller/loan/mapping producers, continuous exclusion and complete invalidation/release order',
        'identity-width/resource/fatal partial-effect qualification', 'unchanged pinned-application corpus and cold public activation',
        'original complete Linux UAPI, distribution, Docker/Podman and VM-host objectives']
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,entries=len(manifest),inputs=len(inputs),controls=sum(len(r['controls']) for r in report['cohorts']))))


if __name__=='__main__':main()
