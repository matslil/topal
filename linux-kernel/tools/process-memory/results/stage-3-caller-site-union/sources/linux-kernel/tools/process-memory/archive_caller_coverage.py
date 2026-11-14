#!/usr/bin/env python3
"""Collect/replay first-reach coverage from identified previous native runs.

Run through scripts/run_bounded.py. No VM, compiler or guest executes here.
TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from caller_coverage import combine, require

ROOT=Path(__file__).resolve().parents[3]
SOURCES=['linux-kernel/tools/process-memory/'+name for name in
         ['caller_coverage.py','test_caller_coverage.py','archive_caller_coverage.py']]
SOURCES+=['linux-kernel/tools/user-memory/inspect_call_inventory.py']
ORIGINS=dict(applications='stage-3-application-corpus',interfaces='stage-3-interface-regressions')


def fact(raw):
    return dict(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())


def member_path(base,name):
    require(isinstance(name,str) and name and not Path(name).is_absolute(),'caller member path')
    path=(base/name).resolve()
    require(path!=base.resolve() and path.is_relative_to(base.resolve()),'caller member escapes archive')
    return path


def group():
    name=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    path=Path('/sys/fs/cgroup')/name.lstrip('/')
    require('topal-bounded-' in name and (path/'memory.swap.max').read_text().strip()=='0',
            'caller census requires the normal bounded no-swap wrapper')
    return path


def origin_report(base,identity):
    report=(base/'report.json').read_bytes();manifest=(base/'manifest.json').read_bytes()
    require(fact(manifest)==identity['manifest'],'caller original manifest')
    require(fact(report)==identity['report'],'caller original report')
    require(json.loads(manifest).get('report.json')==fact(report),
            'caller original report/manifest disagreement')
    result=json.loads(report);require(result['passed'] is True,'caller original report failed')
    return result


def inspect(bundle):
    origins=json.loads((bundle/'origins.json').read_text())
    require(set(origins)==set(ORIGINS),'caller origin closure')
    def packed(origin,name):
        base=bundle/'evidence'/origin
        manifest=json.loads((base/'manifest.json').read_text())
        raw=member_path(base,name).read_bytes()
        require(fact(raw)==manifest[name],'caller packed origin differs: '+name)
        return gzip.decompress(raw)
    def read(origin,name):return json.loads(packed(origin,name))
    reports={}
    for origin,identity in origins.items():
        base=bundle/'evidence'/origin
        reports[origin]=origin_report(base,identity)
    inventory_raw=packed('applications','build/extent-call-inventory.json.gz')
    inventory=json.loads(inventory_raw)
    kernel=packed('applications','build/kernel.elf.gz')
    provider=packed('applications','build/provider.o.gz')
    kernel_sha=fact(kernel)['sha256'];inventory_sha=fact(inventory_raw)['sha256']
    require(fact(kernel)==reports['applications']['artifacts']['kernel.elf'],'caller kernel provenance')
    require(fact(provider)==reports['applications']['artifacts']['provider.o'],'caller provider provenance')
    sys.path.insert(0,str(ROOT/'linux-kernel/tools/user-memory'))
    from inspect_call_inventory import Elf, verify_calls
    with tempfile.TemporaryDirectory(prefix='caller-inventory-',dir=ROOT/'linux-kernel/.cache/x86_64') as temp:
        temp=Path(temp);(temp/'kernel.elf').write_bytes(kernel);(temp/'provider.o').write_bytes(provider)
        verify_calls(Elf(temp/'kernel.elf'),inventory)
        verify_calls(Elf(temp/'provider.o'),inventory)
    aggregate=read('applications','build/process-mm-applications/observations.json.gz')
    selected=aggregate['selected']
    require(aggregate['passed'] and aggregate['complete_corpus'] and len(selected)==len(set(selected))==51,
            'caller complete historical application corpus')
    require(set(selected)==set(reports['applications']['cases'])==set(aggregate['workloads']),
            'caller application closure')
    observations={};first={};where={}
    for name in selected:
        raw=packed('applications','cases/'+name+'/observations.json.gz')
        require(fact(raw)==reports['applications']['cases'][name]['observation'],
                'caller application observation provenance')
        require(fact(raw)['sha256']==aggregate['workloads'][name]['observation_sha256'],
                'caller application attribution')
        value=json.loads(raw);require('extent_call_coverage' in value,'caller application coverage absent')
        observations['application/'+name]=value
        for row in value['extent_call_coverage']['reached']:
            first.setdefault(row['offset'],row);where.setdefault(row['offset'],[]).append(name)
    expected=[dict(first[offset],sample_workload=where[offset][0],observed_in_workloads=where[offset])
              for offset in sorted(first)]
    coverage=aggregate['extent_call_coverage']
    require(coverage['observation_passed'] and coverage['kernel_sha256']==kernel_sha
            and coverage['inventory_sha256']==inventory_sha and coverage['total_sites']==len(inventory['calls'])
            and coverage['reached']==expected and coverage['reached_sites']==len(expected),
            'caller historical aggregate reconstruction')
    interfaces=reports['interfaces']['cohorts'];require(len(interfaces)==9,'caller interface cohort closure')
    for index,cohort in enumerate(interfaces):
        require(cohort['passed'] and cohort['artifacts']['kernel.elf']==fact(kernel),
                'caller interface native kernel')
        raw=packed('interfaces','cohorts/'+str(index)+'/observations.json.gz')
        require(fact(raw)==cohort['user_observation'],'caller interface observation provenance')
        observations['interface/'+str(index)]=json.loads(raw)
    result=combine(inventory,inventory_sha,kernel_sha,observations)
    require(result['excluded_without_coverage']==['interface/1','interface/2'],
            'caller unexpected missing observation field')
    result.update(previous_native_executions=60,application_aggregate_reached=len(first),
                  original_manifests=origins,native_executions_in_this_stage=0)
    return result


def collect(output):
    directory=group();began=time.monotonic()
    before={name:fact((ROOT/name).read_bytes()) for name in SOURCES}
    require(all(not output.is_relative_to(ROOT/'linux-kernel/tools/process-memory/results'/name)
                for name in ORIGINS.values()),'caller output overlaps original archive')
    output.mkdir(parents=True,exist_ok=False)
    origins={}
    for origin,name in ORIGINS.items():
        base=ROOT/'linux-kernel/tools/process-memory/results'/name
        for filename in ['report.json','manifest.json']:
            raw=(base/filename).read_bytes();dest=output/'evidence'/origin/filename
            dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
        origins[origin]=dict(path=base.relative_to(ROOT).as_posix(),
            report=fact((base/'report.json').read_bytes()),manifest=fact((base/'manifest.json').read_bytes()))
        names=[]
        if origin=='applications':
            report=json.loads((base/'report.json').read_bytes())
            names=['build/extent-call-inventory.json.gz','build/kernel.elf.gz','build/provider.o.gz',
                   'build/process-mm-applications/observations.json.gz']
            names+=['cases/'+case+'/observations.json.gz' for case in report['cases']]
        else:names=['cohorts/'+str(index)+'/observations.json.gz' for index in range(9)]
        manifest=json.loads((base/'manifest.json').read_bytes())
        for name in names:
            raw=member_path(base,name).read_bytes();require(fact(raw)==manifest[name],'caller selected packed origin')
            target=member_path(output/'evidence'/origin,name);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    (output/'origins.json').write_text(json.dumps(origins,indent=2,sort_keys=True)+'\n')
    for name in SOURCES:
        path=output/'sources'/name;path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes((ROOT/name).read_bytes())
    result=inspect(output)
    command=['rtk','proxy','python3','-m','unittest','discover','-s','linux-kernel/tools/process-memory','-p','test_caller_coverage.py']
    test=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
    (output/'tests.stdout.log').write_text(test.stdout);(output/'tests.stderr.log').write_text(test.stderr)
    require(test.returncode==0,'caller census tests failed')
    require(before=={name:fact((ROOT/name).read_bytes()) for name in SOURCES},'caller source changed during collection')
    resources={name:(directory/name).read_text().strip() for name in ['memory.max','memory.swap.max','memory.peak','memory.events']}
    require(all(int(line.split()[1])==0 for line in resources['memory.events'].splitlines()),'caller resource events')
    result.update(sources=before,validation=dict(argv=command,exit_code=0),
                  elapsed_seconds=time.monotonic()-began,resources=resources)
    (output/'report.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    manifest={p.relative_to(output).as_posix():fact(p.read_bytes()) for p in output.rglob('*') if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,reached=result['reached_sites'],total=result['total_sites'],entries=len(manifest))))


def replay(bundle,output):
    group();manifest=json.loads((bundle/'manifest.json').read_bytes())
    require(not output.is_relative_to(bundle),'caller replay overlaps archive')
    require(set(manifest)|{'manifest.json'}=={p.relative_to(bundle).as_posix() for p in bundle.rglob('*') if p.is_file()},
            'caller archive closure')
    for name,expected in manifest.items():
        path=(bundle/name).resolve();require(path.is_relative_to(bundle) and fact(path.read_bytes())==expected,'caller archive bytes')
    output.mkdir(parents=True,exist_ok=False)
    report=json.loads((bundle/'report.json').read_bytes())
    require(set(report['sources'])==set(SOURCES),'caller frozen source closure')
    for name,expected in report['sources'].items():
        raw=(bundle/'sources'/name).read_bytes();require(fact(raw)==expected,'caller frozen source')
        target=output/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    (output/'linux-kernel/.cache/x86_64').mkdir(parents=True)
    # Execute the frozen collector's reader, not the current worktree model.
    program='import json,sys; sys.path.insert(0,"linux-kernel/tools/process-memory"); from archive_caller_coverage import inspect; from pathlib import Path; print(json.dumps(inspect(Path(sys.argv[1]))))'
    checked=subprocess.run(['rtk','proxy','python3','-c',program,str(bundle)],cwd=output,capture_output=True,text=True)
    (output/'reader.stdout.log').write_text(checked.stdout);(output/'reader.stderr.log').write_text(checked.stderr)
    require(checked.returncode==0,'caller frozen reader failed: '+checked.stderr[-2000:])
    expected={key:report[key] for key in json.loads(checked.stdout)}
    require(json.loads(checked.stdout)==expected,'caller frozen result differs')
    command=['rtk','proxy','python3','-m','unittest','discover','-s','linux-kernel/tools/process-memory','-p','test_caller_coverage.py']
    tested=subprocess.run(command,cwd=output,capture_output=True,text=True)
    (output/'tests.stdout.log').write_text(tested.stdout);(output/'tests.stderr.log').write_text(tested.stderr)
    require(tested.returncode==0,'caller frozen tests failed')
    (output/'replayed.json').write_text(json.dumps(dict(passed=True,archive_manifest=fact((bundle/'manifest.json').read_bytes()),
        reached_sites=report['reached_sites'],total_sites=report['total_sites'],native_executions=0,
        scope='Frozen saved-data/native-byte reader and ten model tests only; no VM or kernel rebuild'),indent=2)+'\n')
    print(json.dumps(dict(passed=True,reached=report['reached_sites'],total=report['total_sites'])))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('action',choices=['collect','replay'])
    parser.add_argument('input',type=Path);parser.add_argument('output',type=Path,nargs='?');args=parser.parse_args()
    first=args.input.resolve();require(first.is_relative_to(ROOT/'linux-kernel'),'caller path outside task')
    if args.action=='collect':
        require(args.output is None,'caller collect takes one output');collect(first)
    else:
        require(args.output is not None,'caller replay needs output');second=args.output.resolve()
        require(second.is_relative_to(ROOT/'linux-kernel'),'caller replay outside task');replay(first,second)
