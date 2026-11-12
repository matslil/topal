"""Replay the selected saved-code inventory and nine reader tests, without a VM."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parent
def digest(raw):return hashlib.sha256(raw).hexdigest()
def read(path):return json.loads(path.read_bytes())
def unpack(row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(root.parents[4])
    packed=path.read_bytes()
    assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
    return raw

def replay(work):
    work=Path(work).resolve()
    work.mkdir(parents=True,exist_ok=False)
    meta=read(root/'inputs.json')
    for group in ['sources','artifacts','records']:
        for name,row in meta[group].items():
            assert not Path(name).is_absolute() and '..' not in Path(name).parts
            path=work/name
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_bytes(unpack(row))
    commands=[('models',[sys.executable,'-m','unittest','discover','-s','linux-kernel/tools/user-memory','-p','test_paging_controls.py','-v']),
        ('inventory',[sys.executable,'linux-kernel/tools/user-memory/inventory_paging_controls.py',
            'kernel.elf','llvm.stdout.log','gnu.stdout.log','process-access-inventory.json','inventory-replay.json'])]
    for label,command in commands:
        with (work/(label+'.log')).open('wb') as log:
            result=subprocess.run(command,cwd=work,stdout=log,stderr=subprocess.STDOUT,
                env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'))
        assert result.returncode==0,(label,result.returncode)
    assert b'Ran 9 tests' in (work/'models.log').read_bytes()
    assert read(work/'inventory-replay.json')==read(root/'inventory.json')
    report=dict(passed=True,selected_sources=3,inherited_artifacts=2,reader_tests=9,
        exact_inventory_equal=True,new_native_execution=False,
        full_decoder_control_agreement=False,
        scope='Frozen Python-only inventory and targeted reader replay; unresolved decoder/entry/runtime qualifications remain')
    (work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))

if __name__=='__main__':replay(sys.argv[1])
