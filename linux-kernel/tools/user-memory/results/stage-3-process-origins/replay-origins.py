"""Replay the frozen byte/source/access join without LLVM, GDB or a VM."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
identity=json.loads((root/'source-identity.json').read_text())
for path,row in identity['selected_inputs'].items():
    data=gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data)==row['sha256']
    target=work/'source-root'/path
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
readers=work/'readers'
readers.mkdir()
for name in ['inspect_call_inventory.py','inventory_process_accesses.py',
             'inspect_callback_bindings.py','trace_process_origins.py']:
    row=identity['selected_inputs']['linux-kernel/tools/user-memory/'+name]
    (readers/name).write_bytes(gzip.decompress((root/row['snapshot']).read_bytes()))
build=work/'build'
build.mkdir()
for name,row in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed=(root/row['snapshot']).read_bytes()
    assert digest(compressed)==row['snapshot_sha256']
    data=gzip.decompress(compressed)
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
    target=build/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
expected=json.loads((build/'process-access-origins.json').read_text())
command=[sys.executable,str(readers/'trace_process_origins.py'),str(build),str(work/'source-root')]
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(work/'replay.log').write_bytes(result.stdout)
assert result.returncode==0,result.stdout.decode(errors='replace')
assert json.loads((build/'process-access-origins.json').read_text())==expected
report=dict(passed=True,exact_process_origin_report_equal=True,
    spans=expected['spans'],source_locations=len(expected['locations']),
    potential_access_instructions=expected['access_instructions'],
    scope='Frozen first-party artifact/source join replay only; no kernel rebuild, VM/native execution, compiler conformance, pointer authority or lifetime qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
