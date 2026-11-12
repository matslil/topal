#!/usr/bin/env python3
"""Replay process/MM boundary observations and saved-data negative controls.

This checks saved observations, not native execution, continuous exclusion,
each invalidation instruction, fatal cleanup, or public kernel qualification.
"""
import argparse
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from process_native import event as boundary_event, require, snapshot


def event(row):
    result = boundary_event(row)
    if row['operation'] == 5:
        before, after = row['before'], row['after']
        request = row['request']
        current = before['state'][1] if before['state'][0] else 0x5f7000
        require(request == before['context']['user_arg'], 'captured break argument')
        require(row['result'] == after['state'][1], 'raw break return')
        if request == 0 or not 0x5f7000 <= request <= 0x7fe000:
            require(row['result'] == current, 'query/refused break result')
        else:
            require(row['result'] in [request,current], 'resize result')
        old, new = bytes.fromhex(before['active']['bytes']), bytes.fromhex(after['active']['bytes'])
        retained = min(len(old),len(new))
        require(old[:retained] == new[:retained], 'resize retained prefix')
        if len(new)>len(old):
            require(not any(new[len(old):]), 'new heap pages not zero')
    return result


def digest(raw):
    return dict(sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))


def negative_controls(events):
    selected = {op: next(row for row in events if row['operation'] == op)
                for op in [5, 9, 10, 11, 12]}
    query = next(row for row in events if row['operation'] == 5 and row['request'] == 0)
    changes = [
        ('prepared-pin-missing', 9, ['after', 'prepared', 'record', 6], 0),
        ('prepared-wrong-epoch', 9, ['after', 'prepared_state', 6], 0),
        ('prepared-wrong-source-generation', 9, ['after', 'prepared_owner', 2], 999),
        ('prepared-truncated-to-old-34-pages', 9, ['after', 'prepared', 'bytes'],
         selected[9]['after']['prepared']['bytes'][:34*4096*2]),
        ('child-adopts-parent-MM', 10, ['after', 'ids', 1], selected[10]['before']['ids'][1]),
        ('child-identity-counter-restored', 10, ['after', 'ids', 6], selected[10]['before']['ids'][6]),
        ('handoff-loses-parent', 11, ['after', 'inactive_state', 0], 0),
        ('handoff-reuses-heap-PTE', 11,
         ['after', 'tables', str(selected[11]['after']['context']['low']), 503],
         selected[11]['before']['tables'][str(selected[11]['before']['context']['low'])][503]),
        ('exit-missing-retirement', 12, ['retired'], []),
        ('exit-truncated-erasure', 12, ['retired', 0, 'bytes'], ''),
        ('exit-retains-child-MM', 12, ['after', 'ids', 3], selected[12]['before']['ids'][0]),
        ('hardware-root-changed', 9, ['after', 'cr3'], 0),
        ('write-protection-disabled', 9, ['after', 'cr0'], selected[9]['after']['cr0'] & ~(1 << 16)),
        ('retained-resize-candidate', 9, ['after', 'tokens', 3], 1),
    ]
    results = []
    for name, op, path, replacement in changes:
        row = deepcopy(selected[op])
        target = row
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = replacement
        try:
            event(row)
        except ValueError as error:
            results.append(dict(name=name, rejected=True, reason=str(error)))
        else:
            raise AssertionError('accepted saved-data control: ' + name)
    row = deepcopy(query)
    row['after']['allocator'][2] += 1
    try:
        event(row)
    except ValueError as error:
        results.append(dict(name='query-consumes-frame-identity', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted query allocation')
    row = deepcopy(selected[9])
    row['before']['epoch'][1] = row['after']['epoch'][1] = 12
    try:
        event(row)
    except ValueError as error:
        results.append(dict(name='wrong-syscall', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted wrong syscall')
    row = deepcopy(selected[9])
    raw = bytearray.fromhex(row['after']['prepared']['bytes'])
    raw[34*4096+128] ^= 1
    row['after']['prepared']['bytes'] = raw.hex()
    try:
        event(row)
    except ValueError as error:
        results.append(dict(name='wrong-child-copy-beyond-page-34', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted incomplete child copy')
    row = deepcopy(selected[12])
    row['retired'][0]['bytes'] = '01' + row['retired'][0]['bytes'][2:]
    try:
        event(row)
    except ValueError as error:
        results.append(dict(name='exit-nonzero-erasure', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted nonzero retired backing')
    resize = next(r for r in events if r['operation']==5 and r['before']['state'][2]>34
                  and r['after']['state'][2]>r['before']['state'][2])
    for name,offset in [('resize-loses-retained-byte',34*4096+128),
                        ('resize-new-page-nonzero',resize['before']['state'][2]*4096)]:
        row=deepcopy(resize)
        raw=bytearray.fromhex(row['after']['active']['bytes'])
        raw[offset]^=1
        row['after']['active']['bytes']=raw.hex()
        try:
            event(row)
        except ValueError as error:
            results.append(dict(name=name,rejected=True,reason=str(error)))
        else:
            raise AssertionError('accepted saved-data control: '+name)
    return results


def readback(value):
    require(value['passed'] and value.get('failure') is None, 'native run incomplete')
    require(value['pending'] is None and value['active_case'] is None, 'native work retained')
    events, cases = value['events'], value['cases']
    comparisons = [event(row) for row in events]
    require(all(a == b['comparison'] for a, b in zip(comparisons, events)), 'recorded comparison differs')
    counts = Counter(row['operation'] for row in events)
    require(len(cases) == 4 and [counts[n] for n in [9,10,11,12]] == [8,8,8,8], 'native process scenarios')
    require(sorted(row['before']['state'][2] for row in events if row['operation']==9)
            == [34]*4+[67]*4, 'original and large-heap creation cases')
    require([row['before']['ids'][2] for row in cases] == [0,1,2,3], 'image generations')
    for row in cases:
        before, after = row['before'], row['after']
        snapshot(before)
        snapshot(after)
        require(row['returned']['result']==0 and row['returned']['cs']&3==3
                and row['returned']['sp']==row['sp']+8 and row['returned']['callee']==row['callee'], 'ordinary native case return')
        require(before['ids'][:3] == after['ids'][:3] and after['ids'][3:6] == [0]*3, 'parent MM changed')
        require(after['ids'][6] == before['ids'][6] + 1, 'fork MM issuance')
        require(before['state'][1:3] == after['state'][1:3] == [0x619000,34], 'parent break restoration')
        require(digest(bytes.fromhex(before['active']['bytes']))['sha256'] == row['prefix']
                == digest(bytes.fromhex(after['active']['bytes']))['sha256'], 'parent prefix restoration')
    require(counts[7] == 3 and counts[6] == 1, 'outer exec/retirement count')
    require(set(value['final'])=={'ids','inactive_token','inactive_state','prepared_token',
                               'prepared_state','prepared_owner','transition'}
            and all(not any(v) for v in value['final'].values()), 'final owner storage')
    retired = [r for row in events for r in row['retired']]
    return dict(passed=True, cases=len(cases), operations=dict(sorted(counts.items())),
        comparisons=comparisons, controls=negative_controls(events),
        copied_heap_bytes=sum(row['before']['state'][2]*4096 for row in events if row['operation']==9),
        erased_released_bytes=sum(r['count']*4096 for r in retired),
        retired_extents=len(retired),
        allocation_free_queries=sum(row['operation']==5 and row['request']==0 for row in events),
        allocation_free_within_page_changes=sum(row['operation']==5 and row['request']!=0
            and row['request']==row['after']['state'][1]
            and row['before']['state'][2]==row['after']['state'][2] for row in events),
        scope='Experimental native boundary records and saved-data controls; continuous publication/loan/fatal/public qualification remains open')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observations', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    output = args.output.resolve()
    require(output.is_relative_to(root/'linux-kernel'), 'Linux output directory')
    raw = args.observations.read_bytes()
    result = readback(json.loads(raw))
    result['observations'] = digest(raw)
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('x') as stream:
        stream.write(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:result[k] for k in ['passed','cases','operations','copied_heap_bytes','erased_released_bytes','retired_extents']}))


if __name__ == '__main__':
    main()
