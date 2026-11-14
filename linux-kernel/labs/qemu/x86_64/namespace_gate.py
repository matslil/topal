#!/usr/bin/env python3
"""Build a V6 candidate and execute independent rename/link Linux callers."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import topal_gate as GATE
import static_namespace_transaction_checks as CHECKS

LAB = Path(__file__).resolve().parent


def build(root, fixture_builder, library_root=None):
    """Use the public compiler/boot builder with a private V6 caller fixture."""
    original_fixture, original_source = GATE.static_user_fixture, GATE.SOURCE_PATH
    source = original_source.read_text().replace('static-linux-dispatch-v5', 'static-linux-dispatch-v6')
    declaration = '  namespace-library is lang systems source-library (\n    source is "policy/namespace-action.t",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n'
    if '  namespace-library is' not in source:
        source = source.replace('  inode-mode-library is', declaration + '  inode-mode-library is')
    selected = root / 'namespace-root.t'
    selected.write_text(source)
    calls = []

    def fixture(mode='maximum'):
        nonlocal calls
        image, calls = fixture_builder(original_fixture(mode))
        return image

    GATE.static_user_fixture, GATE.SOURCE_PATH = fixture, selected
    try:
        values = GATE.build_image(root, library_root=library_root)
    finally:
        GATE.static_user_fixture, GATE.SOURCE_PATH = original_fixture, original_source
    return values, calls


def verify(root, timeout, dispositions=False, physical=False):
    root.mkdir(parents=True)
    GATE.verify_host()

    def fixture(original):
        image, calls = CHECKS.fixture(original)
        if dispositions:
            import static_namespace_disposition_checks as disposition_checks
            image, additional = disposition_checks.fixture(image)
            calls += additional
        return image, calls

    values, calls = build(root, fixture)
    artifact = json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface'] == 'static-linux-dispatch-v6'
    assert artifact['namespace_runtime']['schema'] == 'topal.namespace-owner-binding.x86_64/2'
    debug, script, failed = root/'debug.sock', root/'namespace.gdb', root/'failed.txt'
    inspector = LAB/'namespace_syscall_inspect.py'
    config = dict(output=str(root), cases=calls, inspector_sha256=GATE.digest(inspector))
    if physical:
        config['artifact'] = artifact
        config['physical_sha256'] = GATE.digest(LAB/'namespace_syscall_physical.py')
        config['source_physical_sha256'] = GATE.digest(LAB/'namespace_source_physical.py')
    (root/'caller-cases.json').write_text(json.dumps(calls, indent=2)+'\n')
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_syscall_inspect\nnamespace_syscall_inspect.install({config!r})\nend\ncontinue\ndetach\n")
    outcome = []

    def debugger():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic()<until: time.sleep(.02)
            result = subprocess.run(['gdb','-q','--batch','-ex','set debuginfod enabled off','-x',str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout+10)
            (root/'gdb.stdout.log').write_bytes(result.stdout)
            (root/'gdb.stderr.log').write_bytes(result.stderr)
            outcome.append(result.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=debugger, daemon=True)
    original_command = GATE.qemu_command

    def command(*args):
        thread.start()
        return original_command(*args)+['-S','-gdb',f'unix:{debug},server=on,wait=off']

    GATE.qemu_command = command
    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'namespace debugger failed: ' + repr(outcome)
        return None

    try:
        serial = GATE.observe_gate(values[0], values[1], root/'serial.log', root/'qemu.log', timeout,
            failure_check=failure)
    finally:
        GATE.qemu_command = original_command
        thread.join(timeout=15)
    assert not failed.exists(), failed.read_text() if failed.exists() else ''
    assert outcome == [0], ('debugger did not detach successfully', outcome)
    returned = json.loads((root/'syscall-returns.json').read_text())
    assert len(returned) == len(calls), (len(returned), len(calls))
    record = dict(schema='topal.namespace-syscall-caller/1', result='pass', independent_calls=len(calls),
        kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'), artifact_sha256=GATE.digest(values[3]),
        image_sha256=GATE.digest(values[5]), caller_sha256=GATE.digest(LAB/'static_namespace_transaction_checks.py'),
        inspector_sha256=GATE.digest(inspector), serial_sha256=GATE.digest(root/'serial.log'), serial_bytes=len(serial),
        scope='actual user syscall results and independent user memory assertions; complete physical source/lifetime coverage remains separate')
    record['dispositions'] = dispositions
    if physical:
        observation = json.loads((root/'refusal-physical.json').read_text())
        assert observation['result'] == 'pass'
        assert len(observation['calls']) == observation['expected_calls']
        record['physical_sha256'] = config['physical_sha256']
        record['source_physical_sha256'] = config['source_physical_sha256']
        record['physical_observation_sha256'] = GATE.digest(root/'refusal-physical.json')
    record['harness_sha256'] = GATE.digest(Path(__file__))
    record['cases_sha256'] = GATE.digest(root/'caller-cases.json')
    record['returns_sha256'] = GATE.digest(root/'syscall-returns.json')
    record['debugger_logs'] = {name: GATE.digest(root/name) for name in ('gdb.stdout.log', 'gdb.stderr.log')}
    if dispositions:
        baseline = GATE.LINUX_KERNEL/'knowledge/x86_64/namespace-native-dispositions.json'
        record['linux_baseline_sha256'] = GATE.digest(baseline)
        record['disposition_caller_sha256'] = GATE.digest(LAB/'static_namespace_disposition_checks.py')
    (root/'caller-observation.json').write_text(json.dumps(record, indent=2)+'\n')
    print(f'NAMESPACE_SYSCALL_CALLER_PASS calls={len(calls)} work={root}', flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--dispositions', action='store_true')
    parser.add_argument('--physical', action='store_true')
    args = parser.parse_args()
    verify(args.output.resolve(), args.timeout, args.dispositions, args.physical)
