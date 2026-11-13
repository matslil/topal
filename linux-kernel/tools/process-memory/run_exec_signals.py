#!/usr/bin/env python3
"""Run the ordinary child-exec signal cohort inside the standard resource bound."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import traceback

from archive_process import ROOT
from run_policy_failure import hashes, watched


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--failed-exec', action='store_true', help='also check rejected exec state and continuation')
    args = parser.parse_args()
    output = args.output.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    directory = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (directory / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()
    paths = watched()
    before = hashes(paths)
    report = dict(passed=False, source_hashes_before=before, commands={},
                  scope='One fresh child-exec Q35 VM with ordinary application syscalls and read-only observations. No full compiler suite or public activation claim.')
    environment = {k: v for k, v in os.environ.items() if not k.startswith('TOPAL_TEST_')}
    environment.update(RUST_MIN_STACK='16777216', PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(temporary))

    def save():
        report['resources'] = {n: (directory / n).read_text().strip()
                               for n in ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (output / 'run.json').write_text(json.dumps(report, indent=2) + '\n')

    def command(name, arguments):
        began = time.monotonic()
        argv = ['rtk', 'proxy', *map(str, arguments)]
        with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
            result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=out, stderr=err)
        report['commands'][name] = dict(argv=argv, exit_code=result.returncode, elapsed_seconds=time.monotonic() - began)
        save()
        print(name, result.returncode, flush=True)
        assert result.returncode == 0, name

    began = time.monotonic()
    save()
    try:
        command('native', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                          'systems_frame_target::root_tests::process_mm_child_exec_preserves_inactive_parent_and_fresh_child_heap',
                          '--', '--ignored', '--exact', '--test-threads=1', '--nocapture'])
        text = (output / 'native.stdout.log').read_text() + (output / 'native.stderr.log').read_text()
        builds = [Path(p.strip()) for p in re.findall(r'actual frame source root: ([^\n]+)', text)]
        assert len(builds) == 1, builds
        build = builds[0].resolve()
        report['native_build'] = str(build.relative_to(ROOT))
        reader = 'readback_exec_failures.py' if args.failed_exec else 'readback_exec_signals.py'
        report['reader'] = reader
        command('readback', ['python3', ROOT / 'linux-kernel/tools/process-memory' / reader,
                             build / 'process-mm-users/observations.json', output / 'readback.json'])
        for label, folder in [('process-models', 'process-memory'), ('user-models', 'user-memory')]:
            command(label, ['python3', '-m', 'unittest', 'discover', '-s', 'linux-kernel/tools/' + folder, '-p', 'test_*.py'])
        report['artifacts'] = {name: dict(bytes=(build / name).stat().st_size,
                                         sha256=hashlib.sha256((build / name).read_bytes()).hexdigest())
                               for name in ['kernel.elf', 'bzImage', 'initramfs.cpio', 'source-component.json', 'source-user.elf']}
        report['passed'] = True
    except BaseException:
        report['failure'] = traceback.format_exc()
        print(report['failure'], flush=True)
    finally:
        report['source_hashes_after'] = hashes(paths)
        report['passed'] &= report['source_hashes_after'] == before
        report['elapsed_seconds'] = time.monotonic() - began
        save()
    print(json.dumps({k: report.get(k) for k in ['passed', 'native_build', 'elapsed_seconds', 'failure']}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
