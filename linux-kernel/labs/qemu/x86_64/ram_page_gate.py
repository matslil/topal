#!/usr/bin/env python3
"""Qualify ordinary RAM-page actions against real x86-64 filesystem state."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time

import positional_gate as POSITIONAL
from qemu_control import stopped_status

GATE = POSITIONAL.GATE
LAB = Path(__file__).resolve().parent


def selected_source(text):
    if '  ram-page-pool-library is' in text:
        return text
    declaration = ('  ram-page-pool-library is lang systems source-library (\n'
        '    source is "memory/ram-page-pool.t",\n'
        '    export is apply,\n'
        '    interface is borrowed-byte-region-instance-action-v1,\n'
        '    stack-limit-bytes is 4096,\n'
        '    runtime-limit-bytes is 65536\n  ),\n')
    assert text.count('  region-library is') == 1
    return text.replace('  region-library is', declaration+'  region-library is')


def build(destination, lifetime=False, libraries=None, source_text=None):
    source = destination.parent/(destination.name+'-root.t')
    source.write_text(selected_source(GATE.SOURCE_PATH.read_text()) if source_text is None else source_text)
    original_source, original_fixture = GATE.SOURCE_PATH, GATE.static_user_fixture
    calls = []

    def fixture(mode='maximum'):
        nonlocal calls
        image, calls = POSITIONAL.CHECKS.fixture(original_fixture(mode), lifetime)
        return image

    GATE.SOURCE_PATH, GATE.static_user_fixture = source, fixture
    try:
        values = GATE.build_image(destination, library_root=libraries)
    finally:
        GATE.SOURCE_PATH, GATE.static_user_fixture = original_source, original_fixture
    return values, calls


def observe(root, values, calls, timeout, **mode):
    root.mkdir(parents=True)
    artifact = json.loads(values[3].read_text())
    debug, failed, result, script = (root/name for name in
        ('debug.sock', 'failed.txt', 'observation.json', 'ram-page.gdb'))
    config = dict(output=str(root), artifact=artifact, cases=calls, **mode)
    controlled = bool(mode.get('fault') or mode.get('bad_action') or mode.get('terminate_inactive'))
    qmp = root/'qmp.sock'
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import ram_page_inspect\nram_page_inspect.install({config!r})\nend\ncontinue\n"
        + ('disconnect\n' if controlled else 'detach\n'))
    outcome = []
    stopped = []

    def inspect():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic() < until: time.sleep(.02)
            run = subprocess.run(['gdb', '-q', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+10)
            (root/'debug.log').write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=inspect, daemon=True)
    original_command, original_markers = GATE.qemu_command, GATE.MARKERS

    def command(*args):
        thread.start()
        command = original_command(*args)+['-S', '-gdb', f'unix:{debug},server=on,wait=off']
        if controlled: command += ['-qmp', f'unix:{qmp},server=on,wait=off']
        return command

    def failure():
        if failed.exists(): return failed.read_text()
        if outcome and outcome != [0]: return 'RAM-page debugger failed: '+repr(outcome)
        if controlled and result.exists() and outcome == [0]:
            try:
                if not stopped: stopped.append(stopped_status(qmp))
            except BaseException as error:
                failed.write_text(repr(error))
                return failed.read_text()
            return 'controlled RAM-page refusal observed'
        return None

    GATE.qemu_command = command
    if controlled: GATE.MARKERS = (*original_markers, b'RAM_PAGE_REQUIRES_DEBUGGER_DISCONNECT')
    try:
        try:
            runtime = GATE.observe_gate(values[0], values[1], root/'serial.log',
                root/'qemu.log', timeout, failure_check=failure)
        except GATE.GateError as error:
            if not controlled or failed.exists() or not result.exists():
                raise GATE.GateError(str(error)+'\n'+(failed.read_text() if failed.exists() else '')) from error
            runtime = None
        else:
            if controlled: raise GATE.GateError('refused page action resumed normal completion')
    finally:
        GATE.qemu_command, GATE.MARKERS = original_command, original_markers
    thread.join(timeout=5)
    if outcome != [0] or failed.exists() or not result.exists():
        raise GATE.GateError('RAM-page inspector did not complete: '+repr(outcome))
    observation = json.loads(result.read_text())
    expected = ('pass-controlled-inactive-terminal' if mode.get('terminate_inactive')
                else 'expected-admitted-action-fatal' if mode.get('bad_action')
                else 'expected-pre-entry-refusal' if controlled else 'pass')
    if observation['result'] != expected or (controlled and not stopped):
        raise GATE.GateError('RAM-page inspector missed its disposition')
    record = dict(observation=observation,
        serial_sha256=GATE.digest(root/'serial.log'), serial_bytes=len(runtime) if runtime is not None else None,
        linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        static_user_image_sha256=GATE.digest(values[5]),
        source_library=artifact['ram_page_pool_library'], binding=artifact['ram_page_pool_runtime'])
    if controlled: record.update(debugger_completed_before_teardown=True, stopped_target=stopped[0])
    return record


def reverse_order(text):
    changes = 0
    result = []
    for line in text.splitlines():
        for slot in range(8):
            expected = f'try-reserve (pool, {slot})'
            if line.strip() == expected:
                line = line.replace(expected, f'try-reserve (pool, {7-slot})', 1)
                changes += 1
                break
        result.append(line)
    assert changes == 8
    return '\n'.join(result)+'\n'


def verify(timeout=120, write_evidence=False, smoke=False, keep=None):
    GATE.verify_host()
    temporary = tempfile.TemporaryDirectory(prefix='topal-ram-pages-', dir='/tmp') if keep is None else None
    root = Path(temporary.name) if temporary else Path(keep)
    root.mkdir(exist_ok=True, parents=True)
    print('RAM-page work directory:', root, flush=True)
    record = dict(schema='topal-ram-page-region-qemu/1',
        harness_sha256=GATE.digest(Path(__file__)), inspector_sha256=GATE.digest(LAB/'ram_page_inspect.py'),
        caller_sha256=GATE.digest(LAB/'static_positional_file_checks.py'))
    try:
        values, calls = build(root/'primary-artifact')
        record['primary'] = observe(root/'primary', values, calls, timeout)
        if not smoke:
            values_lifetime, lifetime = build(root/'lifetime-artifact', True)
            record['lifetime'] = observe(root/'lifetime', values_lifetime, lifetime, timeout)
            record['inactive_terminal'] = observe(root/'inactive-terminal', values_lifetime, lifetime,
                                                   timeout, terminate_inactive=True)
            libraries = root/'last-fit-library'
            shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
            helper = libraries/'support/page-slots.t'
            helper.write_text(reverse_order(helper.read_text()))
            alternate, alternate_calls = build(root/'last-fit-artifact', libraries=libraries)
            record['last_fit'] = observe(root/'last-fit', alternate, alternate_calls, timeout, reverse=True)
            first = record['primary']['observation']['calls'][0]
            last = record['last_fit']['observation']['calls'][0]
            assert first['selected'] == 0 and last['selected'] == 7
            for name in ('false', 'wrong-byte', 'multiple-delta', 'false-release',
                         'wrong-byte-release', 'multiple-delta-release'):
                libraries = root/(name+'-library')
                shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
                if name.startswith('false'):
                    source = libraries/'memory/ram-page-pool.t'
                    text = source.read_text()
                    old = '    true then dispatch (pool, operation, slot)'
                    new = '    true then refuse-result (pool, operation, slot)'
                    result = 'operation = 0' if name.endswith('-release') else 'false'
                    text = text.replace('pub apply is fn',
                        'refuse-result is fn ((pool : KernelByteRegion, operation : Nat, slot : Nat)) -> Boolean\n'
                        f'  dispatch (pool, operation, slot)\n    true then {result}\n    false then false\n\npub apply is fn')
                else:
                    source = libraries/'support/page-slots.t'
                    text = source.read_text()
                    releasing = name.endswith('-release')
                    old = f'    true then pool byte store (offset-bytes is slot, value is {0 if releasing else 1})'
                    new = ('    true then pool byte store (offset-bytes is slot, value is 2)'
                           if name.startswith('wrong-byte') else '    true then reserve-two (pool, slot)')
                    if name.startswith('multiple-delta'):
                        first_value = 0 if releasing else 1
                        second_store = ('flip-other (pool, slot)' if releasing
                                        else 'pool byte store (offset-bytes is 7, value is 1)')
                        extra = ('' if not releasing else
                            'flip-value is fn ((pool : KernelByteRegion, slot : Nat, occupied : Nat)) -> Boolean\n'
                            '  occupied = 0\n    true then pool byte store (offset-bytes is slot, value is 1)\n'
                            '    false then pool byte store (offset-bytes is slot, value is 0)\n\n'
                            'flip-slot is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean\n'
                            '  pool byte load (offset-bytes is slot)\n    Some occupied then flip-value (pool, slot, occupied)\n'
                            '    None then false\n\n'
                            'flip-other is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean\n'
                            '  slot = 7\n    true then flip-slot (pool, 0)\n    false then flip-slot (pool, 7)\n\n')
                        text = text.replace('reserve-if-free is fn',
                            extra +
                            'reserve-two is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean\n'
                            f'  pool byte store (offset-bytes is slot, value is {first_value})\n'
                            f'    true then {second_store}\n'
                            '    false then false\n\nreserve-if-free is fn', 1)
                assert text.count(old) == 1
                source.write_text(text.replace(old, new))
                changed, changed_calls = build(root/(name+'-artifact'), libraries=libraries)
                record[name] = observe(root/name, changed, changed_calls, timeout, bad_action=name)
            for fault in ('foreign-owner', 'object-stride', 'interrupts-enabled', 'dirty-arena', 'generation-exhausted'):
                for release in (False, True):
                    name = fault+('-release' if release else '-allocation')
                    record[name] = observe(root/name, values, calls, timeout, fault=fault, release_fault=release)
            text = selected_source(GATE.SOURCE_PATH.read_text())
            start = text.index('  ram-page-pool-library is')
            end = text.index('  ),\n', start)+len('  ),\n')
            absent, absent_calls = build(root/'native-only-artifact', source_text=text[:start]+text[end:])
            metadata = json.loads(absent[3].read_text())
            assert 'ram_page_pool_library' not in metadata and 'ram_page_pool_runtime' not in metadata
            assert metadata['volatile_files']['schema'] == 'topal.volatile-file.x86_64/2'
            serial = GATE.observe_gate(absent[0], absent[1], root/'native-only-serial.log',
                                        root/'native-only-qemu.log', timeout)
            record['native_only'] = dict(result='pass', independent_syscall_checks=len(absent_calls),
                source_and_binding_absent=True, profile=metadata['volatile_files'],
                linked_kernel_sha256=GATE.digest(absent[3].parent/'kernel.elf'),
                serial_sha256=GATE.digest(root/'native-only-serial.log'), serial_bytes=len(serial))
        if write_evidence:
            assert not smoke
            (LAB/'results/topal-ram-page-region-gate.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
        print('PASS real RAM-page source allocation/release and physical owner cleanup', flush=True)
        return record
    finally:
        if temporary: temporary.cleanup()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--write-evidence', action='store_true')
    parser.add_argument('--smoke', action='store_true')
    parser.add_argument('--keep', type=Path)
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence, args.smoke, args.keep)
