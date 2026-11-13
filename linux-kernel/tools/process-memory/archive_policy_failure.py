#!/usr/bin/env python3
"""Collect or replay source-bound diagnostic evidence; replay runs no VM.

TOPAL-SYSTEMS-PROCESS-MM-001. Every archived byte is manifested. Frozen
readers validate saved observations and CPU logs, not another execution.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def fact(raw):
    return dict(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())


def collect(matrix,output,validation=None,original_collector=None):
    from run_policy_failure import ROOT,CASES,VALIDATIONS,hashes
    from readback_policy_failure import readback as failure_readback
    from readback_publication import readback as ordinary_readback
    run=json.loads((matrix/'run.json').read_text())
    assert not run['source_changes_during_run'] and run['source_hashes_before']==run['source_hashes_after']
    assert [r['label'] for r in run['cohorts']]==[r[0] for r in CASES]
    assert all(r['exit_code']==r['readback']['exit_code']==0 for r in run['cohorts'])
    current=hashes(run['source_hashes_before'])
    changed={name for name in current if current[name]!=run['source_hashes_before'][name]}
    collector=Path(__file__).resolve().relative_to(ROOT).as_posix()
    original=None
    if changed:
        # Only this non-native collector may be repaired after the matrix.
        # Preserve its exact former bytes and every original native-phase hash.
        # No compiler, policy, application, observer or reader change is allowed.
        assert changed=={collector} and original_collector is not None
        assert original_collector.is_relative_to(ROOT/'linux-kernel')
        original=original_collector.read_bytes()
        assert fact(original)['sha256']==run['source_hashes_before'][collector]
    qualification=run
    if validation is not None:
        assert validation.is_relative_to(ROOT/'linux-kernel')
        qualification=json.loads((validation/'qualification.json').read_text())
        assert qualification['native_matrix']==fact((matrix/'run.json').read_bytes())
        assert qualification['source_hashes_before']==qualification['source_hashes_after']==current
    else:
        assert not changed
    assert qualification['passed'] and qualification['exit_code']==0
    assert set(qualification['validation'])==set(VALIDATIONS)
    assert all(v['exit_code']==0 for v in qualification['validation'].values())
    output.mkdir(parents=True,exist_ok=False)
    manifest={}
    def save(name,raw):
        path=output/name;path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(raw);manifest[name]=fact(raw)
    def compressed(name,raw):save(name+'.gz',gzip.compress(raw,mtime=0))
    report=dict(passed=True,schema='topal-process-mm-source-policy-failure-v1',
        sources={},cohorts={},scope='One ordinary positive VM and two source-built fatal diagnostics; frozen replay is saved-data evidence. Every other fatal path, complete caller/lifetime qualification and public activation remain open.')
    report['native_phase_sources']=run['source_hashes_before']
    report['post_native_collector_repair']=None
    if original is not None:
        compressed('history/native-phase-collector.py',original)
        report['post_native_collector_repair']=dict(path=collector,original=fact(original),
            current=fact(Path(__file__).read_bytes()),
            scope='Collector-only JSON key normalization and explicit recovered-validation provenance. Native compiler/policy/application/observer/reader inputs are unchanged; original matrix and collector bytes are retained.')
    for name,sha in current.items():
        raw=(ROOT/name).read_bytes();assert fact(raw)['sha256']==sha
        report['sources'][name]=fact(raw);compressed('inputs/'+name,raw)
    for path in sorted(matrix.rglob('*')):
        if path.is_file():compressed('matrix/'+path.relative_to(matrix).as_posix(),path.read_bytes())
    if validation is not None:
        for path in sorted(validation.rglob('*')):
            if path.is_file():compressed('validation/'+path.relative_to(validation).as_posix(),path.read_bytes())
    for row in run['cohorts']:
        assert row['exit_code']==row['readback']['exit_code']==0 and len(row['builds'])==1
        build=ROOT/row['builds'][0];label=row['label']
        assert build.resolve().is_relative_to(ROOT/'linux-kernel/.cache/x86_64')
        expected=json.loads(json.dumps((ordinary_readback if label=='ordinary' else failure_readback)(build)))
        name='process-mm-users/publication-readback.json' if label=='ordinary' else 'process-mm-policy-failure/readback.json'
        assert expected==json.loads((build/name).read_text())
        opt=json.loads((build/'root-target-verify.command.json').read_text())['argv'][0]
        verified={}
        for role in ['source','runtime','owner','controller','target','dispatch','extent']:
            command=['rtk','proxy',opt,'-passes=verify','-disable-output',str(build/(role+'.ll'))]
            result=subprocess.run(command,capture_output=True)
            assert result.returncode==0,result.stderr
            verified[role]=dict(passed=True,input=fact((build/(role+'.ll')).read_bytes()),argv=command)
        report['cohorts'][label]=dict(summary=expected,llvm=verified,files={})
        for path in sorted(build.rglob('*')):
            if path.is_file():
                relative=path.relative_to(build).as_posix();raw=path.read_bytes()
                report['cohorts'][label]['files'][relative]=fact(raw)
                compressed('native/'+label+'/'+relative,raw)
    save('report.json',(json.dumps(report,indent=2,sort_keys=True)+'\n').encode())
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,archive=str(output),entries=len(manifest),sources=len(report['sources']),
                         manifest=fact((output/'manifest.json').read_bytes()))))


def replay(archive,output):
    manifest=json.loads((archive/'manifest.json').read_text())
    assert {p.relative_to(archive).as_posix() for p in archive.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}
    for name,expected in manifest.items():
        path=(archive/name).resolve();assert path.is_relative_to(archive)
        assert fact(path.read_bytes())==expected
    report=json.loads((archive/'report.json').read_text())
    assert report['passed'] and report['schema']=='topal-process-mm-source-policy-failure-v1'
    assert set(report['cohorts'])=={'ordinary','allocate','release'}
    output.mkdir(parents=True,exist_ok=False)
    def restore(target,name,expected):
        assert target.resolve().is_relative_to(output)
        raw=gzip.decompress((archive/(name+'.gz')).read_bytes());assert fact(raw)==expected
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    for name,expected in report['sources'].items():restore(output/name,'inputs/'+name,expected)
    summaries={}
    for label,case in report['cohorts'].items():
        build=output/'linux-kernel/.cache/x86_64/policy-failure-replay'/label
        for name,expected in case['files'].items():restore(build/name,'native/'+label+'/'+name,expected)
        reader='readback_publication.py' if label=='ordinary' else 'readback_policy_failure.py'
        command=['rtk','proxy','python3','linux-kernel/tools/process-memory/'+reader,str(build)]
        result=subprocess.run(command,cwd=output,capture_output=True,text=True)
        (output/(label+'.stdout.log')).write_text(result.stdout)
        (output/(label+'.stderr.log')).write_text(result.stderr)
        assert result.returncode==0,result.stderr
        name='process-mm-users/publication-readback.json' if label=='ordinary' else 'process-mm-policy-failure/readback.json'
        summaries[label]=json.loads((build/name).read_text());assert summaries[label]==case['summary']
    result=dict(passed=True,cohorts=summaries,manifest=fact((archive/'manifest.json').read_bytes()),
                scope='Frozen saved-record readers, complete CPU logs and controls; no native rerun or kernel rebuild')
    (output/'replayed.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True,cohorts=list(summaries),output=str(output))))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['collect','replay']);parser.add_argument('input',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--validation',type=Path)
    parser.add_argument('--original-collector',type=Path)
    args=parser.parse_args();root=Path(__file__).resolve().parents[3]
    source,destination=args.input.resolve(),args.output.resolve()
    assert source.is_relative_to(root/'linux-kernel') and destination.is_relative_to(root/'linux-kernel')
    if args.action=='collect':
        collect(source,destination,args.validation.resolve() if args.validation else None,
                args.original_collector.resolve() if args.original_collector else None)
    else:
        assert args.validation is args.original_collector is None
        replay(source,destination)
