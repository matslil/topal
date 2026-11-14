"""Replay the exact archived object audit without booting or modifying a VM."""
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
for name in ['inspect_call_inventory.py', 'audit_transfer_guards.py']:
    source = 'linux-kernel/tools/user-memory/'+name
    row = identity['selected_inputs'][source]
    data = gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    (work/name).write_bytes(data)
for name, row in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed = (root/row['snapshot']).read_bytes()
    assert digest(compressed) == row['snapshot_sha256']
    data = gzip.decompress(compressed)
    assert len(data) == row['bytes'] and digest(data) == row['sha256']
    (work/name).write_bytes(data)
inventory = (root/'extent-call-inventory.json').read_bytes()
expected = json.loads((root/'transfer-guard-audit.json').read_text())
assert digest(inventory) == expected['inventory_sha256']
(work/'extent-call-inventory.json').write_bytes(inventory)
result = subprocess.run([sys.executable, str(work/'audit_transfer_guards.py'), str(work)],
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
(work/'replay.log').write_text(result.stdout)
assert result.returncode == 0, result.stdout
actual = json.loads((work/'transfer-guard-audit.json').read_text())
assert actual == expected
report = dict(passed=True, exact_archived_audit_equal=True, carry_gates=actual['call_count'],
    fallback_local_reads=len(actual['fallback_local_reads']),
    scope='Offline archived first-party object audit; no kernel rebuild, VM execution or full access proof')
(work/'replay.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report))
