#!/usr/bin/env python3
"""Qualify natural parent exit while a namespace-sharing child is parked."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import namespace_gate
import static_namespace_cancel_checks as CHECKS
import topal_gate as GATE
from qemu_control import stopped_status

LAB=Path(__file__).resolve().parent


def verify(root,timeout):
    root.mkdir(parents=True)
    GATE.verify_host()
    values,cases=namespace_gate.build(root,CHECKS.fixture)
    artifact=json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface']=='static-linux-dispatch-v6'
    debug,qmp,failed,result=(root/n for n in ('debug.sock','qmp.sock','failed.txt','observation.json'))
    assert max(len(str(debug)),len(str(qmp)))<108
    inspector=LAB/'namespace_cancel_inspect.py'
    config=dict(artifact=artifact,output=str(root),cases=cases,
        inspector_sha256=GATE.digest(inspector),
        physical_sha256=GATE.digest(LAB/'namespace_syscall_physical.py'),
        source_physical_sha256=GATE.digest(LAB/'namespace_source_physical.py'))
    (root/'caller-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
    script=root/'cancel.gdb'
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_cancel_inspect\nnamespace_cancel_inspect.install({config!r})\nend\ncontinue\ndisconnect\n")
    outcome,stopped=[],[]

    def debugger():
        try:
            until=time.monotonic()+10
            while not debug.exists() and time.monotonic()<until:time.sleep(.02)
            command=['gdb','-q','--batch','-ex','set debuginfod enabled off','-x',str(script)]
            (root/'debugger-command.json').write_text(json.dumps(command,indent=2)+'\n')
            run=subprocess.run(command,capture_output=True,timeout=timeout+10)
            (root/'gdb.stdout.log').write_bytes(run.stdout)
            (root/'gdb.stderr.log').write_bytes(run.stderr)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread=threading.Thread(target=debugger,daemon=True)
    original,markers=GATE.qemu_command,GATE.MARKERS

    def command(*args):
        thread.start()
        arguments=original(*args)+['-S','-gdb',f'unix:{debug},server=on,wait=off','-qmp',f'unix:{qmp},server=on,wait=off']
        (root/'qemu-command.json').write_text(json.dumps(arguments,indent=2)+'\n')
        return arguments

    def failure():
        if failed.exists():return failed.read_text()
        if outcome and outcome!=[0]:return 'inactive cancellation debugger failed: '+repr(outcome)
        if result.exists() and outcome==[0]:
            stopped.append(stopped_status(qmp))
            return 'natural inactive cancellation observed at process-controller return'
        return None

    GATE.qemu_command=command
    GATE.MARKERS=(*markers,b'NATURAL_CANCELLATION_REQUIRES_DISCONNECT_AND_QMP')
    try:
        try:
            GATE.observe_gate(values[0],values[1],root/'serial.log',root/'qemu.log',timeout,failure_check=failure)
        except GATE.GateError:
            if failed.exists() or not result.exists():raise
        else:raise GATE.GateError('cancellation observer did not stop at the qualified boundary')
    finally:
        GATE.qemu_command,GATE.MARKERS=original,markers
        thread.join(timeout=15)
    assert outcome==[0] and len(stopped)==1 and not failed.exists()
    observation=json.loads(result.read_text())
    assert observation['result']=='pass'
    source=json.loads((root/'refusal-physical.json').read_text())
    assert source['result']=='pass' and len(source['calls'])==3
    paths=[values[3],values[5],values[3].parent/'kernel.elf',values[0],values[1],
        root/'observation.json',root/'caller-cases.json',root/'syscall-returns.json',root/'refusal-physical.json',
        root/'owner-before-terminal.bin',root/'owner-after-terminal.bin',
        root/'qemu-command.json',root/'debugger-command.json',root/'gdb.stdout.log',root/'gdb.stderr.log',root/'serial.log',root/'qemu.log',
        inspector,LAB/'namespace_syscall_physical.py',LAB/'namespace_source_physical.py',
        Path(CHECKS.__file__),Path(namespace_gate.__file__),Path(GATE.__file__),Path(__file__)]
    record=dict(schema='topal.namespace-inactive-cancellation-physical/1',result='pass',
        stopped_target=stopped[0],observation=observation,hashes={str(p):GATE.digest(p) for p in paths})
    (root/'physical-observation.json').write_text(json.dumps(record,indent=2)+'\n')
    print('NAMESPACE_INACTIVE_CANCELLATION_PASS returns='+str(observation['ordinary_returns'])+' work='+str(root),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    verify(args.output.resolve(),args.timeout)
