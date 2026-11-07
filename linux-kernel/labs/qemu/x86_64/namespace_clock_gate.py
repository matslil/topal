#!/usr/bin/env python3
"""Execute namespace clock-error and no-clock-on-refusal/no-op observations."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import topal_gate as GATE
import namespace_gate
import namespace_clock_probe
import static_namespace_clock_checks as CHECKS

LAB = Path(__file__).resolve().parent


def verify(root,timeout,operation=None,fixture=None):
    root.mkdir(parents=True)
    GATE.verify_host()
    if fixture:
        import static_namespace_transaction_checks as transaction
        import static_namespace_disposition_checks as disposition
        image,cases = transaction.fixture(GATE.static_user_fixture())
        image,additional = disposition.fixture(image)
        cases += additional
        assert (fixture/'linked/static-user.elf').read_bytes() == image
        values = (fixture/'boot/bzImage',fixture/'boot/initramfs.cpio',fixture/'boot/boot-provenance.json',
            fixture/'linked/provenance.json',fixture/'generated/initial-user.elf',fixture/'linked/static-user.elf')
        mode = 'observe'
    else:
        assert operation in CHECKS.OPERATIONS
        values,cases = namespace_gate.build(root,lambda original: CHECKS.fixture(original,operation))
        mode = 'busy-rtc'
    linked = values[3].parent
    artifact = json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface'] == 'static-linux-dispatch-v6'
    assert next(o['sha256'] for o in artifact['outputs'] if o['identity'] == 'kernel.elf') == GATE.digest(linked/'kernel.elf')
    debug,script,failed = root/'debug.sock',root/'clock.gdb',root/'failed.txt'
    assert len(str(debug)) < 108
    inspector = LAB/'namespace_clock_inspect.py'
    config = dict(output=str(root),mode=mode,artifact=artifact,cases=cases,
        clock_probe=namespace_clock_probe.binding(linked/'kernel.elf'),inspector_sha256=GATE.digest(inspector))
    (root/'caller-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
    (root/'clock-probe.json').write_text(json.dumps(config['clock_probe'],indent=2)+'\n')
    script.write_text(f"set pagination off\nset confirm off\nfile {linked/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_clock_inspect\nnamespace_clock_inspect.install({config!r})\nend\ncontinue\ndetach\n")
    outcome = []

    def debugger():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic()<until:
                time.sleep(.02)
            command = ['gdb','-q','--batch','-ex','set debuginfod enabled off','-x',str(script)]
            (root/'debugger-command.json').write_text(json.dumps(command,indent=2)+'\n')
            result = subprocess.run(command,capture_output=True,timeout=timeout+10)
            (root/'gdb.stdout.log').write_bytes(result.stdout)
            (root/'gdb.stderr.log').write_bytes(result.stderr)
            outcome.append(result.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=debugger,daemon=True)
    original = GATE.qemu_command

    def command(*args):
        thread.start()
        arguments = original(*args)+['-S','-gdb',f'unix:{debug},server=on,wait=off']
        (root/'qemu-command.json').write_text(json.dumps(arguments,indent=2)+'\n')
        return arguments

    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'namespace clock debugger failed: '+repr(outcome)
        return None

    GATE.qemu_command = command
    try:
        serial = GATE.observe_gate(values[0],values[1],root/'serial.log',root/'qemu.log',timeout,failure_check=failure)
    finally:
        GATE.qemu_command = original
        thread.join(timeout=15)
    assert outcome == [0] and not failed.exists()
    returns = json.loads((root/'syscall-returns.json').read_text())
    assert len(returns) == len(cases)
    observation = json.loads((root/'clock-observation.json').read_text())
    assert observation['result'] == 'pass'
    files = [linked/'kernel.elf',values[3],values[0],values[1],values[5],
        root/'clock-observation.json',root/'clock-probe.json',root/'syscall-returns.json',
        root/'caller-cases.json',root/'qemu-command.json',root/'debugger-command.json',
        root/'gdb.stdout.log',root/'gdb.stderr.log',root/'serial.log',root/'qemu.log',
        inspector,LAB/'namespace_source_physical.py',LAB/'namespace_clock_probe.py',
        LAB/'static_namespace_clock_checks.py',LAB/'static_namespace_transaction_checks.py',
        LAB/'static_namespace_disposition_checks.py',LAB/'namespace_gate.py',Path(GATE.__file__),Path(__file__)]
    if not fixture:
        files += [root/'owner-before-clock.bin',root/'owner-after-clock.bin']
    record = dict(schema='topal.namespace-clock-physical/1',result='pass',mode=mode,operation=operation,
        returns=len(returns),serial_bytes=len(serial),observation=observation,
        hashes={str(path):GATE.digest(path) for path in files})
    (root/'physical-observation.json').write_text(json.dumps(record,indent=2)+'\n')
    print(f'NAMESPACE_CLOCK_PASS mode={mode} operation={operation} returns={len(returns)} work={root}',flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument('--operation',choices=CHECKS.OPERATIONS)
    choice.add_argument('--fixture',type=Path)
    args = parser.parse_args()
    verify(args.output.resolve(),args.timeout,args.operation,args.fixture.resolve() if args.fixture else None)
