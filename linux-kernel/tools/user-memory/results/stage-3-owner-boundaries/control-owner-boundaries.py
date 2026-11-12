"""Alter saved observation records only; execute no guest program or debugger."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
reader=Path('linux-kernel/.cache/x86_64/readback-owner-boundaries.py').resolve()
raw=(build/'source-user-extents/observations.json').read_bytes()
original=json.loads(raw)
sha=lambda data:hashlib.sha256(data).hexdigest()

def mutate_snapshots(obs,operation):
    native=obs['owner_boundary_observation']
    renamed={}
    snapshots={}
    for key,snapshot in native['snapshots'].items():
        operation(snapshot)
        newkey=sha(json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode())
        renamed[key]=newkey
        snapshots[newkey]=snapshot
    native['snapshots']=snapshots
    for row in native['records']:
        for name in ['before','after']:row[name]=renamed[row[name]]

def missing_pin(snapshot):snapshot['roles'][2]['record'][6]=0
def stale_cell(snapshot):snapshot['roles'][0]['cells'][0]=snapshot['allocator'][2]
def heap_alias(snapshot):
    base=snapshot['roles'][2]['cells'][2]
    snapshot['state'][3]=base*4096
    snapshot['heap']['cells'][2::3]=list(range(base,base+snapshot['state'][2]))
def foreign_table(snapshot):
    root=snapshot['context']['root']
    snapshot['tables'][str(root)][0]=snapshot['context']['low']|7
def stale_entry(snapshot):snapshot['epoch'][0]=0
def no_wp(snapshot):snapshot['cr0']&=~(1<<16)
def foreign_leaf(snapshot):
    root=snapshot['context']['root']
    snapshot['tables'][str(root+16384)][0x638000//4096%512]=0x100007

controls=[]
for name,operation in [('missing-pin',missing_pin),('stale-cell',stale_cell),
        ('heap-alias',heap_alias),('foreign-table',foreign_table),('stale-entry',stale_entry),
        ('write-protection-disabled',no_wp),('foreign-leaf',foreign_leaf)]:
    controls.append((name,lambda obs,op=operation:mutate_snapshots(obs,op)))
controls.extend([
    ('missing-helper',lambda obs:obs['owner_boundary_observation']['records'].pop()),
    ('false-page-evidence',lambda obs:obs['owner_boundary_observation']['records'][0]['evidence']['pages'].pop()),
    ('wrong-direction',lambda obs:obs['owner_boundary_observation']['records'][0].update(direction=0)),
    ('wrong-model-hash',lambda obs:obs['owner_boundary_observation'].update(model_sha256='0'*64)),
    ('wrong-syscall-order',lambda obs:obs['owner_boundary_observation']['records'].__setitem__(2,copy.deepcopy(obs['owner_boundary_observation']['records'][0])))])
rows=[]
for name,operation in controls:
    obs=copy.deepcopy(original)
    operation(obs)
    path=output/(name+'.json')
    path.write_text(json.dumps(obs,separators=(',',':'))+'\n')
    result=subprocess.run([sys.executable,str(reader),str(build),str(path),str(output/(name+'-readback.json'))],
        env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (output/(name+'.log')).write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout[-3000:])
    rows.append(dict(name=name,rejected=True,exit_code=result.returncode,log_sha256=sha(result.stdout)))
    path.unlink()
report=dict(passed=True,controls=rows,reader_sha256=sha(reader.read_bytes()),original_observation_sha256=sha(raw),
    scope='Twelve saved JSON observation controls, with coherent snapshot-digest rebinding for semantic controls. No ELF, GDB or VM execution, register edit or guest memory mutation')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
