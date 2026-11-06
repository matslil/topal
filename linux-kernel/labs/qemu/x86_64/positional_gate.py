#!/usr/bin/env python3
"""Qualify actual positional-file transactions, independent descriptions and reclamation."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import threading
import time

LAB = Path(__file__).resolve().parent


def module(name):
    spec = importlib.util.spec_from_file_location(name, LAB / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


GATE = module('topal_gate')
CHECKS = module('static_positional_file_checks')


def build(destination, lifetime):
    original = GATE.static_user_fixture
    calls = []
    def fixture(mode='maximum'):
        nonlocal calls
        image, calls = CHECKS.fixture(original(mode), lifetime)
        return image
    original_source=GATE.SOURCE_PATH
    selected=destination.parent/'positional-root.t'
    selected.write_text(original_source.read_text().replace('policy/boot-open-access.t','policy/file-open-access.t')
        .replace('export is boot-open-decision','export is file-open-decision')
        .replace('interface is boot-open-decision-v1','interface is file-open-decision-v1')
        .replace('policy/static-linux-dispatch-v2.t','policy/static-linux-dispatch-v4.t')
        .replace('policy/static-linux-dispatch-v3.t','policy/static-linux-dispatch-v4.t')
        .replace('interface is static-linux-dispatch-v2','interface is static-linux-dispatch-v4')
        .replace('interface is static-linux-dispatch-v3','interface is static-linux-dispatch-v4'))
    GATE.static_user_fixture=fixture;GATE.SOURCE_PATH=selected
    try:values=GATE.build_image(destination)
    finally:GATE.static_user_fixture=original;GATE.SOURCE_PATH=original_source
    return values, calls


def observe(root, values, calls, timeout, *, lifetime=False, corrupt=False, terminate=False):
    root.mkdir(parents=True)
    artifact = json.loads(values[3].read_text())
    if artifact['dispatch_library']['interface'] not in ('static-linux-dispatch-v4', 'static-linux-dispatch-v5'):
        raise GATE.GateError('file service needs the explicitly selected extended operation vocabulary')
    profile = artifact['volatile_files']
    if profile['private_umask_offset'] == 32 or profile['resident_pages'] != 8 or profile['object_count'] != 4:
        raise GATE.GateError('file witness does not match the selected ownership/capacity profile')
    debug, failed, result, script = (root / name for name in ('debug.sock', 'failed.txt', 'observation.json', 'file.gdb'))
    config = dict(calls=calls, failed=str(failed), result=str(result), retained_for_exit=lifetime,
                  page_map_offset=profile.get('page_allocation_map_offset'),
                  corrupt=corrupt, terminate_inactive=terminate)
    if corrupt:
        code,_=CHECKS.code()
        comparison=bytes.fromhex('48a1')+struct.pack('<Q',CHECKS.DATA)
        comparison+=bytes.fromhex('49bb')+b'aXYdefgh'+bytes.fromhex('4c39d875fe')
        witness=next(case for case in calls if case.get('tag')=='positional-data-read')
        at=code.index(comparison,witness['returned']-CHECKS.ENTRY)
        if at-witness['returned']+CHECKS.ENTRY>64:
            raise GATE.GateError('positional counterexample lost its immediate independent byte comparison')
        config['rejection_pc']=CHECKS.ENTRY+at+len(comparison)-2
    script.write_text(f'''set pagination off
file {values[3].parent / 'kernel.elf'}
target remote {debug}
python
import sys
sys.path.insert(0,{str(LAB)!r})
import positional_file_inspect
positional_file_inspect.install({config!r})
end
continue
detach
''')
    outcome = []
    def inspect():
        try:
            until = time.monotonic() + 10
            while not debug.exists() and time.monotonic() < until:
                time.sleep(.02)
            run = subprocess.run(['gdb', '-q', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout + 10)
            (root / 'debug.log').write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))
    thread = threading.Thread(target=inspect, daemon=True)
    original_build, original_command, original_observe = GATE.build_image, GATE.qemu_command, GATE.observe_gate
    def command(*args):
        thread.start()
        return original_command(*args) + ['-S', '-gdb', f'unix:{debug},server=on,wait=off']
    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'file debugger failed: ' + repr(outcome)
        if (corrupt or terminate) and result.exists() and outcome == [0]:
            return 'controlled file observation completed'
        return None
    original_source=GATE.SOURCE_PATH
    selected=values[3].parent.parent.parent / 'positional-root.t'
    if not selected.is_file():raise GATE.GateError('named artifact lost its actual selected source snapshot')
    GATE.SOURCE_PATH=selected
    GATE.build_image = lambda *_args, **_kwargs: values
    GATE.qemu_command = command
    GATE.observe_gate = lambda *args, **kwargs: original_observe(*args, **kwargs, failure_check=failure)
    runtime = None
    try:
        try:
            runtime = GATE.verify(timeout, False)
        except GATE.GateError:
            if not (corrupt or terminate) or not result.exists() or failed.exists():
                raise
        else:
            if corrupt or terminate:
                raise GATE.GateError('controlled file run unexpectedly reached ordinary completion')
    finally:
        GATE.build_image, GATE.qemu_command, GATE.observe_gate = original_build, original_command, original_observe
        GATE.SOURCE_PATH=original_source
    thread.join(timeout=5)
    if outcome != [0] or failed.exists() or not result.exists():
        raise GATE.GateError('file debugger did not complete: ' + repr(outcome))
    observation = json.loads(result.read_text())
    expected = 'expected-wrong-positional-data-rejection' if corrupt else 'pass-controlled-inactive-positional-terminal' if terminate else 'pass'
    if observation['result'] != expected:
        raise GATE.GateError('file observation missed its exact disposition')
    return dict(runtime=runtime, observation=observation, linked_kernel_sha256=GATE.digest(values[3].parent / 'kernel.elf'),
                static_user_image_sha256=GATE.digest(values[5]), profile=profile)


def verify(timeout=120, write_evidence=False):
    control_path = LAB / 'results/topal-positional-file-linux-control.json'
    control = json.loads(control_path.read_text())
    for field, path in [('harness_sha256', LAB / 'positional_control.py'), ('source_sha256', LAB / 'positional_file_control.c')]:
        if control[field] != GATE.digest(path):
            raise GATE.GateError('Linux file control needs current source/harness qualification')
    if control['result'] != 'pass' or control['kernel_release'] != '7.2.9':
        raise GATE.GateError('file control must run on the pinned Linux baseline')
    with tempfile.TemporaryDirectory(prefix='topal-positional-files-', dir='/tmp') as temporary:
        root = Path(temporary)
        primary, primary_calls = build(root / 'primary-artifact', False)
        lifetime, lifetime_calls = build(root / 'lifetime-artifact', True)
        if GATE.digest(primary[3].parent / 'kernel.elf') != GATE.digest(lifetime[3].parent / 'kernel.elf'):
            raise GATE.GateError('file fixtures changed the qualified kernel')
        record = dict(schema='topal-positional-file-qemu/1', harness_sha256=GATE.digest(Path(__file__)),
            inspector_sha256=GATE.digest(LAB / 'positional_file_inspect.py'),
            caller_sha256=GATE.digest(LAB / 'static_positional_file_checks.py'), linux_control_sha256=GATE.digest(control_path))
        record['primary'] = observe(root / 'primary', primary, primary_calls, timeout)
        record['lifetime'] = observe(root / 'lifetime', lifetime, lifetime_calls, timeout, lifetime=True)
        record['inactive_terminal'] = observe(root / 'inactive-terminal', lifetime, lifetime_calls, timeout,
                                               lifetime=True, terminate=True)
        record['counterexample'] = observe(root / 'wrong-data', primary, primary_calls, timeout, corrupt=True)
    if write_evidence:
        (LAB / 'results/topal-positional-file-gate.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    print('PASS positional bytes, unchanged descriptions, signed/append ordering, lifetimes and complete reclamation', flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--write-evidence', action='store_true')
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence)
