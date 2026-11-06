#!/usr/bin/env python3
"""Execute independent Linux chmod callers on the selected Topal kernel."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time

import topal_gate as GATE
import static_inode_mode_checks as CHECKS
from qemu_control import stopped_status

LAB = Path(__file__).resolve().parent


def build(root, lifetime=False, libraries=None):
    original = GATE.static_user_fixture
    calls = []

    def fixture(mode='maximum'):
        nonlocal calls
        image, calls = CHECKS.fixture(original(mode), lifetime)
        return image

    GATE.static_user_fixture = fixture
    try:
        values = GATE.build_image(root, library_root=libraries)
    finally:
        GATE.static_user_fixture = original
    return values, calls


def inspect(root, values, calls, timeout, **mode):
    root.mkdir(parents=True)
    artifact = json.loads(values[3].read_text())
    debug, result, failed, script = (root/n for n in
        ('debug.sock', 'observation.json', 'failed.txt', 'inode.gdb'))
    inspector_hash = GATE.digest(LAB/'inode_mode_inspect.py')
    config = dict(output=str(root), artifact=artifact, cases=calls,
        inspector_sha256=inspector_hash, **mode)
    controlled = bool(mode.get('fault') or mode.get('bad_action') or mode.get('terminate_inactive'))
    qmp = root/'qmp.sock'
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import inode_mode_inspect\ninode_mode_inspect.install({config!r})\nend\ncontinue\n"
        + ('disconnect\n' if controlled else 'detach\n'))
    outcome = []
    stopped = []

    def debugger():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic() < until: time.sleep(.02)
            run = subprocess.run(['gdb', '-q', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+10)
            (root/'debug.log').write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=debugger, daemon=True)
    original = GATE.qemu_command
    original_markers = GATE.MARKERS

    def command(*args):
        thread.start()
        command = original(*args)+['-S', '-gdb', f'unix:{debug},server=on,wait=off']
        if controlled: command += ['-qmp', f'unix:{qmp},server=on,wait=off']
        return command

    def failure():
        if failed.exists(): return failed.read_text()
        if outcome and outcome != [0]: return 'inode debugger failed: '+repr(outcome)
        if controlled and result.exists() and outcome == [0]:
            stopped.append(stopped_status(qmp))
            return 'controlled inode refusal observed'
        return None

    GATE.qemu_command = command
    if controlled:
        GATE.MARKERS = (*original_markers, b'INODE_REQUIRES_DEBUGGER_DISCONNECT_AND_QMP_CONFIRMATION')
    try:
        try:
            serial = GATE.observe_gate(values[0], values[1], root/'serial.log',
                root/'qemu.log', timeout, failure_check=failure)
        except GATE.GateError:
            if not controlled or failed.exists() or not result.exists(): raise
            serial = None
        else:
            if controlled: raise GATE.GateError('controlled source refusal resumed normal completion')
    finally:
        GATE.qemu_command = original
        GATE.MARKERS = original_markers
        thread.join(timeout=15)
    if outcome != [0] or failed.exists() or not result.exists():
        raise GATE.GateError('inode physical observation did not complete: '+repr(outcome))
    observation = json.loads(result.read_text())
    expected = ('pass-controlled-inactive-terminal' if mode.get('terminate_inactive') else
                'expected-admitted-action-fatal' if mode.get('bad_action') else
                'expected-pre-entry-refusal' if controlled else 'pass')
    assert observation['result'] == expected
    assert not controlled or len(stopped) == 1
    record = dict(schema='topal-inode-mode-physical-smoke/1', result=expected,
        linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        static_user_image_sha256=GATE.digest(values[5]), observation=observation,
        caller_sha256=GATE.digest(LAB/'static_inode_mode_checks.py'),
        harness_sha256=GATE.digest(Path(__file__)), inspector_sha256=inspector_hash,
        gate_sha256=GATE.digest(Path(GATE.__file__)),
        serial_sha256=GATE.digest(root/'serial.log'), serial_bytes=len(serial) if serial is not None else None,
        source_library=artifact['inode_mode_library'], binding=artifact['inode_mode_runtime'])
    if controlled: record['stopped_target'] = stopped[0]
    (root/'physical-observation.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print(f"PASS physical inode disposition {expected} ({len(observation['calls'])} completed actions)", flush=True)
    return record


def verify(root, timeout, lifetime=False, bad_action=None):
    root.mkdir(parents=True, exist_ok=True)
    GATE.verify_host()
    libraries = None
    if bad_action:
        libraries = root/'libraries'
        shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
        path = libraries/'support/inode-metadata.t'
        source = path.read_text()
        old, new = (('value is mode)', 'value is 0)') if bad_action == 'wrong-mode' else
                    ('true then write-time (region, seconds)', 'true then false'))
        assert source.count(old) == 1
        path.write_text(source.replace(old, new))
    values, calls = build(root, lifetime, libraries)
    artifact = json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface'] == 'static-linux-dispatch-v5'
    assert artifact['inode_mode_runtime']['combined_stack_bytes'] <= 4096
    if not bad_action: assert artifact['inode_mode_runtime']['combined_stack_bytes'] == 1544
    if bad_action:
        return inspect(root/'physical', values, calls, timeout, bad_action=bad_action)
    serial = GATE.observe_gate(values[0], values[1], root/'serial.log', root/'qemu.log', timeout)
    record = dict(schema='topal-inode-mode-caller-smoke/1', result='pass',
        linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        caller_sha256=GATE.digest(LAB/'static_inode_mode_checks.py'),
        harness_sha256=GATE.digest(Path(__file__)),
        source_library=artifact['inode_mode_library'], binding=artifact['inode_mode_runtime'],
        static_user_image_sha256=GATE.digest(values[5]), independent_calls=len(calls),
        serial_sha256=GATE.digest(root/'serial.log'), serial_bytes=len(serial),
        scope='caller assertions only; physical action debugger qualification remains required')
    (root/'caller-observation.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print(f'PASS independent mode caller assertions ({len(calls)} returns)', flush=True)
    inspect(root/'physical', values, calls, timeout)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--keep', type=Path)
    parser.add_argument('--inspect-existing', type=Path,
        help='inspect an earlier artifact after checking the exact caller bytes')
    parser.add_argument('--lifetime', action='store_true')
    parser.add_argument('--fault', choices=('foreign-owner', 'record-stride', 'interrupts-enabled',
        'permissions-range', 'dirty-arena', 'generation-exhausted', 'bad-type'))
    parser.add_argument('--bad-action', choices=('wrong-mode', 'false-after-mode'))
    parser.add_argument('--inspection-output', type=Path,
        help='fresh output directory when inspecting an existing artifact')
    parser.add_argument('--terminate-inactive', action='store_true')
    args = parser.parse_args()
    if args.inspect_existing:
        root = args.inspect_existing.resolve()
        GATE.verify_host()
        image, calls = CHECKS.fixture(GATE.static_user_fixture(), args.lifetime)
        values = (root/'boot/bzImage', root/'boot/initramfs.cpio', root/'boot/boot-provenance.json',
            root/'linked/provenance.json', root/'generated/initial-user.elf', root/'linked/static-user.elf')
        assert values[5].read_bytes() == image, 'existing artifact does not contain these caller checks'
        name = 'physical-'+args.fault if args.fault else 'physical'
        inspect(args.inspection_output or root/name, values, calls, args.timeout, fault=args.fault,
            bad_action=args.bad_action, terminate_inactive=args.terminate_inactive)
    elif args.keep:
        verify(args.keep, args.timeout, args.lifetime, args.bad_action)
    else:
        with tempfile.TemporaryDirectory(prefix='topal-inode-mode-vm-') as directory:
            verify(Path(directory), args.timeout, args.lifetime, args.bad_action)
