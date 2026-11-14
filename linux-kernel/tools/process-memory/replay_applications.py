#!/usr/bin/env python3
"""Replay frozen application, interface and process histories without a VM."""
import argparse
import gzip
import json
from pathlib import Path
import subprocess

from replay_process import checked


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();archive=args.archive.resolve();output=args.output.resolve()
    root=Path(__file__).resolve().parents[3]
    if not all(path.is_relative_to(root/'linux-kernel') for path in [archive,output]):raise ValueError('Linux archive/replay paths')
    manifest=json.loads((archive/'manifest.json').read_text())
    if {p.relative_to(archive).as_posix() for p in archive.rglob('*') if p.is_file()}!=set(manifest)|{'manifest.json'}:
        raise ValueError('archive file closure')
    for name,fact in manifest.items():
        target=(archive/name).resolve()
        if not target.is_relative_to(archive):raise ValueError('archive path escapes root')
        checked(target.read_bytes(),fact)
    report=json.loads((archive/'report.json').read_text())
    if not report['passed'] or report['schema']!='topal-process-mm-native-application-corpus-v1':raise ValueError('archive schema')
    output.mkdir(parents=True,exist_ok=False)
    def write(name,raw):
        target=(output/name).resolve()
        if not target.is_relative_to(output):raise ValueError('output path escapes root')
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    def unpack(name,fact):return checked(gzip.decompress((archive/(name+'.gz')).read_bytes()),fact)
    unpack('execution/driver.py',report['execution']['driver'])
    for name,fact in report['selected_inputs'].items():write(name,unpack('inputs/'+name,fact))
    prior=json.loads(unpack('prior/report.json',report['prior_archive']['report']))
    unpack('prior/manifest.json',report['prior_archive']['manifest'])
    if not prior['passed'] or prior['schema']!='topal-process-mm-native-interface-regressions-v1':raise ValueError('preceding archive identity')
    if set(report['retained_interfaces'])!={row['mode'] for row in prior['cohorts']} or set(report['retained_process'])!=set(prior['retained_regressions']):
        raise ValueError('preceding native-history closure')
    for row in prior['cohorts']:
        retained=report['retained_interfaces'][row['mode']]
        if row['artifacts']['kernel.elf']!=report['artifacts']['kernel.elf'] or retained['observation']!=row['user_observation'] or retained['mm_observation']!=row['mm_observation']:
            raise ValueError('preceding interface/kernel binding')
    for name,row in prior['retained_regressions'].items():
        if row['kernel']!=report['artifacts']['kernel.elf'] or report['retained_process'][name]['observation']!=row['observation']:
            raise ValueError('preceding process/kernel binding')
    for name in ['kernel.elf','initramfs.cpio','extent-call-inventory.json']:
        fact=report['artifacts'].get(name)
        raw=gzip.decompress((archive/('build/'+name+'.gz')).read_bytes())
        if fact is not None:checked(raw,fact)
        write('linux-kernel/native-build/'+name,raw)
    for name,row in report['cases'].items():
        for file,key in [('observations.json','observation'),('process-mm-observations.json','mm_observation'),('serial.log','serial')]:
            write('cases/'+name+'/'+file,unpack('cases/'+name+'/'+file,row[key]))
    for name,row in report['retained_interfaces'].items():
        for file,key in [('observations.json','observation'),('mm-observations.json','mm_observation')]:
            write('retained/interfaces/'+name+'/'+file,unpack('retained/interfaces/'+name+'/'+file,row[key]))
    for name,row in report['retained_process'].items():
        write('retained/process/'+name+'/observations.json',unpack('retained/process/'+name+'/observations.json',row['observation']))
    write('expected.json',json.dumps(report).encode())
    script='''import json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from readback_applications import facts,readback
from readback_interfaces import MODES,readback as interface_readback
from archive_applications import process_reader
import topal_gate
expected=json.loads(Path('expected.json').read_text())
binding=facts(Path('linux-kernel/native-build'))
assert binding==expected['facts']
assert len(topal_gate.PROCESS_WORKLOADS)==51 and set(expected['cases'])==set(topal_gate.PROCESS_WORKLOADS)
rows={};controls={}
for name in topal_gate.PROCESS_WORKLOADS:
    folder=Path('cases')/name
    user=json.loads((folder/'observations.json').read_text())
    mm=json.loads((folder/'process-mm-observations.json').read_text())
    serial=(folder/'serial.log').read_bytes()
    actual=json.loads(json.dumps(readback(user,mm,serial,name,binding)))
    assert all(actual[key]==expected['cases'][name][key] for key in actual)
    rows[name]=actual;controls[name]=len(actual['controls'])
    del user,mm,serial
summary={key:sum(row[key] for row in rows.values()) for key in ['boundaries','retired_extents','erased_released_bytes']}
summary.update(workloads=51,controls=sum(controls.values()))
assert summary==expected['native_summary']
assert set(expected['retained_interfaces'])==set(MODES)
interfaces={}
for name,row in expected['retained_interfaces'].items():
    folder=Path('retained/interfaces')/name
    user=json.loads((folder/'observations.json').read_text())
    mm=json.loads((folder/'mm-observations.json').read_text())
    actual=json.loads(json.dumps(interface_readback(user,mm,binding['kernel_sha256'])))
    assert actual==row['summary'];interfaces[name]=len(actual['controls'])
    del user,mm
names={'maximum-56','maximum-57','allocation-refusal','child-exec'}|{
    f'{wait}-cancellation-{stage}-{exit_kind}' for wait in ['reader','writer']
    for stage in ['early','late'] for exit_kind in ['exit','exit-group']}|{
    f'relationship-{z}-{e}-{s}' for z in [0,1] for e in [0,1] for s in [60,231]}
assert set(expected['retained_process'])==names
processes={}
for name,row in expected['retained_process'].items():
    raw=json.loads((Path('retained/process')/name/'observations.json').read_text())
    actual=json.loads(json.dumps(process_reader(name)(raw)))
    assert actual==row['summary'];processes[name]=len(actual['controls'])+len(actual.get('child_exec_controls',[]))
    del raw
Path('records.json').write_text(json.dumps(dict(passed=True,summary=summary,application_controls=controls,
    retained_interface_controls=interfaces,retained_process_controls=processes))+'\\n')
print('Frozen 51-workload application corpus and identified interface/process records passed')
'''
    subprocess.run(['rtk','proxy','python3','-c',script],cwd=output,check=True)
    models={}
    for name,folder,count in [('user-memory','user-memory',203),('process-memory','process-memory',37)]:
        command=['rtk','proxy','python3','-m','unittest','discover','-s','linux-kernel/tools/'+folder,'-p','test_*.py']
        result=subprocess.run(command,cwd=output,capture_output=True,text=True)
        write(name+'-models.stdout.log',result.stdout.encode());write(name+'-models.stderr.log',result.stderr.encode())
        write(name+'-models.command.json',json.dumps(dict(argv=command,exit_code=result.returncode)).encode())
        if result.returncode or f'Ran {count} tests' not in result.stderr:raise RuntimeError('frozen '+name+' model coverage failed')
        models[name]=dict(passed=True,tests=count,exit_code=result.returncode)
    completed=json.loads((output/'records.json').read_text())
    completed.update(model_suites=models,
        scope='Frozen saved-record comparisons and Python suites; no VM execution or kernel rebuild')
    write('replayed.json',(json.dumps(completed,indent=2,sort_keys=True)+'\n').encode())
    print('Frozen 203 user-memory and 37 process-memory tests passed')


if __name__=='__main__':main()
