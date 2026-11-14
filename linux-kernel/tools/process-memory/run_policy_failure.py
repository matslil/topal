#!/usr/bin/env python3
"""Run the positive process cohort and four source-built fatal diagnostics.

Invoke through scripts/run_bounded.py. This driver retains commands, output,
actual source identities and cumulative cgroup resource observations.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

from archive_publication import ROOT, selected_inputs


CASES = [
    ('ordinary', 'process_mm_fork_handoff_and_exit_preserve_independent_heap_owners', 'readback_publication.py'),
    ('allocate', 'policy_failure::allocation_source_failure_retains_real_process_owners_and_performed_effects', 'readback_policy_failure.py'),
    ('release', 'policy_failure::release_source_failure_retains_published_candidate_and_performed_effects', 'readback_policy_failure.py'),
    ('partial-allocate', 'policy_failure::partial_allocation_source_status_retains_written_prefix_and_process_owners', 'readback_policy_failure.py'),
    ('partial-release', 'policy_failure::partial_release_source_status_retains_published_candidate_and_written_prefix', 'readback_policy_failure.py'),
]

VALIDATIONS = {
    'workspace-check':['cargo','check','--workspace','--all-targets','--locked','-j1'],
    'compiler-units':['cargo','test','-p','topal-compiler','--lib','--locked','-j1'],
    'user-memory-models':['python3','-m','unittest','discover','-s','linux-kernel/tools/user-memory','-p','test_*.py'],
    'process-memory-models':['python3','-m','unittest','discover','-s','linux-kernel/tools/process-memory','-p','test_*.py'],
    'default-ABI-fixture':['cargo','test','-p','topal-compiler','--lib','--locked','-j1','default_abi_fixture_preserves_committed_generator_bytes'],
    'format':['cargo','fmt','--all','--','--check'],
    'Rust-file-ceiling':['python3','scripts/check_rust_file_sizes.py'],
}

REFERENCE_COMMIT = '9c84c21a305f502a29aaa0cce974ba85fc879c59'
REFERENCE_KERNELS = dict(
    ordinary='b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c',
    allocate='541a0e59e0c03faec805e361fad01bccafcab16c8e3490cad77cba8cb169a2fb',
    release='72c7827a529a3abbdb968b1f30f95195d8bac49983be3ab608476b96683e6159')


def validation_commands(profile):
    assert profile in ['full','observer']
    return {name:command for name,command in VALIDATIONS.items()
            if profile=='full' or name!='compiler-units'}


def observer_reference():
    """Reuse the full suite only for a private-test/observation-only increment.

    The new matrix still checks all three existing native kernel identities.
    Any other compiler, dependency or higher-authority source change refuses
    this profile; select full validation for those changes.
    """
    paths=['src','Cargo.toml','Cargo.lock','docs','se','spec','scripts','linux-kernel/kernel','linux-kernel/design']
    changed=subprocess.check_output(['rtk','proxy','git','diff','--name-only',REFERENCE_COMMIT,'--']+paths,cwd=ROOT).decode().splitlines()
    assert set(changed)<={'src/topal-compiler/src/systems_process_mm_fault_tests.rs'}
    path=ROOT/'linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json'
    raw=path.read_bytes();receipt=json.loads(raw);qualification=receipt['qualification']
    assert receipt['passed'] and qualification['passed'] and qualification['exit_code']==0
    assert set(qualification['validation'])==set(VALIDATIONS)
    assert all(row['exit_code']==0 for row in qualification['validation'].values())
    before=qualification['source_hashes_before']
    assert before==qualification['source_hashes_after']
    exempt='src/topal-compiler/src/systems_process_mm_fault_tests.rs'
    compiler={name:sha for name,sha in before.items() if name.startswith('src/topal-compiler/') and name!=exempt}
    actual={name for name in watched() if name.startswith('src/topal-compiler/') and name!=exempt}
    assert actual==set(compiler) and hashes(compiler)==compiler
    return dict(commit=REFERENCE_COMMIT,receipt=path.relative_to(ROOT).as_posix(),
        receipt_sha256=hashlib.sha256(raw).hexdigest(),unchanged_compiler_inputs=len(compiler),
        scope='Earlier full383-test compiler suite, not rerun. Current changes are private test policies and observation/readback tools; fresh workspace/models/ABI/format/ceiling and five native cases remain mandatory.')


def watched():
    paths=set(selected_inputs([]))
    for base,patterns in [
            ('src/topal-compiler/src',['**/*.rs','**/*.ll']),
            ('linux-kernel/kernel',['**/*.t']),
            ('linux-kernel/labs/qemu/x86_64',['*.py','*.c']),
            ('linux-kernel/tools/user-memory',['*.py']),
            ('linux-kernel/tools/process-memory',['*.py'])]:
        for pattern in patterns:
            paths.update(p.relative_to(ROOT).as_posix() for p in (ROOT/base).glob(pattern) if p.is_file())
    paths.update(['Cargo.toml','Cargo.lock','src/topal-compiler/Cargo.toml','AGENTS.md','scripts/run_bounded.py'])
    paths.add('linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json')
    return sorted(paths)


def hashes(paths):
    return {name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in paths}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path)
    parser.add_argument('--validation-profile',choices=['full','observer'],default='full')
    args=parser.parse_args();output=args.output.resolve()
    assert output.is_relative_to(ROOT/'linux-kernel')
    # Require the normal wrapper's active no-swap cgroup, rather than silently
    # permitting an unbounded invocation of this potentially long matrix.
    group=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    directory=Path('/sys/fs/cgroup')/group.lstrip('/')
    assert 'topal-bounded-' in group and (directory/'memory.swap.max').read_text().strip()=='0'
    output.mkdir(parents=True,exist_ok=False)
    paths=watched();before=hashes(paths);start=time.monotonic()
    report=dict(passed=False,source_hashes_before=before,cohorts=[],validation_profile=args.validation_profile,
        compiler_reference=observer_reference() if args.validation_profile=='observer' else None)
    (output/'request.json').write_text(json.dumps(report,indent=2)+'\n')
    (output/'driver.py').write_bytes(Path(__file__).read_bytes())
    failure=0
    for label,test,reader in CASES:
        folder=output/label;folder.mkdir()
        command=['rtk','proxy','cargo','test','-p','topal-compiler','--lib','--locked','-j1',
            'systems_frame_target::root_tests::'+test,'--','--ignored','--exact','--test-threads=1','--nocapture']
        environment=dict(os.environ,TOPAL_TEST_PROCESS_MM_PUBLICATION='1')
        began=time.monotonic()
        with (folder/'test.log').open('w') as stream:
            result=subprocess.run(command,cwd=ROOT,env=environment,stdout=stream,stderr=subprocess.STDOUT)
        text=(folder/'test.log').read_text()
        builds=[Path(name.strip()).resolve() for name in re.findall(r'actual frame source root: ([^\n]+)',text)]
        row=dict(label=label,argv=command,environment={'TOPAL_TEST_PROCESS_MM_PUBLICATION':'1'},
            exit_code=result.returncode,elapsed_seconds=time.monotonic()-began,
            builds=[p.relative_to(ROOT).as_posix() for p in builds])
        report['cohorts'].append(row)
        (folder/'native.json').write_text(json.dumps(row,indent=2)+'\n')
        (output/'progress.json').write_text(json.dumps(report,indent=2)+'\n')
        print(label,'native',result.returncode,'builds',len(builds),flush=True)
        failure=result.returncode
        if failure:print(text[-8000:],flush=True);break
        assert len(builds)==1 and builds[0].is_relative_to(ROOT/'linux-kernel/.cache/x86_64')
        if label in REFERENCE_KERNELS:
            assert hashlib.sha256((builds[0]/'kernel.elf').read_bytes()).hexdigest()==REFERENCE_KERNELS[label]
        command=['rtk','proxy','python3','linux-kernel/tools/process-memory/'+reader,str(builds[0])]
        result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
        (folder/'readback.stdout.log').write_text(result.stdout)
        (folder/'readback.stderr.log').write_text(result.stderr)
        row['readback']=dict(argv=command,exit_code=result.returncode)
        print(label,'readback',result.returncode,flush=True)
        failure=result.returncode
        if failure:print(result.stderr[-8000:],flush=True);break
    report['validation']={}
    if not failure:
        folder=output/'validation';folder.mkdir()
        for name,arguments in validation_commands(args.validation_profile).items():
            command=['rtk','proxy']+arguments;began=time.monotonic()
            result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
            (folder/(name+'.stdout.log')).write_text(result.stdout)
            (folder/(name+'.stderr.log')).write_text(result.stderr)
            report['validation'][name]=dict(argv=command,exit_code=result.returncode,
                elapsed_seconds=time.monotonic()-began)
            print(name,result.returncode,flush=True)
            failure=result.returncode
            if failure:print(result.stderr[-8000:],flush=True);break
    resources={name:(directory/name).read_text().strip() for name in
               ['memory.max','memory.swap.max','memory.peak','memory.events']}
    after=hashes(paths)
    report.update(passed=not failure and len(report['cohorts'])==len(CASES) and before==after,
        exit_code=failure,elapsed_seconds=time.monotonic()-start,source_hashes_after=after,
        source_changes_during_run=before!=after,group=group,resources=resources,
        scope='Five actual isolated VMs, complete saved-record readbacks and explicit validation profile; cumulative bounded resources, not per-VM peaks. Four diagnostic policies are not canonical allocator qualification.')
    (output/'run.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['source_hashes_before','source_hashes_after','cohorts']}),flush=True)
    return failure or int(not report['passed'])


if __name__=='__main__':
    sys.exit(main())
