"""Freeze native ordinary pipe observations and the expanded conditional index."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path('linux-kernel/tools/user-memory/results/stage-3-fixed-pipe-native')
prior = root.parent/'stage-3-fixed-pipe-audit'
staging = root.parent/'stage-3-staging-copies'
cache = Path('linux-kernel/.cache/x86_64')
run = cache/'tk283-staging-focused-1794445505844594624'
build = cache/'user-extent-source-pipes-2369600-1794445506947104708'
digest = lambda data:hashlib.sha256(data).hexdigest()
native = json.loads((run/'run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert native['modified_source_hashes_before'] == native['modified_source_hashes_after']
assert native['resources']['memory.swap.max'] == '0'
assert all(int(line.split()[1]) == 0 for line in native['resources']['memory.events'].splitlines())
readback = json.loads((cache/'fixed-pipe-native-readback/readback.json').read_text())
assert readback['passed'] and readback['user_access_observations'] == 28
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
for path,row in json.loads(old_bytes)['selected_inputs'].items():
    data = gzip.decompress((prior/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    inputs[path] = dict(sha256=row['sha256'],snapshot='../stage-3-fixed-pipe-audit/'+row['snapshot'])
for path,sha in native['modified_source_hashes_before'].items():
    data = Path(path).read_bytes()
    assert digest(data) == sha
    inputs[path] = zipped('inputs/'+path+'.gz',data)
for model_path in ['linux-kernel/tools/user-memory/test_model.py','linux-kernel/tools/user-memory/model.py']:
    inputs[model_path] = zipped('inputs/'+model_path+'.gz',Path(model_path).read_bytes())
assert len(inputs) == 141 and len(native['modified_source_hashes_before']) == 5
write('source-identity.json',dict(base_commit='269156515e459c110b72a11db1726741329d4660',
    inherited_record='../stage-3-fixed-pipe-audit/source-identity.json',
    inherited_record_sha256=digest(old_bytes),selected_inputs=inputs,
    scope='Selected snapshots only; no complete clean producer or canonical reconstruction'))
fields = [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions')]
old_artifacts = json.loads((staging/'artifact-identity.json').read_text())
artifacts = {}
for name in ['provider.o','kernel.elf','extent-call-inventory.json','process-access-inventory.json',
             'process-access-origins.json']+[name for name,_ in fields[:-1]]:
    row = old_artifacts[name]
    assert digest((build/name).read_bytes()) == row['sha256']
    artifacts[name] = dict(row,snapshot='../stage-3-staging-copies/'+row['snapshot'])
for name in ['pipe-fixed-output-audit.json','static-elf-abi-fixture.elf','initramfs.cpio','source-user.elf']:
    artifacts[name] = zipped('artifacts/'+name+'.gz',(build/name).read_bytes())
write('artifact-identity.json',artifacts)
selected,accesses = {},set()
for name,field in fields:
    report = json.loads((build/name).read_text())
    assert report['passed'] and report['kernel_sha256'] == readback['kernel_sha256']
    for row in report['cases']:
        at = row['call_offset']
        assert at not in selected
        selected[at] = dict(name=row['name'],call_offset=at,audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field == 'copy_offset' else [r['offset'] for r in row[field]])
census = json.loads((build/'extent-call-inventory.json').read_text())
remaining = [r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining)) == (81,309,10)
write('caller-access-index.json',dict(complete=False,kernel_sha256=readback['kernel_sha256'],
    emitted_helper_calls=len(census['calls']),calls_with_selected_local_intervals=len(selected),
    distinct_user_access_instructions=len(accesses),selected_intervals=[selected[at] for at in sorted(selected)],
    calls_without_selected_local_intervals=remaining,
    scope='Six exact matching-kernel conditional reports; native fixed-pipe observations are separately scoped. Not complete callers, owners/nonalias/lifetime/helper/entry/callee/exception or production qualification'))
save('native/run.json',(run/'run.json').read_bytes())
zipped('native/test.log.gz',(run/'test.log').read_bytes())
for path in (build/'source-user-extents').iterdir():
    if path.is_file() and path.suffix in ['.json','.log','.gdb']:
        zipped('native/observation/'+path.name+'.gz',path.read_bytes())
zipped('native/source-component.json.gz',(build/'source-component.json').read_bytes())
save('native-readback.json',(cache/'fixed-pipe-native-readback/readback.json').read_bytes())
write('baseline-identities.json',readback['byte_identical_artifacts'])
# The prior committed component record supplies the preceding first-party
# linked/application image identities; the eight-file equality was also read
# directly from both actual cache builds before freezing this record.
component_data = gzip.decompress((prior/'native/source-component.json.gz').read_bytes())
component = json.loads(component_data)
assert component['image_sha256'] == artifacts['static-elf-abi-fixture.elf']['sha256']
current_component = json.loads((build/'source-component.json').read_text())
assert current_component['linked_sha256'] == artifacts['source-user.elf']['sha256']
write('prior-component-binding.json',dict(record='../stage-3-fixed-pipe-audit/native/source-component.json.gz',
    record_sha256=digest((prior/'native/source-component.json.gz').read_bytes()),
    image_sha256=component['image_sha256'],linked_sha256=component['linked_sha256'],
    current_linked_sha256=current_component['linked_sha256'],
    raw_linked_images_equal=component['linked_sha256'] == current_component['linked_sha256'],
    scope='Installed first-party application image is identical; raw source-user link ELF equality is not claimed'))
section_bytes = (cache/'fixed-pipe-source-link-section-difference.json').read_bytes()
section_report = json.loads(section_bytes)
assert section_report['passed'] and section_report['allocated_sections_equal']
assert section_report['previous']['sha256'] == component['linked_sha256']
assert section_report['current']['sha256'] == current_component['linked_sha256']
save('source-link-section-difference.json',section_bytes)
write('preceding-link-identity.json',zipped('history/preceding-source-user.elf.gz',
    (cache/'user-extent-source-pipes-2367848-1794444753502668964/source-user.elf').read_bytes()))
for name in ['readback-fixed-pipe-native.py','archive-fixed-pipe-native.py','run-staging-focused-tests.py',
             'readback-source-link-sections.py']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-REQ-SYSTEMS-USER-EXTENT-001','TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High if boundary observations are mistaken for continuous lifetime or complete ownership; observer/test/record changes only, no kernel or authority-design change',
    review='Single-agent authority/source review; actual observer model counterexamples, source-issued invalid/valid syscalls, independent payload/queue/translation and source-order readback, preserved byte identities and frozen replay',
    model_reader_tests=69,new_observer_controls=8,source_syscalls=144,
    completed_transfers=24,refused_admissions=40,scalar_access_observations=28,
    changed_sources_stable_during_run=5,selected_inputs=141,byte_identical_artifacts=8,
    conditional_index_sites=81,conditional_index_access_instructions=309,sites_without_intervals=10,
    native_helper_union_unchanged='88/91',core_language_or_authority_meaning_changed=False,
    native_failures=[],safeguard_notices=[],packaging_failures=[
        'Initial archive incorrectly required raw source-user link ELF equality in addition to the independently verified identical installed application image; retained as a failed packaging check, not a native or AI failure',
        'Initial selected replay ran 52 tests because the unchanged model tests were not among inherited inputs; the next replay found their missing model module. Final selected snapshots include both unchanged files and require all 69 tests'],
    remaining=['Complete actual table/backing/output/stack ownership and nonaliasing',
        'Continuous entry-bound mapping lifetime; boundary equality is insufficient',
        'Complete helper/entry/callee/return/fatal/exception effects',
        'Ten sites without selected intervals and all three fallback obligations',
        'Canonical production qualification and full original Step-4 acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=81,user_access_instructions=309,remaining_sites=10)))
