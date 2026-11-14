"""Read-only stat records and private-state observations for source requests."""

PROGRAM = r'''
def stat_record(fields):
    result = bytearray(144)
    for offset,value in fields.items():
        assert offset in [0,8,16,24,40,48,56,64,72,88,104]
        struct.pack_into('<Q',result,offset,value)
    return bytes(result)

def stat_identity(table,identity):
    fields = {0:1,8:identity,16:1,56:4096}
    if identity == 5:
        fields.update({24:0o100755,48:2024544,64:3955})
        return stat_record(fields),'immutable-file'
    if identity in [3,4]:
        fields.update({16:4 if identity == 3 else 2,24:0o40755})
        return stat_record(fields),'directory-root' if identity == 3 else 'directory-bin'
    if identity == 6:
        at = table+0x43020
        fields.update({0:2,16:2,24:word(at),72:word(at+8),88:word(at+16),104:word(at+24)})
        return stat_record(fields),'directory-tmp'
    assert identity >= 7
    objects = [table+0x43100+i*128 for i in range(4)
               if word(table+0x43100+i*128) == 1 and word(table+0x43108+i*128) == identity]
    assert len(objects) == 1
    at = objects[0]
    blocks = sum(word(table+0x43400+i*32) == identity for i in range(8))*8
    fields.update({0:2,16:word(at+56),24:word(at+24),48:word(at+16),64:blocks,
                   72:word(at+32),88:word(at+40),104:word(at+48)})
    return stat_record(fields),'volatile-file'

def stat_descriptor(table,fd):
    if fd >= 16: return None,'bad-descriptor',-9
    index = word(table+64+fd*16)
    if not index: return None,'bad-descriptor',-9
    assert 1 <= index <= 16
    at = table+320+(index-1)*64
    kind,identity = word(at),word(at+32)
    if kind in [3,4,7]:
        expected,variant = stat_identity(table,identity)
        return expected,variant,None
    if kind in [5,6]:
        objects = [table+0x3e000+i*128 for i in range(4)
                   if word(table+0x3e000+i*128) == 1 and word(table+0x3e008+i*128) == identity]
        assert len(objects) == 1
        obj = objects[0]
        fields = {0:2,8:identity,16:1,24:word(obj+64),56:4096,
                  72:word(obj+72),88:word(obj+80),104:word(obj+88)}
        return stat_record(fields),'pipe-reader' if kind == 5 else 'pipe-writer',None
    assert kind in [1,2]
    return stat_record({0:1,8:1,16:1,24:0x2180,40:0x4400,56:4096}),'terminal',None

def stat_snapshot():
    table = word(context+928)
    assert word(table) == 1
    return dict(table=table,regions=[dict(pointer=pointer,size=size,
        sha256=hashlib.sha256(memory(pointer,size)).hexdigest()) for pointer,size in
        [(table,8192),(table+0x3e000,0x5000),(table+0x43000,0x9000)]],
        context=pipe_context())

def stat_request(call):
    before = stat_snapshot()
    table = before['table']
    n,args = call['number'],call['args']
    assert n in [5,262]
    if n == 5:
        expected,variant,error = stat_descriptor(table,args[0])
        pointer = args[1]
    else:
        pointer = args[2]
        if args[1] == 0:
            assert args[3] == 0x1000
            expected,variant,error = stat_descriptor(table,args[0])
        else:
            name = pipe_visible(args[1],256)
            name = name[:name.index(0)]
            call['stat_path_hex'] = name.hex()
            identities = {b'/':3,b'/bin':4,b'/bin/busybox':5,b'/tmp':6}
            if name in identities:
                expected,variant = stat_identity(table,identities[name])
                error = None
            elif name == b'/tmp/stat':
                slots = [i for i in range(4) if word(table+0x43040+i*16) and
                    memory(table+0x43600+i*256,word(table+0x43048+i*16)) == b'stat']
                assert len(slots) == 1
                expected,variant = stat_identity(table,word(table+0x43040+slots[0]*16))
                error = None
            else:
                assert name == b'/missing'
                expected,variant,error = None,'missing-path',-2
    call['_stat_private_before'] = before
    call['stat_variant'] = variant
    call['stat_expected_hex'] = expected.hex() if expected is not None else None
    call['stat_expected_error'] = error
    call['stat_pointer'] = pointer
    call['stat_buffer_before'] = pipe_visible(pointer,144).hex()
    call['stat_window_pointer'] = pointer-1
    call['stat_window_before'] = pipe_visible(pointer-1,146).hex()

def stat_result(call):
    before = call.pop('_stat_private_before')
    assert stat_snapshot() == before, 'stat mutated table, FIFO, filesystem or protected context'
    result,pointer = call['result'],call['stat_pointer']
    expected = bytes.fromhex(call['stat_expected_hex']) if call['stat_expected_hex'] is not None else None
    window = bytearray.fromhex(call['stat_window_before'])
    if expected is None:
        assert result == call['stat_expected_error']
    elif result < 0:
        assert result == -14
    else:
        assert result == 0 and len(window) == 146
        actual = memory(pointer,144)
        assert actual == expected, (call['stat_variant'],actual.hex(),expected.hex())
        call['stat_output_hex'] = actual.hex()
        window[1:145] = expected
    if result < 0:
        raw = bytes.fromhex(call['stat_buffer_before'])
        assert pipe_visible(pointer,len(raw)) == raw
    assert pipe_visible(call['stat_window_pointer'],len(window)) == bytes(window)
    call['stat_private_state_preserved'] = call['stat_output_window_verified'] = True

def stat_returned(row,after):
    global current
    calls = row['calls']
    assert len(calls) == 38
    assert {n:sum(c['number'] == n for c in calls) for n in [12,5,262,257,293,1,263,3]} == {
        12:3,5:13,262:9,257:4,293:1,1:1,263:1,3:6}
    queries = [c for c in calls if c['number'] in [5,262]]
    assert len(queries) == 22 and sum(c['result'] == 0 for c in queries) == 15
    assert {error:sum(c['result'] == error for c in queries) for error in [-14,-9,-2]} == {-14:5,-9:1,-2:1}
    assert all(c['stat_private_state_preserved'] and c['stat_output_window_verified'] for c in queries)
    assert {c['stat_variant'] for c in queries if c['result'] == 0} == {
        'terminal','immutable-file','directory-root','directory-bin','directory-tmp',
        'pipe-reader','pipe-writer','volatile-file'}
    assert hashlib.sha256(memory(word(context+928),8192)).hexdigest() == row['stat_table_before_sha256']
    row['after'] = after
    observed['executions'].append(row)
    print('SOURCE_STAT_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current = None
    return False
'''
