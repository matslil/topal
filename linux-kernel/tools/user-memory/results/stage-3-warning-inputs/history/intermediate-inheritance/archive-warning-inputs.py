"""Freeze variant-sensitive warning input evidence on an unchanged kernel."""
import gzip
import hashlib
import json
from pathlib import Path

cache=Path('linux-kernel/.cache/x86_64')
config=json.loads((cache/'warning-input-runs.json').read_text())
builds={k:cache/v for k,v in config['builds'].items()}
runs={k:cache/v for k,v in config['runs'].items()}
root=Path('linux-kernel/tools/user-memory/results/stage-3-warning-inputs')
prior=root.parent/'stage-3-file-inputs'
digest=lambda raw:hashlib.sha256(raw).hexdigest()
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
assert len(frozen)==5 and not root.exists()
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
inputs={name:dict(row,snapshot='../stage-3-file-inputs/'+row['snapshot'])
    for name,row in json.loads(old_inputs)['selected_inputs'].items()}
for name,sha in frozen.items():
    raw=Path(name).read_bytes()
    assert digest(raw)==sha
    inputs[name]=zipped('inputs/'+name+'.gz',raw)
assert len(inputs)==158
write('source-identity.json',dict(base_commit='d0987eeba04ffa18e026de9043c484721ddbf911',
    inherited_record='../stage-3-file-inputs/source-identity.json',inherited_record_sha256=digest(old_inputs),
    selected_inputs=inputs,scope='Selected frozen sources only; not a complete clean producer or production qualification'))
old_artifacts=json.loads((prior/'artifact-identity.json').read_text())
artifacts={name:dict(row,snapshot='../stage-3-file-inputs/'+row['snapshot']) for name,row in old_artifacts.items()}
for name,row in artifacts.items():
    raw=gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(raw)==row['sha256'] and (builds['storage']/name).read_bytes()==raw
artifacts['warning-input-audit.json']=zipped('artifacts/warning-input-audit.json.gz',
    (builds['storage']/'warning-input-audit.json').read_bytes())
assert len(artifacts)==34
write('artifact-identity.json',artifacts)
kernel_sha=artifacts['kernel.elf']['sha256']
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),
    ('pipe-loop-audit.json','user_accesses'),('file-input-audit.json','copy_offset'),
    ('warning-input-audit.json','user_accesses')]
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
assert (len(selected),len(accesses),len(remaining))==(89,323,2)
write('caller-access-index.json',dict(complete=False,kernel_sha256=kernel_sha,emitted_helper_calls=91,
    calls_with_selected_local_intervals=89,distinct_user_access_instructions=323,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Eleven same-kernel conditional local reports; not complete owners/lifetime/callers/fallbacks or production qualification'))
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
        if not (build/name).exists(): continue
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
    fresh_reached=sorted(union),inherited_record='../stage-3-file-inputs/native-coverage.json',
    inherited_record_sha256=digest(old_coverage),matching_kernel_union=len(combined),reached=sorted(combined),
    unreached=[r for r in census['calls'] if r['offset'] not in combined],
    scope='Actual fresh first-reach records plus explicitly bound preceding same-kernel union; warning staging observations do not manufacture first-reach records or complete caller qualification'))
core=['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
    'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o',
    'predecessor-gateway.o','provider.o','kernel.elf','bzImage']
metadata=['extent-call-inventory.json','process-access-inventory.json','process-access-origins.json',
    'process-byte-origins.json']+[name for name,_ in fields[:-1]]
baseline=cache/'user-extent-source-storage-2382778-1794455130144072641'
comparisons={}
for label,build in builds.items():
    comparisons[label]=[]
    for name in core+metadata:
        raw=(build/name).read_bytes()
        assert raw==(baseline/name).read_bytes()
        comparisons[label].append(dict(path=name,sha256=digest(raw),bytes=len(raw)))
write('artifact-comparison.json',dict(passed=True,baseline_build=str(baseline),
    core_objects_images_per_cohort=17,metadata_reports_per_cohort=14,comparisons=comparisons,
    all_storage_fixture_artifacts_equal=True,
    scope='Actual unchanged core/metadata bytes in three fresh cohorts and all inherited storage fixture artifacts; not a complete independent clean producer'))
for src,dest in [('warning-input-native-readback/readback.json','native-readback.json'),
                 ('warning-input-gnu-readback/readback.json','gnu-operands.json')]: save(dest,(cache/src).read_bytes())
write('gnu-identity.json',{name:dict(row,snapshot='../stage-3-file-inputs/'+row['snapshot'])
    for name,row in json.loads((prior/'gnu-identity.json').read_text()).items()})
controls=json.loads((cache/'warning-input-controls/controls.json').read_text())
assert controls['passed'] and len(controls['controls'])==15
save('controls/controls.json',(cache/'warning-input-controls/controls.json').read_bytes())
save('controls/positive.log',(cache/'warning-input-controls/positive.log').read_bytes())
for row in controls['controls']:
    raw=(cache/'warning-input-controls'/row['name']/'rejection.log').read_bytes()
    assert digest(raw)==row['log_sha256']
    save('controls/'+row['name']+'.log',raw)
for name in ['archive-warning-inputs.py','readback-warning-input-native.py','readback-warning-input-gnu.py',
             'check-warning-input-controls.py','run-staging-focused-tests.py','warning-input-runs.json']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001','TOPAL-REQ-SYSTEMS-USER-EXTENT-001',
    'TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High: selector-dependent admission and scalar bounds/evidence accuracy; executable kernel, core language, authority meaning and canonical production selection unchanged',
    review='Single-agent higher-authority/source/full-diff review, separate saved GNU selector/register/scalar/cut readback, fifteen offline controls, 104 model/reader tests, three native regressions and two fixture tests',
    selected_inputs=158,changed_inputs_stable_in_all_four_commands=5,model_reader_tests=104,
    warning_variants=3,storage_warning_admissions=3,storage_warning_bytes=45,storage_warning_scalar_loads=7,
    conditional_sites=89,user_access_instructions=323,remaining_sites=2,
    kernel_sha256=kernel_sha,executable_kernel_changed=False,core_language_or_authority_meaning_changed=False,
    failures=[],safeguard_notices=[],retired_clock_spill_erasure_verified=False,
    remaining=['Actual private/input owners, nonalias and continuous entry/mapping lifetime',
        'Shared helper RSI/RDX and retained selector preservation; complete output/staging/callee/entry/return/fatal/exception effects',
        'Two sites, fallback obligations, retired clock spills, further profiles and canonical clean production qualification',
        'Complete original Linux-compatible kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=len(selected),user_access_instructions=len(accesses),fresh_union=len(union),same_kernel_union=len(combined))))
