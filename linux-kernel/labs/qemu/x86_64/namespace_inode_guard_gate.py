#!/usr/bin/env python3
"""Observe V6 inode admission refusals after actual forked-path lookup."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import topal_gate as GATE
import static_namespace_fork_checks as CHECKS
from qemu_control import stopped_status

LAB=Path(__file__).resolve().parent


def inspect(fixture,root,mode,timeout):
    GATE.verify_host()
    root.mkdir(parents=True)
    linked,boot=fixture/'linked',fixture/'boot'
    image,cases=CHECKS.fixture(GATE.static_user_fixture())
    assert (linked/'static-user.elf').read_bytes()==image
    artifact=json.loads((linked/'provenance.json').read_text())
    assert artifact['inode_mode_runtime']['schema']=='topal.inode-region-invocation.x86_64/2'
    assert next(o['sha256'] for o in artifact['outputs'] if o['identity']=='kernel.elf')==GATE.digest(linked/'kernel.elf')
    debug,qmp,failed,result=(root/n for n in ('debug.sock','qmp.sock','failed.txt','observation.json'))
    assert max(len(str(debug)),len(str(qmp)))<108
    inspector=LAB/'namespace_inode_guard_inspect.py'
    config=dict(artifact=artifact,output=str(root),mode=mode,cases=cases,inspector_sha256=GATE.digest(inspector))
    script=root/'guard.gdb'
    script.write_text(f"set pagination off\nset confirm off\nfile {linked/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import namespace_inode_guard_inspect\nnamespace_inode_guard_inspect.install({config!r})\nend\ncontinue\ndisconnect\n")
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
        if outcome and outcome!=[0]:return 'inode admission debugger failed: '+repr(outcome)
        if result.exists() and outcome==[0]:
            stopped.append(stopped_status(qmp))
            return 'controlled inode admission refusal observed'
        return None

    GATE.qemu_command=command
    GATE.MARKERS=(*markers,b'INODE_ADMISSION_REQUIRES_DISCONNECT_AND_QMP')
    try:
        try:
            GATE.observe_gate(boot/'bzImage',boot/'initramfs.cpio',root/'serial.log',root/'qemu.log',timeout,failure_check=failure)
        except GATE.GateError:
            if failed.exists() or not result.exists():raise
        else:raise GATE.GateError('invalid inode admission resumed completion')
    finally:
        GATE.qemu_command,GATE.MARKERS=original,markers
        thread.join(timeout=15)
    assert outcome==[0] and len(stopped)==1 and not failed.exists()
    observation=json.loads(result.read_text())
    assert observation['result']=='pass'
    paths=[linked/'kernel.elf',linked/'provenance.json',linked/'static-user.elf',boot/'bzImage',boot/'initramfs.cpio',
        root/'observation.json',root/'bad-owner-before.bin',root/'owner-at-refusal.bin',
        root/'qemu-command.json',root/'debugger-command.json',root/'gdb.stdout.log',root/'gdb.stderr.log',root/'serial.log',root/'qemu.log',
        inspector,LAB/'namespace_source_physical.py',LAB/'static_namespace_fork_checks.py',Path(GATE.__file__),Path(__file__)]
    record=dict(schema='topal.inode-namespace-admission-physical/1',result='pass',mode=mode,
        stopped_target=stopped[0],observation=observation,hashes={str(p):GATE.digest(p) for p in paths})
    (root/'physical-observation.json').write_text(json.dumps(record,indent=2)+'\n')
    print('INODE_NAMESPACE_GUARD_PASS mode='+mode+' work='+str(root),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--mode',required=True,choices=('bad-link-count','overlap-namespace'))
    parser.add_argument('--timeout',type=float,default=120)
    args=parser.parse_args()
    inspect(args.fixture.resolve(),args.output.resolve(),args.mode,args.timeout)
