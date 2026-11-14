#!/usr/bin/env python3
"""Qualify the complete selected BusyBox corpus using one immutable artifact."""
import argparse
import json
from pathlib import Path
import tempfile

import topal_gate as GATE


def verify(root,external,timeout):
    root.mkdir(parents=True)
    GATE.checked_static_user_image(external)
    executions={}

    def verify_one(label,*arguments):
        captured=[]
        original_observe=GATE.observe_gate

        def observe(image,resource,serial,qemu_log,*positional,**keywords):
            result=original_observe(image,resource,serial,qemu_log,*positional,**keywords)
            captured.append(dict(serial=str(serial),serial_sha256=GATE.digest(serial),
                qemu_log=str(qemu_log),qemu_log_sha256=GATE.digest(qemu_log)))
            return result

        GATE.observe_gate=observe
        try:
            record=GATE.verify(*arguments)
        finally:
            GATE.observe_gate=original_observe
        assert len(captured)==1
        assert captured[0]['serial_sha256']==record['observations']['serial_sha256']
        executions[label]=captured[0]
        (root/'execution-logs.json').write_text(json.dumps(executions,indent=2)+'\n')
        return record

    clean=verify_one('clean',timeout,True)
    (root/'clean.json').write_text(json.dumps(clean,indent=2)+'\n')
    records={}
    with tempfile.TemporaryDirectory(prefix='topal-corpus-',dir='/tmp') as temporary:
        values=GATE.build_image(Path(temporary)/'artifact',external)
        original=GATE.build_image
        GATE.build_image=lambda *_args,**_keywords:values
        try:
            for workload in GATE.PROCESS_WORKLOADS:
                record=verify_one(workload,timeout,True,external,workload)
                assert record['linked_kernel_sha256']==clean['linked_kernel_sha256']
                assert record['static_user_image_sha256']==GATE.PINNED_STATIC_BUSYBOX_SHA256
                (root/(workload+'.json')).write_text(json.dumps(record,indent=2)+'\n')
                records[workload]=dict(path=workload+'.json',sha256=GATE.digest(root/(workload+'.json')),
                    exit_status=record['process_exit_status'])
                print('COMPLETE_CORPUS_PASS '+workload,flush=True)
        finally:
            GATE.build_image=original
    environments={}
    for mode in ('empty','null'):
        record=verify_one('environment-'+mode,timeout,True,None,'true',mode)
        assert record['linked_kernel_sha256']==clean['linked_kernel_sha256']
        path=root/('environment-'+mode+'.json')
        path.write_text(json.dumps(record,indent=2)+'\n')
        environments[mode]=dict(path=path.name,sha256=GATE.digest(path))
    record=dict(schema='topal.complete-busybox-workload-corpus/1',result='pass',
        primary_kernel_sha256=clean['linked_kernel_sha256'],
        busybox_sha256=GATE.PINNED_STATIC_BUSYBOX_SHA256,
        clean_record_sha256=GATE.digest(root/'clean.json'),workloads=records,environments=environments,
        harness_sha256=GATE.digest(Path(__file__)),gate_sha256=GATE.digest(Path(GATE.__file__)),
        execution_logs_sha256=GATE.digest(root/'execution-logs.json'),
        scope='complete selected unchanged static BusyBox corpus and clean environment controls; dynamic distribution, containers and VM hosting remain required')
    (root/'qualification.json').write_text(json.dumps(record,indent=2)+'\n')
    print('COMPLETE_CORPUS_PASS workloads='+str(len(records)),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--external-static-user-image',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=120)
    args=parser.parse_args()
    verify(args.output.resolve(),args.external_static_user_image.resolve(),args.timeout)
