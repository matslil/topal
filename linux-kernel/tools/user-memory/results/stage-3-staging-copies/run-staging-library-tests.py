"""Record the full compiler suite with disk-backed temporary artifacts."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

root=Path.cwd()
work=root/'linux-kernel/.cache/x86_64'/('tk283-staging-library-'+str(time.time_ns()))
work.mkdir()
temporary=work/'temporary'
temporary.mkdir()
os.environ['TMPDIR']=str(temporary)
with (work/'workload-budget-preflight.log').open('w') as preflight_log:
    preflight=subprocess.run([sys.executable,'linux-kernel/.cache/x86_64/apply-full-workload-budget.py'],
        stdout=preflight_log,stderr=subprocess.STDOUT)
assert preflight.returncode==0,(work/'workload-budget-preflight.log').read_text()
paths=subprocess.check_output(['git','diff','--name-only'],text=True).splitlines()
paths+=subprocess.check_output(['git','ls-files','--others','--exclude-standard','--','linux-kernel','src'],text=True).splitlines()
def hashes(): return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths}
before=hashes()
command=['cargo','test','-p','topal-compiler','--lib','--locked','-j1','--','--test-threads=1']
print('library record:',work,flush=True)
start=time.monotonic()
with (work/'test.log').open('w') as handle:
    result=subprocess.run(command,stdout=handle,stderr=subprocess.STDOUT)
group=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
cgroup=Path('/sys/fs/cgroup'+group)
resources={name:(cgroup/name).read_text().strip() for name in ['memory.max','memory.swap.max','memory.peak','memory.events']}
after=hashes()
record=dict(command=command,exit_code=result.returncode,elapsed_seconds=time.monotonic()-start,
    temporary_directory=str(temporary),resources=resources,modified_source_hashes_before=before,
    modified_source_hashes_after=after,source_changes_during_run=before!=after)
(work/'run.json').write_text(json.dumps(record,indent=2)+'\n')
print((work/'test.log').read_text()[-12000:],flush=True)
print(json.dumps(record),flush=True)
sys.exit(result.returncode or (2 if before!=after else 0))
