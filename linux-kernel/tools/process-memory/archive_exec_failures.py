#!/usr/bin/env python3
"""Freeze failed-exec records over the explicitly pinned signal-stage archive."""
import argparse
import gzip
import json
import os
from pathlib import Path
import subprocess

from archive_process import ROOT
from archive_publication import closed_archive
from readback_exec_failures import readback
from readback_process import digest
from run_policy_failure import hashes

BASE = 'linux-kernel/tools/process-memory/results/stage-3-exec-signals'
BASE_MANIFEST = '335595b6c687e48f6606ba0f8da73d9bbbe284d38a0dcd9b294fd1b5167ea5e3'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['run', 'output', 'replay']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    run, output, replay = [p.resolve() for p in [args.run, args.output, args.replay]]
    assert all(p.is_relative_to(ROOT / 'linux-kernel') for p in [run, output, replay])
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    base = ROOT / BASE
    assert digest(closed_archive(base))['sha256'] == BASE_MANIFEST
    base_report = json.loads((base / 'report.json').read_text())
    qualification = json.loads((run / 'run.json').read_text())
    assert qualification['passed'] and qualification['source_hashes_before'] == qualification['source_hashes_after']
    sources = dict(qualification['source_hashes_before'])
    assert hashes(sources) == sources
    sources[str(Path(__file__).resolve().relative_to(ROOT))] = digest(Path(__file__).read_bytes())['sha256']
    output.mkdir(parents=True, exist_ok=False)
    manifest, origins = {}, {}

    def save(name, raw):
        target = output / name
        assert target.resolve().is_relative_to(output) and name not in manifest
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        manifest[name] = digest(raw)

    def frozen(name, raw):
        save(name + '.gz', gzip.compress(raw, mtime=0))

    for name, sha in sources.items():
        if base_report['selected_inputs'].get(name) == sha:
            origins[name] = 'base'
            raw = gzip.decompress((base / ('inputs/' + name + '.gz')).read_bytes())
            assert digest(raw)['sha256'] == sha
        else:
            origins[name] = 'overlay'
            frozen('inputs/' + name, (ROOT / name).read_bytes())
    for path in sorted(run.glob('*')):
        if path.is_file():
            frozen('qualification/' + path.name, path.read_bytes())
    build = ROOT / qualification['native_build']
    for path in sorted(build.rglob('*')):
        if path.is_file() and 'baseline' not in path.relative_to(build).parts:
            frozen('native/' + str(path.relative_to(build)), path.read_bytes())
    for name, fact in qualification['artifacts'].items():
        assert digest((build / name).read_bytes()) == fact
    component = json.loads((build / 'source-component.json').read_text())
    assert component['linked_sha256'] == qualification['artifacts']['source-user.elf']['sha256']
    assert component['process_mm_exec_source_sha256'] == sources['linux-kernel/labs/qemu/x86_64/user_extent_process_mm_exec_cases.c']
    assert component['conservative_user_stack_bytes'] <= 2048
    observed = (build / 'process-mm-users/observations.json').read_bytes()
    actual = json.loads(json.dumps(readback(json.loads(observed))))
    expected = json.loads((run / 'readback.json').read_text())
    assert all(actual[k] == expected[k] for k in actual)
    report = dict(passed=True, schema='topal-process-mm-exec-failures-v1', summary=actual,
                  selected_inputs=sources, input_origins=origins,
                  dependency=dict(path=BASE, manifest_sha256=BASE_MANIFEST),
                  qualification=qualification, source_component=component, observation=digest(observed),
                  scope='Selected ordinary ENOENT/EFAULT returns and subsequent successful exec. Saved replay is not native execution or public qualification.')
    save('report.json', (json.dumps(report, indent=2, sort_keys=True) + '\n').encode())
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    closed_archive(output)
    replay.mkdir(parents=True, exist_ok=False)
    for name, sha in sources.items():
        archive = base if origins[name] == 'base' else output
        raw = gzip.decompress((archive / ('inputs/' + name + '.gz')).read_bytes())
        assert digest(raw)['sha256'] == sha
        target = replay / name
        assert target.resolve().is_relative_to(replay)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
    script = '''import gzip,json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from readback_exec_failures import readback
a=Path(sys.argv[1]);expected=json.loads((a/'report.json').read_text())
raw=gzip.decompress((a/'native/process-mm-users/observations.json.gz').read_bytes())
actual=json.loads(json.dumps(readback(json.loads(raw))))
assert actual==expected['summary']
Path('replayed.json').write_text(json.dumps(actual,indent=2,sort_keys=True)+'\\n')
print('Frozen failed-exec, signal, process and continuation checks passed')
'''
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    commands = [(['python3', '-c', script, str(output)], 'replay')]
    for folder in ['process-memory', 'user-memory']:
        commands.append((['python3', '-m', 'unittest', 'discover', '-s', 'linux-kernel/tools/' + folder, '-p', 'test_*.py'], folder))
    for argv, label in commands:
        result = subprocess.run(['rtk', 'proxy', *argv], cwd=replay, env=environment, capture_output=True)
        (replay / (label + '.stdout.log')).write_bytes(result.stdout)
        (replay / (label + '.stderr.log')).write_bytes(result.stderr)
        assert result.returncode == 0, (label, result.stderr.decode())
    assert hashes(sources) == sources
    review = dict(passed=True, entries=len(manifest), frozen_sources=len(sources),
                  base_inputs=list(origins.values()).count('base'), overlays=list(origins.values()).count('overlay'),
                  archive_manifest=digest((output / 'manifest.json').read_bytes()), dependency=report['dependency'],
                  summary=actual, replay='Record/control checks and 330 models; no VM or compiler execution',
                  resources={n: (resources / n).read_text().strip() for n in ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']})
    with output.with_name(output.name + '-review.json').open('x') as stream:
        stream.write(json.dumps(review, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: review[k] for k in ['passed', 'entries', 'frozen_sources', 'base_inputs', 'overlays', 'archive_manifest', 'resources']}))


if __name__ == '__main__':
    main()
