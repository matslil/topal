"""Check finite indexed loops and unrolled staged outputs at owned admissions.

TOPAL-SYSTEMS-USER-EXTENT-001. GS/captured-entry/counterpart ownership, helper
soundness and continuous mapping lifetime are conditional, not metadata grants.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable, scalar
from inspect_call_inventory import Elf, digest, verify_calls


# Labels identify census entries, never new execution roots or memory owners.
CASES = (
    ('uname-loop', 'static-process-shell-uname', 'uname', 390, 7, None),
    ('close-message', 'static-process-shell-bin-busybox-explicit-close-write-profile-ready', 'input-loop', 14, 6, 744),
    ('root-condition', 'static-process-shell-root-condition-write-profile-ready', 'input-loop', 5, 6, 744),
    ('empty-bin-message', 'static-process-shell-empty-bin-glob-write-profile-ready', 'input-loop', 13, 6, 744),
    ('executable-message', 'static-process-shell-bin-busybox-executable-write-profile-ready', 'input-loop', 19, 6, 744),
    ('nonempty-message', 'static-process-shell-bin-busybox-nonempty-write-profile-ready', 'input-loop', 17, 6, 744),
    ('open-message', 'static-process-shell-bin-busybox-open-write-profile-ready', 'input-loop', 13, 6, 744),
    ('header-message', 'static-process-shell-bin-busybox-header-write-profile-ready', 'input-loop', 13, 6, 744),
    ('banner', 'static-process-shell-banner', 'input-loop', 109, 6, 624),
    ('console-stat', 'static-process-shell-console-fstat-candidate-ready', 'stat', 144, 6, 472),
    ('root-stat', 'static-process-shell-root-glob-fstat-profile-ready', 'stat', 144, 6, 744),
    ('empty-bin-stat', 'static-process-shell-empty-bin-glob-fstat-profile-ready', 'stat', 144, 6, 744),
    ('profile-stat', 'static-process-shell-profile-fstat', 'stat', 144, 6, 744),
    ('named-stat', 'static-process-shell-named-root-directory-stat-path-ready', 'stat', 144, 2, 744),
    ('root-directory', 'static-process-shell-root-glob-getdents-first', 'directory', 72, 6, 744),
    ('empty-bin-directory', 'static-process-shell-empty-bin-glob-getdents-first', 'directory', 80, 6, 744),
)


def gs_effect(raw):
    """Exact absolute-GS scalar subset; an offset establishes no ownership."""
    assert raw[0] == 0x65
    at, rex = 1, 0
    if raw[at] in [0x48, 0x4c]:
        rex, at = raw[at], at+1
    op, modrm, sib = raw[at:at+3]
    assert modrm & 0xc7 == 4 and sib == 0x25
    field = int.from_bytes(raw[at+3:at+7], 'little')
    width = 8 if rex & 8 else 1
    register, immediate, action = None, 0, 'read'
    if op == 0x8a:
        assert not rex
        register = (modrm>>3)&7
        assert register < 4
    elif op == 0x8b:
        assert rex & 8
        register = ((modrm>>3)&7)+(8 if rex & 4 else 0)
    elif op == 0x89:
        assert rex & 8 and (modrm>>3)&7 == 0 and not (rex & 4)
        action = 'write'
    elif op in [0xc6, 0xc7]:
        assert (modrm>>3)&7 == 0 and bool(rex & 8) == (op == 0xc7)
        action, immediate = 'write', 1 if op == 0xc6 else 4
    elif op in [0x80, 0x83]:
        assert (modrm>>3)&7 == 7 and (op == 0x83) == bool(rex & 8)
        immediate = 1
    else:
        raise AssertionError(('unsupported GS preparation', raw.hex()))
    assert len(raw) == at+7+immediate
    return dict(field=field, width=width, action=action, register_write=register)


def finite_indices(length):
    # These loops execute their body before testing, so zero is not a valid
    # trip count. The selected small constants cannot overflow a target word.
    assert 0 < length < 1 << 31
    return range(length)


def preparation(nodes, successors, predecessors, roots, body, gate, start, user_register, captured_field):
    success = gate+6
    seen, pending, effects = set(), [success], []
    while pending:
        at = pending.pop()
        if at == start or at in seen:
            continue
        assert success <= at < start and at in nodes and at not in roots
        seen.add(at)
        row, raw = nodes[at], bytes.fromhex(nodes[at]['bytes'])
        if raw[0] == 0x65:
            effect = gs_effect(raw)
            if effect['action'] == 'write':
                assert captured_field is not None
                assert effect['field']+effect['width'] <= captured_field or captured_field+8 <= effect['field'], 'captured pointer overwritten'
            if effect['register_write'] is not None:
                assert effect['register_write'] == 0 or (
                    effect['register_write'] == user_register and effect['field'] == captured_field and effect['width'] == 8), 'candidate reload has no captured-value contract'
        elif raw[:2] == b'\x48\xb8':
            assert len(raw) == 10
            effect = dict(register_write=0)
        elif raw[:1] == b'\x3c':
            assert len(raw) == 2
            effect = dict(kind='byte-register-compare')
        elif raw[:1] == b'\xe9' or (raw[:1] == b'\x0f' and 0x80 <= raw[1] <= 0x8f):
            effect = dict(kind='branch')
        else:
            raise AssertionError(('unsupported preparation instruction', at, raw.hex()))
        effects.append(dict(row, preparation_effect=effect))
        for target in successors[at]:
            if body[target:target+2] != bytes.fromhex('fa f4'):
                assert success <= target <= start
                pending.append(target)
    assert start == success or any(start in successors[at] for at in seen)
    allowed = seen | {gate}
    assert all(predecessors[at] <= allowed for at in seen | {start})
    return sorted(effects,key=lambda r:r['offset'])


def audit(build):
    census_bytes = (build/'extent-call-inventory.json').read_bytes()
    census = json.loads(census_bytes)
    provider, kernel = Elf(build/'provider.o'), Elf(build/'kernel.elf')
    verify_calls(provider,census)
    symbol = verify_calls(kernel,census)
    _, body = kernel.function(census['function'])
    graph_bytes = (build/'process-access-inventory.json').read_bytes()
    graph = json.loads(graph_bytes)
    origin_bytes = (build/'process-access-origins.json').read_bytes()
    origins = json.loads(origin_bytes)
    assert graph['passed'] and origins['passed']
    assert graph['function'] == origins['function'] == census['function']
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    nodes, roots, successors, predecessors = check_graph(graph,kernel,symbol,body)
    access_origins = {a['offset']:a for a in origins['accesses']}
    results = []
    for name,label,kind,length,user,staging in CASES:
        matches = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1
        call = matches[0]['offset']
        direction = 'read' if kind == 'input-loop' else 'write'
        assert matches[0]['target'] == 'runtime-'+('readable' if direction == 'read' else 'writable')+'-extent'
        gate,success = call+14,call+20
        assert body[gate:gate+2] == b'\x0f\x82'
        move = {6:'49 89 f0', 7:'49 89 f8', 2:'49 89 d0'}[user]
        captured = {6:320,2:328}.get(user)
        prefix = b'' if captured is None else bytes.fromhex('65 48 8b '+('34' if user == 6 else '14')+' 25')+struct.pack('<I',captured)
        setup = call-13-len(prefix)
        assert body[setup:call] == prefix+bytes.fromhex(move)+b'\x49\xb9'+struct.pack('<Q',length)
        prelude_offsets = [setup] if prefix else []
        prelude_offsets += [call-13,call-10,call,call+5,gate]
        for left,right in zip(prelude_offsets,prelude_offsets[1:]):
            assert predecessors[right] == {left} and right not in roots
        core,accesses,allowed_loop_edges = [],[],set()
        if kind == 'input-loop':
            assert body[success:success+3] == bytes.fromhex('45 31 d2')
            start = success+3
            pattern = bytes.fromhex('42 8a 04 16 65 41 88 82')+struct.pack('<I',staging)+bytes.fromhex('49 ff c2 49 83 fa')+bytes([length])
            assert 0 < length < 128 and body[start:start+len(pattern)] == pattern
            back = start+len(pattern)
            assert body[back:back+2] == b'\x0f\x85' and back+6+struct.unpack_from('<i',body,back+2)[0] == start
            end,index = back+6,10
            counterpart = dict(kind='gs',offset=staging)
            accesses = [dict(offset=start,action='read',width=1,indices=list(finite_indices(length)),base_register=user,index_register=index)]
            prefix_nodes = [nodes[success]]
            allowed_loop_edges.add((back,start))
        elif kind == 'uname':
            assert body[success:success+3] == bytes.fromhex('4c 8d 05')
            source = symbol['value']+success+7+struct.unpack_from('<i',body,success+3)[0]
            assert body[success+7:success+10] == bytes.fromhex('45 31 c9')
            start = success+10
            pattern = bytes.fromhex('43 8a 04 08 42 88 04 0f 49 ff c1 49 81 f9')+struct.pack('<I',length)
            assert body[start:start+len(pattern)] == pattern
            back = start+len(pattern)
            assert body[back:back+2] == b'\x0f\x85' and back+6+struct.unpack_from('<i',body,back+2)[0] == start
            end,index = back+6,9
            counterpart = dict(kind='linked-address',address=source)
            assert symbol['value'] <= source and source+length <= symbol['value']+len(body)
            accesses = [dict(offset=start+4,action='write',width=1,indices=list(finite_indices(length)),base_register=user,index_register=index)]
            prefix_nodes = [nodes[success],nodes[success+7]]
            allowed_loop_edges.add((back,start))
        else:
            source_load = bytes.fromhex('65 48 8b 04 25')+struct.pack('<I',staging)
            store = bytes.fromhex('48 89 '+('06' if user == 6 else '02'))
            start = body.index(source_load+store,success,success+1500)
            prefix_nodes = preparation(nodes,successors,predecessors,roots,body,gate,start,user,captured)
            at,expected = start,[(action,off) for action in (['write','read'] if kind == 'stat' else ['write']) for off in range(0,length,8)]
            while len(accesses) < len(expected):
                row,raw = nodes[at],bytes.fromhex(nodes[at]['bytes'])
                if raw[0] == 0x65:
                    effect = gs_effect(raw)
                    assert effect['register_write'] == 0 and effect['action'] == 'read'
                    last_source = effect['field']
                elif raw == bytes.fromhex('0f ae f8'):
                    pass
                elif raw[:2] == b'\x0f\x85':
                    target = at+6+struct.unpack_from('<i',raw,2)[0]
                    assert body[target:target+2] == bytes.fromhex('fa f4')
                else:
                    action,off,width,base,destination = scalar(raw)
                    assert base == user and destination is None and width == 8
                    assert (action,off) == expected[len(accesses)] and last_source == staging+off
                    assert 0 <= off and off+width <= length
                    accesses.append(dict(offset=at,action=action,candidate_offset=off,width=width,base_register=user))
                core.append(row)
                at += row['size']
            if kind == 'stat':
                raw = bytes.fromhex(nodes[at]['bytes'])
                assert raw[:2] == b'\x0f\x85' and body[at+6+struct.unpack_from('<i',raw,2)[0]:][:2] == bytes.fromhex('fa f4')
                core.append(nodes[at])
                at += nodes[at]['size']
            end,index = at,None
            counterpart = dict(kind='gs',offset=staging)
        if kind in ['input-loop','uname']:
            core = [n for at,n in sorted(nodes.items()) if start <= at < end]
            assert sum(n['size'] for n in core) == end-start
        interval = prefix_nodes+core
        expected_edges = {(a['offset'],b['offset']) for a,b in zip(interval,interval[1:])}
        expected_edges |= {(gate,success)} | allowed_loop_edges
        if kind not in ['input-loop','uname']:
            # Preparation may branch between two payload constructions. Its
            # checked DAG accounts for its own internal edges and reloads.
            prep_offsets = {r['offset'] for r in prefix_nodes}
            expected_edges |= {(a,b) for a in prep_offsets for b in successors[a] if b in prep_offsets or b == start}
        for n in interval:
            assert n['offset'] not in roots
            assert all((p,n['offset']) in expected_edges for p in predecessors[n['offset']]), ('alternate staged entry',name,n['offset'])
        bypass = reachable(roots,successors,(gate,success))
        assert all(n['offset'] not in bypass for n in interval), ('staged success dominance',name)
        for access in accesses:
            origin = access_origins[access['offset']]
            access.update(bytes=origin['bytes'],instruction=origin['instruction'],byte_origins=origin['byte_origins'])
        results.append(dict(name=name,census_label=label,kind=kind,call_offset=call,
            candidate_setup_offset=setup,carry_gate=gate,success_offset=success,
            start_offset=start,end_offset=end,length=length,direction=direction,
            user_register=user,index_register=index,counterpart=counterpart,
            preparation=prefix_nodes,transfer=core,user_accesses=accesses,
            success_edge_dominates=True))
    outside = [r for r in census['calls'] if r['offset'] not in nodes]
    report = dict(passed=True,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=results,transfer_regions=len(results),user_access_instructions=sum(len(r['user_accesses']) for r in results),
        out_of_graph_calls=outside,
        locations={str(i):origins['locations'][i] for i in sorted({o['location'] for r in results for a in r['user_accesses'] for o in a['byte_origins']})},
        scope='Selected finite indexed loops and unrolled staged outputs: exact admission/direction/length, full index or scalar byte bounds, preservation/reload of the candidate under the captured-entry contract, staging-value load/store/readback relationships, checked preparation DAG and nonvacuous direct-graph success-edge dominance. GS/linked numeric expressions grant no owner authority. Complete helper/root/callee/exception, counterpart ownership and continuous lifetime qualification remain separate; no whole-kernel/access or production claim')
    (build/'staged-transfer-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['cases','locations']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
