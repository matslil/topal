"""Validate inactive table-byte preparation and its ordinary userspace VM test."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'linux-kernel/tools/process-memory'))
from run_policy_failure import hashes, watched
from run_compiler_image_layout import COMMANDS


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
    paths = watched()
    before = hashes(paths)
    environment = {name: value for name, value in os.environ.items()
                   if not name.startswith('TOPAL_TEST_') and name != 'TOPAL_PINNED_STATIC_USER_IMAGE'}
    environment.update(RUST_MIN_STACK='16777216', RUST_TEST_THREADS='1',
                       PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(temporary))
    report = dict(passed=False, commands={}, source_hashes_before=before,
                  scope='Inactive x86 bank geometry and byte preparation, seven static tests and an ordinary source-built guest program; initializer output remains application data, not kernel translation authority or effective image protection')
    began = time.monotonic()

    def save():
        report['resources'] = {name: (resources / name).read_text().strip() for name in
                               ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (output / 'run.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')

    def command(name, argv):
        started = time.monotonic()
        report['commands'][name] = dict(argv=['rtk', 'proxy', *argv], state='running')
        save()
        print(name, 'running', flush=True)
        with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
            result = subprocess.run(['rtk', 'proxy', *argv], cwd=ROOT, env=environment,
                                    stdout=out, stderr=err)
        report['commands'][name].update(state='finished', exit_code=result.returncode,
                                        seconds=time.monotonic() - started)
        save()
        print(name, result.returncode, flush=True)
        assert result.returncode == 0, name

    def build(name):
        text = (output / (name + '.stdout.log')).read_text() + (output / (name + '.stderr.log')).read_text()
        matches = re.findall(r'actual frame source root: ([^\n]+)', text)
        assert len(matches) == 1, matches
        path = Path(matches[0].strip()).resolve()
        assert path.is_relative_to(ROOT / 'linux-kernel')
        return path

    save()
    try:
        command('table-data', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                              'systems_kernel_image_layout::translation_tables::tests', '--',
                              '--include-ignored', '--test-threads=1', '--nocapture'])
        text = (output / 'table-data.stdout.log').read_text()
        assert '8 passed; 0 failed; 0 ignored' in text
        native = build('table-data')
        observations = json.loads((native / 'observations.json').read_text())
        assert observations['passed'] and observations['native_execution']
        assert observations['native_execution_attempted']
        assert observations['completed_calls'] == 4
        assert observations['valid_requests'] == 16 and observations['refused_requests'] == 92
        assert observations['protected_context'] is False
        binding = json.loads((native / 'private-image-binding.json').read_text())
        assert binding['production_qualified'] is False and binding['protected_context'] is False
        assert binding['layout']['kernel_sha256'] == observations['kernel_sha256']
        assert hashlib.sha256((native / 'kernel.elf').read_bytes()).hexdigest() == observations['kernel_sha256']
        report['native'] = observations
        report['native_build'] = str(native.relative_to(ROOT))
        report['native_binding'] = dict(layout=binding['layout'], contributions=len(binding['placements']),
                                        production_qualified=False, protected_context=False)
        save()
        for name, argv in COMMANDS.items():
            command(name, argv)
        command('legacy-private-build', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                'systems_frame_target::root_tests::process_mm_closed_link_reconstructs_complete_private_image',
                '--', '--exact', '--test-threads=1', '--nocapture'])
        legacy = build('legacy-private-build')
        identity = hashlib.sha256((legacy / 'kernel.elf').read_bytes()).hexdigest()
        assert identity == '28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1'
        assert json.loads((legacy / 'build-only.json').read_text())['native_execution'] is False
        report['legacy_private_build'] = dict(kernel_sha256=identity, native_execution=False,
                scope='Fresh original-profile compile/link identity; its preceding 51-workload native evidence retains its own source snapshot and scope')
        report['passed'] = True
    except BaseException:
        report['failure'] = traceback.format_exc()
        print(report['failure'], flush=True)
    finally:
        report['source_hashes_after'] = hashes(watched())
        report['passed'] &= before == report['source_hashes_after']
        report['seconds'] = time.monotonic() - began
        save()
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
