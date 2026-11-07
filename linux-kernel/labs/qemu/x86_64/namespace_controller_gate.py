#!/usr/bin/env python3
"""Observe the bound namespace controller in QEMU; no Linux adapter claim."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import topal_gate as GATE
from qemu_control import stopped_status

LAB = Path(__file__).resolve().parent


def inspect(fixture, output, mode, timeout):
    GATE.verify_host()
    output.mkdir(parents=True)
    linked, boot = fixture / 'linked', fixture / 'boot'
    artifact = json.loads((linked / 'provenance.json').read_text())
    assert artifact['namespace_runtime']['invocation']['combined_stack_bytes'] <= 4096
    assert next(o['sha256'] for o in artifact['outputs'] if o['identity'] == 'kernel.elf') == GATE.digest(linked / 'kernel.elf')
    assert artifact['namespace_library']['source_sha256'] == GATE.digest(linked / 'namespace-library.t')
    source = (linked / 'namespace-library.t').read_text()
    if mode == 'source-false': assert '_ is support namespace-metadata apply' in source
    if mode == 'source-wrong-association': assert 'damage is fn' in source
    debug, qmp = output / 'debug.sock', output / 'qmp.sock'
    failed, result = output / 'failed.txt', output / 'observation.json'
    script = output / 'namespace.gdb'
    inspector = LAB / 'namespace_controller_inspect.py'
    config = dict(artifact=artifact, output=str(output), mode=mode, inspector_sha256=GATE.digest(inspector))
    script.write_text(f"set pagination off\nset confirm off\nfile {linked / 'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_controller_inspect\nnamespace_controller_inspect.install({config!r})\nend\n"
        "continue\ndisconnect\n")
    outcome = []
    stopped = []

    def debugger():
        try:
            until = time.monotonic() + 10
            while not debug.exists() and time.monotonic() < until: time.sleep(.02)
            command = ['gdb', '-q', '-batch', '-ex', 'set debuginfod enabled off', '-x', str(script)]
            run = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout + 10)
            (output / 'gdb.stdout.log').write_bytes(run.stdout)
            (output / 'gdb.stderr.log').write_bytes(run.stderr)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=debugger, daemon=True)
    original, markers = GATE.qemu_command, GATE.MARKERS

    def command(*args):
        thread.start()
        return original(*args) + ['-S', '-gdb', f'unix:{debug},server=on,wait=off', '-qmp', f'unix:{qmp},server=on,wait=off']

    def failure():
        if failed.exists(): return failed.read_text()
        if outcome and outcome != [0]: return 'namespace debugger failed: ' + repr(outcome)
        if result.exists() and outcome == [0]:
            stopped.append(stopped_status(qmp))
            return 'controlled namespace component observed'
        return None

    GATE.qemu_command = command
    GATE.MARKERS = (*markers, b'NAMESPACE_COMPONENT_REQUIRES_DEBUGGER_AND_QMP')
    try:
        try:
            GATE.observe_gate(boot / 'bzImage', boot / 'initramfs.cpio', output / 'serial.log', output / 'qemu.log', timeout, failure_check=failure)
        except GATE.GateError:
            if failed.exists() or not result.exists(): raise
        else:
            raise GATE.GateError('component resumed the old personality instead of stopping at its checked boundary')
    finally:
        GATE.qemu_command, GATE.MARKERS = original, markers
        thread.join(timeout=15)
    assert outcome == [0] and len(stopped) == 1 and not failed.exists()
    observation = json.loads(result.read_text())
    expected = 'pass-native-namespace-controller-component' if mode == 'normal' else 'pass-native-namespace-fatal-component'
    assert observation['result'] == expected
    record = dict(schema='topal.namespace-controller-physical-component/1', result=expected,
        mode=mode, observation=observation, stopped_target=stopped[0],
        linked_kernel_sha256=GATE.digest(linked / 'kernel.elf'),
        artifact_provenance_sha256=GATE.digest(linked / 'provenance.json'),
        boot_image_sha256=GATE.digest(boot / 'bzImage'), boot_resource_sha256=GATE.digest(boot / 'initramfs.cpio'),
        inspector_sha256=GATE.digest(inspector), harness_sha256=GATE.digest(Path(__file__)),
        gdb_stdout_sha256=GATE.digest(output / 'gdb.stdout.log'), gdb_stderr_sha256=GATE.digest(output / 'gdb.stderr.log'),
        serial_sha256=GATE.digest(output / 'serial.log'), qemu_sha256=GATE.digest(output / 'qemu.log'))
    (output / 'physical-observation.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    print(f"NAMESPACE_CONTROLLER_PHYSICAL_PASS mode={mode} calls={len(observation['calls'])} work={output}", flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--mode', default='normal', choices=['normal', 'runtime-abort', 'post-frame', 'post-control', 'source-false', 'source-wrong-association', 'pre-owner', 'pre-view', 'pre-input', 'pre-generation', 'pre-overlap-position', 'pre-overlap-page', 'pre-overlap-inode'])
    parser.add_argument('--timeout', default=120, type=float)
    args = parser.parse_args()
    inspect(args.fixture.resolve(), args.output.resolve(), args.mode, args.timeout)
