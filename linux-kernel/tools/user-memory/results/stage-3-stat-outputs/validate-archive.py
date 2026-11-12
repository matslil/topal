"""Check selected frozen stat evidence, source equality, references and links."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
repo = root.parents[4]
digest = lambda data:hashlib.sha256(data).hexdigest()
read = lambda path:json.loads(path.read_text())
identity = read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
references = {}
def unpack(label,row):
    path = (root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed = path.read_bytes()
    if 'snapshot_sha256' in row: assert digest(packed) == row['snapshot_sha256']
    data = gzip.decompress(packed)
    assert digest(data) == row['sha256']
    if 'bytes' in row: assert len(data) == row['bytes']
    references[label] = dict(snapshot=row['snapshot'],sha256=digest(data),snapshot_sha256=digest(packed))
    return data
sources = {name:unpack('sources/'+name,row) for name,row in identity['selected_inputs'].items()}
assert len(sources) == 147
assert all((repo/name).read_bytes() == data for name,data in sources.items())
artifacts = {name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts) == 16
kernel_sha = digest(artifacts['kernel.elf'])
origin = json.loads(artifacts['process-access-origins.json'])
assert origin['origin_sha256'] == digest(artifacts['process-byte-origins.json'])
assert origin['kernel_sha256'] == kernel_sha
assert all(digest(sources[name]) == sha for name,sha in origin['source_hashes'].items())
fields = [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset')]
selected,accesses = {},set()
for name,field in fields:
    report = json.loads(artifacts[name])
    assert report['passed'] and report['kernel_sha256'] == kernel_sha
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']] = row
        accesses.update([row[field]] if field == 'copy_offset' else [r['offset'] for r in row[field]])
index = read(root/'caller-access-index.json')
census = json.loads(artifacts['extent-call-inventory.json'])
assert not index['complete'] and index['kernel_sha256'] == kernel_sha
assert (len(selected),len(accesses),len(census['calls'])) == (83,310,91)
assert {r['call_offset'] for r in index['selected_intervals']} == set(selected)
assert {r['offset'] for r in index['calls_without_selected_local_intervals']} == {r['offset'] for r in census['calls']}-set(selected)
assert len(index['calls_without_selected_local_intervals']) == 8
for row in index['selected_intervals']:
    report = json.loads(gzip.decompress((root/row['audit']).read_bytes()))
    assert any(case['call_offset'] == row['call_offset'] and case['name'] == row['name'] for case in report['cases'])
audit_sha = digest(artifacts['stat-output-audit.json'])
observations = {}
frozen = None
for label in ['stat','source','pinned','fixtures']:
    run = read(root/'native'/label/'run.json')
    assert run['exit_code'] == 0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before'] == run['modified_source_hashes_after']
    assert len(run['modified_source_hashes_before']) == 13
    assert all(digest(sources[name]) == sha for name,sha in run['modified_source_hashes_before'].items())
    if frozen is None: frozen = run['modified_source_hashes_before']
    assert run['modified_source_hashes_before'] == frozen
    assert run['resources']['memory.swap.max'] == '0'
    assert all(int(line.split()[1]) == 0 for line in run['resources']['memory.events'].splitlines())
    if label != 'fixtures':
        raw = gzip.decompress((root/'native'/label/'observation/observations.json.gz').read_bytes())
        observations[label] = json.loads(raw)
        assert observations[label]['passed']
        if label == 'stat': stat_observation_sha = digest(raw)
        else:
            inputs = read(root/'native'/label/'input-identities.json')
            assert inputs['kernel.elf']['sha256'] == kernel_sha
            assert inputs['stat-output-audit.json']['sha256'] == audit_sha
native = read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256'] == kernel_sha
assert native['audit_sha256'] == audit_sha and native['observation_sha256'] == stat_observation_sha
assert native['source_syscalls'] == 152 and native['stat_queries'] == 88
assert len(native['completed_copies']) == 60 and len(native['variants']) == 8
assert native['saved_observer_programs_present']
gnu = read(root/'gnu/readback.json')
assert gnu['passed'] and gnu['kernel_sha256'] == kernel_sha and gnu['audit_sha256'] == audit_sha
assert gnu['gnu_instructions'] == 1384 and gnu['gnu_windows'] == 6
assert gnu['reverse_admission_group_cut'] and gnu['raw_and_gnu_register_and_memory_effect_counts_equal']
controls = read(root/'controls/controls.json')
assert controls['passed'] and controls['positive_report_sha256'] == audit_sha
assert controls['reader_sha256'] == digest(sources['linux-kernel/tools/user-memory/audit_stat_outputs.py'])
assert controls['effects_reader_sha256'] == digest(sources['linux-kernel/tools/user-memory/stat_instruction_effects.py'])
assert len(controls['controls']) == 14
for row in controls['controls']:
    assert row['rejected'] and row['exit_code'] != 0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes()) == row['log_sha256']
comparison = read(root/'artifact-comparison.json')
assert comparison['passed']
for label in ['stat','source','pinned']:
    compared = comparison['kernel_artifact_comparisons'][label]
    assert len(compared) == 17
    assert next(row for row in compared if row['path'] == 'kernel.elf')['sha256'] == kernel_sha
coverage = read(root/'native-coverage.json')
prior_bytes = (root/coverage['preceding_record']).read_bytes()
assert coverage['preceding_record_sha256'] == digest(prior_bytes)
prior_sites = set(json.loads(prior_bytes)['reached'])
assert coverage['passed'] and coverage['matching_kernel_union'] == len(prior_sites) == 88
for label,obs in observations.items():
    sites = {r['offset'] for r in obs['extent_call_coverage']['reached']}
    assert sites <= prior_sites and len(sites) == coverage['fresh_first_reach_sites'][label]
for label,count,status in [('first-reach-failure',12,101),('first-stat-pass',13,0)]:
    folder = root/'history'/label
    historical,run = read(folder/'identity.json'),read(folder/'run.json')
    assert historical['passed'] and len(historical['inputs']) == count and run['exit_code'] == status
    for name,row in historical['inputs'].items():
        data = gzip.decompress((folder/row['snapshot']).read_bytes())
        assert digest(data) == row['sha256'] == run['modified_source_hashes_before'][name]
replay = read(root/'replay.json')
assert replay['passed'] and replay['models'] == 82
assert replay['selected_inputs'] == 147 and replay['artifacts'] == 16
assert replay['exact_source_join_equal'] and replay['exact_seven_reports_equal'] and replay['exact_native_record_readback_equal']
links = []
for doc in [root/'README.md',repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'): continue
        assert (doc.parent/target.split('#',1)[0]).resolve().is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files = {}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name == 'manifest.json': continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    data = path.read_bytes()
    files[str(path.relative_to(root))] = dict(sha256=digest(data),bytes=len(data))
manifest = dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected byte/reference/link/current-source/report/index/saved-record bindings only; not full producer, actual ownership/lifetime or canonical qualification')
if '--check' in sys.argv[1:]: assert read(root/'manifest.json') == manifest
else: (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),snapshot_references=len(references),
    selected_inputs=len(sources),links=len(links),conditional_sites=len(selected),access_instructions=len(accesses))))
