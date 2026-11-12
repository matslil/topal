"""Retain focused unchanged-application results and bounded resource evidence."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

root = Path.cwd()
temporary = root / 'linux-kernel/.cache/x86_64/compiler-temporaries'
temporary.mkdir(exist_ok=True)
os.environ['TMPDIR'] = str(temporary)
work = root / 'linux-kernel/.cache/x86_64' / ('tk283-source-run-' + str(time.time_ns()))
work.mkdir()
print('focused application record:', work, flush=True)
paths = subprocess.check_output(['git', 'diff', '--name-only'], text=True).splitlines()
paths += subprocess.check_output([
    'git', 'ls-files', '--others', '--exclude-standard', '--', 'linux-kernel', 'src'
], text=True).splitlines()
def hashes():
    return {path: hashlib.sha256((root / path).read_bytes()).hexdigest() for path in paths}

before = hashes()
command = ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
           'systems_frame_target::root_tests::owned_source_fixture_checks_native_user_buffer_boundaries',
           '--', '--ignored', '--exact', '--test-threads=1', '--nocapture']
start = time.monotonic()
with (work / 'test.log').open('w') as log:
    result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
             if line.startswith('0::'))
directory = Path('/sys/fs/cgroup' + group)
resources = {name: (directory / name).read_text().strip() for name in [
    'memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
after = hashes()
record = dict(command=command, exit_code=result.returncode,
              elapsed_seconds=time.monotonic() - start, resources=resources,
              modified_source_hashes_before=before, modified_source_hashes_after=after,
              source_changes_during_run=before != after,
              scope='Focused source-built ordinary user-buffer cases; no production or complete all-caller qualification')
(work / 'run.json').write_text(json.dumps(record, indent=2) + '\n')
print((work / 'test.log').read_text(), flush=True)
print(json.dumps(dict(exit_code=result.returncode, resources=resources,
                     source_changes_during_run=before != after)), flush=True)
sys.exit(result.returncode or (2 if before != after else 0))
