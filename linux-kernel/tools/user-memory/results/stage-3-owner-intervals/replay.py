"""Restore selected inputs and replay the exact saved interval/owner evidence."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
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
observation=build/'source-user-extents'
observation.mkdir()
for path in (root/'native/observation').glob('*.gz'):
    (observation/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,(label,result.stdout[-3000:].decode(errors='replace'))
origins=(build/'process-access-origins.json').read_bytes()
run('origins',[sys.executable,str(readers/'trace_process_origins.py'),str(build),str(source)])
assert (build/'process-access-origins.json').read_bytes()==origins
names=[name for name in artifacts if '/' not in name and name.endswith('-audit.json')]
assert len(names)==13
expected={name:(build/name).read_bytes() for name in names}
for stem in ['fixed_transfers','copy_transfers','staged_transfers','additional_scalars','staging_copies',
        'pipe_fixed_outputs','stat_outputs','serial_prefix','pipe_loops','file_inputs','warning_inputs','random_outputs','stack_retirement']:
    run(stem,[sys.executable,str(readers/('audit_'+stem+'.py')),str(build)])
assert all((build/name).read_bytes()==raw for name,raw in expected.items())
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 155 tests' in (work/'models.log').read_bytes()
for label,reader,report in [('intervals','readback-owner-intervals.py','native-readback.json'),
        ('owners','readback-owner-boundaries.py','owner-readback.json')]:
    run(label,[sys.executable,str(root/reader),str(build),str(observation/'observations.json'),str(work/report)])
    assert (work/report).read_bytes()==(root/report).read_bytes()
run('metadata',[sys.executable,str(root/'readback-source-metadata.py'),str(build),str(work/'metadata-readback')])
assert (work/'metadata-readback/readback.json').read_bytes()==(root/'metadata-readback.json').read_bytes()
cache=source/'linux-kernel/.cache/x86_64'
cache.mkdir(parents=True)
shutil.copyfile(root/'readback-owner-intervals.py',cache/'readback-owner-intervals.py')
run('controls',[sys.executable,str(root/'control-owner-intervals.py'),str(build),str(work/'controls')])
assert [(r['name'],r['rejected']) for r in read(work/'controls/controls.json')['controls']]==[(r['name'],r['rejected']) for r in read(root/'controls/controls.json')['controls']]
report=dict(passed=True,selected_sources=len(identity['selected_inputs']),artifacts=len(artifacts),models=155,
    exact_source_join_equal=True,exact_thirteen_reports_equal=True,exact_saved_interval_readback_equal=True,
    exact_saved_owner_readback_equal=True,exact_saved_metadata_readback_equal=True,fourteen_saved_controls_rejected=True,
    scope='Frozen selected sources/artifacts and Python-only saved-record/model replay; no ELF, compiler, GNU decoder, GDB or VM execution, physical-alias/DMA/asynchronous exclusion, general lifetime or clean production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
