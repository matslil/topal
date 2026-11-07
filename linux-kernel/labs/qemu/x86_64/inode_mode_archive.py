#!/usr/bin/env python3
"""Publish fresh inode consumer observations without replacing historical proof."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil

import topal_gate as GATE

LAB=Path(__file__).resolve().parent
WORKLOADS=('mode-numeric','mode-symbolic','mode-directory',
    'mode-retained-reader','mode-umask-independent')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(suite,command_record,corpus_cohort,rust_record,output):
    physical=json.loads((suite/'qualification.json').read_text())
    assert physical['result']=='pass' and len(physical['cases'])==12
    assert physical['suite_sha256']==GATE.digest(LAB/'inode_mode_suite.py')
    command=json.loads(command_record.read_text())
    assert command['complete'] and command['passed']
    assert command['gate']=='linux-kernel/labs/qemu/x86_64/inode_mode_refresh.py'
    assert command['gate_sha256']==GATE.digest(LAB/'inode_mode_refresh.py')
    assert command['wrapper_sha256']==GATE.digest(LAB/'qualification_gate.py')
    cohort=json.loads((corpus_cohort/'cohort.json').read_text())
    assert cohort['phase']=='corpus' and cohort['complete'] and cohort['passed']
    rust=json.loads(rust_record.read_text())
    assert rust['complete'] and rust['passed']
    assert command['head']==cohort['head']==rust['head']
    corpus=corpus_cohort/'corpus'
    collection=json.loads((corpus/'qualification.json').read_text())
    assert collection['result']=='pass' and len(collection['workloads'])==51
    kernel=physical['linked_kernel_sha256']
    selected=json.loads((LAB/'results/topal-toolchain-gate.json').read_text())
    assert kernel==selected['linked_kernel_sha256']==collection['primary_kernel_sha256']
    for name,case in physical['cases'].items():
        for key,filename in (('harness_sha256','inode_mode_gate.py'),
            ('inspector_sha256','inode_mode_inspect.py'),('caller_sha256','static_inode_mode_checks.py'),
            ('gate_sha256','topal_gate.py')):
            assert case[key]==GATE.digest(LAB/filename)
        assert case['binding']['schema']=='topal.inode-region-invocation.x86_64/2'
        if name not in ('false-after-mode','wrong-mode'):
            assert case['linked_kernel_sha256']==kernel
    utilities={}
    executions=json.loads((corpus/'execution-logs.json').read_text())
    assert GATE.digest(corpus/'execution-logs.json')==collection['execution_logs_sha256']
    utility_logs={}
    for name in WORKLOADS:
        entry=collection['workloads'][name]
        path=corpus/entry['path']
        assert GATE.digest(path)==entry['sha256']
        record=json.loads(path.read_text())
        assert record['observations']['result']=='pass' and record['linked_kernel_sha256']==kernel
        assert record['static_user_image_sha256']==GATE.PINNED_STATIC_BUSYBOX_SHA256
        assert record['process_output_utf8'].encode()==GATE.PROCESS_WORKLOADS[name][1]
        execution=executions[name]
        serial=Path(execution['serial'])
        qemu_log=Path(execution['qemu_log'])
        assert serial.resolve().is_relative_to(corpus_cohort)
        assert qemu_log.parent==serial.parent
        assert GATE.digest(serial)==execution['serial_sha256']==record['observations']['serial_sha256']
        assert GATE.digest(qemu_log)==execution['qemu_log_sha256']
        utilities[name]=record
        utility_logs[name]=serial.parent
    output.mkdir(parents=True)
    shutil.copyfile(suite/'qualification.json',output/'inode-qualification.json')
    utility=dict(result='pass',busybox_sha256=GATE.PINNED_STATIC_BUSYBOX_SHA256,
        workloads=utilities,corpus_record_sha256=GATE.digest(corpus/'qualification.json'))
    (output/'inode-busybox.json').write_text(json.dumps(utility,indent=2)+'\n')
    logs=[]

    def archive(path,name):
        data=path.read_bytes()
        target=output/(name+'.gz')
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(data,mtime=0))
        assert gzip.decompress(target.read_bytes())==data
        logs.append(dict(path=str(target.relative_to(output)),uncompressed_bytes=len(data),
            uncompressed_sha256=digest(data)))

    for name in physical['cases']:
        directory=suite/('inactive-terminal' if name=='inactive_terminal' else name)
        for filename in ('serial.log','qemu.log','debug.log'):
            archive(directory/filename,'machine/'+name+'/'+filename)
    for name in WORKLOADS:
        for filename in ('serial.log','qemu.log'):
            archive(utility_logs[name]/filename,'busybox/'+name+'/'+filename)
    assert len(logs)==46
    record=dict(schema='topal.inode-consumer-archive/1',machine_dispositions=12,
        busybox_workloads=5,logs=logs,qualified_code_commit=rust['head'],
        qualified_code_tree=rust['tree'],rust_record_sha256=GATE.digest(rust_record),
        command_record_sha256=GATE.digest(command_record),
        corpus_record_sha256=GATE.digest(corpus/'qualification.json'),
        archiver_sha256=GATE.digest(Path(__file__)),
        scope='fresh V6 inode consumer machine and unchanged BusyBox proof; historical TK280 evidence remains intact')
    (output/'archive.json').write_text(json.dumps(record,indent=2)+'\n')
    print('INODE_CONSUMER_ARCHIVE_PASS dispositions=12 workloads=5 logs=46',flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite',required=True,type=Path)
    parser.add_argument('--command-record',required=True,type=Path)
    parser.add_argument('--corpus-cohort',required=True,type=Path)
    parser.add_argument('--rust-record',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    verify(args.suite.resolve(),args.command_record.resolve(),args.corpus_cohort.resolve(),
        args.rust_record.resolve(),args.output.resolve())
