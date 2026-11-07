#!/usr/bin/env python3
"""Qualify fatal source recovery reached through an actual namespace syscall."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import threading
import time

import topal_gate as GATE
import namespace_gate
import static_namespace_fault_checks as CHECKS
from qemu_control import stopped_status

LAB = Path(__file__).resolve().parent
MODES = ('source-false','source-wrong-association','runtime-abort')


def libraries(root,mode):
    if mode == 'runtime-abort':
        return None
    destination = root/'libraries'
    shutil.copytree(GATE.LINUX_KERNEL/'kernel',destination)
    path = destination/'policy/namespace-action.t'
    if mode == 'source-false':
        old = '  support namespace-metadata apply (region, operation, seconds)'
        original = path.read_text()
        assert original.count(old) == 1
        path.write_text(original.replace(old,'  _ is support namespace-metadata apply (region, operation, seconds)\n  false'))
    else:
        path.write_text('use language (version is v0.1, features is (systems))\n'
            'use library support (version is v0.1)\n'
            'damage is fn (region : KernelByteRegion) -> Boolean\n'
            '  region unsigned store (offset-bytes is 64, byte-count is 8, byte-order is little-endian, value is 999)\n'
            'pub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean\n'
            '  changed is support namespace-metadata apply (region, operation, seconds)\n'
            '  changed\n    true then damage region\n    false then false\n')
    return destination


def verify(root,mode,operation,timeout):
    root.mkdir(parents=True)
    GATE.verify_host()
    library_root = libraries(root,mode)
    values,cases = namespace_gate.build(root,lambda original: CHECKS.fixture(original,operation),library_root)
    artifact = json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface'] == 'static-linux-dispatch-v6'
    assert artifact['namespace_runtime']['schema'] == 'topal.namespace-owner-binding.x86_64/2'
    entries = artifact['volatile_files']['namespace_entries']
    complete_stack = artifact['namespace_library']['maximum_stack_bytes']+entries['namespace_caller_stack_bytes']
    assert complete_stack == (4096 if mode == 'source-wrong-association' else 4080)
    linked = values[3].parent
    assert artifact['namespace_library']['source_sha256'] == GATE.digest(linked/'namespace-library.t')
    debug,qmp = root/'debug.sock',root/'qmp.sock'
    assert max(len(str(debug)),len(str(qmp))) < 108, 'choose a shorter fresh output path for Unix sockets'
    failed,result,script = root/'failed.txt',root/'observation.json',root/'namespace.gdb'
    inspector = LAB/'namespace_fault_inspect.py'
    config = dict(output=str(root),artifact=artifact,cases=cases,fault_mode=mode,
        inspector_sha256=GATE.digest(inspector),source_physical_sha256=GATE.digest(LAB/'namespace_source_physical.py'))
    (root/'caller-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
    script.write_text(f"set pagination off\nset confirm off\nfile {linked/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_fault_inspect\nnamespace_fault_inspect.install({config!r})\nend\ncontinue\ndisconnect\n")
    outcome,stopped = [],[]

    def debugger():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic()<until:
                time.sleep(.02)
            command = ['gdb','-q','--batch','-ex','set debuginfod enabled off','-x',str(script)]
            (root/'debugger-command.json').write_text(json.dumps(command,indent=2)+'\n')
            run = subprocess.run(command,capture_output=True,timeout=timeout+10)
            (root/'gdb.stdout.log').write_bytes(run.stdout)
            (root/'gdb.stderr.log').write_bytes(run.stderr)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=debugger,daemon=True)
    original,markers = GATE.qemu_command,GATE.MARKERS

    def command(*args):
        thread.start()
        arguments = original(*args)+['-S','-gdb',f'unix:{debug},server=on,wait=off','-qmp',f'unix:{qmp},server=on,wait=off']
        (root/'qemu-command.json').write_text(json.dumps(arguments,indent=2)+'\n')
        return arguments

    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'fatal syscall debugger failed: '+repr(outcome)
        if result.exists() and outcome == [0]:
            stopped.append(stopped_status(qmp))
            return 'controlled namespace syscall fatal boundary observed'
        return None

    GATE.qemu_command = command
    GATE.MARKERS = (*markers,b'NAMESPACE_FATAL_REQUIRES_DISCONNECT_AND_STOPPED_QMP')
    try:
        try:
            GATE.observe_gate(values[0],values[1],root/'serial.log',root/'qemu.log',timeout,failure_check=failure)
        except GATE.GateError:
            if failed.exists() or not result.exists():
                raise
        else:
            raise GATE.GateError('fatal namespace action resumed normal completion')
    finally:
        GATE.qemu_command,GATE.MARKERS = original,markers
        thread.join(timeout=15)
    assert outcome == [0] and len(stopped) == 1 and not failed.exists()
    observation = json.loads(result.read_text())
    assert observation['result'] == 'pass' and observation['case'] == cases[-1]
    files = [values[3],linked/'kernel.elf',values[0],values[1],values[5],linked/'namespace-library.t',
        root/'namespace-root.t',root/'observation.json',root/'owner-before-apply.bin',root/'owner-at-halt.bin',
        root/'caller-cases.json',root/'qemu-command.json',root/'debugger-command.json',
        root/'gdb.stdout.log',root/'gdb.stderr.log',root/'serial.log',root/'qemu.log',
        inspector,LAB/'namespace_source_physical.py',LAB/'static_namespace_fault_checks.py',
        LAB/'static_namespace_transaction_checks.py',LAB/'namespace_gate.py',Path(GATE.__file__),Path(__file__)]
    record = dict(schema='topal.namespace-real-syscall-fatal-physical/1',result='pass',mode=mode,operation=operation,
        complete_stack_bytes=complete_stack,stopped_target=stopped[0],observation=observation,
        hashes={str(path):GATE.digest(path) for path in files})
    (root/'physical-observation.json').write_text(json.dumps(record,indent=2)+'\n')
    print(f'NAMESPACE_REAL_FATAL_PASS mode={mode} operation={operation} stores={len(observation["stores"])} work={root}',flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--mode',required=True,choices=MODES)
    parser.add_argument('--operation',required=True,choices=CHECKS.OPERATIONS)
    parser.add_argument('--timeout',type=float,default=120)
    args = parser.parse_args()
    verify(args.output.resolve(),args.mode,args.operation,args.timeout)
