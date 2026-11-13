"""Reproduce saved boot/IDT binding and the enlarged conditional entry graph.

Run through scripts/run_bounded.py. Inputs come from pinned existing archives;
no compiler, disassembler, VM or external target is invoked by this driver.
"""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback


ROOT = Path(__file__).resolve().parents[3]
TOOLS = ROOT / 'linux-kernel/tools/user-memory'
ARCHIVES = {
    'callee': ('linux-kernel/tools/user-memory/results/stage-3-callee-graph',
               '45227f2173edcfad8a5e9d083dd0b2c0da05bd9bc49b553579a728941c9ebb3e'),
    'native': ('linux-kernel/tools/process-memory/results/stage-3-interface-callers',
               '0d5697ba20e1ceddf5c12187daaa82b21f34872e1e04325edc5df020224cf7ea'),
}


def fact(raw):
    return dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


def read_archived(name, relative, receipts):
    folder, expected = ARCHIVES[name]
    manifest_raw = (ROOT / folder / 'manifest.json').read_bytes()
    assert fact(manifest_raw)['sha256'] == expected, 'upstream manifest identity'
    manifest = json.loads(manifest_raw)
    path = ROOT / folder / relative
    raw = path.read_bytes()
    assert fact(raw) == manifest[relative], 'upstream compressed input identity'
    content = gzip.decompress(raw)
    receipts[path.relative_to(ROOT).as_posix()] = dict(compressed=fact(raw), content=fact(content),
                                                      manifest_sha256=expected)
    return content


def watched():
    result = []
    for folder in [TOOLS, ROOT / 'linux-kernel/tools/process-memory']:
        result += list(folder.glob('*.py'))
    return {str(path.relative_to(ROOT)): fact(path.read_bytes()) for path in sorted(result)}


def run(output):
    output = output.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel/.cache'), 'keep replay work under the kernel cache'
    output.mkdir(parents=True, exist_ok=False)
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    directory = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (directory / 'memory.swap.max').read_text().strip() == '0'

    def resources():
        return {name: (directory / name).read_text().strip() for name in
                ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}

    report = dict(passed=False, upstream_inputs={}, source_hashes_before=watched(), commands={},
                  controls=[], group=group, resource_samples=[resources()],
                  scope='Fresh saved-byte readback and model suites, using pinned prior native/listing inputs. No native run, compiler/disassembler rerun or production qualification.')
    before = time.monotonic()
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    (output / 'driver.py').write_bytes(Path(__file__).read_bytes())

    def save():
        report['resource_samples'].append(resources())
        (output / 'run.json').write_text(json.dumps(report, indent=2) + '\n')

    def command(label, arguments):
        argv = ['rtk', 'proxy', *map(str, arguments)]
        started = time.monotonic()
        with (output / (label + '.stdout.log')).open('w') as out, (output / (label + '.stderr.log')).open('w') as err:
            result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=out, stderr=err)
        report['commands'][label] = dict(argv=argv, exit_code=result.returncode,
                                       elapsed_seconds=time.monotonic() - started)
        save()
        assert result.returncode == 0, 'failed ' + label

    failure = 0
    try:
        build = output / 'build'
        build.mkdir()
        for name in ['kernel.elf', 'process-access-inventory.json', 'callback-bindings.json']:
            (build / name).write_bytes(read_archived('callee', 'native/' + name + '.gz', report['upstream_inputs']))
        image = read_archived('native', 'native/poll/bzImage.gz', report['upstream_inputs'])
        (build / 'bzImage').write_bytes(image)
        for name in ['llvm', 'gnu', '45864290-llvm', '45864290-gnu', '45841237-llvm', '45841237-gnu']:
            (output / (name + '.log')).write_bytes(read_archived('callee', 'matrix/' + name + '.stdout.log.gz', report['upstream_inputs']))
        command('boot', ['python3', '-B', TOOLS / 'inspect_boot_entries.py', build / 'kernel.elf',
                         build / 'bzImage', output / 'boot.json'])
        # Actual-image counterexamples are offline copies only; never execute.
        from inspect_boot_entries import inspect
        boot = json.loads((output / 'boot.json').read_bytes())
        assert boot['kernel_sha256'] == 'b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c'
        setup = boot['header']['setup_bytes']
        bss = next(row for row in boot['load_segments'] if row['memory_bytes'] > row['file_bytes'])
        changes = [
            ('wrong-size', 0x1f4, 1), ('wrong-protocol', 0x206, 1),
            ('wrong-protected-entry', 0x214, 1), ('wrong-root', setup + 25, 1),
            ('wrong-stack', setup + 11, 1), ('wrong-far-selector', setup + 62, 1),
            ('wrong-idtr-operand', setup + boot['transition']['lidt_address'] - 0x100000 - 8, 1),
            ('wrong-kernel-handoff', setup + boot['transition']['bytes'] - 10, 1),
            ('wrong-idt-target', setup + 0x5000 + 3 * 16, 1),
            ('user-callable-gate', setup + 0x5000 + 14 * 16 + 5, 0x60),
            ('unexpected-idt-vector', setup + 0x5000 + 16, 1),
            ('nonzero-bss', setup + bss['address'] - 0x100000 + bss['file_bytes'], 1),
            ('hidden-user-paging-grant', setup + 0x1000, 4),
        ]
        for label, offset, mask in changes:
            assert 0 <= offset < len(image)
            altered = bytearray(image)
            altered[offset] ^= mask
            try:
                inspect(build / 'kernel.elf', altered)
            except (ValueError, AssertionError) as error:
                report['controls'].append(dict(name=label, file_offset=offset, xor_mask=mask,
                                               rejected=True, reason=str(error)))
            else:
                raise AssertionError('accepted image counterexample ' + label)
        del image, altered
        graph_command = ['python3', '-B', TOOLS / 'inspect_callee_graph.py', build,
                         output / 'llvm.log', output / 'gnu.log', output / 'graph.json',
                         '--boot-image', build / 'bzImage']
        for address in [45864290, 45841237]:
            graph_command += ['--recovery', str(address), output / (str(address) + '-llvm.log'),
                              output / (str(address) + '-gnu.log')]
        command('graph', graph_command)
        graph_raw = (output / 'graph.json').read_bytes()
        graph = json.loads(graph_raw)
        assert graph['passed'] and graph['boot_entries'] == boot and not graph['qualification_complete']
        assert len(graph['roots']) == 19 and graph['selected_edge_set_closed'] and not graph['frontiers']
        assert graph['reached_instructions'] == 206533 and graph['reached_bytes'] == 1230917
        report['results'] = dict(boot=fact((output / 'boot.json').read_bytes()), graph=fact(graph_raw),
                                reached_instructions=graph['reached_instructions'], reached_bytes=graph['reached_bytes'],
                                roots=len(graph['roots']), functions=len(graph['functions']),
                                tables=len(graph['guarded_tables']),
                                table_entries=sum(len(row['targets']) for row in graph['guarded_tables']),
                                helpers=graph['process_helpers'],
                                previously_unmatched_control_reached=any(row['address'] == 45861106 for row in graph['instructions']),
                                qualification_complete=False)
        del graph_raw, graph
        for label, folder in [('user-memory', 'user-memory'), ('process-memory', 'process-memory')]:
            command(label, ['python3', '-B', '-m', 'unittest', 'discover', '-s',
                            ROOT / 'linux-kernel/tools' / folder, '-p', 'test_*.py'])
    except BaseException:
        failure = 1
        report['failure'] = traceback.format_exc()
    finally:
        report['source_hashes_after'] = watched()
        report['elapsed_seconds'] = time.monotonic() - before
        report['passed'] = not failure and report['source_hashes_before'] == report['source_hashes_after']
        save()
    print(json.dumps({key: report.get(key) for key in ['passed', 'elapsed_seconds', 'results', 'failure']}))
    return failure or int(not report['passed'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    raise SystemExit(run(parser.parse_args().output))
