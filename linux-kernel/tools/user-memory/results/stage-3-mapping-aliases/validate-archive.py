"""Bind selected source, inherited native data and complete saved alias analysis."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parent
repo=root.parents[4]
read=lambda path:json.loads(path.read_bytes())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
canonical=lambda value:json.dumps(value,sort_keys=True,separators=(',',':')).encode()
references={}
def unpack(label,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed=path.read_bytes()
    assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
    references[label]=dict(snapshot=row['snapshot'],sha256=digest(raw),snapshot_sha256=digest(packed))
    return raw
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
sources={name:unpack('source/'+name,row) for name,row in identity['selected_inputs'].items()}
assert len(sources)==183 and all((repo/name).read_bytes()==raw for name,raw in sources.items())
artifacts={name:unpack('artifact/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts)==93
kernel=digest(artifacts['kernel.elf'])
native=read(root/'native-source.json')
assert not native['new_native_execution']
raw=unpack('native/observation',native['observation'])
assert digest((root/native['run']).read_bytes())==native['run_sha256']
run=read(root/native['run'])
assert run['exit_code']==0 and not run['source_changes_during_run']
assert run['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
assert all(digest(sources[name])==sha for name,sha in run['modified_source_hashes_before'].items())
assert digest((root/native['preceding_manifest']).read_bytes())==native['preceding_manifest_sha256']
old_manifest=read(root/native['preceding_manifest'])
for name,row in old_manifest['files'].items():
    data=((root/native['preceding_manifest']).parent/name).read_bytes()
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
for name,row in read(root/'inherited-evidence.json').items():
    data=(root/name).read_bytes()
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
obs=json.loads(raw)
assert obs['passed'] and len(obs['executions'])==4
assert obs['owner_boundary_observation']['passed'] and obs['owner_interval_observation']['passed']
report=json.loads(gzip.decompress((root/'mapping-aliases.json.gz').read_bytes()))
assert report['passed'] and report['kernel_sha256']==kernel and report['observation_sha256']==digest(raw)
for field,name in [('alias_model_sha256','alias_snapshot.py'),('reader_sha256','audit_mapping_aliases.py')]:
    assert report[field]==digest(sources['linux-kernel/tools/user-memory/'+name])
assert report['source_bindings_sha256']==digest(artifacts['user-owners.json'])
assert (report['transfers'],report['boundary_observations'],report['distinct_snapshots'],report['distinct_inventories'])==(52,156,116,6)
assert (report['user_pages_per_inventory'],report['supervisor_leaves_per_inventory'],report['leaves_per_inventory'],
    report['present_entries_per_inventory'],report['table_visits_per_inventory'],report['protected_ranges_per_inventory'])==(572,512,1084,1089,6,26)
assert len(report['events'])==156 and len(report['inventories'])==6
assert all(digest(canonical(row))==key for key,row in report['inventories'].items())
independent=read(root/'independent.json')
assert independent['passed'] and independent['exact_flat_topology_and_inverse_routes']
assert independent['observation_sha256']==digest(raw) and independent['kernel_sha256']==kernel
assert independent['alias_report_sha256']==digest(canonical(report))
assert independent['reader_sha256']==digest(sources['linux-kernel/tools/user-memory/readback_mapping_aliases.py'])
controls=read(root/'checks/controls/controls.json')
assert controls['passed'] and len(controls['controls'])==20
assert controls['original_observation_sha256']==digest(raw) and controls['alias_report_sha256']==digest(canonical(report))
assert controls['runner_sha256']==digest(sources['linux-kernel/tools/user-memory/control_mapping_aliases.py'])
assert all(row['rejected'] and row['expected'] in row['assertion'] for row in controls['controls'])
qualification=read(root/'checks/qualification.json')
assert qualification['passed'] and qualification['model_tests']==164 and qualification['rejected_saved_data_controls']==20
assert qualification['source_hashes_before']==qualification['source_hashes_after']
assert all(digest(sources['linux-kernel/tools/user-memory/'+name])==sha for name,sha in qualification['source_hashes_before'].items())
assert len(qualification['commands'])==4
for row in qualification['commands']:
    assert row['exit_code']==0 and digest((root/'checks'/(row['label']+'.log')).read_bytes())==row['log_sha256']
assert b'Ran 164 tests' in (root/'checks/models.log').read_bytes()
failure=read(root/'history/first-control-failure/status.json')
assert not failure['passed'] and not failure['full_initial_source_set_retained']
assert digest((root/'history/first-control-failure/controls.log').read_bytes())==failure['failure_log_sha256']
assert digest(gzip.decompress((root/'history/first-control-failure/initial-control-source.py.gz').read_bytes()))==failure['exact_control_source_sha256']
assert read(root/'history/first-control-failure/commands.json')[-1]['exit_code']!=0
replay_failure=read(root/'history/first-replay-failure/status.json')
assert not replay_failure['passed']
assert digest(gzip.decompress((root/'history/first-replay-failure/replay.py.gz').read_bytes()))==replay_failure['reader_sha256']
assert digest((root/'history/first-replay-failure/intervals.log').read_bytes())==replay_failure['log_sha256']
assert b'qemu.command.json' in (root/'history/first-replay-failure/intervals.log').read_bytes()
replay=read(root/'replay.json')
assert replay['passed'] and not replay['new_native_execution']
assert (replay['selected_sources'],replay['artifacts'],replay['models'])==(183,93,164)
assert all(replay[field] for field in ['exact_thirteen_reports_equal','exact_source_join_equal',
    'exact_saved_interval_owner_metadata_readbacks_equal','inherited_fourteen_controls_rejected',
    'exact_alias_report_equal','exact_independent_readback_equal','exact_twenty_alias_controls_equal'])
review=read(root/'review.json')
assert review['passed'] and review['risk']=='high'
assert not any(review[field] for field in ['executable_kernel_changed','core_language_or_authority_meaning_changed','new_native_execution'])
links=[]
for doc in [root/'README.md',repo/'linux-kernel/README.md',repo/'linux-kernel/step-4-status.md',
        repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
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
    scope='Selected sources, inherited artifact/native inputs, complete recorded graphs, two data readers, controls and replay; no new native execution, general stable lifetime or clean production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,sources=len(sources),artifacts=len(artifacts),archive_files=len(files),snapshot_references=len(references))))
