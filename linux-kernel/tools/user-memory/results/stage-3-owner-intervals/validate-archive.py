"""Check frozen selected interval inputs, native records and replay bindings."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parent
repo=root.parents[4]
read=lambda path:json.loads(path.read_text())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
references={}
def unpack(label,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed=path.read_bytes()
    if 'snapshot_sha256' in row:assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256']
    if 'bytes' in row:assert len(raw)==row['bytes']
    references[label]=dict(snapshot=row['snapshot'],sha256=digest(raw),snapshot_sha256=digest(packed))
    return raw
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
sources={name:unpack('sources/'+name,row) for name,row in identity['selected_inputs'].items()}
assert len(sources)==178 and all((repo/name).read_bytes()==raw for name,raw in sources.items())
artifacts={name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts)==93
kernel=digest(artifacts['kernel.elf'])
audits={name:json.loads(raw) for name,raw in artifacts.items() if '/' not in name and name.endswith('-audit.json')}
assert len(audits)==13 and all(row['passed'] and row['kernel_sha256']==kernel for row in audits.values())
run=read(root/'native/run.json')
assert run['exit_code']==0 and run['selected_tests']==1 and not run['source_changes_during_run']
assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
assert len(run['modified_source_hashes_before'])==5
assert all(digest(sources[name])==sha for name,sha in run['modified_source_hashes_before'].items())
assert run['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
assert read(root/'native/invocation.json')['environment']['TOPAL_TEST_EXTENT_INTERVALS']=='metadata'
raw=gzip.decompress((root/'native/observation/observations.json.gz').read_bytes())
obs=json.loads(raw)
assert obs['passed'] and len(obs['executions'])==4
native=read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256']==kernel and native['observation_sha256']==digest(raw)
assert (native['selected_intervals'],native['protected_ranges_per_interval'],native['native_positive_controls'],native['interval_snapshots'],native['watchpoints_created'])==(52,26,12,64,27)
assert native['owner_interval_model_sha256']==digest(sources['linux-kernel/tools/user-memory/owner_interval.py'])
assert native['observed_identity_address_cpu_store_changes']==0 and native['table_status_set_events']==8
interval=obs['owner_interval_observation']
assert interval['passed'] and interval['all_watchpoints_hardware'] and interval['all_watchpoints_disabled']
assert interval['inactive_watchpoints_deleted_after_cases']
assert len(interval['records'])==52 and len(interval['controls'])==12
assert len(interval['snapshots'])==64 and len(interval['created_watchpoints'])==27
progress=json.loads(gzip.decompress((root/'native/observation/observations.json.interval-progress.json.gz').read_bytes()))
assert not progress['passed'] and progress['selected_intervals_complete'] and not progress['full_guest_retirement_complete']
assert progress['inactive_watchpoints_deleted']
for name in ['records','controls','snapshots','created_watchpoints']:assert progress[name]==interval[name]
for name in ['owner-readback.json','metadata-readback.json']:
    record=read(root/name)
    assert record['passed'] and record['kernel_sha256']==kernel and record['observation_sha256']==digest(raw)
assert read(root/'owner-readback.json')['selected_native_helpers']==52
assert read(root/'metadata-readback.json')['syscalls']==136
comparison=read(root/'artifact-comparison.json')
assert comparison['passed'] and comparison['kernel_sha256']==kernel
assert (comparison['unchanged_core_objects_images'],comparison['unchanged_audits'],comparison['unchanged_other_metadata'])==(17,13,4)
for row in comparison['comparisons']:
    assert digest(artifacts[row['path']])==row['sha256'] and len(artifacts[row['path']])==row['bytes']
for prefix in ['', 'owner-before/']:
    assert json.loads(artifacts[prefix+'callback-bindings.json'])['baseline_provenance_sha256']==digest(artifacts[prefix+'baseline/provenance.json'])
    assert json.loads(artifacts[prefix+'extent-call-inspection.json'])['disassembly_sha256']==digest(artifacts[prefix+'extent-call-disassembly.stdout.log'])
console=read(root/'console-readback.json')
assert console['passed'] and console['bytes']==7606
assert digest((root/console['preceding_record']).read_bytes())==console['preceding_snapshot_sha256']
serial=gzip.decompress((root/'native/observation/serial.log.gz').read_bytes())
assert serial==gzip.decompress((root/console['preceding_record']).read_bytes())
assert len(serial)==7606 and digest(serial)==console['sha256']
for label,size in [('head',4096),('tail',12072)]:
    pilot=json.loads(gzip.decompress((root/'pilots'/label/'observations.json.gz').read_bytes()))
    assert pilot['passed'] and pilot['watch_type']==pilot['watch_info']['type']==7
    before,after=bytes.fromhex(pilot['before']),bytes.fromhex(pilot['after'])
    assert len(before)==len(after)==size and before!=after
    assert pilot['changed_bytes']==[i for i,(a,b) in enumerate(zip(before,after)) if a!=b]
    if label=='tail':assert pilot['changed_bytes']==[12065,12066]
query=json.loads(gzip.decompress((root/'pilots/query-failure/observations.json.gz').read_bytes()))
assert not query['passed']
assert b'gdb.error: negative value' in gzip.decompress((root/'pilots/query-failure/gdb.stderr.log.gz').read_bytes())
timeout=read(root/'history/timeout/run.json')
assert timeout['exit_code']==101 and not timeout['source_changes_during_run']
assert timeout['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in timeout['resources']['memory.events'].splitlines())
failure_sources=read(root/'history/timeout/source-identity.json')
assert len(failure_sources)==5
for name,row in failure_sources.items():
    assert digest(unpack('timeout-sources/'+name,row))==timeout['modified_source_hashes_before'][name]
status=read(root/'history/timeout/status.json')
assert not status['passed'] and not status['full_guest_retirement_complete'] and status['kernel_sha256']==kernel
controls=read(root/'controls/controls.json')
assert controls['passed'] and controls['original_observation_sha256']==digest(raw) and len(controls['controls'])==14
assert controls['reader_sha256']==digest((root/'readback-owner-intervals.py').read_bytes())
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
checks=read(root/'checks/commands.json')
assert len(checks)==7 and all(row['exit_code']==0 for row in checks)
assert b'Ran 155 tests' in (root/'checks/models.log').read_bytes()
replay=read(root/'replay.json')
assert replay['passed'] and (replay['selected_sources'],replay['artifacts'],replay['models'])==(178,93,155)
assert all(replay[key] for key in ['exact_source_join_equal','exact_thirteen_reports_equal',
    'exact_saved_interval_readback_equal','exact_saved_owner_readback_equal','exact_saved_metadata_readback_equal','fourteen_saved_controls_rejected'])
review=read(root/'review.json')
assert review['passed'] and review['risk']=='high' and not review['executable_kernel_changed']
assert not review['core_language_or_authority_meaning_changed'] and review['safeguard_notices']==[]
links=[]
for doc in [root/'README.md',repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'):continue
        assert (doc.parent/target.split('#',1)[0]).resolve().is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files={}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name=='manifest.json':continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    data=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(data),bytes=len(data))
manifest=dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected sources, packed inputs, native watch/copy/control records, pilots, failures, models and replay bindings only; not general stable lifetime or complete clean production')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,sources=len(sources),artifacts=len(artifacts),archive_files=len(files),
    snapshot_references=len(references),selected_native_intervals=52)))
