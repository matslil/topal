"""Bounded saved-code qualification and retained diagnostic inventory.

No compiler, VM or debugger is invoked. Guard counterexamples alter Python byte
copies only. Historical native records
are producer assertions, kept distinct from fresh host interpretation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import traceback

from gateway_machine import ModelGap, Program
from gateway_contract import machine_inputs
from inspect_gateway_admission import cases, check, inspect, linked


ROOT = Path(__file__).resolve().parents[3]
TOOLS = Path(__file__).resolve().parent
MODES = ['return-site', 'gs-context', 'address-space', 'entry-identity', 'abi-number',
         'captured-stack', 'request', 'busy', 'direction-flag', 'partial-source',
         'published-unpin', 'no-fit']


def fact(raw):
    return dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


def watched():
    paths = set()
    for folder in [TOOLS, ROOT / 'linux-kernel/tools/process-memory']:
        paths.update(folder.glob('*.py'))
    paths.update(ROOT / name for name in [
        'src/topal-compiler/src/systems_frame_gateway.rs',
        'src/topal-compiler/src/systems_frame_gateway_memory.rs',
        'src/topal-compiler/src/systems_frame_gateway_process_memory.rs',
        'src/topal-compiler/src/systems_frame_memory_tests.rs',
        'src/topal-compiler/src/systems_frame_artifact_tests.rs',
        'src/topal-compiler/src/systems_frame_publication_guest_tests.rs',
        'linux-kernel/labs/qemu/x86_64/frame_memory_fault_inspect.py',
        'linux-kernel/design/dynamic-frame-memory.md', 'spec/systems-profile.md',
        'src/topal-compiler/se-requirements.md', 'AGENTS.md', 'scripts/run_bounded.py'])
    return {str(path.relative_to(ROOT)): fact(path.read_bytes()) for path in sorted(paths)}


def legacy_inventory(folder):
    """Inventory retained original assertions; do not synthesize missing states."""
    result = {}
    for mode in MODES:
        work = folder / ('memory-' + mode)
        raw = (work / 'observations.json').read_bytes()
        value = json.loads(raw)
        assert value['passed'] is True and value['mode'] == mode
        stdout = (work / 'gdb.stdout.log').read_bytes()
        assert ('FRAME_MEMORY_FAULT_GDB_PASS ' + mode).encode() in stdout
        assert b'FRAME_ROOT_GDB_PASS' in stdout
        commands = json.loads((work / 'qemu.command.json').read_bytes())
        image = Path(commands[commands.index('-kernel') + 1]).resolve()
        assert image == (folder / 'bzImage').resolve(), 'retained VM command/image path'
        if mode in MODES[:9]:
            assert value['refused_without_retirement_claim'] is True
            evidence = 'retained pre-invocation refusal assertion'
        elif mode == 'no-fit':
            assert value['guest_state_modified'] is False and value['ordinary_no_fit'] is True
            evidence = 'retained ordinary exhaustion and no-state-edit assertions'
        else:
            assert all(value[key] is True for key in ['performed_effects_retained',
                'physical_caller_retired', 'process_not_resumed'])
            evidence = 'retained post-write fatal/retirement assertions'
        files = {name: fact((work / name).read_bytes()) for name in [
            'observations.json', 'gdb.stdout.log', 'gdb.stderr.log', 'inspect.gdb',
            'qemu.command.json', 'qemu.stdout.log', 'qemu.stderr.log', 'serial.log']}
        result[mode] = dict(completed_assertions=True, observations=fact(raw), files=files,
                            evidence=evidence,
                            limitation='Original producer assertions. Complete CPU inputs and before/after bytes were not retained for independent replay of every assertion; this is not fresh execution or current-kernel fault qualification.')
    return result


def mutation_controls(kernel):
    program, layout, _ = linked(kernel)
    result = []
    for name, case in cases(layout):
        if not name.startswith('legacy-'):
            continue
        original = check(program, layout, case)
        assert not original['admitted']
        guard = original['branches'][-1]
        assert guard['taken'] and guard['target'] == program.refusal
        at = guard['offset']
        assert program.body[at:at + 2] in [bytes.fromhex('0f84'), bytes.fromhex('0f85')]
        raw = bytearray(program.body)
        before = bytes(raw[at:at + 6])
        raw[at + 2:at + 6] = bytes(4)  # This Python-only guard now falls through.
        altered = Program(raw, program.boundary, program.refusal)
        assert altered.run(**machine_inputs(case, layout))['admitted']
        try:
            check(altered, layout, case)
        except ModelGap as error:
            assert str(error) == 'compiled admission differs from independent predicate'
            result.append(dict(case=name, offset=at, original=before.hex(),
                               altered=bytes(raw[at:at + 6]).hex(), rejected=True, reason=str(error)))
        else:
            raise AssertionError('model accepted weakened admission guard')
    assert len(result) == 9
    return result


def run(output, current, legacy):
    output, current = output.resolve(), current.resolve()
    assert output.is_relative_to(ROOT / 'linux-kernel/.cache')
    output.mkdir(parents=True, exist_ok=False)
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    directory = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (directory / 'memory.swap.max').read_text().strip() == '0'

    def resources():
        return {name: (directory / name).read_text().strip() for name in
                ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}

    report = dict(passed=False, source_hashes_before=watched(), kernels={}, roles={},
                  legacy={}, controls={}, commands={}, resource_samples=[resources()], group=group,
                  scope='Fresh host interpretation/model tests and inventory of already retained diagnostic assertions. No compiler/VM/debugger invocation, injected native state, test removal or production activation.')
    start = time.monotonic()
    (output / 'driver.py').write_bytes(Path(__file__).read_bytes())
    failure = 0

    def save():
        report['resource_samples'].append(resources())
        (output / 'run.json').write_text(json.dumps(report, indent=2) + '\n')

    save()
    try:
        roles = [('current', current)]
        for label, path in legacy:
            folder = Path(path).resolve()
            assert folder.is_relative_to(ROOT / 'linux-kernel/.cache')
            assert label not in report['legacy'] and label != 'current'
            report['legacy'][label] = legacy_inventory(folder)
            roles.append((label, folder / 'kernel.elf'))
        for label, kernel in roles:
            assert kernel.is_relative_to(ROOT / 'linux-kernel/.cache')
            raw = kernel.read_bytes()
            identity = fact(raw)
            key = identity['sha256']
            report['roles'][label] = dict(kernel=str(kernel.relative_to(ROOT)), identity=identity)
            if key not in report['kernels']:
                value = inspect(kernel)
                assert value['cases'] == 5126 and value['admitted'] == 1173 and value['refused'] == 3953
                assert not value['qualification_complete']
                for row in value['results']:
                    if row['name'].startswith('legacy-'):
                        assert not row['admitted']
                (output / (key + '.json')).write_text(json.dumps(value, indent=2) + '\n')
                report['kernels'][key] = dict(report=fact((output / (key + '.json')).read_bytes()),
                    cases=value['cases'], admitted=value['admitted'], refused=value['refused'],
                    prefix_instructions=value['binding']['prefix_instruction_count'],
                    selected_interpreted_instructions=len(value['interpreted_offsets']),
                    unvisited_prefix_instructions=len(value['unvisited_prefix_offsets']))
                report['controls'][key] = mutation_controls(kernel)
            assert fact(kernel.read_bytes()) == identity, 'native file changed during model controls'
            save()
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
        for name, count in [('user-memory', 248), ('process-memory', 82)]:
            argv = ['rtk', 'proxy', 'python3', '-B', '-m', 'unittest', 'discover',
                    '-s', str(ROOT / 'linux-kernel/tools' / name), '-p', 'test_*.py']
            began = time.monotonic()
            with (output / (name + '.stdout.log')).open('w') as out, (output / (name + '.stderr.log')).open('w') as err:
                result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=out, stderr=err)
            report['commands'][name] = dict(argv=argv, exit_code=result.returncode,
                                            elapsed_seconds=time.monotonic() - began)
            assert result.returncode == 0
            text = (output / (name + '.stderr.log')).read_text()
            assert f'Ran {count} tests' in text and text.rstrip().endswith('OK')
    except BaseException:
        failure = 1
        report['failure'] = traceback.format_exc()
    finally:
        report['source_hashes_after'] = watched()
        report['elapsed_seconds'] = time.monotonic() - start
        report['passed'] = not failure and report['source_hashes_before'] == report['source_hashes_after']
        save()
    print(json.dumps({key: report.get(key) for key in ['passed', 'elapsed_seconds', 'kernels', 'failure']}))
    return failure or int(not report['passed'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('current', type=Path)
    parser.add_argument('--legacy-root', nargs=2, action='append', default=[], metavar=('LABEL', 'DIRECTORY'))
    args = parser.parse_args()
    raise SystemExit(run(args.output, args.current, args.legacy_root))
