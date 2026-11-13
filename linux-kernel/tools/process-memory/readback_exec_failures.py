#!/usr/bin/env python3
"""Check failed ordinary child exec without changing the native application."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from process_native import require
from readback_exec_signals import readback as signal_readback
from readback_process import digest


def preservation(row):
    require(row['result'] in [-2, -14], 'exec failure errno')
    require(row['returned_cs'] & 3 == 3, 'exec failure returned to userspace')
    require(row['returned_sp'] == row['sp'] + 8 and row['returned_pc'] == row['return_pc'], 'exec failure native continuation')
    require(row['returned_callee'] == row['callee'], 'exec failure callee registers')
    before = row['before']
    require(before == row['after'], 'failed exec changed process state')
    require(before['ids'][0] >= 2 and before['ids'][3] == 1, 'child and retained parent identities')
    require(before['state'][2] == 101 and before['inactive_state'][2] == 67, 'nonempty independent heaps')
    require(before['fs_base'] == before['tls'] == 0x5f7500, 'nonempty child TLS')
    require(before['registered_tid'] == before['robust'] == (1 << 64) - 1, 'nonempty registrations')
    require(before['signals']['blocked'] == 1 << 14 and before['signals']['quit'][0] == 1
            and before['signals']['user'][0] > 1, 'nonempty signal state')
    require(len(before['descriptors']) == 32 and len(before['descriptions']) == len(before['shared_descriptions']) == 128,
            'complete selected descriptor banks')


def readback(value):
    result = signal_readback(value)
    rows = value['failed_exec_calls']
    require(len(rows) == 16, 'four exec refusals for each child')
    grouped = {}
    for row in rows:
        preservation(row)
        grouped.setdefault(tuple(row['before']['ids']), []).append(row)
    calls = value['signal_exec_calls']
    require(set(grouped) == {tuple(call['ids']) for call in calls}, 'failed and successful exec identity binding')
    for call in calls:
        attempts = grouped[tuple(call['ids'])]
        require([r['result'] for r in attempts] == [-2, -14, -14, -14], 'missing/faulting name, argv and env sequence')
        require([attempts[i]['args'][i - 1] for i in range(1, 4)] == [(1 << 64) - 1] * 3,
                'faults use ordinary syscall pointer arguments')
        require(all(r['before']['signals'] == call['signals'] for r in attempts), 'state retained until successful exec')
    controls = []
    for name, field, replacement in [
        ('changed-heap-bytes', 'heap_sha256', '0' * 64),
        ('changed-parent-bytes', 'inactive_heap_sha256', '0' * 64),
        ('lost-TLS', 'fs_base', 0),
        ('lost-TID', 'registered_tid', 0),
        ('lost-robust-list', 'robust', 0),
        ('changed-MM-generation', 'ids', [0] * 7),
        ('lost-descriptor', 'descriptors', [0] * 32),
        ('changed-description', 'descriptions', [0] * 128),
    ]:
        row = deepcopy(rows[0])
        row['after'][field] = replacement
        require(row['after'] != row['before'], 'vacuous saved-record control: ' + name)
        try:
            preservation(row)
        except ValueError as error:
            controls.append(dict(name=name, rejected=True, reason=str(error)))
        else:
            raise AssertionError('accepted altered failed-exec record: ' + name)
    result.update(failed_exec_calls=len(rows), failed_exec_controls=controls,
                  failed_exec_scope='Ordinary ENOENT/EFAULT returns, retained heap owners/bytes, TLS, registrations, selected descriptor banks and signal state; not every exec error or continuous lifetime')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observations', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    require(args.output.resolve().is_relative_to(Path(__file__).resolve().parents[3] / 'linux-kernel'), 'Linux output directory')
    raw = args.observations.read_bytes()
    result = readback(json.loads(raw))
    result['observations'] = digest(raw)
    with args.output.open('x') as stream:
        stream.write(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: result[k] for k in ['passed', 'cases', 'failed_exec_calls']}))


if __name__ == '__main__':
    main()
