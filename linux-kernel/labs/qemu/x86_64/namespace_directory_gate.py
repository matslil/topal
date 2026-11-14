#!/usr/bin/env python3
"""Execute and bind the directory/metadata caller through the V6 public gate."""
import argparse
import json
from pathlib import Path

import namespace_gate
import static_namespace_directory_checks as CHECKS


def verify(root,timeout):
    original=namespace_gate.CHECKS.fixture
    namespace_gate.CHECKS.fixture=CHECKS.fixture
    try:
        record=namespace_gate.verify(root,timeout,physical=True)
    finally:
        namespace_gate.CHECKS.fixture=original
    record.update(schema='topal.namespace-directory-consumers/1',
        directory_caller_sha256=namespace_gate.GATE.digest(Path(CHECKS.__file__)),
        directory_harness_sha256=namespace_gate.GATE.digest(Path(__file__)),
        caller_record_sha256=namespace_gate.GATE.digest(root/'caller-observation.json'),
        scope='actual independently encoded user getdents/stat/chmod/open/close calls after entry and inode slots diverge, source effects and retirement for link/exchange')
    (root/'directory-qualification.json').write_text(json.dumps(record,indent=2)+'\n')
    print('NAMESPACE_DIRECTORY_PASS calls='+str(record['independent_calls']),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    verify(args.output.resolve(),args.timeout)
