"""Bounded saved-artifact calendar audit; no VM or generated-code execution."""
import argparse
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'linux-kernel/tools/process-memory'))
from archive_publication import closed_archive
from readback_process import digest as fact
from run_policy_failure import hashes, watched
from calendar_data import inspect as tables, graph_exclusion
from inspect_calendar_data import FUNCTION, context, inspect
from inspect_call_inventory import Elf


DEPENDENCIES = {
    'linux-kernel/tools/user-memory/results/stage-3-paging-controls': '4620f05d40628d574e2d1fd8be3eb5131eb3d8764bc08547ccf9b8f318694046',
    'linux-kernel/tools/user-memory/results/stage-3-callee-graph': '45227f2173edcfad8a5e9d083dd0b2c0da05bd9bc49b553579a728941c9ebb3e',
    'linux-kernel/tools/user-memory/results/stage-3-boot-entries': '60aff8277ed4c9f2cac318a46a845a1a1ea30f628c9aa26c18f4d8bda36a5c6d',
    'linux-kernel/tools/process-memory/results/stage-3-exec-failures': '257079b0d45900342497fe7a3dd6d08ad82b2e30d90a9b67517a942731fd4bfb',
    'linux-kernel/tools/process-memory/results/stage-3-exec-signals': '335595b6c687e48f6606ba0f8da73d9bbbe284d38a0dcd9b294fd1b5167ea5e3',
}


def verify_dependency(folder, expected):
    raw = (folder / 'manifest.json').read_bytes()
    assert fact(raw)['sha256'] == expected
    value = json.loads(raw)
    if 'files' not in value:
        closed_archive(folder)
        return
    # The explicitly pinned historical paging archive predates flat manifests.
    assert value['passed'] is True
    files = value['files']
    assert {p.relative_to(folder).as_posix() for p in folder.rglob('*') if p.is_file()} == set(files) | {'manifest.json'}
    for name, expected_file in files.items():
        path = (folder / name).resolve()
        assert path.is_relative_to(folder) and fact(path.read_bytes()) == expected_file


def artifact_specs():
    old = ROOT / 'linux-kernel/tools/user-memory/results/stage-3-paging-controls'
    metadata = json.loads((old / 'inputs.json').read_text())
    specs = {}
    for name in ['kernel.elf', 'llvm.stdout.log', 'gnu.stdout.log']:
        group = 'artifacts' if name == 'kernel.elf' else 'records'
        row = metadata[group][name]
        specs['historical/' + name] = dict(path=str((old / row['snapshot']).resolve().relative_to(ROOT)),
            packed_sha256=row['snapshot_sha256'], sha256=row['sha256'], bytes=row['bytes'])
    pairs = {
        'kernel.elf': 'linux-kernel/tools/process-memory/results/stage-3-exec-failures/native/kernel.elf.gz',
        'provider.o': 'linux-kernel/tools/process-memory/results/stage-3-exec-failures/native/provider.o.gz',
        'process-byte-origins.json': 'linux-kernel/tools/process-memory/results/stage-3-exec-failures/native/process-byte-origins.json.gz',
        'graph.json': 'linux-kernel/tools/user-memory/results/stage-3-boot-entries/qualification/graph.json.gz',
        'llvm.stdout.log': 'linux-kernel/tools/user-memory/results/stage-3-callee-graph/matrix/llvm.stdout.log.gz',
        'gnu.stdout.log': 'linux-kernel/tools/user-memory/results/stage-3-callee-graph/matrix/gnu.stdout.log.gz',
    }
    for name, path in pairs.items():
        packed = (ROOT / path).read_bytes()
        raw = gzip.decompress(packed)
        specs['current/' + name] = dict(path=path, packed_sha256=fact(packed)['sha256'], **fact(raw))
    return specs


def unpack(spec):
    path = (ROOT / spec['path']).resolve()
    assert path.is_relative_to(ROOT / 'linux-kernel')
    packed = path.read_bytes()
    assert fact(packed)['sha256'] == spec['packed_sha256']
    raw = gzip.decompress(packed)
    assert fact(raw) == {k: spec[k] for k in ['bytes', 'sha256']}
    return raw


def inspect_case(folder, address, current):
    return inspect(folder / 'kernel.elf', address,
                   (folder / 'llvm.stdout.log').read_bytes(), (folder / 'gnu.stdout.log').read_bytes(),
                   folder / 'graph.json' if current else None,
                   folder / 'provider.o' if current else None,
                   folder / 'process-byte-origins.json' if current else None)


def controls(folder):
    elf = Elf(folder / 'kernel.elf')
    symbol, body = elf.function(FUNCTION)
    result = tables(body, symbol['value'])
    changes = [('calendar-byte', result['year_start'] - symbol['value'] + 105 * 16),
               ('year-maximum', result['year_consumer']['lea'] - symbol['value'] - 21),
               ('year-reference', result['year_consumer']['lea'] - symbol['value'] + 3)]
    records = []
    for name, at in changes:
        copy = bytearray(body)
        copy[at] ^= 1
        try:
            tables(copy, symbol['value'])
        except ValueError as error:
            records.append(dict(name=name, rejected=True, reason=str(error)))
        else:
            raise AssertionError('accepted changed calendar model bytes: ' + name)
    row = dict(address=45861106, size=1, bytes='cf')
    try:
        graph_exclusion([row], elf, result['year_start'], result['end'])
    except ValueError as error:
        records.append(dict(name='declared-entry-into-data', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted graph entry into data')
    return records


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
    sources = watched()
    before = hashes(sources)
    for path, sha in DEPENDENCIES.items():
        verify_dependency(ROOT / path, sha)
    specs = artifact_specs()
    for name, spec in specs.items():
        target = output / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(unpack(spec))
    report = dict(passed=False, source_hashes_before=before, dependencies=DEPENDENCIES,
                  artifacts=specs, cases={}, controls=controls(output / 'current'), commands={})
    llvm = json.loads((ROOT / 'linux-kernel/tools/user-memory/results/stage-3-paging-controls/llvm.command.json').read_text())['argv'][0]
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    for name, address in [('historical', 0x2bbaa5d), ('current', 45861106)]:
        folder = output / name
        report['cases'][name] = inspect_case(folder, address, name == 'current')
        elf = Elf(folder / 'kernel.elf')
        symbol, body = elf.function(FUNCTION)
        report['cases'][name]['explicit_start_decoders'] = {}
        for label, arguments in [('llvm', [llvm, '-d', '--x86-asm-syntax=intel']),
                                 ('gnu', ['objdump', '-d', '-M', 'intel', '--insn-width=16'])]:
            argv = ['rtk', 'proxy', *arguments, '--start-address=' + str(address),
                    '--stop-address=' + str(address + 1), str(folder / 'kernel.elf')]
            result = subprocess.run(argv, cwd=ROOT, env=environment, capture_output=True)
            (folder / (label + '-explicit.stdout.log')).write_bytes(result.stdout)
            (folder / (label + '-explicit.stderr.log')).write_bytes(result.stderr)
            record = dict(argv=argv, exit_code=result.returncode)
            (folder / (label + '-explicit.command.json')).write_text(json.dumps(record, indent=2) + '\n')
            assert result.returncode == 0
            rows = context(result.stdout, body, symbol['value'], address)
            assert len(rows) == 1 and rows[0]['address'] == address and rows[0]['bytes'] == 'cf'
            assert rows[0]['instruction'].split()[0] in ['iretd', 'iret']
            report['cases'][name]['explicit_start_decoders'][label] = rows[0]
    for label, directory in [('user', 'user-memory'), ('process', 'process-memory')]:
        argv = ['rtk', 'proxy', 'python3', '-m', 'unittest', 'discover', '-s', 'linux-kernel/tools/' + directory, '-p', 'test_*.py']
        result = subprocess.run(argv, cwd=ROOT, env=environment, capture_output=True)
        (output / (label + '.stdout.log')).write_bytes(result.stdout)
        (output / (label + '.stderr.log')).write_bytes(result.stderr)
        report['commands'][label] = dict(argv=argv, exit_code=result.returncode)
        assert result.returncode == 0, (label, result.stderr.decode())
    report['source_hashes_after'] = hashes(sources)
    report['passed'] = before == report['source_hashes_after']
    report['resources'] = {n: (resources / n).read_text().strip() for n in ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
    (output / 'run.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    assert report['passed']
    print(json.dumps(dict(passed=True, sources=len(sources), controls=len(report['controls']),
                         sites={k: v['site'] for k, v in report['cases'].items()}, resources=report['resources'])))


if __name__ == '__main__':
    main()
