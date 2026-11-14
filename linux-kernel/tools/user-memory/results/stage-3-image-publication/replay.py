"""Replay selected frozen image/heap publication checks without executing a guest."""
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
    assert not Path(name).is_absolute() and '..' not in Path(name).parts
    path=source/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(root,row))
build=work/'build'
for name,row in artifacts.items():
    assert not Path(name).is_absolute() and '..' not in Path(name).parts
    path=build/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(unpack(root,row))
native=build/'source-user-extents'
native.mkdir()
for name,row in read(root/'native/files.json').items():
    assert Path(name).name==name
    (native/name).write_bytes(unpack(root,row))
inherited={}
for name,row in read(root/'inherited-readers.json').items():
    path=(root/row['path']).resolve()
    assert path.is_relative_to(root.parents[4])
    raw=path.read_bytes()
    assert len(raw)==row['bytes'] and digest(raw)==row['sha256']
    inherited[name]=path
readers=source/'linux-kernel/tools/user-memory'
def run(label,command):
    result=subprocess.run(command,cwd=source,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,(label,result.stdout[-3000:].decode(errors='replace'))
    print(label+' passed',flush=True)
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 196 tests' in (work/'models.log').read_bytes()
for label,script in [('layout','inspect_image_publication.py'),('images','readback_image_publication.py'),
        ('heap','readback_translation_publication.py')]:
    run(label,[sys.executable,str(readers/script),str(build),str(work/(label+'.json'))])
    assert (work/(label+'.json')).read_bytes()==(root/'checks'/(label+'.json')).read_bytes()
run('owners',[sys.executable,str(inherited['readback-owner-boundaries.py']),str(build),
    str(native/'observations.json'),str(work/'owners.json')])
assert (work/'owners.json').read_bytes()==(root/'checks/owners.json').read_bytes()
run('metadata',[sys.executable,str(inherited['readback-source-metadata.py']),str(build),str(work/'metadata')])
assert (work/'metadata/readback.json').read_bytes()==(root/'checks/metadata/readback.json').read_bytes()
run('image-controls',[sys.executable,str(readers/'control_image_publication.py'),str(build),str(work/'image-controls.json')])
assert (work/'image-controls.json').read_bytes()==(root/'checks/image-controls.json').read_bytes()
run('heap-controls',[sys.executable,str(readers/'control_translation_publication.py'),str(build),str(work/'heap-controls')])
assert (work/'heap-controls/controls.json').read_bytes()==(root/'checks/heap-controls/controls.json').read_bytes()
report=dict(passed=True,selected_sources=len(sources),artifacts=len(artifacts),models=196,
    exact_layout_equal=True,exact_image_readback_equal=True,exact_heap_readback_equal=True,
    exact_owner_readback_equal=True,exact_metadata_readback_equal=True,
    exact_sixteen_image_controls_equal=True,exact_eighteen_heap_controls_equal=True,
    new_native_execution=False,
    scope='Frozen selected source/artifact delta, models and exact saved-data reader/control replay; no compiler, decoder, GDB or VM execution, continuous lifetime or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
