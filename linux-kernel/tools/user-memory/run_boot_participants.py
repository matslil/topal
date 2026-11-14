"""Run the TK-DEC-293 portable models in the repository's bounded scope.

This runs no VM, compiler, native startup or guest-state modification. The
receipt records model evidence only and hashes the watched inputs before/after.
"""
import hashlib
import json
import re
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
USER = ROOT / 'linux-kernel/tools/user-memory'
PROCESS = ROOT / 'linux-kernel/tools/process-memory'


def digest(path):
    data = path.read_bytes()
    return dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())


def main():
    output = Path(sys.argv[1]).resolve()
    if not output.is_relative_to(ROOT / 'linux-kernel/.cache/x86_64'):
        raise ValueError('output must be a fresh x86-64 lab cache directory')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    if 'topal-bounded-' not in group or (resources / 'memory.swap.max').read_text().strip() != '0':
        raise ValueError('use scripts/run_bounded.py with its no-swap scope')
    output.mkdir(exist_ok=False)
    lab = ROOT / 'linux-kernel/labs/qemu/x86_64'
    watched = sorted(set(USER.glob('*.py')) | set(PROCESS.glob('*.py'))
                     | set(lab.glob('*.py')) | set(lab.glob('*.t')) | {
        ROOT / path for path in (
            'docs/systems-profile.md', 'se/requirements.md', 'spec/systems-profile.md',
            'src/topal-compiler/se-requirements.md', 'AGENTS.md',
            'linux-kernel/design/kernel-image-protection.md',
            'linux-kernel/design/boot-processor-topology.md',
            'linux-kernel/design/boot-participant-arrival.md')})
    before = {str(path.relative_to(ROOT)): digest(path) for path in watched}
    record = dict(schema='topal.kernel.boot-participant-arrival.model/1',
                  scope='Portable protocol and memory-model regression only; provider inputs are assumptions',
                  native_execution=False, source_lowering=False, protected_context=False,
                  production_qualified=False, watched_inputs=before, commands=[], passed=False)
    start = time.monotonic()
    for label, directory in [('user-memory', USER), ('process-memory', PROCESS)]:
        argv = ['rtk', 'proxy', sys.executable, '-B', '-m', 'unittest', 'discover',
                '-s', str(directory.relative_to(ROOT)), '-p', 'test_*.py', '-v']
        log = output / f'{label}.log'
        command_start = time.monotonic()
        with log.open('w') as stream:
            result = subprocess.run(argv, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=False)
        text = log.read_text()
        count = re.findall(r'^Ran (\d+) tests? in ', text, re.MULTILINE)
        passed = bool(result.returncode == 0 and len(count) == 1
                      and re.search(r'^OK\s*$', text, re.MULTILINE))
        record['commands'].append(dict(label=label, argv=argv, exit_code=result.returncode,
                                       passed=passed,
                                       seconds=time.monotonic() - command_start,
                                       tests=int(count[-1]) if count else None,
                                       log=dict(path=str(log.relative_to(ROOT)), **digest(log))))
        print(f'{label}: exit={result.returncode}, tests={count[-1] if count else "missing"}', flush=True)
        if not passed:
            break
    after = {str(path.relative_to(ROOT)): digest(path) for path in watched}
    record['unchanged_inputs'] = before == after
    record['resources'] = {name: (resources / name).read_text().strip() for name in (
        'memory.max', 'memory.peak', 'memory.swap.max', 'memory.events')}
    record['seconds'] = time.monotonic() - start
    record['passed'] = (record['unchanged_inputs'] and len(record['commands']) == 2
                        and all(c['passed'] for c in record['commands']))
    (output / 'qualification.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    print(f'passed={record["passed"]}; unchanged_inputs={len(before)}; seconds={record["seconds"]:.3f}')
    return 0 if record['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
