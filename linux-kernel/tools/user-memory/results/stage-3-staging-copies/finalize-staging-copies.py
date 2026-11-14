"""Verify frozen sources, native versions, local artifacts and inherited Git bytes."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

root=Path('linux-kernel/tools/user-memory/results/stage-3-staging-copies')
digest=lambda data:hashlib.sha256(data).hexdigest()
identity=json.loads((root/'source-identity.json').read_text())
references={identity['inherited_record']}
selected=identity['selected_inputs']
review=json.loads((root/'review.json').read_text())
for path,sha in review['source_sha256'].items(): assert digest(Path(path).read_bytes())==sha
for path,sha in review['authority_sha256'].items(): assert digest(Path(path).read_bytes())==sha
for path,row in selected.items():
    data=Path(path).read_bytes()
    assert digest(data)==row['sha256'] and gzip.decompress((root/row['snapshot']).read_bytes())==data
    if row['snapshot'].startswith('../'):
        references.add(row['snapshot'])
        assert subprocess.check_output(['git','show',identity['base_commit']+':'+path])==data
for name,row in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed=(root/row['snapshot']).read_bytes()
    assert digest(compressed)==row['snapshot_sha256']
    data=gzip.decompress(compressed)
    assert len(data)==row['bytes'] and digest(data)==row['sha256'],name
from staging_source_versions import verify_records
version_counts=verify_records(root,selected,lambda path:path.read_bytes(),
    lambda path:subprocess.check_output(['git','show',identity['base_commit']+':'+path]))
for name in ['gnu','graph']:
    rows=json.loads(gzip.decompress((root/(name+'-records.json.gz')).read_bytes()))
    assert len({r['name'] for r in rows})==len(rows)
    records={r['name']:r for r in rows}
    for row in rows:
        data=row['text'].encode()
        assert digest(data)==row['sha256'] and len(data)==row['bytes']
    filename='readback.json' if name=='gnu' else 'gnu-verification.json'
    assert records[filename]['text'].encode()==(root/(name+'-readback.json')).read_bytes()
    report=json.loads(records[filename]['text'])
    artifacts=json.loads((root/'artifact-identity.json').read_text())
    assert report['passed'] and report['kernel_sha256']==artifacts['kernel.elf']['sha256']
    if name=='graph':
        transcript=records['gnu-decoder-transcript.json']['text'].encode()
        assert len(transcript)==report['decoder_transcript_bytes']
        assert digest(transcript)==report['decoder_transcript_sha256']
        assert len(json.loads(transcript))==report['gnu_decoder_nodes']
        assert report['graph_sha256']==artifacts['process-access-inventory.json']['sha256']
assert json.loads((root/'post-validation.json').read_text())['passed']
library=json.loads((root/'library-validation.json').read_text())
assert library['exit_code']==0 and not library['source_changes_during_run']
for path,sha in library['modified_source_hashes_before'].items(): assert digest(Path(path).read_bytes())==sha
assert json.loads((root/'replay.json').read_text())['passed']
full_records=json.loads((root/'full-shared-record-readback.json').read_text())
assert full_records['passed'] and full_records['complete_gzip_stream']
assert full_records['streamed_records']==full_records['summary_records']>0
assert full_records['input_kernel_sha256']==artifacts['kernel.elf']['sha256']
assert full_records['observations_sha256']==digest(gzip.decompress((root/'full-shared/observations.json.gz').read_bytes()))
assert full_records['compressed_records_sha256']==digest((root/'full-shared/observations.json.extent-records.jsonl.gz').read_bytes())
ref_rows=[]
for reference in sorted(references):
    path=(root/reference).resolve()
    assert path.is_relative_to(root.parent.resolve())
    data=path.read_bytes()
    relative=str(path.relative_to(Path.cwd()))
    assert subprocess.check_output(['git','show',identity['base_commit']+':'+relative])==data
    ref_rows.append(dict(path=reference,bytes=len(data),sha256=digest(data),base_git_blob_equal=True))
links=0
for document in [root/'README.md',Path('linux-kernel/tools/user-memory/caller-routing.md'),Path('linux-kernel/step-4-owned-user-extents.md')]:
    for target in re.findall(r'\]\(([^)]+)\)',document.read_text()):
        target=target.split('#',1)[0]
        if not target or re.match(r'^[a-z]+:',target) or target=='manifest.json': continue
        assert (document.parent/target).exists(),(str(document),target)
        links+=1
files=[]
for path in sorted(root.rglob('*')):
    if '__pycache__' in path.relative_to(root).parts or path.suffix in ['.pyc','.pyo']:
        continue
    assert not path.is_symlink()
    if path.is_file() and path.name!='manifest.json':
        data=path.read_bytes()
        files.append(dict(path=str(path.relative_to(root)),bytes=len(data),sha256=digest(data)))
(root/'manifest.json').write_text(json.dumps(dict(files=files,inherited_references=ref_rows),indent=2)+'\n')
print(json.dumps(dict(passed=True,selected_inputs=len(selected),local_files=len(files),inherited_references=len(ref_rows),links=links)))
