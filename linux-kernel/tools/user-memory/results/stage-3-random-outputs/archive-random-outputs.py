"""Freeze bounded-random evidence and exact historical model-test versions."""
import gzip
import hashlib
import json
from pathlib import Path

cache=Path('linux-kernel/.cache/x86_64')
config=json.loads((cache/'random-output-runs.json').read_text())
builds={k:cache/v for k,v in config['builds'].items()}
runs={k:cache/v for k,v in config['runs'].items()}
root=Path('linux-kernel/tools/user-memory/results/stage-3-random-outputs')
prior=root.parent/'stage-3-warning-inputs'
digest=lambda raw:hashlib.sha256(raw).hexdigest()
assert set(builds)=={'source','pinned','storage'} and set(runs)=={'source','pinned','storage','fixtures'}
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
assert len(frozen)==8 and not root.exists()
model_path='linux-kernel/tools/user-memory/test_random_observer.py'
recorded_model=(cache/'random-table-model-namespace-failure/test_random_observer.py').read_bytes()
assert digest(recorded_model)==frozen[model_path]
assert Path(model_path).read_bytes()!=recorded_model
root.mkdir()
def save(name,raw):
    path=root/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(raw)
def write(name,value): save(name,(json.dumps(value,indent=2)+'\n').encode())
def zipped(name,raw):
    packed=gzip.compress(raw,mtime=0)
    save(name,packed)
    return dict(snapshot=name,sha256=digest(raw),bytes=len(raw),snapshot_sha256=digest(packed))
old_inputs=(prior/'source-identity.json').read_bytes()
inputs={name:dict(row,snapshot='../stage-3-warning-inputs/'+row['snapshot'])
    for name,row in json.loads(old_inputs)['selected_inputs'].items()}
for name,sha in frozen.items():
    raw=Path(name).read_bytes()
    if name!=model_path: assert digest(raw)==sha
    inputs[name]=zipped('inputs/'+name+'.gz',raw)
assert len(inputs)==164
old_model_row=zipped('history/model-namespace/native-recorded-test.py.gz',recorded_model)
write('source-identity.json',dict(base_commit='d57eb98e4460b30bdb794fdac839690e90d4b418',
    inherited_record='../stage-3-warning-inputs/source-identity.json',inherited_record_sha256=digest(old_inputs),
    selected_inputs=inputs,nonexecuted_model_input_in_native_commands=dict(path=model_path,**old_model_row),
    model_repair='Only the unittest mock namespace gains struct=struct after the native commands. The Rust native targets execute explicit audit/observer scripts and do not discover this Python test module; its recorded old version remains separately hash-bound.',
    scope='Selected frozen sources and one explicit nonexecuted model-test version; not a complete clean producer or production qualification'))
old_artifacts=json.loads((prior/'artifact-identity.json').read_text())
artifacts={name:dict(row,snapshot='../stage-3-warning-inputs/'+row['snapshot']) for name,row in old_artifacts.items()}
for name,row in artifacts.items():
    raw=gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(raw)==row['sha256']
    if name not in ['source-user.elf','before/source-user.elf']:
        assert (builds['storage']/name).read_bytes()==raw
artifacts['before/source-user.elf']=dict(artifacts['source-user.elf'])
artifacts['source-user.elf']=zipped('artifacts/source-user.elf.gz',(builds['storage']/'source-user.elf').read_bytes())
artifacts['random-output-audit.json']=zipped('artifacts/random-output-audit.json.gz',
    (builds['storage']/'random-output-audit.json').read_bytes())
assert len(artifacts)==36
write('artifact-identity.json',artifacts)
kernel_sha=artifacts['kernel.elf']['sha256']
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),
    ('pipe-loop-audit.json','user_accesses'),('file-input-audit.json','copy_offset'),
    ('warning-input-audit.json','user_accesses'),('random-output-audit.json','user_accesses')]
selected,accesses={},set()
for name,field in fields:
    report=json.loads((builds['storage']/name).read_text())
    assert report['passed'] and report['kernel_sha256']==kernel_sha
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']]=dict(name=row['name'],call_offset=row['call_offset'],audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field=='copy_offset' else [r['offset'] for r in row[field]])
census=json.loads((builds['storage']/'extent-call-inventory.json').read_text())
remaining=[r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining))==(90,325,1)
write('caller-access-index.json',dict(complete=False,kernel_sha256=kernel_sha,emitted_helper_calls=91,
    calls_with_selected_local_intervals=90,distinct_user_access_instructions=325,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Twelve same-kernel conditional local reports; not complete owners/lifetime/callers/fallbacks or production qualification'))
for label,path in runs.items():
    save('native/'+label+'/run.json',(path/'run.json').read_bytes())
    zipped('native/'+label+'/test.log.gz',(path/'test.log').read_bytes())
coverage={}
identities={}
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
    identities[label]={}
    for name in artifacts:
        if name.startswith('before/') or not (build/name).exists(): continue
        raw=(build/name).read_bytes()
        identities[label][name]=dict(sha256=digest(raw),bytes=len(raw))
        if name not in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
            assert digest(raw)==artifacts[name]['sha256']
write('native-input-identities.json',identities)
old_coverage=(prior/'native-coverage.json').read_bytes()
previous=json.loads(old_coverage)
assert previous['kernel_sha256']==kernel_sha
union=set().union(*coverage.values())
combined=union|set(previous['reached'])
write('native-coverage.json',dict(passed=True,kernel_sha256=kernel_sha,
    fresh_first_reach_sites={k:len(v) for k,v in coverage.items()},fresh_matching_kernel_union=len(union),
    fresh_reached=sorted(union),inherited_record='../stage-3-warning-inputs/native-coverage.json',
    inherited_record_sha256=digest(old_coverage),matching_kernel_union=len(combined),reached=sorted(combined),
    unreached=[r for r in census['calls'] if r['offset'] not in combined],
    scope='Actual fresh first reach plus explicitly bound preceding same-kernel union; native byte-loop records do not manufacture first-reach or complete caller qualification'))
core=['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
    'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o',
    'predecessor-gateway.o','provider.o','kernel.elf','bzImage']
metadata=['extent-call-inventory.json','process-access-inventory.json','process-access-origins.json',
    'process-byte-origins.json']+[name for name,_ in fields[:-1]]
baseline=cache/'user-extent-source-storage-2386238-1794456983427127007'
comparisons={}
for label,build in builds.items():
    comparisons[label]=[]
    for name in core+metadata:
        raw=(build/name).read_bytes()
        assert raw==(baseline/name).read_bytes()
        comparisons[label].append(dict(path=name,sha256=digest(raw),bytes=len(raw)))
write('artifact-comparison.json',dict(passed=True,baseline_build=str(baseline),
    core_objects_images_per_cohort=17,metadata_reports_per_cohort=15,comparisons=comparisons,
    other_inherited_storage_artifacts_equal=33,
    scope='Actual unchanged core/metadata bytes in three fresh cohorts, all other inherited storage artifacts and a separately recorded intermediate comparison; not a complete independent clean producer'))
save('native-readback.json',(cache/'random-output-native-readback-final/readback.json').read_bytes())
save('gnu-operands.json',(cache/'random-output-gnu-readback/readback.json').read_bytes())
save('source-intermediate-comparison.json',(cache/'random-source-intermediate-comparison/comparison.json').read_bytes())
write('gnu-identity.json',{name:dict(row,snapshot='../stage-3-warning-inputs/'+row['snapshot'])
    for name,row in json.loads((prior/'gnu-identity.json').read_text()).items()})
controls=json.loads((cache/'random-output-controls/controls.json').read_text())
assert controls['passed'] and len(controls['controls'])==23
save('controls/controls.json',(cache/'random-output-controls/controls.json').read_bytes())
save('controls/positive.log',(cache/'random-output-controls/positive.log').read_bytes())
for row in controls['controls']:
    raw=(cache/'random-output-controls'/row['name']/'rejection.log').read_bytes()
    assert digest(raw)==row['log_sha256']
    save('controls/'+row['name']+'.log',raw)
for name in ['archive-random-outputs.py','readback-random-output-native.py','readback-random-output-gnu.py',
             'check-random-output-controls.py','compare-random-source-intermediate.py',
             'run-staging-focused-tests.py','run-random-storage-diagnostic.py','random-output-runs.json']:
    save(name,(cache/name).read_bytes())
for folder in ['random-first-model-failure','random-table-model-namespace-failure','random-storage-observer-first-failure']:
    for path in (cache/folder).iterdir():
        if path.is_file(): save('history/'+folder+'/'+path.name,path.read_bytes())
diagnostic=cache/'random-storage-mapping-diagnostic'
save('history/mapping-diagnostic/run.json',(diagnostic/'run.json').read_bytes())
zipped('history/mapping-diagnostic/diagnostic.log.gz',(diagnostic/'diagnostic.log').read_bytes())
for path in (diagnostic/'build/source-user-extents').iterdir():
    if path.is_file() and path.suffix in ['.json','.log','.gdb']:
        zipped('history/mapping-diagnostic/observation/'+path.name+'.gz',path.read_bytes())
failed_build=cache/'user-extent-source-storage-2390370-1794459181759936035/source-user-extents'
for path in failed_build.iterdir():
    if path.is_file() and path.suffix in ['.json','.log','.gdb']:
        zipped('history/random-storage-observer-first-failure/observation/'+path.name+'.gz',path.read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001','TOPAL-REQ-SYSTEMS-USER-EXTENT-001',
    'TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High: suspended-stack shape, extraction scratch retirement, user output and evidence accuracy. Executable kernel, core language, authority meaning and canonical production selection unchanged.',
    review='Single-agent higher-authority/source/full-diff review, complete selected extractor effects/frame dataflow, separate saved GNU effects/cuts, 23 offline controls, 121 model/reader tests, three native regressions and two fixture tests. The isolated model mock repair is explicitly versioned after native commands.',
    selected_inputs=164,native_recorded_inputs=8,native_nonexecuted_model_repair=1,model_reader_tests=121,
    source_admissions=48,source_refusals=40,source_completions=8,source_bytes=64,source_early_results=24,
    native_lengths=[8],extract_instructions=1843,extract_scratch_bytes=128,selected_switched_stack_bytes=168,
    retired_random_outer_spill_bytes=32,retired_random_outer_spill_erasure_verified=False,
    conditional_sites=90,user_access_instructions=325,remaining_sites=1,
    kernel_sha256=kernel_sha,executable_kernel_changed=False,core_language_or_authority_meaning_changed=False,
    failures=['history/random-first-model-failure/failure.json','history/random-table-model-namespace-failure/failure.json',
        'history/random-storage-observer-first-failure/failure.json'],
    diagnostic_exit_code=1,safeguard_notices=[],retired_clock_spill_erasure_verified=False,
    remaining=['Actual key/output/stack/user owners, nonalias and continuous entry/mapping lifetime',
        'Complete shared helper/entry/callee/return/fatal/exception and whole-syscall retirement',
        'Unrun native random lengths, extraction scratch failure paths and outer spill/return-word cleanup',
        'One site, three fallback obligations, clock spills, further profiles and canonical clean production qualification',
        'Complete original Linux-compatible kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=len(selected),user_access_instructions=len(accesses),fresh_union=len(union),same_kernel_union=len(combined))))
