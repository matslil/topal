#!/usr/bin/env python3
"""Freeze the complete unchanged application corpus and identified regressions."""
import argparse
import gzip
import json
from pathlib import Path
import sys

from archive_interfaces import selected_inputs as interface_inputs
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_allocation import readback as allocation_readback
from readback_applications import facts, readback
from readback_cancel import readback as cancel_readback
from readback_exec import readback as exec_readback
from readback_geometry import readback as geometry_readback
from readback_interfaces import MODES, readback as interface_readback
from readback_process import digest
from readback_relationship import readback as relationship_readback
from replay_process import checked

sys.path.insert(0,str(ROOT/'linux-kernel/labs/qemu/x86_64'))
import process_mm_application_inspect as application
import topal_gate
from user_extent_application_inspect import members


def selected_inputs():
    paths=set(interface_inputs())|{
        'linux-kernel/labs/qemu/x86_64/frame_application_inspect.py',
        'linux-kernel/labs/qemu/x86_64/frame_memory_inspect.py',
        'linux-kernel/labs/qemu/x86_64/frame_root_inspect.py',
        'linux-kernel/labs/qemu/x86_64/topal_gate.py',
        'linux-kernel/labs/qemu/x86_64/process_mm_application_inspect.py',
        'linux-kernel/tools/process-memory/readback_applications.py',
        'linux-kernel/tools/process-memory/archive_applications.py',
        'linux-kernel/tools/process-memory/replay_applications.py',
        'linux-kernel/tools/process-memory/test_application_serial.py',
    }
    fixture='linux-kernel/tools/tail-traversal/results/stage-4-frame-production/components/'
    paths.update(fixture+'production-busybox/ownership/application-'+name+'/serial.log.gz'
                 for name in topal_gate.PROCESS_WORKLOADS)
    paths.add(fixture+'production-abi/clean-abi/serial.log.gz')
    return sorted(paths)


def process_reader(name):
    if name.startswith('relationship-'):return relationship_readback
    if name.startswith('maximum-'):return geometry_readback
    if name=='allocation-refusal':return allocation_readback
    if name=='child-exec':return exec_readback
    return cancel_readback


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('--reference',required=True,type=Path)
    parser.add_argument('--prior',required=True,type=Path)
    parser.add_argument('--run',required=True,type=Path)
    parser.add_argument('--driver',required=True,type=Path)
    parser.add_argument('--history',action='append',default=[],type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    build,reference,prior,run,driver,output=[p.resolve() for p in [args.build,args.reference,args.prior,args.run,args.driver,args.output]]
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in [build,reference,prior,run,driver,output]+[p.resolve() for p in args.history]), 'Linux archive paths')
    output.mkdir(parents=True,exist_ok=False)
    def save(name,raw):
        target=output/(name+'.gz');target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(raw,mtime=0))
    binding=facts(build);kernel_fact=digest((build/'kernel.elf').read_bytes())
    image=(build/'static-elf-abi-fixture.elf').read_bytes()
    require(digest(image)['sha256']==topal_gate.PINNED_STATIC_BUSYBOX_SHA256, 'unchanged pinned application')
    boot=members((build/'initramfs.cpio').read_bytes())
    require(set(boot)=={'init','static-user.elf'} and boot['static-user.elf']==image, 'actual application boot-resource binding')
    require(json.loads((build/'process-mm-application.json').read_text())['process_memory'] is True, 'experimental application selection')
    batch=json.loads((build/'process-mm-applications/observations.json').read_text())
    names=list(topal_gate.PROCESS_WORKLOADS)
    require(len(names)==51 and batch['passed'] and batch['complete_corpus']
            and len(batch['selected'])==51 and set(batch['selected'])==set(batch['workloads'])==set(names), 'complete 51-workload execution')
    require(batch['kernel_sha256']==kernel_fact['sha256'] and batch['image_sha256']==digest(image)['sha256'], 'batch artifact binding')
    require(json.loads((build/'guest.command.json').read_text())['exit_code']==0, 'native corpus command failure')
    frame=json.loads((build/'frame-inspection.json').read_text())
    require(all(frame['user_extent_callers'][key]<=4096 for key in ['complete_stack_bytes','complete_physical_bytes'])
            and {'p.prepare','p.commit','p.switch','p.child_exit','p.cancel_inactive'}<=set(frame['calls']), 'native process caller bounds')
    artifacts={p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
               if p.suffix in ['.o','.ll','.elf'] or p.name in ['bzImage','initramfs.cpio']}
    preserved={}
    for actual,old in [(name,name) for name in PRESERVED]+[('retained-owned-provider.o','provider.o'),('retained-owned-gateway.o','gateway.o')]:
        data=(build/actual).read_bytes();require(data==(reference/old).read_bytes(), 'changed predecessor '+actual)
        preserved[actual]=digest(data)
    verification=json.loads((build/'llvm-verification.json').read_text())
    require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'}
            and all(row['passed'] and row['sha256']==artifacts[name+'.ll']['sha256'] for name,row in verification.items()), 'seven LLVM modules')
    graph=json.loads((build/'process-access-inventory.json').read_text())
    require(graph['passed'] and graph['kernel_sha256']==kernel_fact['sha256'], 'linked process graph')
    offsets={row['offset'] for row in graph['instructions']}
    report=dict(passed=True,schema='topal-process-mm-native-application-corpus-v1',facts=binding,
        application=digest(image),artifacts=artifacts,preserved=preserved,native_bounds=frame['user_extent_callers'],cases={})
    for name in names:
        native=build/'process-mm-applications'/name
        raw=(native/'observations.json').read_bytes();mm_raw=(native/'process-mm-observations.json').read_bytes()
        serial=(native/'serial.log').read_bytes();user,mm=json.loads(raw),json.loads(mm_raw)
        command,expected,status=topal_gate.PROCESS_WORKLOADS[name]
        row=batch['workloads'][name]
        require(row==dict(passed=True,command_hex=command.hex(),output_hex=expected.hex(),status=status,
            serial_sha256=digest(serial)['sha256'],mm_boundaries=len(mm['events']),
            observation_sha256=digest(raw)['sha256'],mm_observation_sha256=digest(mm_raw)['sha256']), 'actual workload/result/artifact binding')
        result=readback(user,mm,serial,name,binding)
        require({row['offset'] for row in user['extent_call_coverage']['reached']}<=offsets, 'native extent sites outside linked graph')
        script=(native/'inspect.gdb').read_text()
        require(script.count(application.PROGRAM)==1
                and repr(application.BOUNDARY_PROGRAM+application.MM_RETIREMENT_PROGRAM) in script
                and repr(application.EXTENT_PROGRAM) in script, 'exact legacy, extent and MM observer programs')
        qemu=json.loads((native/'qemu.command.json').read_text())
        require(qemu[qemu.index('-m')+1]=='640M' and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
                and qemu[qemu.index('-accel')+1].startswith('tcg,'), 'retained VM configuration')
        stdout=(native/'gdb.stdout.log').read_bytes()
        require(stdout.count(b'FRAME_ROOT_GDB_PASS')==1 and stdout.count(b'FRAME_APPLICATION_GDB_PASS '+name.encode()+b' ')==1,
                'actual debugger application completion')
        result.update(observation=digest(raw),mm_observation=digest(mm_raw),serial=digest(serial),
            command_hex=command.hex(),output_hex=expected.hex(),status=status)
        report['cases'][name]=result
        for p in sorted(native.iterdir()):
            if p.is_file():save('cases/'+name+'/'+p.name,p.read_bytes())
        del user,mm,raw,mm_raw,serial
    totals=['boundaries','retired_extents','erased_released_bytes']
    report['native_summary']={key:sum(row[key] for row in report['cases'].values()) for key in totals}
    report['native_summary'].update(workloads=51,controls=sum(len(row['controls']) for row in report['cases'].values()))
    validation=json.loads((build/'validation.json').read_text())
    require(set(validation)=={'workspace-check','user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(row['exit_code']==0 for row in validation.values()), 'fresh validation closure')
    require('Ran 203 tests' in (build/'user-memory-models.stderr.log').read_text()
            and 'Ran 37 tests' in (build/'process-memory-models.stderr.log').read_text()
            and 'test result: ok. 1 passed' in (build/'default-ABI-fixture.stdout.log').read_text(), 'model/default coverage')
    report['validation']=validation
    execution=json.loads((run/'run.json').read_text())
    require(execution['exit_code']==0 and not execution['source_changes_during_run']
            and execution['source_hashes_before']==execution['source_hashes_after'], 'actual native source stability/result')
    for name,sha in execution['source_hashes_before'].items():
        path=(ROOT/name).resolve()
        require(path.is_relative_to(ROOT/'linux-kernel') or path.is_relative_to(ROOT/'src'), 'native input path')
        require(digest(path.read_bytes())['sha256']==sha, 'native input changed after execution: '+name)
    resources=execution['resources']
    require(int(resources['memory.swap.max'])==0 and 0<int(resources['memory.peak'])<=int(resources['memory.max'])
            and all(int(line.split()[1])==0 for line in resources['memory.events'].splitlines()), 'native resource failure')
    report['execution']=dict(exit_code=0,elapsed_seconds=execution['elapsed_seconds'],group=execution['group'],resources=resources,
        source_changes_during_run=False,driver=digest(driver.read_bytes()),scope=execution['scope'])
    for p in sorted(run.iterdir()):
        if p.is_file():save('execution/'+p.name,p.read_bytes())
    save('execution/driver.py',driver.read_bytes())
    manifest_raw=(prior/'manifest.json').read_bytes();manifest=json.loads(manifest_raw)
    require({p.relative_to(prior).as_posix() for p in prior.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}, 'prior archive closure')
    for name,fact in manifest.items():
        path=(prior/name).resolve();require(path.is_relative_to(prior), 'prior archive path');checked(path.read_bytes(),fact)
    prior_raw=(prior/'report.json').read_bytes();old=json.loads(prior_raw)
    require(old['passed'] and old['schema']=='topal-process-mm-native-interface-regressions-v1'
            and [row['mode'] for row in old['cohorts']]==MODES, 'preceding interface archive')
    report['prior_archive']=dict(path=prior.relative_to(ROOT).as_posix(),manifest=digest(manifest_raw),report=digest(prior_raw))
    save('prior/manifest.json',manifest_raw);save('prior/report.json',prior_raw)
    report['retained_interfaces']={}
    for i,row in enumerate(old['cohorts']):
        require(row['artifacts']['kernel.elf']==kernel_fact, 'retained interface kernel differs')
        raw=checked(gzip.decompress((prior/f'cohorts/{i}/observations.json.gz').read_bytes()),row['user_observation'])
        mm_raw=checked(gzip.decompress((prior/f'cohorts/{i}/mm-observations.json.gz').read_bytes()),row['mm_observation'])
        summary=interface_readback(json.loads(raw),json.loads(mm_raw),kernel_fact['sha256'])
        report['retained_interfaces'][row['mode']]=dict(observation=digest(raw),mm_observation=digest(mm_raw),summary=summary)
        save('retained/interfaces/'+row['mode']+'/observations.json',raw)
        save('retained/interfaces/'+row['mode']+'/mm-observations.json',mm_raw)
    require(len(old['retained_regressions'])==20, 'twenty retained process histories')
    report['retained_process']={}
    for name,row in old['retained_regressions'].items():
        require(row['kernel']==kernel_fact, 'retained process kernel differs')
        raw=checked(gzip.decompress((prior/'regressions'/name/'observations.json.gz').read_bytes()),row['observation'])
        summary=process_reader(name)(json.loads(raw))
        report['retained_process'][name]=dict(observation=digest(raw),summary=summary)
        save('retained/process/'+name+'/observations.json',raw)
    report['retained_scope']='Identified previous native executions on the byte-identical kernel and current record replay; no fresh execution or full historical reconstruction'
    report['history']=[]
    for i,path in enumerate(args.history):
        history=path.resolve()
        report['history'].append(dict(path=history.relative_to(ROOT).as_posix(),
            scope='Selected earlier invocation/preliminary evidence; not current final-digest corpus qualification or complete historical reconstruction'))
        for p in sorted(history.iterdir()):
            if p.is_file() and p.suffix in ['.py','.rs','.ll','.c','.json','.log']:save(f'history/{i}/'+p.name,p.read_bytes())
        nested=history/'process-mm-applications'
        if nested.is_dir():
            for p in sorted(nested.rglob('*')):
                if p.is_file():save(f'history/{i}/process-mm-applications/'+p.relative_to(nested).as_posix(),p.read_bytes())
    for p in sorted(build.iterdir()):
        if p.is_file() and (p.suffix in ['.ll','.o','.elf','.c','.py','.t','.json','.log','.su','.ld'] or p.name in ['bzImage','initramfs.cpio']):
            save('build/'+p.name,p.read_bytes())
    save('build/process-mm-applications/observations.json',(build/'process-mm-applications/observations.json').read_bytes())
    inputs=selected_inputs();report['selected_inputs']={name:digest((ROOT/name).read_bytes()) for name in inputs}
    for name in inputs:save('inputs/'+name,(ROOT/name).read_bytes())
    report['remaining']=['unresolved earlier metadata source-observer assertion and further observer reliability qualification',
        'remaining native caller/loan/mapping producers, continuous lifetime and full invalidation/release ordering',
        'identity-width/resource/fatal partial-effect qualification and cold public reconstruction before activation',
        'original complete Linux UAPI, distribution, Docker/Podman and VM-host objectives']
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,entries=len(manifest),inputs=len(inputs),summary=report['native_summary'])))


if __name__=='__main__':main()
