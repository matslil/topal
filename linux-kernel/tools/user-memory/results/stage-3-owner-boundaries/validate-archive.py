"""Bind selected sources, packed references and actual-owner boundary records."""
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
assert len(sources)==175 and all((repo/name).read_bytes()==raw for name,raw in sources.items())
artifacts={name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts)==89
kernel=digest(artifacts['kernel.elf'])
reports={name:json.loads(raw) for name,raw in artifacts.items() if '/' not in name and name.endswith('-audit.json')}
assert len(reports)==13 and all(row['passed'] and row['kernel_sha256']==kernel for row in reports.values())
run=read(root/'native/run.json')
assert run['exit_code']==0 and run['selected_tests']==1 and not run['source_changes_during_run']
assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
assert len(run['modified_source_hashes_before'])==4
assert all(digest(sources[name])==sha for name,sha in run['modified_source_hashes_before'].items())
assert run['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
assert read(root/'native/invocation.json')['environment']['TOPAL_TEST_EXTENT_OWNERS']=='metadata'
raw=gzip.decompress((root/'native/observation/observations.json.gz').read_bytes())
obs=json.loads(raw)
assert obs['passed']
native=read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256']==kernel and native['observation_sha256']==digest(raw)
assert (native['selected_native_helpers'],native['stat_helpers'],native['pwrite_helpers'],native['boundary_snapshots'],native['translated_pages'])==(52,44,8,104,96)
assert native['owner_model_sha256']==digest(sources['linux-kernel/tools/user-memory/owner_snapshot.py'])
owners=obs['owner_boundary_observation']
assert owners['passed'] and len(owners['records'])==52 and len(owners['snapshots'])==104
assert owners['profile']['kernel_sha256']==kernel
assert owners['profile']['bindings']==json.loads(artifacts['user-owners.json'])
assert owners['profile']['model'].encode()==sources['linux-kernel/tools/user-memory/owner_snapshot.py']
for key,snapshot in owners['snapshots'].items():
    assert digest(json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode())==key
metadata=read(root/'metadata-readback.json')
assert metadata['passed'] and metadata['observation_sha256']==digest(raw) and metadata['kernel_sha256']==kernel
assert (metadata['syscalls'],metadata['modeled_calls'],metadata['stat_outputs'],metadata['clock_returns'])==(136,124,44,44)
comparison=read(root/'artifact-comparison.json')
assert comparison['passed'] and comparison['kernel_sha256']==kernel
assert (comparison['unchanged_core_objects_images'],comparison['unchanged_audits'],comparison['unchanged_other_metadata'])==(17,13,4)
for row in comparison['comparisons']:
    assert digest(artifacts[row['path']])==row['sha256'] and len(artifacts[row['path']])==row['bytes']
assert comparison['normalized_call_inspection']['old_listing_sha256']==digest(artifacts['metadata-before/extent-call-disassembly.stdout.log'])
assert comparison['normalized_call_inspection']['new_listing_sha256']==digest(artifacts['extent-call-disassembly.stdout.log'])
for prefix in ['', 'metadata-before/']:
    assert json.loads(artifacts[prefix+'callback-bindings.json'])['baseline_provenance_sha256']==digest(artifacts[prefix+'baseline/provenance.json'])
    assert json.loads(artifacts[prefix+'extent-call-inspection.json'])['disassembly_sha256']==digest(artifacts[prefix+'extent-call-disassembly.stdout.log'])
assert len(comparison['fresh_cleanup_blocks'])==12 and len(comparison['joined_cleanup_blocks'])==13
assert len(comparison['fresh_helper_sites'])==76 and len(comparison['joined_helper_sites'])==83
controls=read(root/'controls/controls.json')
assert controls['passed'] and controls['original_observation_sha256']==digest(raw) and len(controls['controls'])==12
assert controls['reader_sha256']==digest((root/'readback-owner-boundaries.py').read_bytes())
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
checks=read(root/'checks/commands.json')
assert len(checks)==6 and all(row['exit_code']==0 for row in checks)
assert b'Ran 149 tests' in (root/'checks/models.log').read_bytes()
replay=read(root/'replay.json')
assert replay['passed'] and (replay['selected_sources'],replay['artifacts'],replay['models'])==(175,89,149)
assert all(replay[key] for key in ['exact_source_join_equal','exact_thirteen_reports_equal',
    'exact_saved_owner_readback_equal','exact_saved_metadata_readback_equal','twelve_saved_controls_rejected'])
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
    scope='Selected source, snapshot, native, artifact, model, saved-record, replay and link bindings; not continuous lifetime or clean production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,sources=len(sources),artifacts=len(artifacts),archive_files=len(files),
    snapshot_references=len(references),selected_native_helpers=52)))
