"""Saved native preparation records; no authority, native producer or TLB proof."""
PAGE = 4096
START = 0x5F7000


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def allocation(snapshot, value, count, pins, spans, capture_bytes=True):
    token, record, cells = value['token'], value['record'], value['cells']
    if not count:
        require(token == [0, 0, 0] and record == cells == [] and value['bytes'] == '', 'empty owner')
        return None
    issuer, identity, index = token
    allocator = snapshot['allocator']
    require(issuer == allocator[1] == 1 and 0 < identity < allocator[2] and 0 <= index < allocator[3], 'token')
    require(len(record) == 8 and record[:2] == token[:2] and record[3:5] == [count, 1], 'record')
    require(record[5] > 0 and record[6:] == [pins, 1], 'pin/lifetime')
    require(0 <= record[2] < record[2] + count <= allocator[3], 'cell domain')
    require(len(cells) == count * 3 and cells[0::3] == [identity] * count, 'cell ownership')
    require(cells[1::3] == [record[5]] * count, 'cell provenance')
    first = cells[2]
    require(cells[2::3] == list(range(first, first + count)) and 0 <= first < first + count <= 1 << 18, 'physical extent')
    span = [first * PAGE, (first + count) * PAGE]
    require(all(span[1] <= a or b <= span[0] for a, b in spans), 'reserved/static overlap')
    raw = bytes.fromhex(value['bytes'])
    require(len(raw) == (count * PAGE if capture_bytes else 0), 'whole bytes')
    return dict(span=span, identity=identity, token=token, record=record, cells=cells, bytes=raw)


def snapshot(value):
    allocator = value['allocator']
    require(len(allocator) == 16 and allocator[:2] == [1, 1] and 0 < allocator[3] <= 262144, 'allocator')
    state = value['state']
    require(len(state) == 4 and state[0] == 1 and 0 <= state[2] <= 519, 'active state')
    require(START <= state[1] <= START + 519 * PAGE and
            (state[1] - START + PAGE - 1) // PAGE == state[2], 'actual break geometry')
    require(value['tokens'][3:] == [0, 0, 0] and value['tokens'][:3] == value['active']['token'], 'active token bank')
    require(value['epoch'][0] > 0 and value['epoch'][1] == 12 and value['recovery'][1:3] == [1, 5], 'entry identity/operation')
    require(value['recovery'][5] == value['epoch'][0], 'captured epoch')
    ctx = value['context']
    require(ctx['state'] == 1 and ctx['address'] == ctx['self'] == value['gs_base'], 'current context')
    require(ctx['root'] == value['cr3'] and ctx['heap'] == state[3], 'actual root/heap')
    require(value['cs'] & 3 == 0 and value['flags'] & 0x600 == 0, 'privileged critical state')
    require(value['cr0'] & 0x80010001 == 0x80010001 and value['cr4'] & (1 << 5), 'paging/write protection')
    require(value['cr4'] & ((1 << 7) | (1 << 12) | (1 << 17)) == 0, 'selected paging mode')
    spans = list(value['reserved_spans'])
    require(len(value['static_owners']) == 4, 'static owner roles')
    static = []
    for owned, (count, pins) in zip(value['static_owners'], [(5, 1), (1, 1), (503, 1), (2, 2)]):
        owner = allocation(value, owned, count, pins, spans, False)
        static.append(owner)
        spans.append(owner['span'])
    active = allocation(value, value['active'], state[2], 1, spans)
    require((active['span'][0] if active else 0) == state[3], 'active backing')
    tables = value['tables']
    root, low = ctx['root'], ctx['low']
    require(root == static[0]['span'][0] and low == static[1]['span'][0], 'table ownership')
    require(set(tables) == {str(a) for a in [root, root + PAGE, root + 2 * PAGE,
                                           root + 3 * PAGE, root + 4 * PAGE, low]}, 'complete selected tables')
    require(all(len(words) == 512 for words in tables.values()), 'table words')
    expected = [(root, 0, root + PAGE), (root + PAGE, 0, root + 2 * PAGE),
                (root + 2 * PAGE, 2, low), (root + 2 * PAGE, 3, root + 4 * PAGE)]
    for table, index, target in expected:
        require(tables[str(table)][index] & ~0x60 == target | 7, 'heap hierarchy')
    for i in range(519):
        table, index = (low, 503 + i) if i < 9 else (root + 4 * PAGE, i - 9)
        wanted = ((state[3] + i * PAGE) | (1 << 63) | 7) if i < state[2] else 0
        require(tables[str(table)][index] & ~0x60 == wanted, 'unchanged active heap mapping')
    require(value['details'] == [0] * 4, 'retired resolve facts')
    return active


def check(row):
    before, ready, after = (row[k] for k in ['before', 'ready', 'after'])
    original = snapshot(before)
    for current in [ready, after]:
        snapshot(current)
        for key in ['state', 'tokens', 'epoch', 'recovery', 'context', 'context_bytes', 'reserved_spans',
                    'active', 'static_owners', 'cr0', 'cr3', 'cr4', 'gs_base', 'cs']:
            require(current[key] == before[key], 'changed active owner/context: ' + key)
        for table, words in before['tables'].items():
            for old, new in zip(words, current['tables'][table]):
                require(old & ~0x60 == new & ~0x60 and old | new == new, 'unrelated descriptor change')
    require(before['prepared_state'] == after['prepared_state'] == [0] * 7, 'receipt retirement')
    require(before['prepared']['token'] == after['prepared']['token'] == [0] * 3, 'prepared token retirement')
    count = before['state'][2]
    require(ready['prepared_state'][:3] == [1, before['state'][1], count], 'prepared geometry')
    require(ready['prepared_state'][4:] == [before['cr3'], before['context']['address'], before['epoch'][0]], 'prepared entry binding')
    spans = before['reserved_spans'] + [[o['cells'][2] * PAGE, (o['cells'][2] + o['record'][3]) * PAGE]
                                        for o in before['static_owners']]
    candidate = allocation(ready, ready['prepared'], count, 1, spans)
    expected_issued = before['allocator'][2] + int(count > 0)
    for current in [ready, after]:
        require(current['allocator'] == before['allocator'][:2] + [expected_issued] + before['allocator'][3:], 'allocator issuance/state')
    require(ready['prepared_state'][3] == (candidate['span'][0] if candidate else 0), 'prepared physical alias')
    if count:
        require(candidate['identity'] == before['allocator'][2] and candidate['identity'] != original['identity'], 'distinct issued extent')
        a, b = candidate['span'], original['span']
        require(a[1] <= b[0] or b[1] <= a[0], 'parent/prepared alias')
        require(candidate['bytes'] == original['bytes'], 'complete copied heap')
        erased = row['erased']
        require(erased['base'] == a[0] and erased['count'] == count, 'erasure owner')
        require(erased['token'] == candidate['token'] and erased['record'][:6] == candidate['record'][:6], 'erasure identity')
        require(erased['record'][6:] == [0, 1] and bytes.fromhex(erased['bytes']) == bytes(count * PAGE), 'unpin/physical erasure before release')
        require(after['released_record'] == [0] * 8, 'consumed registry owner')
        cells = after['released_cells']
        require(cells[0::3] == [0] * count and cells[1::3] == candidate['cells'][1::3]
                and cells[2::3] == candidate['cells'][2::3], 'released cell ownership/provenance')
    else:
        require(row['erased'] is None and after['released_record'] == after['released_cells'] == [], 'empty preparation')
    return dict(passed=True, pages=count, copied_bytes=count * PAGE, released_bytes=count * PAGE,
                scope='Observed preparation/cancellation within an actual brk gateway; no process switch, complete mapping-alias, asynchronous or production proof')
