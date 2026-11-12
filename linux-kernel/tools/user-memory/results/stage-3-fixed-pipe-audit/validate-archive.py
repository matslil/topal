"""Verify selected snapshots, local links and archive hashes; never run ELF."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
repo = root.parents[4]
digest = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
references = {}
for label, rows in [('sources',identity['selected_inputs']),
                    ('artifacts',json.loads((root/'artifact-identity.json').read_text()))]:
    for name,row in rows.items():
        path = (root/row['snapshot']).resolve()
        assert path.is_relative_to(repo)
        compressed = path.read_bytes()
        if 'snapshot_sha256' in row:
            assert digest(compressed) == row['snapshot_sha256']
        data = gzip.decompress(compressed)
        assert digest(data) == row['sha256']
        if 'bytes' in row:
            assert len(data) == row['bytes']
        if label == 'sources':
            assert digest((repo/name).read_bytes()) == row['sha256'],name
        references[label+'/'+name] = dict(snapshot=row['snapshot'],sha256=row['sha256'],
                                          snapshot_sha256=digest(compressed))
native = json.loads((root/'native/run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert native['modified_source_hashes_before'] == native['modified_source_hashes_after']
assert all(identity['selected_inputs'][name]['sha256'] == sha
           for name,sha in native['modified_source_hashes_before'].items())
audit_bytes = (root/'audit.json').read_bytes()
audit = json.loads(audit_bytes)
assert audit['passed'] and not audit['complete']
assert (audit['transfer_regions'],audit['scalar_access_instructions']) == (2,3)
assert len(audit['callee_contracts'][0]['instructions']) == 45
gnu = json.loads((root/'gnu/readback.json').read_text())
controls = json.loads((root/'controls/controls.json').read_text())
assert gnu['passed'] and gnu['audit_sha256'] == digest(audit_bytes)
assert controls['passed'] and controls['positive_report_sha256'] == digest(audit_bytes)
assert controls['reader_sha256'] == identity['selected_inputs'][
    'linux-kernel/tools/user-memory/audit_pipe_fixed_outputs.py']['sha256']
assert len(controls['controls']) == 12 and all(row['rejected'] for row in controls['controls'])
for row in controls['controls']:
    assert digest((root/('controls/'+row['name']+'.log')).read_bytes()) == row['log_sha256']
replay = json.loads((root/'replay.json').read_text())
assert replay['passed'] and replay['exact_report_equal'] and replay['focused_models'] == 8
links = []
for doc in [root/'README.md',repo/'linux-kernel/step-4-owned-user-extents.md',
            repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'):
            continue
        path = (doc.parent/target.split('#',1)[0]).resolve()
        assert path.is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files = {}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name == 'manifest.json':
        continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    data = path.read_bytes()
    files[str(path.relative_to(root))] = dict(sha256=digest(data),bytes=len(data))
manifest = dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected archive bytes, current selected-source equality and local-link checks only; no complete producer, native per-access, owner/lifetime or canonical qualification')
if '--check' in sys.argv[1:]:
    assert json.loads((root/'manifest.json').read_text()) == manifest
else:
    (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),snapshot_references=len(references),
                     links=len(links),selected_inputs=len(identity['selected_inputs']))))
