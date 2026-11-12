"""Observe the same unchanged guest artifacts with precise table-delta diagnostics."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

cache=Path('linux-kernel/.cache/x86_64')
original=cache/'user-extent-source-storage-2390370-1794459181759936035'
work=cache/'random-storage-mapping-diagnostic'
work.mkdir()
build=work/'build'
build.mkdir()
for path in original.iterdir():
    if path.is_file(): shutil.copy2(path,build/path.name)
identity=json.loads((cache/'random-storage-observer-first-failure/source-identity.json').read_text())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
before={name:digest(Path(name).read_bytes()) for name in identity}
command=[sys.executable,'linux-kernel/labs/qemu/x86_64/user_extent_source_inspect.py',str(build)]
start=time.monotonic()
with (work/'diagnostic.log').open('w') as handle:
    result=subprocess.run(command,stdout=handle,stderr=subprocess.STDOUT)
after={name:digest(Path(name).read_bytes()) for name in identity}
group=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
cgroup=Path('/sys/fs/cgroup'+group)
record=dict(exit_code=result.returncode,elapsed_seconds=time.monotonic()-start,
    kernel_sha256=digest((build/'kernel.elf').read_bytes()),copied_artifacts_equal=True,
    source_hashes_before=before,source_hashes_after=after,source_changes_during_run=before!=after,
    resources={name:(cgroup/name).read_text().strip() for name in ['memory.max','memory.swap.max','memory.peak','memory.events']},
    scope='Diagnostic replay of copied unchanged first-party guest artifacts with read-only observer delta recording; not a fresh producer or passing regression')
(work/'run.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(dict(exit_code=result.returncode,elapsed_seconds=record['elapsed_seconds'],
    source_changes_during_run=record['source_changes_during_run'])))
print((work/'diagnostic.log').read_text()[-1800:])
sys.exit(result.returncode)
