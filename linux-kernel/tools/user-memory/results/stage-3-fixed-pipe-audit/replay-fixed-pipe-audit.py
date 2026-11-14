"""Restore selected frozen sources and inspect ELF data without execution."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
work = Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
digest = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
for path,row in identity['selected_inputs'].items():
    compressed = (root/row['snapshot']).read_bytes()
    if 'snapshot_sha256' in row:
        assert digest(compressed) == row['snapshot_sha256']
    data = gzip.decompress(compressed)
    assert digest(data) == row['sha256']
    target = work/'source-root'/path
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
build = work/'build'
build.mkdir()
for name,row in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed = (root/row['snapshot']).read_bytes()
    assert digest(compressed) == row['snapshot_sha256']
    data = gzip.decompress(compressed)
    assert digest(data) == row['sha256'] and len(data) == row['bytes']
    (build/name).write_bytes(data)
readers = work/'source-root/linux-kernel/tools/user-memory'
command = [sys.executable,str(readers/'audit_pipe_fixed_outputs.py'),str(build)]
result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(work/'audit.log').write_bytes(result.stdout)
assert result.returncode == 0,result.stdout.decode(errors='replace')
assert (build/'pipe-fixed-output-audit.json').read_bytes() == (root/'audit.json').read_bytes()
models = subprocess.run([sys.executable,'-m','unittest','discover','-s',str(readers),
    '-p','test_pipe_fixed_outputs.py','-v'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(work/'models.log').write_bytes(models.stdout)
assert models.returncode == 0 and b'Ran 8 tests' in models.stdout,models.stdout.decode(errors='replace')
report = dict(passed=True,selected_inputs=len(identity['selected_inputs']),inherited_artifacts=5,
    exact_report_equal=True,focused_models=8,
    scope='Frozen selected source and artifact replay only. ELF files are inspected as data; no compiler, GNU, GDB or VM execution, no complete producer/owner/lifetime/native qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
