"""Reproduce frozen readers/models and saved evidence without native execution."""
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
read=lambda path:json.loads(path.read_text())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
def unpack(row):
    packed=(root/row['snapshot']).read_bytes()
    if 'snapshot_sha256' in row:assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256']
    if 'bytes' in row:assert len(raw)==row['bytes']
    return raw
source=work/'source-root'
for name,row in identity['selected_inputs'].items():
    path=source/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(row))
build=work/'build'
artifacts=read(root/'artifact-identity.json')
for name,row in artifacts.items():
    path=build/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(row))
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,(label,result.stdout[-4000:].decode(errors='replace'))
origin=(build/'process-access-origins.json').read_bytes()
run('origins',[sys.executable,str(readers/'trace_process_origins.py'),str(build),str(source)])
assert (build/'process-access-origins.json').read_bytes()==origin
stems=['fixed_transfers','copy_transfers','staged_transfers','additional_scalars','staging_copies',
    'pipe_fixed_outputs','stat_outputs','serial_prefix','pipe_loops','file_inputs','warning_inputs','random_outputs','stack_retirement']
names=[name for name in artifacts if not name.startswith('before/') and name.endswith('-audit.json')]
assert len(names)==13
expected={name:(build/name).read_bytes() for name in names}
for stem in stems:run(stem,[sys.executable,str(readers/('audit_'+stem+'.py')),str(build)])
assert all((build/name).read_bytes()==raw for name,raw in expected.items())
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 132 tests' in (work/'models.log').read_bytes()
gnu=work/'gnu'
gnu.mkdir()
for name,row in read(root/'gnu-identity.json').items():(gnu/name).write_bytes(unpack(row))
run('gnu-operands',[sys.executable,str(root/'readback-stack-gnu.py'),str(build),str(gnu),str(work/'gnu-operands')])
assert (work/'gnu-operands/readback.json').read_bytes()==(root/'gnu-operands.json').read_bytes()
compare=work/'compare-stack-emission.py'
compare.write_bytes((root/'compare-stack-emission.py').read_bytes())
(work/'baseline-linear-transcript.json').write_bytes(gzip.decompress((root/'baseline-linear-transcript.json.gz').read_bytes()))
run('emission',[sys.executable,str(compare),str(build/'before'),str(build),str(work/'emission')])
assert (work/'emission/comparison.json').read_bytes()==gzip.decompress((root/'emission-comparison.json.gz').read_bytes())
builds={}
for label in ['source','storage','pinned']:
    target=work/label
    observation=target/('first-image-exit' if label=='pinned' else 'source-user-extents')
    observation.mkdir(parents=True)
    for name in ['kernel.elf','stack-retirement-audit.json']:(target/name).write_bytes((build/name).read_bytes())
    for path in (root/'native'/label/'observation').glob('*.gz'):
        (observation/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
    builds[label]=str(target)
mapping=work/'builds.json'
mapping.write_text(json.dumps(builds))
run('native-records',[sys.executable,str(root/'readback-stack-native.py'),str(mapping),str(work/'native')])
assert (work/'native/readback.json').read_bytes()==(root/'native-readback.json').read_bytes()
report=dict(passed=True,selected_sources=len(identity['selected_inputs']),artifacts=len(artifacts),models=132,
    exact_source_join_equal=True,exact_thirteen_reports_equal=True,exact_gnu_operand_readback_equal=True,
    exact_complete_emission_comparison_equal=True,exact_saved_native_readback_equal=True,
    scope='Frozen selected source/artifact reconstruction, Python-only readers/models and saved GNU/native record joins. No ELF, compiler, GNU decoder, GDB or VM execution; not a complete clean producer, actual owners/lifetime, unreached paths or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
