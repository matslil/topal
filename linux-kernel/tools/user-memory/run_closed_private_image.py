"""Qualify closed private image linking with ordinary child/BusyBox VM workloads."""
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
from readback_exec_failures import readback as exec_readback
from readback_applications import facts as app_facts, readback as app_readback
from run_compiler_image_layout import COMMANDS

PINNED = '7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--static-user-image', type=Path, required=True)
    args = parser.parse_args()
    output, pinned = args.output.resolve(), args.static_user_image.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel') and pinned.is_relative_to(ROOT / 'linux-kernel')
    assert hashlib.sha256(pinned.read_bytes()).hexdigest() == PINNED
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()
    paths = watched()
    before = hashes(paths)
    environment = {k: v for k, v in os.environ.items() if not k.startswith('TOPAL_TEST_')}
    environment.update(RUST_MIN_STACK='16777216', PYTHONDONTWRITEBYTECODE='1',
                       RUST_TEST_THREADS='1', TMPDIR=str(temporary))
    report = dict(passed=False, commands={}, builds={}, image_bindings={},
                  source_hashes_before=before, pinned_image=str(pinned), pinned_sha256=PINNED,
                  scope='Private static/local compilation and complete fresh object/link/allocation reconstruction, ordinary child exec and unchanged pinned BusyBox corpus; not native image protection or production qualification')
    began = time.monotonic()

    def save():
        report['resources'] = {name: (resources / name).read_text().strip() for name in
                               ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (output / 'run.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')

    def command(name, command, extra=None):
        started = time.monotonic()
        argv = ['rtk', 'proxy', *command]
        report['commands'][name] = dict(argv=argv, environment=extra or {}, state='running')
        save()
        print(name, 'running', flush=True)
        with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
            result = subprocess.run(argv, cwd=ROOT, env=dict(environment, **(extra or {})), stdout=out, stderr=err)
        report['commands'][name].update(state='finished', exit_code=result.returncode,
                                        seconds=time.monotonic() - started)
        save()
        print(name, result.returncode, flush=True)
        assert result.returncode == 0, name

    def private(name, test, native, extra=None):
        argv = ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                'systems_frame_target::root_tests::' + test, '--', '--exact', '--test-threads=1', '--nocapture']
        if native:
            argv.append('--ignored')
        command(name, argv, extra)
        text = (output / (name + '.stdout.log')).read_text() + (output / (name + '.stderr.log')).read_text()
        builds = [Path(value.strip()).resolve() for value in re.findall(r'actual frame source root: ([^\n]+)', text)]
        assert len(builds) == 1 and builds[0].is_relative_to(ROOT / 'linux-kernel'), builds
        build = builds[0]
        binding = json.loads((build / 'private-image-binding.json').read_text())
        records = json.loads((build / 'linked-inputs.json').read_text())
        controls = json.loads((build / 'private-image-controls.json').read_text())
        assert binding['production_qualified'] is False and binding['protected_context'] is False
        assert len(records) == 23 and len(binding['placements']) > 800
        assert len(binding['original_inputs']) == len(records)
        for original in binding['original_inputs']:
            assert hashlib.sha256((build / original['file']).read_bytes()).hexdigest() == original['sha256']
        assert controls == ['code', 'immutable', 'zero', 'omitted-library']
        assert binding['layout']['kernel_sha256'] == hashlib.sha256((build / 'kernel.elf').read_bytes()).hexdigest()
        assert (build / 'build-only.json').exists() == (not native)
        report['builds'][name] = str(build.relative_to(ROOT))
        report['image_bindings'][name] = dict(layout=binding['layout'], objects=len(records),
                                             contributions=len(binding['placements']), controls=controls,
                                             native_execution=native, original_inputs_unchanged=True)
        save()
        return build

    save()
    try:
        for name, argv in COMMANDS.items():
            command(name, argv)
        private('private-build', 'process_mm_closed_link_reconstructs_complete_private_image', False)
        build = private('child-exec', 'process_mm_child_exec_preserves_inactive_parent_and_fresh_child_heap', True)
        report['child_exec'] = exec_readback(json.loads((build / 'process-mm-users/observations.json').read_bytes()))
        assert report['child_exec']['passed']
        save()
        build = private('applications', 'process_mm_preserves_pinned_busybox_workloads', True,
                        {'TOPAL_PINNED_STATIC_USER_IMAGE': str(pinned)})
        facts = app_facts(build)
        batch = json.loads((build / 'process-mm-applications/observations.json').read_bytes())
        assert batch['passed'] and batch['complete_corpus'] and len(batch['workloads']) == 51
        summaries = {}
        for name in batch['selected']:
            folder = build / 'process-mm-applications' / name
            summaries[name] = app_readback(json.loads((folder / 'observations.json').read_bytes()),
                json.loads((folder / 'process-mm-observations.json').read_bytes()),
                (folder / 'serial.log').read_bytes(), name, facts)
        assert len(summaries) == 51
        report['applications'] = dict(passed=True, workloads=len(summaries), facts=facts, summaries=summaries)
        assert len({entry['layout']['kernel_sha256'] for entry in report['image_bindings'].values()}) == 1
        assert hashlib.sha256(pinned.read_bytes()).hexdigest() == PINNED
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
