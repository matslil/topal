"""Verify native source freezes, inherited Git bytes, compressed data and links."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote

root = Path('linux-kernel/tools/user-memory/results/stage-3-staged-refusals')
digest = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((root / 'source-identity.json').read_text())
references = {identity['inherited_record']}
for path, row in identity['selected_inputs'].items():
    data = Path(path).read_bytes()
    assert digest(data) == row['sha256']
    assert gzip.decompress((root / row['snapshot']).read_bytes()) == data
    if row['snapshot'].startswith('../'):
        references.add(row['snapshot'])
        assert subprocess.check_output(['git', 'show', identity['base_commit'] + ':' + path]) == data
for name, row in json.loads((root / 'artifact-identity.json').read_text()).items():
    references.add(row['snapshot'])
    data = (root / row['snapshot']).read_bytes()
    raw = gzip.decompress(data) if row.get('compressed', True) else data
    assert digest(raw) == row['sha256'] and len(raw) == row['bytes'], name
for row in json.loads((root / 'inherited-evidence.json').read_text()):
    references.add(row['path'])
    data = (root / row['path']).read_bytes()
    assert digest(data) == row['sha256'] and len(data) == row['bytes']
native = json.loads((root / 'native-run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
for path, sha in native['modified_source_hashes_before'].items():
    assert digest(Path(path).read_bytes()) == sha
if (root / 'application/native-run.json').exists():
    pinned = json.loads((root / 'application/native-run.json').read_text())
    assert pinned['exit_code'] == 0 and not pinned['source_changes_during_run']
    assert pinned['modified_source_hashes_before'] == native['modified_source_hashes_before']
for attempt in root.glob('*-timeout'):
    failed = json.loads((attempt / 'run.json').read_text())
    assert failed['exit_code'] != 0 and not failed['source_changes_during_run']
    for path, sha in failed['modified_source_hashes_before'].items():
        data = gzip.decompress((attempt / 'inputs' / (path + '.gz')).read_bytes())
        assert digest(data) == sha
for relative, record in [('guest/observations.json.gz', 'observation-identity.json'),
                         ('source-user-image.elf.gz', 'image-identity.json'),
                         ('staged-transfer-audit.json.gz', 'staged-audit-identity.json')]:
    data = gzip.decompress((root / relative).read_bytes())
    row = json.loads((root / record).read_text())
    assert digest(data) == row['sha256'] and len(data) == row['bytes']
ref_rows = []
for reference in sorted(references):
    path = (root / reference).resolve()
    assert path.is_relative_to(root.parent.resolve())
    data = path.read_bytes()
    relative = str(path.relative_to(Path.cwd()))
    assert subprocess.check_output(['git', 'show', identity['base_commit'] + ':' + relative]) == data
    ref_rows.append(dict(path=reference, bytes=len(data), sha256=digest(data), base_git_blob_equal=True))
links = 0
for document in [root / 'README.md', Path('linux-kernel/tools/user-memory/caller-routing.md'),
                 Path('linux-kernel/step-4-owned-user-extents.md')]:
    for raw in re.findall(r'\]\(([^)]+)\)', document.read_text()):
        target = raw.split('#', 1)[0]
        if not target or re.match(r'^[a-z]+:', target) or target == 'manifest.json':
            continue
        assert (document.parent / unquote(target)).exists(), (str(document), target)
        links += 1
files = []
for path in sorted(root.rglob('*')):
    assert not path.is_symlink()
    if path.is_file() and path.name != 'manifest.json':
        data = path.read_bytes()
        files.append(dict(path=str(path.relative_to(root)), bytes=len(data), sha256=digest(data)))
(root / 'manifest.json').write_text(json.dumps(dict(files=files, inherited_references=ref_rows), indent=2) + '\n')
print(json.dumps(dict(passed=True, selected_inputs=len(identity['selected_inputs']),
    local_evidence_files=len(files), inherited_references=len(ref_rows), links=links)))
