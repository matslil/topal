#!/usr/bin/env python3
"""Check ordinary child-exec signal observations; no guest execution or edits."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from process_native import require
from readback_exec import readback as exec_readback
from readback_process import digest


RESET = dict(blocked=1 << 14, quit=[1, 0, 0, 0], user=[0] * 4)


def transition(before, after, entry):
    require(before['blocked'] == 1 << 14, 'nonempty child blocked mask before exec')
    require(before['quit'][0] == 1 and before['user'][0] > 1, 'ignored and caught input dispositions')
    for key in ['quit', 'user']:
        require(before[key][1] == 0x04000000 and before[key][2] > 1
                and before[key][3] == 1 << 14, 'nonempty input action metadata')
    require(after == entry == RESET, 'exec reset and blocked-mask preservation')


def parent(before, after):
    require(before == after, 'parent signal state restored after child exec/wait')


def readback(value):
    result = exec_readback(value)
    executions = [r for r in value['events'] if r['operation'] == 7 and r['before']['ids'][0] >= 2]
    entries = value['child_exec_entries']
    calls = value['signal_exec_calls']
    require(len(calls) == len(executions), 'missing ordinary exec call observations')
    for call, row, entry in zip(calls, executions, entries, strict=True):
        require(call['ids'] == row['before']['ids'], 'ordinary exec call/process binding')
        transition(call['signals'], row['after']['signals'], entry['snapshot']['signals'])
        require(row['before']['signals'] == RESET, 'reset precedes memory gateway')
    for row in value['cases']:
        parent(row['before']['signals'], row['after']['signals'])
    first = executions[0]
    controls = []
    for name, where, field, index, replacement in [
        ('lost-ignore', 1, 'quit', 0, 0),
        ('retained-caught-handler', 1, 'user', 0, calls[0]['signals']['user'][0]),
        ('retained-flags', 1, 'quit', 1, 0x04000000),
        ('retained-restorer', 1, 'user', 2, calls[0]['signals']['user'][2]),
        ('retained-action-mask', 1, 'quit', 3, 1 << 14),
        ('lost-blocked-mask', 1, 'blocked', None, 0),
        ('entry-lost-mask', 2, 'blocked', None, 0),
        ('vacuous-before-mask', 0, 'blocked', None, 0),
    ]:
        rows = deepcopy([calls[0]['signals'], first['after']['signals'], entries[0]['snapshot']['signals']])
        if index is None:
            rows[where][field] = replacement
        else:
            rows[where][field][index] = replacement
        try:
            transition(*rows)
        except ValueError as error:
            controls.append(dict(name=name, rejected=True, reason=str(error)))
        else:
            raise AssertionError('accepted altered signal record: ' + name)
    changed = deepcopy(value['cases'][0]['after']['signals'])
    changed['blocked'] ^= 1 << 14
    try:
        parent(value['cases'][0]['before']['signals'], changed)
    except ValueError as error:
        controls.append(dict(name='changed-parent-mask', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted changed parent signal state')
    result.update(signal_exec_transitions=len(executions), restored_parent_signals=len(value['cases']),
                  signal_controls=controls,
                  signal_scope='Registration/query through ordinary fork/exec/yield/wait; no delivery, pending signals, frames, restart, rt_sigreturn or complete Linux qualification')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observations', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    require(args.output.resolve().is_relative_to(root / 'linux-kernel'), 'Linux output directory')
    raw = args.observations.read_bytes()
    result = readback(json.loads(raw))
    result['observations'] = digest(raw)
    with args.output.open('x') as stream:
        stream.write(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: result[k] for k in ['passed', 'cases', 'signal_exec_transitions', 'restored_parent_signals']}))


if __name__ == '__main__':
    main()
