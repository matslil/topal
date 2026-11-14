"""Bounded compiler image-layout validation; no guest or host code execution test."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'linux-kernel/tools/process-memory'))
from run_policy_failure import hashes, watched

COMMANDS = {
    'image-layout': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                     'systems_kernel_image_layout::tests', '--', '--test-threads=1', '--nocapture'],
    'default-ABI': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                    'default_abi_fixture_preserves_committed_generator_bytes'],
    'workspace': ['cargo', 'check', '--workspace', '--all-targets', '--locked', '-j1'],
    'format': ['cargo', 'fmt', '--all', '--', '--check'],
    'Rust-ceiling': ['python3', 'scripts/check_rust_file_sizes.py'],
    'user-models': ['python3', '-B', '-m', 'unittest', 'discover', '-s',
                    'linux-kernel/tools/user-memory', '-p', 'test_*.py'],
    'process-models': ['python3', '-B', '-m', 'unittest', 'discover', '-s',
                       'linux-kernel/tools/process-memory', '-p', 'test_*.py'],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    output = parser.parse_args().output.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()
    paths = sorted(set(watched()) | {str(Path(__file__).resolve().relative_to(ROOT))})
    before = hashes(paths)
    report = dict(passed=False, commands={}, source_hashes_before=before,
                  scope='Fresh compiler source/native reconstruction and descriptive image subdivision only; no VM, native activation, ownership or complete root sizing')
    environment = {k: v for k, v in os.environ.items() if not k.startswith('TOPAL_TEST_')}
    environment.update(RUST_MIN_STACK='16777216', PYTHONDONTWRITEBYTECODE='1',
                       RUST_TEST_THREADS='1', TMPDIR=str(temporary))
    began = time.monotonic()

    def save():
        report['resources'] = {name: (resources / name).read_text().strip() for name in
                               ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (output / 'run.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')

    try:
        for name, command in COMMANDS.items():
            started = time.monotonic()
            argv = ['rtk', 'proxy', *command]
            with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
                result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=out, stderr=err)
            report['commands'][name] = dict(argv=argv, exit_code=result.returncode,
                                            seconds=time.monotonic() - started)
            save()
            print(name, result.returncode, flush=True)
            assert result.returncode == 0, name
        text = (output / 'image-layout.stderr.log').read_text()
        fixtures = re.findall(r'kernel image layout fixture: ([^\n]+)', text)
        assert len(fixtures) == 1, fixtures
        fixture = Path(fixtures[0]).resolve()
        assert fixture.is_relative_to(temporary)
        shutil.copyfile(fixture / 'image-layout.json', output / 'image-layout.json')
        report['layout'] = json.loads((output / 'image-layout.json').read_text())
        report['passed'] = True
    except BaseException:
        report['failure'] = traceback.format_exc()
        print(report['failure'], flush=True)
    finally:
        report['source_hashes_after'] = hashes(paths)
        report['passed'] &= report['source_hashes_after'] == before
        report['seconds'] = time.monotonic() - began
        save()
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
