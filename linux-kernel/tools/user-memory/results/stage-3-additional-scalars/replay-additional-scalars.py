"""Replay the frozen source join and staged access proof without a VM."""
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
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
for path,row in identity['selected_inputs'].items():
    data=gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data)==row['sha256']
    target=work/'source-root'/path
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
readers=work/'source-root/linux-kernel/tools/user-memory'
build=work/'build'
build.mkdir()
for name,row in json.loads((root/'artifact-identity.json').read_text()).items():
    data=(root/row['snapshot']).read_bytes()
    if row.get('compressed',True):
        assert digest(data)==row['snapshot_sha256']
        data=gzip.decompress(data)
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
    (build/name).write_bytes(data)
expected=json.loads((build/'process-access-origins.json').read_text())
for name,args in [('trace_process_origins',[str(build),str(work/'source-root')]),
                  ('audit_additional_scalars',[str(build)])]:
    result=subprocess.run([sys.executable,str(readers/(name+'.py')),*args],
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(name+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,result.stdout.decode(errors='replace')
assert json.loads((build/'process-access-origins.json').read_text())==expected
assert (build/'additional-scalar-audit.json').read_bytes()==gzip.decompress((root/'additional-scalar-audit.json.gz').read_bytes())
report=dict(passed=True,selected_inputs=len(identity['selected_inputs']),
    exact_source_join_equal=True,exact_additional_scalar_report_equal=True,
    scalar_regions=33,user_access_instructions=51,
    scope='Frozen selected first-party source/artifact replay only; no compiler rebuild, LLVM/GNU/GDB/VM execution, complete owner/lifetime or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
