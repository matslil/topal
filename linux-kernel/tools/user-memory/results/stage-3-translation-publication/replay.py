"""Restore selected frozen inputs and replay Python-only local-publication checks."""
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys

from records import digest,inputs,read,unpack

root=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
sources,artifacts=inputs(root)
source=work/'source-root'
for name,row in sources.items():
    path=source/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(root,row))
build=work/'build'
for name,row in artifacts.items():
    path=build/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(root,row))
native=build/'source-user-extents'
native.mkdir()
for name,row in read(root/'native/files.json').items():
    assert Path(name).name==name
    (native/name).write_bytes(unpack(root,row))
for name,row in read(root/'inherited-readers.json').items():
    raw=(root/name).read_bytes()
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,(label,result.stdout[-3000:].decode(errors='replace'))
    print(label+' passed',flush=True)
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 177 tests' in (work/'models.log').read_bytes()
for label,script,report in [('layout','inspect_translation_publication.py','layout.json'),
        ('publication','readback_translation_publication.py','publication.json')]:
    run(label,[sys.executable,str(readers/script),str(build),str(work/report)])
    assert (work/report).read_bytes()==(root/'checks'/report).read_bytes()
previous=root.parent/'stage-3-owner-intervals'
run('owners',[sys.executable,str(previous/'readback-owner-boundaries.py'),str(build),
    str(native/'observations.json'),str(work/'owners.json')])
assert (work/'owners.json').read_bytes()==(root/'checks/owners.json').read_bytes()
run('metadata',[sys.executable,str(previous/'readback-source-metadata.py'),str(build),str(work/'metadata')])
assert (work/'metadata/readback.json').read_bytes()==(root/'checks/metadata/readback.json').read_bytes()
run('controls',[sys.executable,str(readers/'control_translation_publication.py'),str(build),str(work/'controls')])
assert (work/'controls/controls.json').read_bytes()==(root/'checks/controls/controls.json').read_bytes()
report=dict(passed=True,selected_sources=len(sources),artifacts=len(artifacts),models=177,
    exact_layout_readback_equal=True,exact_publication_readback_equal=True,
    exact_owner_readback_equal=True,exact_metadata_readback_equal=True,exact_eighteen_controls_equal=True,
    new_native_execution=False,
    scope='Current frozen selected source/artifact delta and Python-only model/saved native-data replay. Preceding mapping and thirteen-audit checks are inherited; no new ELF, compiler, GNU decoder, GDB or VM execution, general stable lifetime or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
