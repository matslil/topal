"""Replay frozen models, saved descriptor graphs and inherited same-kernel data.

This Python-only replay executes no ELF, compiler, GNU decoder, debugger or VM.
Run under scripts/run_bounded.py and supply a new output directory.
"""
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

root=Path(__file__).resolve().parent
previous=root.parent/'stage-3-owner-intervals'
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
read=lambda path:json.loads(path.read_bytes())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
for name,row in read(root/'inherited-evidence.json').items():
    raw=(root/name).read_bytes()
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
def unpack(row):
    packed=(root/row['snapshot']).read_bytes()
    assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
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
native=read(root/'native-source.json')
(observation/'observations.json').write_bytes(unpack(native['observation']))
for path in (previous/'native/observation').glob('*.gz'):
    (observation/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
assert digest((observation/'observations.json').read_bytes())==native['observation']['sha256']
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,(label,result.stdout[-3000:].decode(errors='replace'))
    print(label+' passed',flush=True)
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
assert b'Ran 164 tests' in (work/'models.log').read_bytes()
for label,reader,report in [('intervals','readback-owner-intervals.py','native-readback.json'),
        ('owners','readback-owner-boundaries.py','owner-readback.json')]:
    run(label,[sys.executable,str(previous/reader),str(build),str(observation/'observations.json'),str(work/report)])
    assert (work/report).read_bytes()==(previous/report).read_bytes()
run('metadata',[sys.executable,str(previous/'readback-source-metadata.py'),str(build),str(work/'metadata-readback')])
assert (work/'metadata-readback/readback.json').read_bytes()==(previous/'metadata-readback.json').read_bytes()
cache=source/'linux-kernel/.cache/x86_64'
cache.mkdir(parents=True)
shutil.copyfile(previous/'readback-owner-intervals.py',cache/'readback-owner-intervals.py')
run('inherited-controls',[sys.executable,str(previous/'control-owner-intervals.py'),str(build),str(work/'inherited-controls')])
assert [(r['name'],r['rejected']) for r in read(work/'inherited-controls/controls.json')['controls']]==[(r['name'],r['rejected']) for r in read(previous/'controls/controls.json')['controls']]
report=work/'mapping-aliases.json'
run('aliases',[sys.executable,str(readers/'audit_mapping_aliases.py'),str(build),str(report)])
assert report.read_bytes()==gzip.decompress((root/'mapping-aliases.json.gz').read_bytes())
run('independent',[sys.executable,str(readers/'readback_mapping_aliases.py'),str(build),str(report),str(work/'independent.json')])
assert (work/'independent.json').read_bytes()==(root/'independent.json').read_bytes()
run('alias-controls',[sys.executable,str(readers/'control_mapping_aliases.py'),str(build),str(report),str(work/'alias-controls')])
assert (work/'alias-controls/controls.json').read_bytes()==(root/'checks/controls/controls.json').read_bytes()
result=dict(passed=True,selected_sources=len(identity['selected_inputs']),artifacts=len(artifacts),models=164,
    exact_thirteen_reports_equal=True,exact_source_join_equal=True,exact_saved_interval_owner_metadata_readbacks_equal=True,
    inherited_fourteen_controls_rejected=True,exact_alias_report_equal=True,exact_independent_readback_equal=True,
    exact_twenty_alias_controls_equal=True,new_native_execution=False,
    scope='Frozen selected sources/artifacts and Python-only saved-record/model replay; no ELF, compiler, GNU decoder, GDB or VM execution, TLB/DMA/asynchronous exclusion, general lifetime or clean production qualification')
(work/'replay.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
