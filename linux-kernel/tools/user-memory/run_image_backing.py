"""Validate source-bound inactive image backing inside the bounded test VM."""
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
    parser.add_argument('--tables', action='store_true', help='Construct and verify both inactive table banks and the separate low USER leaf')
    args = parser.parse_args()
    output = args.output.resolve()
    tables = args.tables
    role = 'image-tables' if tables else 'image-backing'
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
                      'linux-kernel/design/kernel-image-backing-preparation.md',
                      'linux-kernel/design/inactive-image-table-construction.md',
                      'linux-kernel/design/kernel-image-boot-binding.md',
                      'linux-kernel/design/boot-processor-topology.md',
                      'linux-kernel/design/inactive-boot-coverage.md',
                      'linux-kernel/design/decision-register.md',
                      'linux-kernel/tools/user-memory/image_backing_source.rs'])
        return sorted(paths)

    before = hashes(inputs())
    environment = {k: v for k, v in os.environ.items()
                   if not k.startswith('TOPAL_TEST_') and k != 'TOPAL_PINNED_STATIC_USER_IMAGE'}
    environment.update(RUST_MIN_STACK='16777216', RUST_TEST_THREADS='1',
                       PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(temporary))
    report = dict(passed=False, commands={}, source_hashes_before=before,
                  scope='Actual final-image-sized whole owned inactive banks, native initialization/zero/erase/release and ordinary guest application completion; no active protected hierarchy or production qualification')
    if tables:
        report['scope'] = 'Actual boot bytes, bounded retained ACPI processor discovery, independent native identity/memory-attribute reads, entry controls and retained hardware state bound to inactive bootstrap/process table construction; separate low USER leaf, whole erase/release, ordinary native guest and previous backing-only regression; inventory/snapshots grant no participation, activation, all-CPU lifetime or production qualification'
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

    def build(name, pattern):
        text = (output / (name + '.stdout.log')).read_text() + (output / (name + '.stderr.log')).read_text()
        matches = re.findall(pattern + r': ([^\n]+)', text)
        assert len(matches) == 1, matches
        path = Path(matches[0].strip()).resolve()
        assert path.is_relative_to(ROOT / 'linux-kernel')
        return path

    save()
    try:
        command('source-and-model', ['cargo', 'test', '-p', 'topal-language', '-p', 'topal-semantics',
                                    '--lib', '--locked', '-j1'])
        command('backing-component', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                     'systems_image_backing::', '--', '--test-threads=1', '--nocapture'])
        command('ordinary-guest', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                  ('image_tables_construct_and_discard_both_final_image_banks_with_separate_low_leaf' if tables else
                                   'kernel_image_backing_allocates_and_discards_actual_image_sized_banks'),
                                  '--', '--ignored', '--test-threads=1', '--nocapture'])
        native = build('ordinary-guest', 'final inactive image tables candidate' if tables else 'final image-backing preparation candidate')
        command('native-readback', ['python3', '-B',
                                  'linux-kernel/tools/user-memory/readback_image_tables.py' if tables else
                                  'linux-kernel/tools/user-memory/readback_image_backing.py', str(native)])
        observations = json.loads((native / (role+'-observations.json')).read_text())
        binding = json.loads((native / 'private-image-binding.json').read_text())
        plan = json.loads((native / (role+'-plan-binding.json')).read_text())
        component = json.loads((native / (role+'-component.json')).read_text())
        identity = hashlib.sha256((native / 'kernel.elf').read_bytes()).hexdigest()
        assert observations['passed'] and observations['native_execution']
        assert observations['native_execution_attempted'] and observations['banks'] == 2
        assert identity == observations['kernel_sha256'] == plan['kernel_sha256'] == binding['layout']['kernel_sha256']
        assert plan['layout'] == binding['layout'] and plan['matched_final_partition_and_template' if tables else 'preparation_sizing_matched']
        required = 5 + binding['layout']['image_leaf_storage_bytes'] // 4096
        if tables:
            from inactive_image_tables import page_count
            required = page_count(binding['layout'])
        assert observations['pages_per_bank'] == required > 5
        assert observations['zeroed_bytes'] == observations['discarded_bytes'] == (required*2+int(tables))*4096
        assert all(item['protected_context'] is False and item['production_qualified'] is False
                   for item in [observations, binding, plan, component])
        report.update(native=observations, native_build=str(native.relative_to(ROOT)),
                      readback=json.loads((native / (role+'-readback.json')).read_text()),
                      image_layout=binding['layout'], component=component,
                      original_inputs=binding['original_inputs'], contributions=len(binding['placements']))
        if tables:
            boot=report['readback']['boot_binding']
            assert boot['passed'] and boot['cache_indices_preserved']
            assert boot['entry_table_words']==2048 and boot['idt_words']==512 and boot['gdt_words']==6
            assert boot['cpus_observed']==2 and boot['boot_snapshots']==2
            assert boot['independently_observed_msr_reads']==2*(5+2*boot['variable_mtrrs']+boot['fixed_mtrrs'])
            assert boot['idt_pointer_bytes']==10 and len(boot['rejected_offline_controls'])==23
            report['boot_binding']=json.loads((native/'kernel-image-boot-binding.json').read_text())
            inventory=report['readback']['processor_inventory']
            assert inventory['passed'] and inventory['inventory_only']
            assert inventory['enabled']==2 and inventory['boot_snapshots']==2
            assert inventory['independent_identity_reads']==4 and len(inventory['rejected_offline_controls'])==24
            coverage=report['readback']['boot_coverage']
            assert coverage['passed'] and coverage['inactive_only']
            assert coverage['roles']==11 and coverage['views']==22
            assert coverage['native_completed_checks']==2 and len(coverage['rejected_offline_controls'])==22
        for name, argv in COMMANDS.items():
            command(name, argv)
        command('legacy-owners', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                                 'systems_frame_ownership::tests', '--', '--test-threads=1'])
        command('legacy-private-build', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                'systems_frame_target::root_tests::process_mm_closed_link_reconstructs_complete_private_image',
                '--', '--exact', '--test-threads=1', '--nocapture'])
        legacy = build('legacy-private-build', 'actual frame source root')
        legacy_identity = hashlib.sha256((legacy / 'kernel.elf').read_bytes()).hexdigest()
        assert legacy_identity == '28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1'
        assert json.loads((legacy / 'build-only.json').read_text())['native_execution'] is False
        report['legacy_private_build'] = dict(kernel_sha256=legacy_identity, native_execution=False)
        if tables:
            command('backing-regression', ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                    'kernel_image_backing_allocates_and_discards_actual_image_sized_banks',
                    '--', '--ignored', '--test-threads=1', '--nocapture'])
            backing = build('backing-regression','final image-backing preparation candidate')
            saved = json.loads((backing/'image-backing-observations.json').read_text())
            assert saved['passed'] and saved['native_execution']
            assert saved['kernel_sha256'] == '064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19'
            command('backing-readback', ['python3','-B','linux-kernel/tools/user-memory/readback_image_backing.py',str(backing)])
            report['backing_regression'] = saved
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
