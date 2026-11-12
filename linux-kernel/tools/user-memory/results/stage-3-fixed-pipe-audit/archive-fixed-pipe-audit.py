"""Freeze selected fixed-pipe audit evidence after terminal validation."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import sys

root = Path('linux-kernel/tools/user-memory/results/stage-3-fixed-pipe-audit')
prior = root.parent/'stage-3-staging-copies'
cache = Path('linux-kernel/.cache/x86_64')
run = cache/'tk283-staging-focused-1794444745930453399'
build = cache/'user-extent-source-pipes-2367848-1794444753502668964'
baseline = cache/'user-extent-source-poll-2340765-1794433349203271869'
digest = lambda data: hashlib.sha256(data).hexdigest()
record = json.loads((run/'run.json').read_text())
assert record['exit_code'] == 0 and not record['source_changes_during_run']
assert record['modified_source_hashes_before'] == record['modified_source_hashes_after']
assert record['resources']['memory.swap.max'] == '0'
events = dict(line.split() for line in record['resources']['memory.events'].splitlines())
assert all(int(events[k]) == 0 for k in ['max','oom','oom_kill','oom_group_kill'])
assert 'test result: ok. 1 passed; 0 failed' in (run/'test.log').read_text()
assert not root.exists()
root.mkdir()

def save(name,data):
    target = root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
    return dict(sha256=digest(data),bytes=len(data))

def zipped(name,data):
    compressed = gzip.compress(data,mtime=0)
    save(name,compressed)
    return dict(snapshot=name,snapshot_sha256=digest(compressed),sha256=digest(data),bytes=len(data))

def write(name,value):
    save(name,(json.dumps(value,indent=2)+'\n').encode())

old_identity_bytes = (prior/'source-identity.json').read_bytes()
old_identity = json.loads(old_identity_bytes)
inputs = {}
for path,row in old_identity['selected_inputs'].items():
    data = gzip.decompress((prior/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    inputs[path] = dict(sha256=row['sha256'],snapshot='../stage-3-staging-copies/'+row['snapshot'])
for path,sha in record['modified_source_hashes_before'].items():
    data = Path(path).read_bytes()
    assert digest(data) == sha
    inputs[path] = zipped('inputs/'+path+'.gz',data)
assert set(record['modified_source_hashes_before']) == {
    'src/topal-compiler/src/systems_frame_root_tests.rs',
    'linux-kernel/tools/user-memory/audit_pipe_fixed_outputs.py',
    'linux-kernel/tools/user-memory/test_pipe_fixed_outputs.py'}
write('source-identity.json',dict(base_commit='d17ced5fcf80f38b392f98479105d85438a58d3e',
    inherited_record='../stage-3-staging-copies/source-identity.json',
    inherited_record_sha256=digest(old_identity_bytes),selected_inputs=inputs,
    scope='Selected snapshots only; not a complete clean producer or canonical reconstruction'))
artifacts = {}
old_artifacts = json.loads((prior/'artifact-identity.json').read_text())
for name in ['provider.o','kernel.elf','extent-call-inventory.json',
             'process-access-inventory.json','process-access-origins.json']:
    data = (build/name).read_bytes()
    old = old_artifacts[name]
    assert digest(data) == old['sha256'] and len(data) == old['bytes']
    assert data == (baseline/name).read_bytes()
    artifacts[name] = dict(old,snapshot='../stage-3-staging-copies/'+old['snapshot'])
write('artifact-identity.json',artifacts)
audit = (build/'pipe-fixed-output-audit.json').read_bytes()
assert audit == (baseline/'pipe-fixed-output-audit.json').read_bytes()
save('audit.json',audit)
save('native/run.json',(run/'run.json').read_bytes())
zipped('native/test.log.gz',(run/'test.log').read_bytes())
for path in build.glob('pipe-fixed-output-audit.*.json'):
    save('native/'+path.name,path.read_bytes())
for path in build.glob('pipe-fixed-output-audit.*.log'):
    save('native/'+path.name,path.read_bytes())
native_files = {}
for path in (build/'source-user-extents').iterdir():
    if path.name.endswith(('.json','.log','.gdb')):
        data = path.read_bytes()
        native_files[path.name] = zipped('native/observation/'+path.name+'.gz',data)
write('native/observation-identity.json',native_files)
for name in ['caller-observation.json','source-component.json']:
    if (build/name).is_file():
        zipped('native/'+name+'.gz',(build/name).read_bytes())
controls = cache/'fixed-pipe-controls-v2'
control_record = json.loads((controls/'controls.json').read_text())
assert control_record['passed'] and len(control_record['controls']) == 12
assert control_record['positive_report_sha256'] == digest(audit)
assert control_record['reader_sha256'] == digest(Path('linux-kernel/tools/user-memory/audit_pipe_fixed_outputs.py').read_bytes())
save('controls/controls.json',(controls/'controls.json').read_bytes())
save('controls/positive.log',(controls/'positive.log').read_bytes())
for row in control_record['controls']:
    log = (controls/row['name']/'rejection.log').read_bytes()
    assert digest(log) == row['log_sha256']
    save('controls/'+row['name']+'.log',log)
gnu = cache/'fixed-pipe-gnu-readback-v2'
gnu_record = json.loads((gnu/'readback.json').read_text())
assert gnu_record['passed'] and gnu_record['audit_sha256'] == digest(audit)
for path in gnu.iterdir():
    if path.is_file():
        save('gnu/'+path.name,path.read_bytes())
for name in ['check-fixed-pipe-controls.py','readback-fixed-pipe.py',
             'run-staging-focused-tests.py','archive-fixed-pipe-audit.py']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-REQ-SYSTEMS-USER-EXTENT-001','TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High if overstated as a complete safety proof; this change only adds conditional readers and test integration, not executable kernel behavior or authority design',
    review='Single-agent source/authority review with separate GNU instruction/effect and reverse-graph readback, positive-backed offline ELF counterexamples, eight graph/operand models and a fresh ordinary source-pipe VM regression',
    inherited_frozen_artifacts=len(artifacts),selected_inputs=len(inputs),local_source_snapshots=3,
    conditional_regions=2,user_access_instructions=3,lookup_instructions=45,
    gnu_instructions=gnu_record['gnu_instructions'],gnu_windows=gnu_record['gnu_windows'],
    offline_rejections=12,model_reader_tests=60,
    native_regression_passed=True,native_new_per_access_observation=False,
    native_helper_union_unchanged='88/91',published_caller_index_unchanged='79/91; 306 instructions',
    kernel_and_provider_and_three_metadata_files_equal=True,
    authority_meaning_changed=False,canonical_production_unchanged=True,
    previous_failures=[dict(kind='control-harness expected-offset error',terminal_status='failed',
                           detail='Initial GS displacement offset was off by one; no rejected action or AI notice'),
                       dict(kind='GNU-reader operand formatting mismatch',terminal_status='failed',
                           detail='The decoder spells zero displacement as [r8+0x0]; no kernel or AI failure')],
    remaining=['Fresh per-access/helper-preservation/permission observation for both outputs',
        'Actual table/output/stack owner and nonalias evidence; continuous lifetime',
        'Complete helper/entry/return/fatal/exception contracts',
        'Twelve sites without published intervals, including these two, and all fallback obligations',
        'Canonical source/native/boot and full Step-4 qualification']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),inherited_artifacts=5,native_regression_passed=True)))
