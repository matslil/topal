"""Retain scoped post-validation results under the ordinary resource wrapper."""
import json
from pathlib import Path
import subprocess
import sys
import time

work = Path('linux-kernel/.cache/x86_64') / ('tk283-startup-post-'+str(time.time_ns()))
work.mkdir()
print('post-validation record:',work,flush=True)
commands = [
    ['cargo','test','-p','topal-compiler','--lib','--locked','-j1',
     'systems_user_extent::source_fixture::','--','--test-threads=1','--nocapture'],
    ['cargo','fmt','--all','--','--check'],
    ['rustfmt','--check','--edition','2024',
     'src/topal-compiler/src/systems_artifact_static_image.rs',
     'src/topal-compiler/src/systems_user_source_fixture.rs'],
    ['cargo','check','--workspace','--all-targets','--locked','-j1'],
    ['python3','scripts/check_rust_file_sizes.py'],
    ['git','-c','core.whitespace=blank-at-eol,blank-at-eof,space-before-tab','diff','--check'],
]
results = []
for index,command in enumerate(commands):
    start = time.monotonic()
    with (work/f'{index}.log').open('w') as log:
        result = subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
    results.append(dict(command=command,exit_code=result.returncode,elapsed_seconds=time.monotonic()-start))
    print(index,result.returncode,command,flush=True)
    if result.returncode:
        print((work/f'{index}.log').read_text(),flush=True)
        break
group = next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
directory = Path('/sys/fs/cgroup'+group)
resources = {name:(directory/name).read_text().strip() for name in ['memory.max','memory.swap.max','memory.peak','memory.events']}
passed = len(results) == len(commands) and all(r['exit_code'] == 0 for r in results)
(work/'validation.json').write_text(json.dumps(dict(passed=passed,results=results,resources=resources),indent=2)+'\n')
print(json.dumps(dict(passed=passed,resources=resources)),flush=True)
sys.exit(0 if passed else 1)
