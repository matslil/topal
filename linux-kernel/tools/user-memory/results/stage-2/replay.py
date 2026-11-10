#!/usr/bin/env python3
"""Restore the frozen first-party VM and observers into a fresh directory.

Run through scripts/run_bounded.py. This replays an unpublished component, not
canonical kernel publication or general application/container compatibility.
"""
from pathlib import Path
import argparse
import gzip
import hashlib
import json
import os
import subprocess
import sys

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('destination',type=Path)
parser.add_argument('--run',action='store_true',help='also run the positive native-extent VM')
args=parser.parse_args()
archive=Path(__file__).resolve().parent
r=json.loads((archive/'qualification.json').read_text())
assert r['complete'] and r['passed']
sha=lambda data:hashlib.sha256(data).hexdigest()
def read(info):
    packed=(archive/info['file']).read_bytes()
    assert sha(packed)==info['sha256']
    raw=gzip.decompress(packed)
    assert sha(raw)==info['uncompressed_sha256'] and len(raw)==info['bytes']
    return raw
destination=args.destination.resolve()
destination.mkdir(parents=True,exist_ok=False)
def restore(name,data):
    name=Path(name)
    assert not name.is_absolute() and '..' not in name.parts
    path=destination/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)

# Import all observer dependencies and the manifest at the same relative paths;
# paths recorded in old diagnostic scripts are not used as runtime authority.
for name,info in r['snapshots'].items():
    if ((name.startswith('linux-kernel/labs/qemu/x86_64/') and
         (name.endswith('.py') or name.endswith('/manifest.json')))
        or (name.startswith('linux-kernel/tools/user-memory/') and name.endswith('.py'))):
        restore(name,read(info))
guests=[c for c in r['components'] if c['label'].startswith('current-') and 'kernel.elf' in c['files']]
assert len(guests)==1
guest=guests[0]
for name in ['kernel.elf','bzImage','initramfs.cpio','root-tokens.json','user-owners.json']:
    restore('component/'+name,read(guest['files'][name]))
(destination/'restore.json').write_text(json.dumps(dict(
    qualification_sha256=sha((archive/'qualification.json').read_bytes()),
    component=guest['label'],native_calls_expected=324,
    scope='Restored native component and frozen observer sources; no rebuilding or external BusyBox needed'),indent=2)+'\n')
if args.run:
    relative=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    group=Path('/sys/fs/cgroup')/relative.lstrip('/')
    assert 'topal-bounded-' in relative and (group/'memory.swap.max').read_text().strip()=='0'
    observer=destination/'linux-kernel/labs/qemu/x86_64/user_extent_inspect.py'
    command=[sys.executable,str(observer),str(destination/'component')]
    env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
    p=subprocess.run(command,cwd=destination,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (destination/'replay.log').write_bytes(p.stdout)
    assert p.returncode==0,p.stdout.decode(errors='replace')[-6000:]
    observed=json.loads((destination/'component/user-extents/observations.json').read_text())
    assert observed['passed'] and len(observed['user_extents'])==324
    assert len(observed['memory'])==80 and observed['probe_counts']==[44,324,24,16]
    print('OWNED_USER_EXTENT_RESTORED_VM_PASS',destination)
else:
    print('RESTORED',destination)
