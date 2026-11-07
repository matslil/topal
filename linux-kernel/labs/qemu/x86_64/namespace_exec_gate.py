#!/usr/bin/env python3
"""Bind namespace and descriptor survival through real same-image exec."""
import argparse
import json
from pathlib import Path

import namespace_gate
import static_namespace_exec_checks as CHECKS


def verify(root,timeout):
    original=namespace_gate.CHECKS.fixture
    namespace_gate.CHECKS.fixture=CHECKS.fixture
    try:
        record=namespace_gate.verify(root,timeout,physical=True)
    finally:
        namespace_gate.CHECKS.fixture=original
    snapshots={name:namespace_gate.GATE.digest(root/name) for name in
        ('filesystem-first-exec-save.bin','filesystem-first-exec-compare.bin')}
    assert len(set(snapshots.values()))==1
    record.update(schema='topal.namespace-exec-lifetime/1',
        exec_caller_sha256=namespace_gate.GATE.digest(Path(CHECKS.__file__)),
        exec_harness_sha256=namespace_gate.GATE.digest(Path(__file__)),
        caller_record_sha256=namespace_gate.GATE.digest(root/'caller-observation.json'),
        shared_filesystem_snapshots=snapshots,
        scope='successful same-image exec chain: exact shared filesystem preservation, aliases/inode identity, descriptor CLOEXEC/access/position, and nonrewinding namespace epochs; inactive cancellation remains separate')
    (root/'exec-qualification.json').write_text(json.dumps(record,indent=2)+'\n')
    print('NAMESPACE_EXEC_PASS calls='+str(record['independent_calls']),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    verify(args.output.resolve(),args.timeout)
