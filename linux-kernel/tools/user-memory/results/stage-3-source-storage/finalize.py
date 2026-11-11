"""Verify frozen evidence, selected sources and report links before publication."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote

root = Path('linux-kernel/tools/user-memory/results/stage-3-source-storage')
identity = json.loads((root/'source-identity.json').read_text())
native = json.loads((root/'native-run.json').read_text())
digest = lambda data: hashlib.sha256(data).hexdigest()
assert native['exit_code'] == 0 and not native['source_changes_during_run']
for source, row in identity['selected_inputs'].items():
    data = Path(source).read_bytes()
    assert digest(data) == row['sha256'], source
    assert gzip.decompress((root/row['snapshot']).read_bytes()) == data, source
    if source not in native['modified_source_hashes_before']:
        assert subprocess.check_output(['git','show',identity['base_commit']+':'+source]) == data, source
for source, value in native['modified_source_hashes_before'].items():
    assert digest(Path(source).read_bytes()) == value, source
links = 0
for document in [root/'README.md', Path('linux-kernel/tools/user-memory/caller-routing.md'),
                 Path('linux-kernel/step-4-owned-user-extents.md')]:
    for raw in re.findall(r'\]\(([^)]+)\)',document.read_text()):
        target = raw.split('#',1)[0]
        if not target or re.match(r'^[a-z]+:',target) or target == 'manifest.json':
            continue
        assert (document.parent/unquote(target)).exists(), (str(document),target)
        links += 1
files = []
for path in sorted(root.rglob('*')):
    assert not path.is_symlink(), path
    if not path.is_file() or path.name == 'manifest.json':
        continue
    data = path.read_bytes()
    files.append(dict(path=str(path.relative_to(root)),bytes=len(data),sha256=digest(data)))
(root/'manifest.json').write_text(json.dumps(dict(files=files),indent=2)+'\n')
print(json.dumps(dict(passed=True, selected_inputs=len(identity['selected_inputs']),
                     files=len(files), links=links)))
