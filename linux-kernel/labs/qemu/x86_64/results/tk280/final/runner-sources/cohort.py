"""Fresh TK280 affected VM cohort; every heavy child uses the normal wrapper."""
from pathlib import Path
import os
import subprocess
import sys

ROOT = Path('/home/mats/git/topal')
LAB = ROOT/'linux-kernel/labs/qemu/x86_64'
LOGS = Path('/tmp/topal-tk280-cohort')
LOGS.mkdir(exist_ok=True)
EXTERNAL = '/tmp/topal-busybox-elf.myVNMa/busybox'
ENVIRONMENT = ['RUST_MIN_STACK=16777216', 'RUST_TEST_THREADS=1', 'CARGO_BUILD_JOBS=1',
    'CARGO_PROFILE_TEST_DEBUG=0', 'CARGO_PROFILE_DEV_DEBUG=0',
    'CARGO_PROFILE_TEST_CODEGEN_UNITS=1024', 'CARGO_PROFILE_DEV_CODEGEN_UNITS=1024',
    'CARGO_INCREMENTAL=0', 'MALLOC_ARENA_MAX=1',
    'MALLOC_CONF=narenas:1,dirty_decay_ms:0,muzzy_decay_ms:0,background_thread:false,retain:false,tcache:false']
commands = [(name, ['python3', str(LAB/(name+'_gate.py')), '--write-evidence'])
    for name in ('file_region', 'region', 'positional', 'named', 'file_open', 'volatile',
                 'header', 'dispatch', 'identity', 'retained', 'structured')]
commands.insert(0, ('ram_page', ['python3', str(LAB/'ram_page_gate.py'), '--write-evidence']))
commands += [('corpus', ['python3', '/tmp/topal-current-corpus-reuse.py']),
             ('environments', ['python3', '/tmp/topal-empty-environments-evidence.py'])]
commands += [(name, ['python3', str(LAB/(name+'_gate.py')), '--write-evidence'])
    for name in ('numeric', 'library')]
commands += [(name, ['python3', str(LAB/(name+'_gate.py')), '--external-static-user-image',
    EXTERNAL, '--write-evidence']) for name in ('realtime', 'time', 'children', 'pair', 'pipes')]
commands += [('deadline', ['python3', str(LAB/'deadline_gate.py'),
    '--external-static-user-image', EXTERNAL, '--predecessor-artifact',
    '/tmp/topal-runtime-debug-qbzhvatz/artifact', '--write-evidence'])]
start = sys.argv[1] if len(sys.argv)>1 else None
if start: commands = commands[next(i for i, (name, _) in enumerate(commands) if name == start):]
for name, command in commands:
    path = LOGS/(name+'.log')
    attempt = 1
    while path.exists():
        attempt += 1
        path = LOGS/(name+'-attempt-'+str(attempt)+'.log')
    print('START '+name, flush=True)
    with path.open('w') as output:
        result = subprocess.run(['rtk', 'scripts/run_bounded.py', '--', 'env', *ENVIRONMENT, *command],
                                cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
    if result.returncode:
        print(path.read_text()[-14000:], flush=True)
        print('FAILED '+name+'; log '+str(path), flush=True)
        sys.exit(result.returncode)
    print('PASS '+name, flush=True)
(LOGS/'complete').write_text('All fresh TK280 affected-cohort commands passed.\n')
print('PASS complete fresh affected VM cohort', flush=True)
