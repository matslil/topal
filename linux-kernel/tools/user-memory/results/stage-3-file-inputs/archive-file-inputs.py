"""Freeze volatile file input evidence without changing executable kernel bytes."""
import gzip
import hashlib
import json
from pathlib import Path

cache=Path('linux-kernel/.cache/x86_64')
root=Path('linux-kernel/tools/user-memory/results/stage-3-file-inputs')
prior=root.parent/'stage-3-pipe-loops'
builds=dict(storage=cache/'user-extent-source-storage-2382778-1794455130144072641',
    source=cache/'user-extent-source-2383420-1794455566798945157',
    pinned=cache/'user-extent-application-2383173-1794455447746904465')
runs=dict(storage=cache/'tk283-staging-focused-1794455123493434631',
    source=cache/'tk283-staging-focused-1794455565801746442',
    pinned=cache/'tk283-staging-focused-1794455446647758243',
    fixtures=cache/'tk283-staging-focused-1794455922945760548')
digest=lambda data:hashlib.sha256(data).hexdigest()
frozen=None
for label,path in runs.items():
    run=json.loads((path/'run.json').read_text())
    assert run['exit_code']==0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
    assert run['selected_tests']==(2 if label=='fixtures' else 1)
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
    if frozen is None: frozen=run['modified_source_hashes_before']
    assert frozen==run['modified_source_hashes_before']
assert len(frozen)==9 and not root.exists()
root.mkdir()
def save(name,data):
    path=root/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)
def write(name,value): save(name,(json.dumps(value,indent=2)+'\n').encode())
def zipped(name,data):
    packed=gzip.compress(data,mtime=0)
    save(name,packed)
    return dict(snapshot=name,sha256=digest(data),bytes=len(data),snapshot_sha256=digest(packed))
old_inputs=(prior/'source-identity.json').read_bytes()
inputs={name:dict(row,snapshot='../stage-3-pipe-loops/'+row['snapshot'])
    for name,row in json.loads(old_inputs)['selected_inputs'].items()}
for name,sha in frozen.items():
    raw=Path(name).read_bytes()
    assert digest(raw)==sha
    inputs[name]=zipped('inputs/'+name+'.gz',raw)
assert len(inputs)==155
write('source-identity.json',dict(base_commit='0bdb6fa1c6a918b9e06ef2ede4731efc6bf5a5b5',
    inherited_record='../stage-3-pipe-loops/source-identity.json',inherited_record_sha256=digest(old_inputs),
    selected_inputs=inputs,scope='Selected frozen sources; not a complete clean producer or production qualification'))
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),
    ('pipe-loop-audit.json','user_accesses'),('file-input-audit.json','copy_offset')]
old_artifacts=json.loads((prior/'artifact-identity.json').read_text())
artifacts={name:dict(row,snapshot='../stage-3-pipe-loops/'+row['snapshot'])
    for name,row in old_artifacts.items() if not name.startswith('before/')}
for name in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio','file-input-audit.json']:
    artifacts[name]=zipped('artifacts/'+name+'.gz',(builds['storage']/name).read_bytes())
assert len(artifacts)==33
write('artifact-identity.json',artifacts)
kernel_sha=artifacts['kernel.elf']['sha256']
selected,accesses={},set()
for name,field in fields:
    report=json.loads((builds['storage']/name).read_text())
    assert report['passed'] and report['kernel_sha256']==kernel_sha
    assert digest((builds['storage']/name).read_bytes())==artifacts[name]['sha256']
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']]=dict(name=row['name'],call_offset=row['call_offset'],audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field=='copy_offset' else [r['offset'] for r in row[field]])
census=json.loads((builds['storage']/'extent-call-inventory.json').read_text())
remaining=[r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining))==(88,316,3)
write('caller-access-index.json',dict(complete=False,kernel_sha256=kernel_sha,emitted_helper_calls=91,
    calls_with_selected_local_intervals=88,distinct_user_access_instructions=316,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Ten same-kernel conditional local reports; not complete owners/lifetime/callers/fallbacks or production qualification'))
for label,path in runs.items():
    save('native/'+label+'/run.json',(path/'run.json').read_bytes())
    zipped('native/'+label+'/test.log.gz',(path/'test.log').read_bytes())
coverage={}
cohort_identities={}
for label,build in builds.items():
    folders=[p for p in build.iterdir() if p.is_dir() and (p/'observations.json').is_file()]
    assert len(folders)==1
    obs=json.loads((folders[0]/'observations.json').read_text())
    assert obs['passed'] and obs['extent_call_coverage']['kernel_sha256']==kernel_sha
    coverage[label]={r['offset'] for r in obs['extent_call_coverage']['reached']}
    for path in folders[0].iterdir():
        if path.is_file() and path.suffix in ['.json','.log','.gdb']:
            zipped('native/'+label+'/observation/'+path.name+'.gz',path.read_bytes())
    if (build/'source-component.json').is_file():
        zipped('native/'+label+'/source-component.json.gz',(build/'source-component.json').read_bytes())
    cohort_identities[label]={}
    for name in artifacts:
        if not (build/name).exists(): continue
        raw=(build/name).read_bytes()
        cohort_identities[label][name]=dict(sha256=digest(raw),bytes=len(raw))
        if name not in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
            assert digest(raw)==artifacts[name]['sha256']
write('native-input-identities.json',cohort_identities)
previous_coverage=(prior/'native-coverage.json').read_bytes()
old_coverage=json.loads(previous_coverage)
assert old_coverage['kernel_sha256']==kernel_sha
union=set().union(*coverage.values())
combined=union|set(old_coverage['reached'])
assert (len(union),len(combined))==(83,87)
write('native-coverage.json',dict(passed=True,kernel_sha256=kernel_sha,
    fresh_first_reach_sites={k:len(v) for k,v in coverage.items()},fresh_matching_kernel_union=len(union),
    fresh_reached=sorted(union),inherited_record='../stage-3-pipe-loops/native-coverage.json',
    inherited_record_sha256=digest(previous_coverage),matching_kernel_union=len(combined),
    reached=sorted(combined),unreached=[r for r in census['calls'] if r['offset'] not in combined],
    scope='Three fresh cohorts plus the explicitly referenced preceding three cohorts on the same executable/census; not earlier-kernel coverage, complete runtime instruction coverage or ownership/lifetime qualification'))
save('artifact-comparison.json',(cache/'file-input-artifact-comparison.json').read_bytes())
save('native-readback.json',(cache/'file-input-native-readback/readback.json').read_bytes())
save('gnu-operands.json',(cache/'file-input-gnu-readback-v2/readback.json').read_bytes())
gnu_rows={}
for name in ['gnu-verification.json','gnu-decoder-transcript.json']:
    path=prior/'gnu-graph'/(name+'.gz')
    packed=path.read_bytes()
    raw=gzip.decompress(packed)
    gnu_rows[name]=dict(snapshot='../stage-3-pipe-loops/gnu-graph/'+name+'.gz',
        sha256=digest(raw),bytes=len(raw),snapshot_sha256=digest(packed))
write('gnu-identity.json',gnu_rows)
controls=json.loads((cache/'file-input-controls/controls.json').read_text())
assert controls['passed'] and len(controls['controls'])==21
save('controls/controls.json',(cache/'file-input-controls/controls.json').read_bytes())
save('controls/positive.log',(cache/'file-input-controls/positive.log').read_bytes())
for row in controls['controls']:
    raw=(cache/'file-input-controls'/row['name']/'rejection.log').read_bytes()
    assert digest(raw)==row['log_sha256']
    save('controls/'+row['name']+'.log',raw)
for filename in ['storage-cases.su','source-component.json']:
    zipped('fixture/'+filename+'.gz',(builds['storage']/filename).read_bytes())
for name in ['archive-file-inputs.py','compare-file-input-artifacts.py','readback-file-input-native.py',
             'check-file-input-controls.py','readback-file-input-gnu.py','run-staging-focused-tests.py']:
    save(name,(cache/name).read_bytes())
for folder,label in [('file-input-wipe-prefix-failure','static-wipe-prefix'),
                     ('file-input-gnu-readback','gnu-comment-parser')]:
    for path in (cache/folder).iterdir():
        assert path.is_file()
        save('history/'+label+'/'+path.name,path.read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001','TOPAL-SYSTEMS-POSITIONAL-FILE-TRANSFER-001',
    'TOPAL-REQ-SYSTEMS-USER-EXTENT-001','TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High: conditional user input interval, callee effects, stack/nonalias and evidence accuracy; executable kernel, authority meaning and core language unchanged; no owned production activation',
    review='Single-agent higher-authority/source/full-diff review, saved independent GNU RTC effects/frame/cuts, twenty-one offline controls, 100 models/readers, three fresh native regressions and two fixture build tests',
    selected_inputs=155,changed_inputs_stable_in_all_four_commands=9,model_reader_tests=100,
    source_syscalls=248,input_admissions=40,input_completions=16,input_refusals=24,input_bytes=1136,
    resident_missing_variants=4,user_stack_bound_bytes=1320,user_stack_limit_bytes=2048,
    selected_clock_stack_bytes=112,retired_clock_spill_bytes=96,retired_spill_erasure_verified=False,
    conditional_sites=88,user_access_instructions=316,remaining_sites=3,
    kernel_sha256=kernel_sha,executable_kernel_changed=False,
    failures=['history/static-wipe-prefix/failure.json','history/gnu-comment-parser/failure.json'],
    safeguard_notices=[],core_language_or_authority_meaning_changed=False,
    remaining=['Actual owners/nonaliasing and continuous lifetime',
        'Retired clock spill cleanup and complete helper/entry/callee/return/fatal/exception/post-copy effects',
        'Three sites, fallback obligations, further native profiles and canonical clean production reconstruction',
        'Complete original Linux-compatible kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=len(selected),user_access_instructions=len(accesses),fresh_union=len(union),same_kernel_union=len(combined))))
