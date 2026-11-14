"""Freeze selected serial-prefix inputs and actual native evidence."""
import gzip
import hashlib
import json
from pathlib import Path

cache = Path('linux-kernel/.cache/x86_64')
root = Path('linux-kernel/tools/user-memory/results/stage-3-serial-prefix')
prior = root.parent/'stage-3-stat-outputs'
builds = dict(serial=cache/'user-extent-source-serial-2376795-1794451786977016215',
    pinned=cache/'user-extent-application-2377495-1794452145272725263')
runs = dict(serial=cache/'tk283-staging-focused-1794451780580672837',
    pinned=cache/'tk283-staging-focused-1794452144124842982')
digest = lambda data:hashlib.sha256(data).hexdigest()
frozen = None
for label,path in runs.items():
    run = json.loads((path/'run.json').read_text())
    assert run['exit_code'] == 0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before'] == run['modified_source_hashes_after']
    assert run['resources']['memory.swap.max'] == '0'
    assert all(int(line.split()[1]) == 0 for line in run['resources']['memory.events'].splitlines())
    assert b'running 1 test' in (path/'test.log').read_bytes()
    if frozen is None: frozen = run['modified_source_hashes_before']
    assert frozen == run['modified_source_hashes_before']
assert len(frozen) == 5
assert not root.exists()
root.mkdir()
def save(name,data):
    path = root/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)
def write(name,data): save(name,(json.dumps(data,indent=2)+'\n').encode())
def zipped(name,data):
    packed = gzip.compress(data,mtime=0)
    save(name,packed)
    return dict(snapshot=name,sha256=digest(data),snapshot_sha256=digest(packed),bytes=len(data))
old_bytes = (prior/'source-identity.json').read_bytes()
inputs = {name:dict(row,snapshot='../stage-3-stat-outputs/'+row['snapshot'])
    for name,row in json.loads(old_bytes)['selected_inputs'].items()}
for name,sha in frozen.items():
    data = Path(name).read_bytes()
    assert digest(data) == sha
    inputs[name] = zipped('inputs/'+name+'.gz',data)
assert len(inputs) == 149
write('source-identity.json',dict(base_commit='18b698ebf756b77d489d7873d3d2d3e556c28273',
    inherited_record='../stage-3-stat-outputs/source-identity.json',inherited_record_sha256=digest(old_bytes),
    selected_inputs=inputs,scope='Selected frozen sources only; not complete clean producer or production reconstruction'))
artifacts = {name:dict(row,snapshot='../stage-3-stat-outputs/'+row['snapshot'])
    for name,row in json.loads((prior/'artifact-identity.json').read_text()).items()}
for name,row in artifacts.items():
    if name not in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
        assert digest((builds['serial']/name).read_bytes()) == row['sha256']
for name in ['serial-prefix-audit.json','static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
    artifacts[name] = zipped('artifacts/'+name+'.gz',(builds['serial']/name).read_bytes())
assert len(artifacts) == 17
write('artifact-identity.json',artifacts)
fields = [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset')]
selected,accesses = {},set()
for name,field in fields:
    report = json.loads((builds['serial']/name).read_text())
    assert report['passed'] and report['kernel_sha256'] == artifacts['kernel.elf']['sha256']
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']] = dict(call_offset=row['call_offset'],name=row['name'],audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field == 'copy_offset' else [r['offset'] for r in row[field]])
census = json.loads((builds['serial']/'extent-call-inventory.json').read_text())
remaining = [r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining)) == (84,311,7)
write('caller-access-index.json',dict(complete=False,kernel_sha256=artifacts['kernel.elf']['sha256'],
    emitted_helper_calls=91,calls_with_selected_local_intervals=84,distinct_user_access_instructions=311,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Eight matching-kernel conditional local reports; complete owners/lifetime/callers/fallbacks and production remain open'))
baseline = cache/'user-extent-source-stat-2373431-1794449036311016637'
names = ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
    'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o',
    'predecessor-gateway.o','provider.o','kernel.elf','bzImage']
comparisons = {}
coverage = {}
for label,build in builds.items():
    save('native/'+label+'/run.json',(runs[label]/'run.json').read_bytes())
    zipped('native/'+label+'/test.log.gz',(runs[label]/'test.log').read_bytes())
    folders = [p for p in build.iterdir() if p.is_dir() and (p/'observations.json').is_file()]
    assert len(folders) == 1
    for path in folders[0].iterdir():
        if path.is_file() and path.suffix in ['.json','.log','.gdb']:
            zipped('native/'+label+'/observation/'+path.name+'.gz',path.read_bytes())
    comparisons[label] = []
    for name in names:
        data = (build/name).read_bytes()
        assert data == (baseline/name).read_bytes()
        comparisons[label].append(dict(path=name,sha256=digest(data),bytes=len(data)))
    obs = json.loads((folders[0]/'observations.json').read_text())
    assert obs['passed']
    coverage[label] = {r['offset'] for r in obs['extent_call_coverage']['reached']}
    if label == 'serial':
        zipped('native/serial/source-component.json.gz',(build/'source-component.json').read_bytes())
    else:
        write('native/pinned/input-identities.json',{name:dict(sha256=digest((build/name).read_bytes()),
            bytes=(build/name).stat().st_size) for name in ['kernel.elf','provider.o','static-elf-abi-fixture.elf','initramfs.cpio','serial-prefix-audit.json']})
write('artifact-comparison.json',dict(passed=True,baseline_build=str(baseline),kernel_artifact_comparisons=comparisons))
old_coverage = root.parent/'stage-3-staging-copies/native-summary.json'
old_raw = old_coverage.read_bytes()
union = set(json.loads(old_raw)['reached'])
assert len(union) == 88 and all(s <= union for s in coverage.values())
write('native-coverage.json',dict(passed=True,matching_kernel_union=88,
    fresh_first_reach_sites={k:len(v) for k,v in coverage.items()},
    preceding_record='../stage-3-staging-copies/native-summary.json',preceding_record_sha256=digest(old_raw),
    scope='Matching-kernel first-reach subsets only; not full transfer or production qualification'))
save('native-readback.json',(cache/'serial-prefix-native-readback/readback.json').read_bytes())
for label,directory in [('gnu','serial-prefix-gnu-readback')]:
    for path in (cache/directory).iterdir():
        if path.is_file(): save(label+'/'+path.name,path.read_bytes())
controls = json.loads((cache/'serial-prefix-controls-v2/controls.json').read_text())
assert controls['passed'] and len(controls['controls']) == 12
save('controls/controls.json',(cache/'serial-prefix-controls-v2/controls.json').read_bytes())
save('controls/positive.log',(cache/'serial-prefix-controls-v2/positive.log').read_bytes())
for row in controls['controls']:
    data = (cache/'serial-prefix-controls-v2'/row['name']/'rejection.log').read_bytes()
    assert digest(data) == row['log_sha256']
    save('controls/'+row['name']+'.log',data)
save('history/failed-control-runner.py',(cache/'serial-prefix-controls/failed-runner.py').read_bytes())
for label,run_name in [('zero-selected-tests','tk283-staging-focused-1794452046200474333'),
                       ('missing-pinned-image','tk283-staging-focused-1794452079007490619')]:
    run = cache/run_name
    save('history/'+label+'/run.json',(run/'run.json').read_bytes())
    zipped('history/'+label+'/test.log.gz',(run/'test.log').read_bytes())
for name in ['archive-serial-prefix.py','readback-serial-prefix.py','readback-serial-prefix-native.py',
             'check-serial-prefix-controls.py','run-staging-focused-tests.py']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-REQ-SYSTEMS-USER-EXTENT-001','TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High if selected interval evidence is mistaken for complete user-memory or production qualification; kernel bytes and authoritative meaning unchanged',
    review='Single-agent authority/source/full-diff review, 88 model/reader tests, separate GNU operands/reverse cuts, twelve offline ELF controls, actual serial and unchanged pinned regressions, saved UART/syscall/prefix count reconstruction',
    selected_inputs=149,changed_inputs_stable_in_native_runs=5,model_reader_tests=88,
    gnu_instructions=56,gnu_windows=2,offline_controls=12,source_syscalls=64,
    prefix_admissions=28,prefix_completed=12,prefix_refused=16,prefix_bytes=64,short_prefixes=4,
    conditional_sites=84,user_access_instructions=311,remaining_sites=7,
    failures=['First offline controls stopped at an incorrectly selected MOV byte before running that control; corrected runner and all twelve controls pass',
        'A misspelled Rust selector ran zero tests; its nominal exit zero is not validation, and the focused runner now rejects zero selections',
        'The correct pinned regression first failed before VM execution because TOPAL_PINNED_STATIC_USER_IMAGE was omitted; the corrected run uses the unchanged verified pinned digest'],
    safeguard_notices=[],core_language_or_authority_meaning_changed=False,kernel_production_unchanged=True,
    remaining=['Actual owners/nonaliasing and uninterrupted check-to-copy lifetime',
        'Seven remaining sites and all helper/entry/callee/return/fatal/exception/fallback obligations',
        'Canonical production and complete original Linux-compatible kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),sites=84,access_instructions=311)))
