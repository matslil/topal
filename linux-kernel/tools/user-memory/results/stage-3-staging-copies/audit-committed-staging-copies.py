"""Read back committed snapshots, selected source versions and actual native freezes."""
import gzip
import hashlib
import json
from pathlib import Path
import posixpath
import subprocess

root=Path('linux-kernel/tools/user-memory/results/stage-3-staging-copies')
digest=lambda data:hashlib.sha256(data).hexdigest()
def committed(path,ref='HEAD'):
    path=Path(posixpath.normpath(str(path)))
    if path.is_absolute():
        path=path.relative_to(Path.cwd())
    assert '..' not in path.parts
    return subprocess.check_output(['git','show',ref+':'+path.as_posix()])
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
identity=json.loads(committed(root/'source-identity.json'))
selected=identity['selected_inputs']
review=json.loads(committed(root/'review.json'))
for path,sha in review['authority_sha256'].items():
    assert digest(committed(path))==sha
for path,sha in review['source_sha256'].items():
    assert digest(committed(path))==sha
for path,row in selected.items():
    snapshot=(root/row['snapshot']).resolve().relative_to(Path.cwd())
    data=gzip.decompress(committed(snapshot))
    assert digest(data)==row['sha256'] and committed(path)==data
    if row['snapshot'].startswith('../'): assert committed(path,identity['base_commit'])==data
manifest=json.loads(committed(root/'manifest.json'))
for row in manifest['files']:
    data=committed(root/row['path'])
    assert data==(root/row['path']).read_bytes() and digest(data)==row['sha256'] and len(data)==row['bytes']
for row in manifest['inherited_references']:
    path=(root/row['path']).resolve().relative_to(Path.cwd())
    data=committed(path)
    assert data==committed(path,identity['base_commit']) and digest(data)==row['sha256'] and len(data)==row['bytes']
from staging_source_versions import verify_records
version_counts=verify_records(root,selected,committed,
    lambda path:committed(path,identity['base_commit']))
library=json.loads(committed(root/'library-validation.json'))
assert library['exit_code']==0 and not library['source_changes_during_run']
for path,sha in library['modified_source_hashes_before'].items(): assert digest(committed(path))==sha
full_records=json.loads(committed(root/'full-shared-record-readback.json'))
assert full_records['passed'] and full_records['complete_gzip_stream']
assert full_records['streamed_records']==full_records['summary_records']>0
artifacts=json.loads(committed(root/'artifact-identity.json'))
assert full_records['input_kernel_sha256']==artifacts['kernel.elf']['sha256']
report=dict(passed=True,commit=head,selected_inputs=len(selected),local_files=len(manifest['files']),
    inherited_references=len(manifest['inherited_references']),**version_counts,
    scope='Committed selected-source/archive/version equality; not a clean full producer rebuild or canonical production acceptance')
Path('linux-kernel/.cache/x86_64/committed-staging-copy-audit.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
