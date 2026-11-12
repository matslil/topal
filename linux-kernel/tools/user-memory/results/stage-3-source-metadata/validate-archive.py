"""Bind current selected sources, same-kernel coverage and metadata evidence."""
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
assert len(sources)==172 and all((repo/name).read_bytes()==raw for name,raw in sources.items())
artifacts={name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
kernel=digest(artifacts['kernel.elf'])
reports={name:json.loads(raw) for name,raw in artifacts.items() if name.endswith('-audit.json')}
assert len(reports)==13 and all(row['passed'] and row['kernel_sha256']==kernel for row in reports.values())
before_hashes=None
for label in ['metadata','fixtures']:
    run=read(root/'native'/label/'run.json')
    assert run['exit_code']==0 and not run['source_changes_during_run']
    hashes=run['modified_source_hashes_before']
    assert len(hashes)==7 and hashes==run['modified_source_hashes_after']
    assert all(digest(sources[name])==sha for name,sha in hashes.items())
    if before_hashes is None:before_hashes=hashes
    assert hashes==before_hashes and run['selected_tests']==(1 if label=='metadata' else 2)
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
raw=gzip.decompress((root/'native/metadata/observation/observations.json.gz').read_bytes())
obs=json.loads(raw)
native=read(root/'native-readback.json')
assert obs['passed'] and native['passed'] and native['observation_sha256']==digest(raw)
assert native['kernel_sha256']==kernel and native['complete_saved_delta_replay_equal']
assert (native['iterations'],native['syscalls'],native['modeled_calls'],native['stat_outputs'],native['clock_returns'])==(4,136,124,44,44)
assert len(native['events'])==124 and native['compiled_stack_usage_verified'] and native['user_stack_bound']<=2048
assert native['metadata_model_sha256']==digest(sources['linux-kernel/tools/user-memory/metadata_model.py'])
assert native['metadata_source_sha256']==digest(sources['linux-kernel/labs/qemu/x86_64/user_extent_metadata_cases.c'])
comparison=read(root/'artifact-comparison.json')
assert comparison['passed'] and comparison['kernel_sha256']==kernel
assert (comparison['unchanged_core_objects_images'],comparison['unchanged_audits'],comparison['unchanged_other_metadata'])==(17,13,4)
assert len(comparison['comparisons'])==34
for row in comparison['comparisons']:assert digest(artifacts[row['path']])==row['sha256'] and len(artifacts[row['path']])==row['bytes']
previous=(root/comparison['preceding_record']).read_bytes()
assert digest(previous)==comparison['preceding_record_sha256']
prior=json.loads(previous)
assert prior['passed'] and prior['kernel_sha256']==kernel
fresh={row['name'] for row in obs['stack_retirement_observation']['records']}
joined=fresh|set(prior['reached_blocks'])
assert len(fresh)==12 and len(joined)==13 and joined==set(comparison['joined_cleanup_blocks'])
assert obs['stack_retirement_observation']['audit_sha256']==digest(artifacts['stack-retirement-audit.json'])
assert comparison['cleanup_activations']==146 and comparison['zero_bytes_observed']==5208
assert set(comparison['fresh_helper_sites'])=={row['offset'] for row in obs['extent_call_coverage']['reached']}
assert set(comparison['joined_helper_sites'])==set(comparison['fresh_helper_sites'])|set(prior['fresh_helper_first_reach_union'])
assert len(comparison['joined_helper_sites'])==83
for prefix in ['', 'before/']:
    callbacks=json.loads(artifacts[prefix+'callback-bindings.json'])
    assert callbacks['baseline_provenance_sha256']==digest(artifacts[prefix+'baseline/provenance.json'])
    inspection=json.loads(artifacts[prefix+'extent-call-inspection.json'])
    assert inspection['disassembly_sha256']==digest(artifacts[prefix+'extent-call-disassembly.stdout.log'])
assert comparison['normalized_call_inspection']['every_other_listing_byte_and_inspection_field_equal']
relation=comparison['callback_binding_relation']
assert relation['all_other_callback_fields_equal'] and not relation['retained_intermediate_text_equivalence_claimed']
for row in relation['equal_baseline_source_module_object_inputs']:
    assert digest(artifacts['baseline/'+row['path']])==row['sha256']
assert len(relation['versioned_provenance_changes'])==14
controls=read(root/'controls/controls.json')
assert controls['passed'] and len(controls['controls'])==14 and controls['original_observation_sha256']==digest(raw)
assert controls['reader_sha256']==digest((root/'readback-source-metadata.py').read_bytes())
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
checks=read(root/'checks/commands.json')
assert len(checks)==7 and all(row['exit_code']==0 for row in checks)
assert b'Ran 141 tests' in (root/'checks/models.log').read_bytes()
index=read(root/'caller-access-index.json')
assert index['kernel_sha256']==kernel and (index['calls_with_selected_local_intervals'],index['distinct_user_access_instructions'])==(90,325)
for row in index['selected_intervals']:
    audit=json.loads(gzip.decompress((root/row['audit']).read_bytes()))
    assert any(case['call_offset']==row['call_offset'] and case['name']==row['name'] for case in audit['cases'])
replay=read(root/'replay.json')
assert replay['passed'] and replay['selected_sources']==len(sources) and replay['artifacts']==len(artifacts) and replay['models']==141
assert all(replay[key] for key in ['exact_source_join_equal','exact_thirteen_reports_equal','exact_saved_metadata_delta_stat_clock_image_readback_equal'])
review=read(root/'review.json')
assert review['passed'] and review['risk']=='high' and not review['executable_kernel_changed']
assert review['safeguard_notices']==[] and not review['core_language_or_authority_meaning_changed']
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
    scope='Selected source/snapshot/artifact/report/current-native/model/replay/link bindings only; not complete clean production or actual kernel owner/nonalias/lifetime qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,sources=len(sources),artifacts=len(artifacts),archive_files=len(files),
    snapshot_references=len(references),cleanup_blocks_joined=len(joined),helper_sites_joined=83)))
