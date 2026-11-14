"""Change saved interval records only; do not execute altered guest inputs."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
reader=Path('linux-kernel/.cache/x86_64/readback-owner-intervals.py').resolve()
raw=(build/'source-user-extents/observations.json').read_bytes()
original=json.loads(raw)
sha=lambda data:hashlib.sha256(data).hexdigest()

def endpoint(obs,operation):
    native=obs['owner_interval_observation']
    row=native['records'][0]
    snapshot=copy.deepcopy(native['snapshots'][row['after']])
    operation(snapshot)
    key=sha(json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode())
    old=row['after']
    native['snapshots'][key]=snapshot
    row['after']=key
    if not any(old in [r['before'],r['after']] for r in native['records']):del native['snapshots'][old]

def bad_permission(snapshot):
    root=snapshot['context']['root']
    snapshot['tables'][str(root+16384)][0x638000//4096%512]&=~2
def bad_address(snapshot):
    root=snapshot['context']['root']
    snapshot['tables'][str(root+16384)][0x638000//4096%512]^=4096
def bad_status(snapshot):
    root=snapshot['context']['root']
    snapshot['tables'][str(root+16384)][0]|=0x20

controls=[
    ('missing-range',lambda obs:obs['owner_interval_observation']['records'][0]['ranges'].pop()),
    ('wrong-watch-count',lambda obs:obs['owner_interval_observation']['records'][0].update(hardware_watchpoint_count=25)),
    ('software-watchpoint',lambda obs:obs['owner_interval_observation'].update(all_watchpoints_hardware=False)),
    ('incomplete-disarm',lambda obs:obs['owner_interval_observation'].update(all_watchpoints_disabled=False)),
    ('changed-protected-record',lambda obs:obs['owner_interval_observation']['records'][0].update(identity_address_cpu_store_changes=1)),
    ('wrong-copy-endpoint',lambda obs:obs['owner_interval_observation']['records'][0].update(copy_size=3)),
    ('missing-copy',lambda obs:obs['owner_interval_observation']['records'][0].update(copy_completed=False)),
    ('unrun-positive-control',lambda obs:obs['owner_interval_observation']['controls'][0].update(after=obs['owner_interval_observation']['controls'][0]['before'])),
    ('wrong-control-syscall',lambda obs:obs['owner_interval_observation']['controls'][0].update(epoch=[1,5])),
    ('missing-interval',lambda obs:obs['owner_interval_observation']['records'].pop()),
    ('permission-change',lambda obs:endpoint(obs,bad_permission)),
    ('physical-address-change',lambda obs:endpoint(obs,bad_address)),
    ('unrelated-status-set',lambda obs:endpoint(obs,bad_status)),
    ('entry-change',lambda obs:endpoint(obs,lambda snapshot:snapshot['epoch'].__setitem__(0,snapshot['epoch'][0]+1)))
]
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
    scope='Fourteen saved JSON controls; endpoint semantic controls rebind snapshot hashes. No ELF, GDB or VM execution, register edit or guest-memory mutation')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,controls=len(rows))))
