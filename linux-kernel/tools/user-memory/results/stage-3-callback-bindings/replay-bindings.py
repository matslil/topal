"""Replay frozen callback byte/registration binding without a VM."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
tool=Path(sys.argv[2]).resolve()
assert tool.is_file()
work.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
identity=json.loads((root/'source-identity.json').read_text())
for name in ['inspect_call_inventory.py','inventory_process_accesses.py','inspect_callback_bindings.py']:
    record=identity['selected_inputs']['linux-kernel/tools/user-memory/'+name]
    data=gzip.decompress((root/record['snapshot']).read_bytes())
    assert digest(data)==record['sha256']
    (work/name).write_bytes(data)
for name,record in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed=(root/record['snapshot']).read_bytes()
    assert digest(compressed)==record['snapshot_sha256']
    data=gzip.decompress(compressed)
    assert len(data)==record['bytes'] and digest(data)==record['sha256']
    target=work/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
command=[sys.executable,str(work/'inspect_callback_bindings.py'),str(work),str(tool)]
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(work/'replay.log').write_bytes(result.stdout)
assert result.returncode==0,result.stdout.decode(errors='replace')
expected=json.loads(gzip.decompress((root/'source/callback-bindings.json.gz').read_bytes()))
actual=json.loads((work/'callback-bindings.json').read_text())
assert actual==expected
report=dict(passed=True,exact_callback_binding_equal=True,controllers=len(actual['callbacks']),
    functions=sum(len(r['functions']) for r in actual['callbacks']),
    sections=sum(len(r['sections']) for r in actual['callbacks']),
    scope='Offline archived first-party callback/root/object-to-linked binding; no kernel rebuild, new VM/native execution, full source authentication or callee-effect qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
