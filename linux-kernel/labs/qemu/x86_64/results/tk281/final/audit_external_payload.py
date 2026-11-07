from pathlib import Path
import gzip
import hashlib
import json
root=Path('/home/mats/git/topal/linux-kernel/labs/qemu/x86_64/results/tk281')
payload=Path('/tmp/topal-busybox-elf.myVNMa/busybox').read_bytes()
count=0
for path in root.rglob('*.gz'):
    assert payload not in gzip.decompress(path.read_bytes()),str(path)
    count+=1
print('PUBLISHED_BINARY_BOUNDARY_PASS objects='+str(count)+' dependency_sha256='+hashlib.sha256(payload).hexdigest(),flush=True)
