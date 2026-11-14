"""Replay selected source/report/model/record and complete body comparisons."""
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
def unpack(row):
    packed=(root/row['snapshot']).read_bytes()
    if 'snapshot_sha256' in row: assert digest(packed)==row['snapshot_sha256']
    data=gzip.decompress(packed)
    assert digest(data)==row['sha256']
    if 'bytes' in row: assert len(data)==row['bytes']
    return data
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
comparison=json.loads((root/'emission-comparison.json').read_text())
for row in comparison['unchanged_component_objects']:
    data=(build/row['path']).read_bytes()
    assert digest(data)==row['sha256']
    (build/'before'/row['path']).write_bytes(data)
observations=build/'source-user-extents'
observations.mkdir()
for path in (root/'native/pipes/observation').glob('*.gz'):
    (observations/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,result.stdout[-4000:].decode(errors='replace')
origin=(build/'process-access-origins.json').read_bytes()
run('origins',[sys.executable,str(readers/'trace_process_origins.py'),str(build),str(source)])
assert (build/'process-access-origins.json').read_bytes()==origin
names=['fixed-transfer-audit.json','copy-transfer-audit.json','staged-transfer-audit.json',
    'additional-scalar-audit.json','staging-copy-audit.json','pipe-fixed-output-audit.json',
    'stat-output-audit.json','serial-prefix-audit.json','pipe-loop-audit.json']
expected={name:(build/name).read_bytes() for name in names}
for stem in ['fixed_transfers','copy_transfers','staged_transfers','additional_scalars','staging_copies',
             'pipe_fixed_outputs','stat_outputs','serial_prefix','pipe_loops']:
    run(stem,[sys.executable,str(readers/('audit_'+stem+'.py')),str(build)])
assert all((build/name).read_bytes()==data for name,data in expected.items())
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 94 tests' in (work/'models.log').read_bytes()
run('native-records',[sys.executable,str(root/'readback-pipe-loop-native.py'),str(build),str(work/'native-readback')])
assert (work/'native-readback/readback.json').read_bytes()==(root/'native-readback.json').read_bytes()
run('emission',[sys.executable,str(root/'compare-pipe-loop-emission.py'),str(build/'before'),str(build),str(work/'emission')])
assert (work/'emission/comparison.json').read_bytes()==(root/'emission-comparison.json').read_bytes()
run('provider-functions',[sys.executable,str(root/'compare-pipe-loop-provider.py'),str(build/'before'),str(build),str(work/'provider-functions')])
assert (work/'provider-functions/comparison.json').read_bytes()==(root/'provider-function-comparison.json').read_bytes()
gnu=work/'gnu-graph'
gnu.mkdir()
for path in (root/'gnu-graph').glob('*.gz'): (gnu/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
run('gnu-operands',[sys.executable,str(root/'readback-pipe-loop-operands.py'),str(build),str(gnu),str(work/'gnu-operands')])
assert (work/'gnu-operands/readback.json').read_bytes()==(root/'gnu-operands.json').read_bytes()
report=dict(passed=True,selected_inputs=len(identity['selected_inputs']),artifacts=len(artifacts),
    exact_source_join_equal=True,exact_nine_reports_equal=True,models=94,
    exact_native_record_readback_equal=True,exact_complete_body_comparison_equal=True,
    exact_provider_function_comparison_equal=True,
    exact_saved_gnu_operand_readback_equal=True,
    scope='Frozen selected inputs/reports/models/saved native and GNU records/body comparison only. Equal historical component bytes are restored from their recorded identical hashes. No ELF, compiler, GNU decoder, GDB or VM execution; not an independent baseline producer, complete clean producer, actual owners/lifetime or canonical qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
