#!/usr/bin/env python3
"""Qualify shared namespace/description ownership through real fork and wait."""
import argparse
import json
from pathlib import Path

import namespace_gate
import static_namespace_fork_checks as CHECKS


def verify(root,timeout):
    original=namespace_gate.CHECKS.fixture
    namespace_gate.CHECKS.fixture=CHECKS.fixture
    try:
        record=namespace_gate.verify(root,timeout,physical=True)
    finally:
        namespace_gate.CHECKS.fixture=original
    record.update(schema='topal.namespace-fork-lifetime/1',
        fork_caller_sha256=namespace_gate.GATE.digest(Path(CHECKS.__file__)),
        fork_harness_sha256=namespace_gate.GATE.digest(Path(__file__)),
        caller_record_sha256=namespace_gate.GATE.digest(root/'caller-observation.json'),
        scope='actual fork/child-exit/wait with shared names/inode/descriptions, independent access and positions, and explicit namespace epoch preservation; exec and inactive cancellation remain separate')
    (root/'fork-qualification.json').write_text(json.dumps(record,indent=2)+'\n')
    print('NAMESPACE_FORK_PASS calls='+str(record['independent_calls']),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    verify(args.output.resolve(),args.timeout)
