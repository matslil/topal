"""Replay frozen source joining and conditional scalar checks without a VM."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
work = Path(sys.argv[1]).resolve()
work.mkdir(parents=True, exist_ok=False)
digest = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
for path, row in identity['selected_inputs'].items():
    data = gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    target = work/'source-root'/path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
readers = work/'source-root/linux-kernel/tools/user-memory'
build = work/'build'
build.mkdir()
for name, row in json.loads((root/'artifact-identity.json').read_text()).items():
    archived = (root/row['snapshot']).read_bytes()
    if row.get('compressed', True):
        assert digest(archived) == row['snapshot_sha256']
        data = gzip.decompress(archived)
    else:
        data = archived
    assert digest(data) == row['sha256'] and len(data) == row['bytes']
    (build/name).write_bytes(data)
expected_origins = json.loads((build/'process-access-origins.json').read_text())
for name, arguments in [
        ('trace_process_origins', [str(build), str(work/'source-root')]),
        ('audit_fixed_transfers', [str(build)])]:
    result = subprocess.run([sys.executable, str(readers/(name+'.py')), *arguments],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (work/(name+'.log')).write_bytes(result.stdout)
    assert result.returncode == 0, result.stdout.decode(errors='replace')
assert json.loads((build/'process-access-origins.json').read_text()) == expected_origins
assert json.loads((build/'fixed-transfer-audit.json').read_text()) == json.loads((root/'fixed-transfer-audit.json').read_text())
report = dict(passed=True, selected_inputs=len(identity['selected_inputs']),
    exact_source_join_equal=True, exact_fixed_transfer_report_equal=True,
    transfer_regions=9, scalar_access_instructions=27,
    scope='Frozen first-party artifact/source replay with explicitly inherited snapshots; no compiler rebuild, LLVM/GNU/GDB/VM execution, helper/lifetime or production qualification')
(work/'replay.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report))
