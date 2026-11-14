"""Replay frozen conditional reports, models and saved native/GNU records."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
identity=json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
def unpack(row):
    packed=(root/row['snapshot']).read_bytes()
    if 'snapshot_sha256' in row: assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256']
    if 'bytes' in row: assert len(raw)==row['bytes']
    return raw
source=work/'source-root'
for name,row in identity['selected_inputs'].items():
    path=source/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(row))
build=work/'build'
artifacts=json.loads((root/'artifact-identity.json').read_text())
for name,row in artifacts.items():
    path=build/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(row))
observations=build/'source-user-extents'
observations.mkdir()
for path in (root/'native/storage/observation').glob('*.gz'):
    (observations/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,
        env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'))
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,result.stdout[-4000:].decode(errors='replace')
origin=(build/'process-access-origins.json').read_bytes()
run('origins',[sys.executable,str(readers/'trace_process_origins.py'),str(build),str(source)])
assert (build/'process-access-origins.json').read_bytes()==origin
names=['fixed-transfer-audit.json','copy-transfer-audit.json','staged-transfer-audit.json',
    'additional-scalar-audit.json','staging-copy-audit.json','pipe-fixed-output-audit.json',
    'stat-output-audit.json','serial-prefix-audit.json','pipe-loop-audit.json','file-input-audit.json','warning-input-audit.json']
expected={name:(build/name).read_bytes() for name in names}
for stem in ['fixed_transfers','copy_transfers','staged_transfers','additional_scalars','staging_copies',
             'pipe_fixed_outputs','stat_outputs','serial_prefix','pipe_loops','file_inputs','warning_inputs']:
    run(stem,[sys.executable,str(readers/('audit_'+stem+'.py')),str(build)])
assert all((build/name).read_bytes()==raw for name,raw in expected.items())
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 104 tests' in (work/'models.log').read_bytes()
run('native-records',[sys.executable,str(root/'readback-warning-input-native.py'),str(build),str(work/'native-readback')])
assert (work/'native-readback/readback.json').read_bytes()==(root/'native-readback.json').read_bytes()
gnu=work/'gnu-graph'
gnu.mkdir()
for name,row in json.loads((root/'gnu-identity.json').read_text()).items():
    (gnu/name).write_bytes(unpack(row))
run('gnu-operands',[sys.executable,str(root/'readback-warning-input-gnu.py'),str(build),str(gnu),str(work/'gnu-operands')])
assert (work/'gnu-operands/readback.json').read_bytes()==(root/'gnu-operands.json').read_bytes()
report=dict(passed=True,selected_inputs=len(identity['selected_inputs']),artifacts=len(artifacts),
    exact_source_join_equal=True,exact_eleven_reports_equal=True,models=104,
    exact_native_record_readback_equal=True,exact_saved_gnu_operand_readback_equal=True,
    scope='Frozen selected sources/reports/models and saved native/GNU records only. No ELF, compiler, GNU decoder, GDB or VM execution; not a complete clean producer, actual owners/nonalias/lifetime/retired-spill cleanup or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
