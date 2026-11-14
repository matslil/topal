"""Verify this selected archive, conditional index, references and local links."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
repo = root.parents[4]
digest = lambda data:hashlib.sha256(data).hexdigest()
identity = json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
artifacts = json.loads((root/'artifact-identity.json').read_text())
references = {}
def unpack(label,row):
    path = (root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed = path.read_bytes()
    if 'snapshot_sha256' in row:
        assert digest(packed) == row['snapshot_sha256']
    data = gzip.decompress(packed)
    assert digest(data) == row['sha256']
    if 'bytes' in row:
        assert len(data) == row['bytes']
    references[label] = dict(snapshot=row['snapshot'],sha256=row['sha256'],snapshot_sha256=digest(packed))
    return data
for path,row in identity['selected_inputs'].items():
    data = unpack('sources/'+path,row)
    assert (repo/path).read_bytes() == data,path
assert len(identity['selected_inputs']) == 141
data = {name:unpack('artifacts/'+name,row) for name,row in artifacts.items()}
preceding = unpack('history/preceding-source-user.elf',json.loads((root/'preceding-link-identity.json').read_text()))
binding = json.loads((root/'prior-component-binding.json').read_text())
component = (root/binding['record']).read_bytes()
assert digest(component) == binding['record_sha256']
old_component = json.loads(gzip.decompress(component))
assert digest(preceding) == old_component['linked_sha256'] == binding['linked_sha256']
assert digest(data['static-elf-abi-fixture.elf']) == old_component['image_sha256'] == binding['image_sha256']
assert digest(data['source-user.elf']) == binding['current_linked_sha256']
sections = json.loads((root/'source-link-section-difference.json').read_text())
assert sections['passed'] and sections['allocated_sections_equal']
assert sections['previous']['sha256'] == digest(preceding)
assert sections['current']['sha256'] == digest(data['source-user.elf'])
native = json.loads((root/'native/run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert native['modified_source_hashes_before'] == native['modified_source_hashes_after']
assert all(identity['selected_inputs'][path]['sha256'] == sha for path,sha in native['modified_source_hashes_before'].items())
obs_bytes = gzip.decompress((root/'native/observation/observations.json.gz').read_bytes())
obs = json.loads(obs_bytes)
readback = json.loads((root/'native-readback.json').read_text())
assert obs['passed'] and readback['passed'] and readback['observation_sha256'] == digest(obs_bytes)
assert readback['audit_sha256'] == digest(data['pipe-fixed-output-audit.json'])
assert readback['user_access_observations'] == 28
index = json.loads((root/'caller-access-index.json').read_text())
selected,accesses = set(),set()
for name,field in [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions')]:
    report = json.loads(data[name])
    assert report['passed'] and report['kernel_sha256'] == readback['kernel_sha256'] == digest(data['kernel.elf'])
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected.add(row['call_offset'])
        accesses.update([row[field]] if field == 'copy_offset' else [r['offset'] for r in row[field]])
assert (len(selected),len(accesses)) == (81,309)
census = json.loads(data['extent-call-inventory.json'])
assert len(census['calls']) == 91 and len(index['calls_without_selected_local_intervals']) == 10
assert {row['call_offset'] for row in index['selected_intervals']} == selected
assert {row['offset'] for row in index['calls_without_selected_local_intervals']} == {row['offset'] for row in census['calls']}-selected
for row in index['selected_intervals']:
    assert any(case['call_offset'] == row['call_offset'] and case['name'] == row['name']
               for case in json.loads(gzip.decompress((root/row['audit']).read_bytes()))['cases'])
coverage = json.loads((root/'native-coverage.json').read_text())
prior_coverage = json.loads((root/coverage['preceding_record']).read_text())
prior_sites = {r if isinstance(r,int) else r['offset'] for r in prior_coverage['reached']}
fresh_sites = {r['offset'] for r in obs['extent_call_coverage']['reached']}
assert fresh_sites <= prior_sites and len(fresh_sites) == 79 and len(prior_sites) == 88
replay = json.loads((root/'replay.json').read_text())
assert replay['passed'] and replay['models'] == 69 and replay['exact_six_reports_equal']
assert replay['exact_native_record_readback_equal'] and replay['exact_raw_source_link_section_readback_equal']
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
    raw = path.read_bytes()
    files[str(path.relative_to(root))] = dict(sha256=digest(raw),bytes=len(raw))
manifest = dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected archived byte/reference/link/index/source equality checks only; no complete producer, continuous ownership/lifetime or canonical qualification')
if '--check' in sys.argv[1:]:
    assert json.loads((root/'manifest.json').read_text()) == manifest
else:
    (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),snapshot_references=len(references),
    selected_inputs=len(identity['selected_inputs']),links=len(links),conditional_sites=len(selected),access_instructions=len(accesses))))
