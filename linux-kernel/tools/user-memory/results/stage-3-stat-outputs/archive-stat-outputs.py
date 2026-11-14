"""Freeze selected stat-output readers, native runs and their retained failures."""
import gzip
import hashlib
import json
from pathlib import Path

cache = Path('linux-kernel/.cache/x86_64')
root = Path('linux-kernel/tools/user-memory/results/stage-3-stat-outputs')
prior = root.parent/'stage-3-fixed-pipe-native'
catalog = json.loads((cache/'stat-output-catalog.json').read_text())
builds = dict(stat=cache/catalog['native_stat']['build'],source=cache/catalog['native_boundary']['build'],
              pinned=cache/'user-extent-application-2374272-1794449800958638246')
runs = dict(stat=cache/catalog['native_stat']['run'],source=cache/catalog['native_boundary']['run'],
            pinned=cache/'tk283-staging-focused-1794449799985015269',fixtures=cache/catalog['fixture_builds']['run'])
digest = lambda data:hashlib.sha256(data).hexdigest()
frozen = None
for label,run in runs.items():
    record = json.loads((run/'run.json').read_text())
    assert record['exit_code'] == 0 and not record['source_changes_during_run']
    assert record['modified_source_hashes_before'] == record['modified_source_hashes_after']
    assert record['resources']['memory.swap.max'] == '0'
    assert all(int(line.split()[1]) == 0 for line in record['resources']['memory.events'].splitlines())
    if frozen is None: frozen = record['modified_source_hashes_before']
    assert record['modified_source_hashes_before'] == frozen
assert len(frozen) == 13
assert not root.exists()
root.mkdir()
def save(name,data):
    path = root/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)
def write(name,value): save(name,(json.dumps(value,indent=2)+'\n').encode())
def zipped(name,data):
    packed = gzip.compress(data,mtime=0)
    save(name,packed)
    return dict(snapshot=name,sha256=digest(data),bytes=len(data),snapshot_sha256=digest(packed))
old_bytes = (prior/'source-identity.json').read_bytes()
inputs = {}
for name,row in json.loads(old_bytes)['selected_inputs'].items():
    data = gzip.decompress((prior/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    inputs[name] = dict(sha256=row['sha256'],snapshot='../stage-3-fixed-pipe-native/'+row['snapshot'])
for name,sha in frozen.items():
    data = Path(name).read_bytes()
    assert digest(data) == sha
    inputs[name] = zipped('inputs/'+name+'.gz',data)
assert len(inputs) == 147
write('source-identity.json',dict(base_commit='7151fa1e21144f0dd1d01229cb3d208e90447efc',
    inherited_record='../stage-3-fixed-pipe-native/source-identity.json',
    inherited_record_sha256=digest(old_bytes),selected_inputs=inputs,
    scope='Selected frozen inputs only; not a complete clean producer or canonical reconstruction'))
old_artifacts = json.loads((prior/'artifact-identity.json').read_text())
fields = [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset')]
artifacts = {}
for name in ['provider.o','kernel.elf','extent-call-inventory.json','process-access-inventory.json',
             'process-access-origins.json']+[name for name,_ in fields[:-1]]:
    row = old_artifacts[name]
    assert digest((builds['stat']/name).read_bytes()) == row['sha256']
    artifacts[name] = dict(row,snapshot='../stage-3-fixed-pipe-native/'+row['snapshot'])
for name in ['stat-output-audit.json','static-elf-abi-fixture.elf','initramfs.cpio','source-user.elf']:
    artifacts[name] = zipped('artifacts/'+name+'.gz',(builds['stat']/name).read_bytes())
origin_record = json.loads((root.parent/'stage-3-staging-copies/artifact-identity.json').read_text())['process-byte-origins.json']
assert digest((builds['stat']/'process-byte-origins.json').read_bytes()) == origin_record['sha256']
artifacts['process-byte-origins.json'] = dict(origin_record,
    snapshot='../stage-3-staging-copies/'+origin_record['snapshot'])
write('artifact-identity.json',artifacts)
selected,accesses = {},set()
for name,field in fields:
    report = json.loads((builds['stat']/name).read_text())
    assert report['passed'] and report['kernel_sha256'] == catalog['kernel_unchanged_sha256']
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']] = dict(name=row['name'],call_offset=row['call_offset'],audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field == 'copy_offset' else [r['offset'] for r in row[field]])
census = json.loads((builds['stat']/'extent-call-inventory.json').read_text())
remaining = [r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining)) == (83,310,8)
write('caller-access-index.json',dict(complete=False,kernel_sha256=catalog['kernel_unchanged_sha256'],
    emitted_helper_calls=91,calls_with_selected_local_intervals=83,distinct_user_access_instructions=310,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Seven matching-kernel conditional local reports only; not complete callers, owners/nonalias/lifetime/helper/entry/callee/exception/fallback or production qualification'))
for label,run in runs.items():
    save('native/'+label+'/run.json',(run/'run.json').read_bytes())
    zipped('native/'+label+'/test.log.gz',(run/'test.log').read_bytes())
for label,build in builds.items():
    folders = [p for p in build.iterdir() if p.is_dir() and (p/'observations.json').is_file()]
    assert len(folders) == 1,(label,folders)
    for path in folders[0].iterdir():
        if path.is_file() and path.suffix in ['.json','.log','.gdb']:
            zipped('native/'+label+'/observation/'+path.name+'.gz',path.read_bytes())
    if (build/'source-component.json').is_file():
        zipped('native/'+label+'/source-component.json.gz',(build/'source-component.json').read_bytes())
    if label != 'stat':
        write('native/'+label+'/input-identities.json',{name:dict(bytes=(build/name).stat().st_size,
            sha256=digest((build/name).read_bytes())) for name in ['kernel.elf','provider.o',
            'static-elf-abi-fixture.elf','initramfs.cpio','stat-output-audit.json']})
save('native-readback.json',(cache/'stat-native-readback/readback.json').read_bytes())
save('artifact-comparison.json',(cache/'stat-artifact-comparison.json').read_bytes())
gnu = cache/'stat-gnu-readback-v2'
assert json.loads((gnu/'readback.json').read_text())['passed']
for path in gnu.iterdir():
    if path.is_file(): save('gnu/'+path.name,path.read_bytes())
controls = cache/'stat-output-controls'
control_record = json.loads((controls/'controls.json').read_text())
assert control_record['passed'] and len(control_record['controls']) == 14
save('controls/controls.json',(controls/'controls.json').read_bytes())
save('controls/positive.log',(controls/'positive.log').read_bytes())
for row in control_record['controls']:
    data = (controls/row['name']/'rejection.log').read_bytes()
    assert digest(data) == row['log_sha256']
    save('controls/'+row['name']+'.log',data)
for label,folder in [('first-reach-failure','stat-first-reach-failure-inputs'),
                     ('first-stat-pass','stat-native-first-pass-inputs')]:
    directory = cache/folder
    for path in directory.rglob('*'):
        if path.is_file(): save('history/'+label+'/'+str(path.relative_to(directory)),path.read_bytes())
failed_run = cache/catalog['native_first_stat_failure']['run']
save('history/first-reach-failure/run.json',(failed_run/'run.json').read_bytes())
zipped('history/first-reach-failure/test.log.gz',(failed_run/'test.log').read_bytes())
failed_build = cache/catalog['native_first_stat_failure']['build']
for name in ['observations.json','gdb.stderr.log','gdb.stdout.log','inspect.gdb']:
    zipped('history/first-reach-failure/'+name+'.gz',(failed_build/'source-user-extents'/name).read_bytes())
first_run = cache/catalog['native_first_stat_pass']['run']
save('history/first-stat-pass/run.json',(first_run/'run.json').read_bytes())
zipped('history/first-stat-pass/test.log.gz',(first_run/'test.log').read_bytes())
save('history/gnu-gs-prelude-failure-reader.py',(cache/'stat-gnu-readback/failed-reader.py').read_bytes())
for name in ['archive-stat-outputs.py','readback-stat-native.py','readback-stat-outputs.py',
             'check-stat-output-controls.py','preserve-stat-native-failure.py','run-staging-focused-tests.py']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-REQ-SYSTEMS-USER-EXTENT-001','TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High if conditional effect/normal-return observations are mistaken for complete ownership/lifetime or kernel qualification; kernel and authoritative design bytes are unchanged',
    review='Single-agent authority/source review, raw effect/frame counterexamples, separate GNU boundaries/effects/stack-order and reverse cut, fourteen offline ELF controls, actual ordinary stat/old-source/pinned-application execution and saved-source/native correspondence',
    model_reader_tests=82,new_stat_model_controls=11,coincident_first_reach_control=True,
    selected_inputs=147,changed_inputs_stable_in_each_final_run=13,
    static_stat_regions=2,shared_stat_copy_instructions=1,metadata_validator_instructions=1140,
    metadata_spill_bytes=64,maximum_nested_callee_stack_bytes=96,
    native_stat_syscalls=152,stat_queries=88,completed_stat_copies=60,refused_stat_output_admissions=20,
    lookup_errors_without_registered_output_events=8,stat_variants=8,
    conditional_index_sites=83,conditional_index_access_instructions=310,remaining_sites=8,
    failures=['Initial stat run failed on duplicate first-reach bookkeeping from two coincident read-only observers; exact captured inputs and failure retained',
        'Initial effect decoder rejected reviewed SUB opcode 4c29e0; register ADD/SUB effects were reviewed and tested before native validation',
        'Initial GNU projection applied the stat body decoder to the captured-GS prelude; the failed reader is retained and the exact reviewed GS instruction is checked separately'],
    earlier_positive_scope='The 34-call-per-execution stat run is retained separately; the final fixture also queries stdin and a directory descriptor',
    safeguard_notices=[],kernel_production_unchanged=True,core_language_or_authority_meaning_changed=False,
    remaining=['Actual private/table/stack/backing/output ownership and nonaliasing',
        'Continuous lifetime and full helper/entry/callee/return/fatal/exception guarantees',
        'Eight sites without intervals and all fallback obligations',
        'Canonical clean source/native/boot reconstruction and full original kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=83,access_instructions=310,remaining_sites=8)))
