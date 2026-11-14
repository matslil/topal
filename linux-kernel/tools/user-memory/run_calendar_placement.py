"""Validate separated provider data with bounded static and ordinary VM checks."""
import argparse
import gzip
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
from archive_publication import closed_archive
from run_policy_failure import hashes, watched
from readback_process import digest as fact
from readback_exec_failures import readback as exec_readback
from readback_applications import facts as app_facts, readback as app_readback
from inspect_calendar_placement import REFERENCE, inspect


STATIC = {
    'format': ['cargo', 'fmt', '--all', '--', '--check'],
    'workspace': ['cargo', 'check', '--workspace', '--all-targets', '--locked', '-j1'],
    'calendar-object': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                        'systems_provider_object::tests::calendar_constants_are_data_with_bound_rip_references', '--', '--exact'],
    'provider-closure': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                         'systems_provider_object::tests::emits_a_closed_relocatable_elf64_provider_object', '--', '--exact'],
    'provider-storage': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                         'systems_provider_object::tests::retains_checked_storage_and_provider_provenance', '--', '--exact'],
    'default-ABI': ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1', 'default_abi_fixture_preserves_committed_generator_bytes'],
    'Rust-ceiling': ['python3', 'scripts/check_rust_file_sizes.py'],
    'user-models': ['python3', '-m', 'unittest', 'discover', '-s', 'linux-kernel/tools/user-memory', '-p', 'test_*.py'],
    'process-models': ['python3', '-m', 'unittest', 'discover', '-s', 'linux-kernel/tools/process-memory', '-p', 'test_*.py'],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel')
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    output.mkdir(parents=True, exist_ok=False)
    temporary = output / 'temporary'
    temporary.mkdir()
    paths = watched()
    before = hashes(paths)
    reference = output / 'reference'
    reference.mkdir()
    archive = ROOT / Path(REFERENCE).parent
    assert fact(closed_archive(archive))['sha256'] == '257079b0d45900342497fe7a3dd6d08ad82b2e30d90a9b67517a942731fd4bfb'
    reference_facts = {}
    for name in ['kernel.elf', 'provider.o', 'process-byte-origins.json']:
        raw = gzip.decompress((ROOT / REFERENCE / (name + '.gz')).read_bytes())
        (reference / name).write_bytes(raw)
        reference_facts[name] = fact(raw)
    environment = {k: v for k, v in os.environ.items() if not k.startswith('TOPAL_TEST_')}
    environment.update(RUST_MIN_STACK='16777216', PYTHONDONTWRITEBYTECODE='1', RUST_TEST_THREADS='1', TMPDIR=str(temporary))
    report = dict(passed=False, source_hashes_before=before, commands={}, native_builds={},
                  reference_artifacts=reference_facts, scope='Provider calendar data placement, selected compiler checks, ordinary child exec and pinned 51-workload corpus; not full compiler or public kernel qualification')
    def save():
        report['resources'] = {n: (resources / n).read_text().strip() for n in ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (output / 'run.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    def command(name, arguments, extra=None):
        began = time.monotonic()
        argv = ['rtk', 'proxy', *map(str, arguments)]
        with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
            result = subprocess.run(argv, cwd=ROOT, env=dict(environment, **(extra or {})), stdout=out, stderr=err)
        report['commands'][name] = dict(argv=argv, environment=extra or {}, exit_code=result.returncode, seconds=time.monotonic() - began)
        save()
        print(name, result.returncode, flush=True)
        assert result.returncode == 0, name
    def native(name, test, extra=None):
        command(name, ['cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                      'systems_frame_target::root_tests::' + test,
                      '--', '--ignored', '--exact', '--test-threads=1', '--nocapture'], extra)
        text = (output / (name + '.stdout.log')).read_text() + (output / (name + '.stderr.log')).read_text()
        builds = [Path(p.strip()).resolve() for p in re.findall(r'actual frame source root: ([^\n]+)', text)]
        assert len(builds) == 1, builds
        build = builds[0]
        report['native_builds'][name] = str(build.relative_to(ROOT))
        result = inspect(build, reference)
        (output / (name + '-placement.json')).write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
        report.setdefault('placement', {})[name] = result
        save()
        return build
    began = time.monotonic()
    save()
    try:
        for name, argv in STATIC.items():
            command(name, argv)
        build = native('child-exec', 'process_mm_child_exec_preserves_inactive_parent_and_fresh_child_heap')
        result = exec_readback(json.loads((build / 'process-mm-users/observations.json').read_bytes()))
        (output / 'child-exec-readback.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
        prior = json.loads((ROOT / 'linux-kernel/.cache/x86_64/process-mm-applications-corpus/run.json').read_text())
        pinned = Path(prior['environment']['TOPAL_PINNED_STATIC_USER_IMAGE'])
        assert pinned.resolve().is_relative_to(ROOT / 'linux-kernel')
        build = native('applications', 'process_mm_preserves_pinned_busybox_workloads',
                       {'TOPAL_PINNED_STATIC_USER_IMAGE': str(pinned)})
        binding = app_facts(build)
        batch = json.loads((build / 'process-mm-applications/observations.json').read_bytes())
        assert batch['passed'] and batch['complete_corpus'] and len(batch['workloads']) == 51
        summaries = {}
        for name in batch['selected']:
            folder = build / 'process-mm-applications' / name
            summaries[name] = app_readback(json.loads((folder / 'observations.json').read_bytes()),
                json.loads((folder / 'process-mm-observations.json').read_bytes()),
                (folder / 'serial.log').read_bytes(), name, binding)
        report['applications'] = dict(passed=True, complete_corpus=True, workloads=len(summaries), facts=binding, summaries=summaries)
        assert report['placement']['applications']['kernel_sha256'] == report['placement']['child-exec']['kernel_sha256']
        report['passed'] = True
    except BaseException:
        report['failure'] = traceback.format_exc()
        print(report['failure'], flush=True)
    finally:
        report['source_hashes_after'] = hashes(paths)
        report['passed'] &= before == report['source_hashes_after']
        report['seconds'] = time.monotonic() - began
        save()
    print(json.dumps({k: report.get(k) for k in ['passed', 'seconds', 'native_builds', 'failure']}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
