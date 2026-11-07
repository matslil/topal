#!/usr/bin/env python3
"""Execute unchanged pinned BusyBox mv/ln against the V6 namespace candidate."""
import argparse
import json
from pathlib import Path

import topal_gate as GATE

WORKLOADS={
    'move':("echo alpha >/tmp/a; /bin/busybox mv /tmp/a /tmp/b; exec /bin/busybox cat /tmp/b",b'alpha\n',0,3),
    'link':("echo alpha >/tmp/a; /bin/busybox ln /tmp/a /tmp/b; exec /bin/busybox stat -c %h /tmp/a /tmp/b",b'2\n2\n',0,3),
    'replace-retained-reader':("echo source >/tmp/a; echo victim >/tmp/b; exec 4</tmp/b; /bin/busybox mv /tmp/a /tmp/b; /bin/busybox cat <&4; exec /bin/busybox cat /tmp/b",b'victim\nsource\n',0,4),
    'alias-mode':("echo alpha >/tmp/a; /bin/busybox ln /tmp/a /tmp/b; /bin/busybox chmod 640 /tmp/b; exec /bin/busybox stat -c %a /tmp/a /tmp/b",b'640\n640\n',0,4),
    'unlink-retained-reader':("echo alpha >/tmp/a; /bin/busybox ln /tmp/a /tmp/b; exec 4</tmp/b; /bin/busybox rm /tmp/a /tmp/b; exec /bin/busybox cat <&4",b'alpha\n',0,4),
    'relative-directory':("echo alpha >/tmp/a; cd /tmp; /bin/busybox ln a b; /bin/busybox mv b c; exec /bin/busybox ls -1 /tmp",b'a\nc\n',0,4),
}


def verify(root,external,timeout):
    root.mkdir(parents=True)
    GATE.verify_host()
    image,_=GATE.checked_static_user_image(external)
    assert GATE.digest(external)==GATE.PINNED_STATIC_BUSYBOX_SHA256
    original=GATE.SOURCE_PATH
    source=original.read_text().replace('static-linux-dispatch-v5','static-linux-dispatch-v6')
    declaration='  namespace-library is lang systems source-library (\n    source is "policy/namespace-action.t",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n'
    if '  namespace-library is' not in source:
        source=source.replace('  inode-mode-library is',declaration+'  inode-mode-library is')
    selected=root/'namespace-root.t'
    selected.write_text(source)
    GATE.SOURCE_PATH=selected
    try:
        values=GATE.build_image(root/'artifact',external_static_user_image=external)
    finally:
        GATE.SOURCE_PATH=original
    assert values[5].read_bytes()==image,'pinned application binary must remain unchanged'
    artifact=json.loads(values[3].read_text())
    assert artifact['dispatch_library']['interface']=='static-linux-dispatch-v6'
    results=[]
    for name,(body,expected,status,commits) in WORKLOADS.items():
        directory=root/name
        directory.mkdir()
        command=("exec /bin/busybox sh -c '"+body+"'\n").encode('ascii')
        (directory/'command.txt').write_bytes(command)
        serial=GATE.observe_gate(values[0],values[1],directory/'serial.log',directory/'qemu.log',timeout,command)
        clean=serial.replace(GATE.EXEC_COMMIT_MARKER,b'')
        witness=f'\nTOPAL_KERNEL_PROCESS_EXIT_STATUS={status:02x}\n'.encode()
        assert b'/\n'+expected+witness in clean,('utility output/status',name,serial)
        assert clean.count(b'\n')==12+expected.count(b'\n'),('utility newline count',name,serial)
        assert serial.count(GATE.EXEC_COMMIT_MARKER)==commits,('exec count',name,serial)
        result=dict(name=name,command=command.decode('ascii'),output=expected.decode('ascii'),
            exit_status=status,successful_execs=commits,
            serial_sha256=GATE.digest(directory/'serial.log'),qemu_log_sha256=GATE.digest(directory/'qemu.log'))
        (directory/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        results.append(result)
        print('NAMESPACE_BUSYBOX_PASS '+name,flush=True)
    record=dict(schema='topal.namespace-unchanged-busybox/1',result='pass',workloads=results,
        application_sha256=GATE.digest(values[5]),kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        artifact_sha256=GATE.digest(values[3]),gate_sha256=GATE.digest(Path(GATE.__file__)),
        harness_sha256=GATE.digest(Path(__file__)),source_sha256=GATE.digest(selected),
        scope='unchanged pinned BusyBox shell/mv/ln/chmod/stat/rm/cat/ls with exact output/status and exec-count assertions on the V6 candidate; complete distribution/container/VM-host acceptance remains separate')
    (root/'utility-qualification.json').write_text(json.dumps(record,indent=2)+'\n')
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--external-static-user-image',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=120)
    args=parser.parse_args()
    verify(args.output.resolve(),args.external_static_user_image.resolve(),args.timeout)
