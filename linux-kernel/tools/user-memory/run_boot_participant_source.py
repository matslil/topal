"""Bounded checked-source/IR qualification; participant native startup is refused."""
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
    output = Path(sys.argv[1]).resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel/.cache/x86_64')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()
    paths = set(watched())
    for base in ['src/topal-language/src', 'src/topal-semantics/src']:
        paths.update(str(p.relative_to(ROOT)) for p in (ROOT / base).rglob('*.rs'))
    paths.update(['docs/systems-profile.md', 'se/requirements.md', 'spec/systems-profile.md',
                  'src/topal-compiler/se-requirements.md',
                  'linux-kernel/design/decision-register.md',
                  'linux-kernel/design/boot-participant-arrival.md',
                  'linux-kernel/design/boot-participant-source.md',
                  'linux-kernel/tools/user-memory/boot_participant_source.rs'])
    before = hashes(sorted(paths))
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith('TOPAL_TEST_') and key != 'TOPAL_PINNED_STATIC_USER_IMAGE'}
    environment.update(RUST_MIN_STACK='16777216', RUST_TEST_THREADS='1',
                       PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(temporary))
    test = ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1']
    commands = {
        'source-and-model': ['cargo', 'test', '-p', 'topal-language', '-p', 'topal-semantics', '--lib', '--locked', '-j1'],
        'participant-boundaries': test + ['boot_participant', '--', '--test-threads=1'],
        'provider-regression': test + ['systems_provider::tests::', '--', '--test-threads=1'],
        'backing-regression': test + ['systems_image_backing::', '--', '--test-threads=1'],
        'legacy-owners': test + ['systems_frame_ownership::tests', '--', '--test-threads=1'],
        **COMMANDS,
        'included-format': ['rustfmt', '--check', '--edition', '2024',
            'linux-kernel/tools/user-memory/boot_participant_source.rs',
            'src/topal-language/src/systems_boot_participants.rs',
            'src/topal-language/src/systems_source_tests_boot_participants.rs',
            'src/topal-semantics/src/systems_boot_participants_model.rs',
            'src/topal-semantics/src/systems_model_tests_boot_participants.rs',
            'src/topal-compiler/src/systems_artifact_tests_boot_participants.rs'],
        'legacy-private-build': test + [
            'systems_frame_target::root_tests::process_mm_closed_link_reconstructs_complete_private_image',
            '--', '--exact', '--test-threads=1', '--nocapture'],
    }
    record = dict(schema='topal.kernel.boot-participant-source/1', passed=False,
                  scope='Checked source/IR and explicit native refusal; unchanged legacy image rebuilt without VM execution',
                  participant_native_execution=False, protected_context=False, production_qualified=False,
                  watched_inputs=before, commands={})
    started = time.monotonic()
    def save():
        record['resources'] = {name: (resources / name).read_text().strip() for name in (
            'memory.max', 'memory.peak', 'memory.swap.max', 'memory.events')}
        (output / 'qualification.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    save()
    try:
        for label, command in commands.items():
            log = output / f'{label}.log'
            argv = ['rtk', 'proxy', *command]
            record['commands'][label] = dict(argv=argv, state='running')
            save()
            began = time.monotonic()
            with log.open('w') as stream:
                result = subprocess.run(argv, cwd=ROOT, env=environment,
                                        stdout=stream, stderr=subprocess.STDOUT, check=False)
            text = log.read_text()
            tests = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)
            row = record['commands'][label]
            row.update(state='finished', exit_code=result.returncode, seconds=time.monotonic() - began,
                       test_counts=[list(map(int, values)) for values in tests],
                       log=dict(path=str(log.relative_to(ROOT)), bytes=log.stat().st_size,
                                sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
            save()
            print(label, result.returncode, tests, flush=True)
            assert result.returncode == 0, label
            if command[:2] == ['cargo', 'test']:
                assert tests and any(int(row[0]) > 0 for row in tests) and all(row[1] == '0' for row in tests), label
            if label == 'legacy-private-build':
                candidates = re.findall(r'actual frame source root: ([^\n]+)', text)
                assert len(candidates) == 1, candidates
                build = Path(candidates[0].strip()).resolve()
                assert build.is_relative_to(ROOT / 'linux-kernel')
                kernel = build / 'kernel.elf'
                identity = hashlib.sha256(kernel.read_bytes()).hexdigest()
                assert identity == '28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1'
                assert json.loads((build / 'build-only.json').read_text())['native_execution'] is False
                record['legacy_build'] = dict(path=str(build.relative_to(ROOT)), kernel_sha256=identity,
                    native_execution=False, bytes=kernel.stat().st_size)
        record['passed'] = True
    except BaseException:
        record['failure'] = traceback.format_exc()
        print(record['failure'], flush=True)
    finally:
        record['unchanged_inputs'] = before == hashes(sorted(paths))
        record['passed'] &= record['unchanged_inputs']
        record['seconds'] = time.monotonic() - started
        save()
    return 0 if record['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
