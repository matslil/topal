#!/usr/bin/env python3
"""Build fresh V6 inode source variants and run all physical dispositions."""
import argparse
import json
from pathlib import Path
import shutil

import inode_mode_gate as MODE
import inode_mode_suite as SUITE


def verify(root,timeout):
    root.mkdir(parents=True)
    MODE.GATE.verify_host()
    artifacts={}
    for name,lifetime in (('primary',False),('lifetime',True),
                          ('false-after-mode',False),('wrong-mode',False)):
        output=root/name
        output.mkdir()
        libraries=None
        if name in ('false-after-mode','wrong-mode'):
            libraries=output/'libraries'
            shutil.copytree(MODE.GATE.LINUX_KERNEL/'kernel',libraries)
            path=libraries/'support/inode-metadata.t'
            old,new=(('value is mode)','value is 0)') if name=='wrong-mode' else
                ('true then write-time (region, seconds)','true then false'))
            source=path.read_text()
            assert source.count(old)==1
            path.write_text(source.replace(old,new))
        values,_calls=MODE.build(output/'artifact',lifetime,libraries)
        artifact=json.loads(values[3].read_text())
        assert artifact['dispatch_library']['interface']=='static-linux-dispatch-v6'
        assert artifact['inode_mode_runtime']['schema']=='topal.inode-region-invocation.x86_64/2'
        artifacts[name]=output/'artifact'
    args=argparse.Namespace(output=root/'physical',timeout=timeout,
        primary=artifacts['primary'],lifetime=artifacts['lifetime'],
        false_after_mode=artifacts['false-after-mode'],wrong_mode=artifacts['wrong-mode'])
    result=SUITE.verify(args)
    (root/'refresh.json').write_text(json.dumps(dict(schema='topal.inode-consumer-refresh/1',
        result='pass',physical_record_sha256=MODE.GATE.digest(root/'physical/qualification.json'),
        harness_sha256=MODE.GATE.digest(Path(__file__)),
        artifacts={key:str(value) for key,value in artifacts.items()}),indent=2)+'\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    verify(args.output.resolve(),args.timeout)
