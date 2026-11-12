"""TOPAL-SYSTEMS-PROCESS-MM-001 saved process/MM boundary checks.

Records describe observations; they cannot construct native ownership or prove
continuous exclusion, every invalidation instruction or public qualification.
"""
PAGE = 4096
START = 0x5F7000


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def owner(row, token, allocator, data=True):
    require(row['token'] == token, 'token binding')
    if token == [0, 0, 0]:
        require(row['record'] == row['cells'] == [] and row['bytes'] == '', 'empty allocation')
        return None
    issuer, identity, index = token
    require(issuer == allocator[1] == 1 and 0 < identity < allocator[2] and 0 <= index < allocator[3], 'live issuance')
    r, cells = row['record'], row['cells']
    require(len(r) == 8 and r[:2] == token[:2] and r[7] == 1 and r[5] > 0, 'live record')
    count = r[3]
    require(0 < count <= allocator[3] and r[4] > 0 and r[4] & (r[4] - 1) == 0
            and 0 <= r[2] < r[2] + count <= allocator[3], 'allocation geometry')
    require(len(cells) == count * 3 and cells[0::3] == [identity] * count and cells[1::3] == [r[5]] * count, 'owned cells')
    first = cells[2]
    require(first % r[4] == 0, 'allocation alignment')
    require(cells[2::3] == list(range(first, first + count)) and 0 <= first < first + count <= 1 << 18, 'physical frames')
    raw = bytes.fromhex(row['bytes'])
    require(len(raw) == (count * PAGE if data else 0), 'whole allocation bytes')
    return dict(token=token, span=[first * PAGE, (first + count) * PAGE], count=count,
                pins=r[6], data=raw, record=r, cells=cells)


def bank(value, state, token, allocator):
    allocated = owner(value, token, allocator)
    require(len(state) in [4, 7], 'state width')
    if state == [0] * len(state):
        require(allocated is None, 'owner without state')
        return None
    require(state[0] == 1 and START <= state[1] <= START + 519 * PAGE, 'MM state')
    require((state[1] - START + PAGE - 1) // PAGE == state[2] and 0 <= state[2] <= 519, 'byte break')
    if not state[2]:
        require(state[3] == 0 and allocated is None, 'zero-page heap')
    else:
        require(allocated is not None and allocated['count'] == state[2]
                and allocated['span'][0] == state[3] and allocated['pins'] == 1
                and allocated['record'][4] == 1, 'heap backing/pin')
    return allocated


def mappings(value, count, base):
    ctx, tables = value['context'], value['tables']
    root, low = ctx['root'], ctx['low']
    require(set(tables) == {str(a) for a in [root, root + PAGE, root + 2 * PAGE, root + 3 * PAGE, root + 4 * PAGE, low]}, 'table set')
    require(all(len(words) == 512 for words in tables.values()), 'complete tables')
    for table, index, target in [(root, 0, root + PAGE), (root + PAGE, 0, root + 2 * PAGE),
                                 (root + 2 * PAGE, 2, low), (root + 2 * PAGE, 3, root + 4 * PAGE)]:
        require(tables[str(table)][index] & ~0x60 == target | 7, 'hierarchy')
    for i in range(519):
        table, index = (low, 503 + i) if i < 9 else (root + 4 * PAGE, i - 9)
        expected = ((base + i * PAGE) | (1 << 63) | 7) if i < count else 0
        require(tables[str(table)][index] & ~0x60 == expected, 'selected heap translations')


def snapshot(value, stable=True):
    allocator = value['allocator']
    require(len(allocator) == 16 and allocator[:2] == [1, 1] and 0 < allocator[3] <= 262144, 'allocator')
    require(value['details'] == [0] * 4, 'resolve scratch')
    active = bank(value['active'], value['state'], value['tokens'][:3], allocator)
    inactive = bank(value['inactive'], value['inactive_state'], value['inactive']['token'], allocator)
    prepared = bank(value['prepared'], value['prepared_state'], value['prepared']['token'], allocator)
    candidate = owner(value['candidate'], value['tokens'][3:], allocator)
    ids = value['ids']
    require(len(ids) == 7, 'MM identities')
    if ids[0] == 0:
        require(ids == [0] * 7 and inactive is None and prepared is None, 'unbound identities')
    else:
        require(0 < ids[0] <= 2147483647 and 0 < ids[1] < ids[6], 'active MM identity')
        if ids[3]:
            require(0 < ids[3] <= 2147483647 and ids[3] != ids[0] and 1 in [ids[0], ids[3]], 'pair PIDs')
            require(0 < ids[4] < ids[6] and ids[4] != ids[1] and value['inactive_state'][0] == 1, 'inactive MM identity')
        else:
            require(ids[3:6] == [0] * 3 and value['inactive_state'] == [0] * 4, 'empty inactive owner')
    owned = [o for o in [active, inactive, prepared, candidate] if o is not None]
    for row in value['root_owners']:
        owned.append(owner(row, row['token'], allocator, False))
    require(any(o['span'][0] == value['context']['root'] and o['count'] == 5 and o['pins'] == 1
                for o in owned if o is not None), 'owned hierarchy')
    require(any(o['span'][0] == value['context']['low'] and o['count'] == 1 and o['pins'] == 1
                for o in owned if o is not None), 'owned low table')
    spans = list(value['kernel_spans'])
    identities = []
    for allocation in owned:
        a, b = allocation['span']
        require(all(b <= x or y <= a for x, y in spans), 'backing overlap')
        require(allocation['token'][1] not in identities, 'aliased handle')
        identities.append(allocation['token'][1])
        spans.append([a, b])
    if stable:
        require(candidate is None and value['transition'] == [0] * 3, 'incomplete transfer')
        mappings(value, value['state'][2], value['state'][3])
    return dict(active=active, inactive=inactive, prepared=prepared, candidate=candidate)


def same_bank(a, b, name):
    require(a[name] == b[name] and a[name + '_state'] == b[name + '_state'], 'changed retained ' + name)


def firmware_frames(evidence):
    """All complete RAM pages, excluding every reservation/non-RAM overlap."""
    ram=[];forbidden=list(evidence['reservations'])
    for base,length,kind in evidence['e820']:
        require(0<=base<base+length<=1<<64 and length>0 and kind>0, 'firmware interval')
        if kind==1:
            require(base+length<=1<<30, 'firmware supported RAM domain')
            ram.append((base,base+length))
        else:forbidden.append((base,base+length))
    require(len(evidence['reservations'])==2 and all(0<=a<b<=1<<30 for a,b in evidence['reservations']), 'firmware reservations')
    frames=set()
    for a,b in ram:
        for frame in range((a+PAGE-1)//PAGE,b//PAGE):
            if all((frame+1)*PAGE<=x or y<=frame*PAGE for x,y in forbidden):frames.add(frame)
    return sorted(frames)


def physical_catalog(value):
    """Independent whole-catalog/registry readback for the small-RAM cohort."""
    allocator=value['allocator'];n=allocator[3]
    require('catalog' in value and 'registry' in value, 'missing complete physical catalog')
    cells,registry=value['catalog'],value['registry']
    require(len(cells)==3*n and len(registry)==8*n, 'complete physical catalog/registry')
    physical=cells[2::3]
    require(physical==sorted(set(physical)) and all(0<=p<1<<18 for p in physical), 'catalog physical identity')
    require(all(p>0 for p in cells[1::3]), 'catalog provenance')
    records={};free_records=0
    for index in range(n):
        r=registry[index*8:(index+1)*8]
        if r==[0]*8:
            free_records+=1;continue
        require(r[0]==allocator[1]==1 and 0<r[1]<allocator[2] and r[7]==1, 'catalog live record')
        require(r[1] not in records and 0<=r[2]<r[2]+r[3]<=n and r[3]>0
                and r[4]>0 and r[4]&(r[4]-1)==0 and r[5]>0, 'catalog record geometry')
        part=cells[r[2]*3:(r[2]+r[3])*3]
        require(part[0::3]==[r[1]]*r[3] and part[1::3]==[r[5]]*r[3], 'catalog owned span')
        require(part[2]%r[4]==0 and part[2::3]==list(range(part[2],part[2]+r[3])), 'catalog contiguous record')
        records[r[1]]=(index,r)
    for index in range(n):
        identity=cells[index*3]
        if identity:
            require(identity in records, 'catalog cell lacks record')
            r=records[identity][1]
            require(r[2]<=index<r[2]+r[3], 'catalog cell outside record')
    for owner in [value[k] for k in ['active','inactive','prepared','candidate']]+value['root_owners']:
        if owner['token']!=[0]*3:
            _,identity,index=owner['token'];r=owner['record']
            require(records.get(identity)==(index,r) and owner['cells']==cells[r[2]*3:(r[2]+r[3])*3], 'catalog owner binding')
    free=largest=run=0;previous=None
    for i in range(n):
        identity,provenance,frame=cells[i*3:i*3+3]
        if identity==0:
            free+=1
            run=run+1 if previous is not None and previous==(provenance,frame-1) else 1
            largest=max(largest,run);previous=(provenance,frame)
        else:
            run=0;previous=None
    return dict(frames=n,free=free,largest_run=largest,free_records=free_records)


def preparation_refusal(row):
    before,after=row['before'],row['after']
    require(row['operation']==9 and row['result']==0 and row['request']==0, 'physical refusal result')
    require(before['ids'][0]==before['context']['pid']==1 and before['ids'][3:6]==[0]*3
            and before['context']['relationship']==0, 'physical refusal source')
    require(before['state'][2]>0, 'physical refusal nonempty request')
    for value in [before,after]:
        require(value['prepared_state']==[0]*7 and value['prepared_owner']==[0]*3
                and value['prepared']['token']==[0]*3, 'refusal retained preparation')
    for key in ['allocator','ids','state','active','inactive','inactive_state','catalog','registry','tables','identity_store']:
        require(before[key]==after[key], 'refusal changed '+key)
    catalog=physical_catalog(before);physical_catalog(after)
    require(catalog['free_records']>0 and before['allocator'][2]<(1<<64)-1, 'refusal is not registry/identity exhaustion')
    require(catalog['largest_run']<before['state'][2], 'physical refusal has an eligible run')
    return catalog


def resize_refusal(row):
    before,after=row['before'],row['after'];request=row['request']
    require(START<request<=START+519*PAGE and row['result']==before['state'][1], 'raw resize refusal')
    count=(request-START+PAGE-1)//PAGE
    require(count!=before['state'][2], 'resize refusal geometry')
    for key in ['allocator','ids','state','active','inactive','inactive_state','catalog','registry','tables','identity_store']:
        require(before[key]==after[key], 'resize refusal changed '+key)
    catalog=physical_catalog(before);physical_catalog(after)
    require(catalog['free_records']>0 and before['allocator'][2]<(1<<64)-1, 'resize refusal metadata exhaustion')
    require(catalog['largest_run']<count, 'resize refusal has eligible run')
    return catalog


def event(row):
    before, after = row['before'], row['after']
    first, last = snapshot(before), snapshot(after)
    op, request = row['operation'], row['request']
    for value in [before, after]:
        require(value['cs'] & 3 == 0 and value['flags'] & 0x600 == 0, 'kernel critical state')
        require(value['cr0'] & 0x80010001 == 0x80010001 and value['cr4'] & 0x20, 'paging/write protection')
        require(value['cr4'] & ((1 << 7) | (1 << 12) | (1 << 17)) == 0, 'selected paging mode')
        if op == 6:
            require(value['context']['state'] == 2 and value['cr3'] == value['context']['bootstrap'], 'detached cleanup root')
        else:
            require(value['context']['state'] == 1 and value['cr3'] == value['context']['root']
                    and value['gs_base'] == value['context']['self'] == value['context']['address'], 'actual live root/context')
    require(before['epoch'] == after['epoch'] and before['epoch'][0] > 0, 'entry changed')
    numbers = {5: [12], 6: [60, 231], 7: [59], 9: [56, 57], 10: [56, 57],
               11: [0, 1, 19, 20, 24, 61], 12: [60, 231], 13: [60, 231]}
    require(op in numbers and before['epoch'][1] in numbers[op], 'syscall identity')
    require(before['context']['root'] == after['context']['root'] and before['cr3'] == after['cr3'], 'root changed')
    require(before['tokens'][3:] == after['tokens'][3:] == [0] * 3, 'resize candidate retained')
    require(before['root_owners'] == after['root_owners'], 'root owners changed')
    if op == 9:
        require(row['result'] in [0,1] and request == 0 and before['ids'][0] == 1, 'creation preparation')
        require(before['state'] == after['state'] and before['active'] == after['active'] and before['ids'] == after['ids'], 'changed source during preparation')
        if row['result']==0:
            preparation_refusal(row)
        else:
            require(after['prepared_state'][:3] == before['state'][:3], 'prepared byte geometry')
            require(after['prepared_state'][4:] == [before['context']['root'], before['context']['address'], before['epoch'][0]], 'prepared entry receipt')
            require(after['prepared_owner'] == before['ids'][:3], 'prepared source owner')
            if first['active']:
                require(last['prepared']['data'] == first['active']['data'], 'incomplete fork copy')
            require(after['allocator'][2] == before['allocator'][2] + int(before['state'][2] > 0), 'preparation issuance')
    elif op == 10:
        require(row['result'] == 1 and request >= 2 and before['prepared_owner'] == before['ids'][:3], 'creation commit')
        require(after['active'] == before['prepared'] and after['state'] == before['prepared_state'][:4], 'child adoption')
        require(after['inactive'] == before['active'] and after['inactive_state'] == before['state'], 'parent retention')
        require(after['ids'] == [request, before['ids'][6], before['ids'][2]] + before['ids'][:3] + [before['ids'][6] + 1], 'child MM identity')
    elif op == 11:
        require(row['result'] == 1 and request == 0, 'handoff result')
        require(after['active'] == before['inactive'] and after['state'] == before['inactive_state'], 'target handoff')
        require(after['inactive'] == before['active'] and after['inactive_state'] == before['state'], 'source handoff')
        require(after['ids'] == before['ids'][3:6] + before['ids'][:3] + before['ids'][6:], 'handoff identities')
    elif op == 12:
        require(row['result'] == 1 and before['ids'][0] >= 2 and before['ids'][3] == 1, 'child exit')
        require(after['active'] == before['inactive'] and after['state'] == before['inactive_state'], 'parent restoration')
        require(after['ids'] == before['ids'][3:6] + [0] * 3 + before['ids'][6:], 'retired child MM identity')
    elif op == 13:
        require(row['result'] == 1 and before['ids'][0] == 1, 'outer cancellation')
        require(after['active'] == before['active'] and after['state'] == before['state'], 'active cancellation effect')
        require(after['ids'] == before['ids'][:3] + [0] * 3 + before['ids'][6:], 'inactive cancellation identity')
    elif op == 5:
        same_bank(before, after, 'inactive')
        count = before['state'][2]
        wanted = (request - START + PAGE - 1) // PAGE if START <= request <= START + 519 * PAGE else count
        if request == 0 or wanted == count:
            require(after['allocator'][2] == before['allocator'][2] and after['active'] == before['active'], 'query/within-page allocation')
        elif START<request<=START+519*PAGE and row['result']==before['state'][1]:
            resize_refusal(row)
    elif op == 7:
        same_bank(before, after, 'inactive')
        require(after['state'] == [0] * 4 and after['tokens'] == [0] * 6, 'exec heap retirement')
        require(after['ids'] == [before['ids'][0], before['ids'][6], before['ids'][2] + 1] + before['ids'][3:6] + [before['ids'][6] + 1], 'exec MM generation')
    elif op == 6:
        require(after['state'] == [0] * 4 and after['tokens'] == [0] * 6 and after['ids'] == [0] * 7, 'outer retirement')
    else:
        require(False, 'unknown MM operation')
    if op != 9:
        require(after['prepared_state'] == [0] * 7 and after['prepared_owner'] == [0] * 3 and after['prepared']['token'] == [0] * 3, 'prepared owner retained')
    retained = [after[name]['token'] for name in ['active', 'inactive', 'prepared', 'candidate']]
    expected = {tuple(before[name]['token']): before[name] for name in
                ['active', 'inactive', 'prepared', 'candidate']
                if before[name]['token'] != [0] * 3 and before[name]['token'] not in retained}
    actual = [tuple(retired['token']) for retired in row['retired']]
    require(len(actual) == len(set(actual)) and set(actual) == set(expected), 'complete retirement set')
    for retired in row['retired']:
        original = expected[tuple(retired['token'])]
        count = original['record'][3]
        require(retired['count'] == count and len(bytes.fromhex(retired['bytes'])) == count * PAGE
                and len(retired['cells']) == count * 3, 'complete retirement geometry')
        require(retired['record'] == [0] * 8 and not any(bytes.fromhex(retired['bytes'])), 'retired backing not consumed/zero')
        require(retired['cells'][0::3] == [0] * retired['count'], 'retired cells still owned')
        require(retired['cells'][2::3] == original['cells'][2::3], 'retired physical cell identity')
    return dict(passed=True, operation=op, active_pid=after['ids'][0],
                active_pages=after['state'][2], inactive_pages=after['inactive_state'][2])
