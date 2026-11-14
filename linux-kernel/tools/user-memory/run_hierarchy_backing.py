"""Validate purpose-preserving source requests and their actual VM owner lifecycle."""
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
    parser.add_argument('--profile', choices=['reservation', 'root-owners'], default='reservation')
    args = parser.parse_args()
    output = args.output.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()

    def inputs():
        paths = set(watched())
        for base in ['src/topal-language/src', 'src/topal-semantics/src']:
            paths.update(p.relative_to(ROOT).as_posix() for p in (ROOT / base).rglob('*.rs'))
        paths.update(['docs/systems-profile.md', 'se/requirements.md', 'spec/systems-profile.md',
                      'src/topal-compiler/se-requirements.md',
                      'linux-kernel/design/kernel-image-protection.md',
                      'linux-kernel/design/decision-register.md'])
        return sorted(paths)

    before = hashes(inputs())
    environment = {name: value for name, value in os.environ.items()
                   if not name.startswith('TOPAL_TEST_') and name != 'TOPAL_PINNED_STATIC_USER_IMAGE'}
    environment.update(RUST_MIN_STACK='16777216', RUST_TEST_THREADS='1',
                       PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(temporary))
    report = dict(passed=False, profile=args.profile, commands={}, source_hashes_before=before,
                  scope='Provider-selected static-hierarchy request, checked IR/provenance, exact legacy five-page backing and ordinary source-built guest/exec lifetimes; no image-protected provider or production qualification')
    if args.profile == 'root-owners':
        report['scope'] = ('Compiler-derived bootstrap/process whole-owner purposes, exact root-object binding and '
                           'fresh-image backing requirements; ordinary native regression preserves existing mapping '
                           'and runtime validators. No larger native bank or protected context is qualified.')
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
        if args.profile == 'root-owners':
            command('whole-root-owners', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                         'systems_frame_ownership::tests', '--', '--test-threads=1', '--nocapture'])
            command('legacy-owner-guards', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                    'user_extent_owners_come_from_the_checked_root_without_changing_its_bytes',
                    '--', '--test-threads=1', '--nocapture'])
        else:
            command('source-and-model', ['cargo', 'test', '-p', 'topal-language', '-p', 'topal-semantics',
                                       '--lib', '--locked', '-j1'])
        command('native-lowering', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                    'systems_frame_target::root_tests::hierarchy_backing',
                                    '--', '--test-threads=1', '--nocapture'])
        command('ordinary-guest', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                  'provider_selected_hierarchy_runs_ordinary_guest_and_exec_lifetimes',
                                  '--', '--ignored', '--test-threads=1', '--nocapture'])
        native = build('ordinary-guest')
        observations = json.loads((native / 'observations.json').read_text())
        assert observations['passed'] and observations['native_execution']
        assert observations['native_execution_attempted'] and observations['completed_calls'] == 4
        assert observations['valid_requests'] == 16 and observations['refused_requests'] == 92
        assert observations['protected_context'] is False
        root = (native / 'root.t').read_text()
        assert 'backing is provider-selected' in root and 'frame-count is 5,' not in root
        binding = json.loads((native / 'private-image-binding.json').read_text())
        assert binding['production_qualified'] is False and binding['protected_context'] is False
        identity = hashlib.sha256((native / 'kernel.elf').read_bytes()).hexdigest()
        assert identity == observations['kernel_sha256'] == binding['layout']['kernel_sha256']
        report['native'] = observations
        report['native_build'] = str(native.relative_to(ROOT))
        report['native_binding'] = dict(layout=binding['layout'], contributions=len(binding['placements']),
                                        production_qualified=False, protected_context=False)
        if args.profile == 'root-owners':
            requirements = binding['hierarchy_backing_requirements']
            owners = json.loads((native / 'root-backing-owners.json').read_text())
            root_identity = hashlib.sha256((native / 'root.o').read_bytes()).hexdigest()
            assert requirements['root_object_sha256'] == owners['root_object_sha256'] == root_identity
            assert requirements['kernel_sha256'] == identity
            assert requirements['root_token_bytes'] == owners['token_bytes']
            assert requirements['geometry_template'] == 'draft-x86-image-role-subdivision/1'
            for flag in ['native_selection_available', 'runtime_ownership_qualified',
                         'protected_context', 'production_qualified']:
                assert requirements[flag] is False
            banks = requirements['banks']
            assert [bank['domain'] for bank in banks] == ['bootstrap', 'process']
            assert [bank['current_owner']['purpose'] for bank in banks] == [
                'bootstrap-hierarchy', 'static-image-hierarchy']
            assert banks[0]['current_owner'] == owners['bootstrap']
            assert banks[1]['current_owner'] == owners['roles'][0]
            assert requirements['process_low_leaf'] == owners['roles'][1]
            required = 5 + binding['layout']['image_leaf_storage_bytes'] // 4096
            assert required > 5
            for bank in banks:
                assert bank['current_owner']['count'] == 5
                assert bank['current_owner']['alignment'] == bank['current_owner']['pins'] == 1
                assert bank['required_frame_count'] == required
                assert bank['required_bytes'] == required * 4096
                assert bank['required_alignment_frames'] == 1
                assert bank['whole_extent_size_matches'] is False
            report['hierarchy_backing_requirements'] = requirements
        for name, argv in COMMANDS.items():
            command(name, argv)
        command('legacy-private-build', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                'systems_frame_target::root_tests::process_mm_closed_link_reconstructs_complete_private_image',
                '--', '--exact', '--test-threads=1', '--nocapture'])
        legacy = build('legacy-private-build')
        legacy_identity = hashlib.sha256((legacy / 'kernel.elf').read_bytes()).hexdigest()
        assert identity == legacy_identity == '28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1'
        assert json.loads((legacy / 'build-only.json').read_text())['native_execution'] is False
        if args.profile == 'root-owners':
            legacy_binding = json.loads((legacy / 'private-image-binding.json').read_text())
            assert legacy_binding['hierarchy_backing_requirements'] == report['hierarchy_backing_requirements']
        report['legacy_private_build'] = dict(kernel_sha256=legacy_identity, native_execution=False)
        report['passed'] = True
    except BaseException:
        report['failure'] = traceback.format_exc()
        print(report['failure'], flush=True)
    finally:
        report['source_hashes_after'] = hashes(inputs())
        report['passed'] &= before == report['source_hashes_after']
        report['seconds'] = time.monotonic() - began
        save()
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
