"""Conditional exact scalar bounds for further owned-extent callers.

TOPAL-SYSTEMS-USER-EXTENT-001. Each listed interval is local; neither a register
name nor an exposed memory expression grants ownership or continuous lifetime.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable, scalar
from audit_staged_transfers import gs_effect, preparation
from inspect_call_inventory import Elf, digest, verify_calls


# name, census label, direction, complete admitted bytes, candidate register,
# every selected user access (direction, candidate-relative offset, width).
CASES = (
    ('cwd', 'static-process-shell-getcwd', 'write', 2, 7, [('write',0,2)]),
    ('chdir', 'static-process-shell-chdir-root-profile-ready', 'read', 2, 7, [('read',0,2)]),
    ('pwd-cwd', 'static-process-shell-pwd-getcwd', 'write', 2, 7, [('write',0,2)]),
    ('initial-cwd', 'static-process-shell-initial-getcwd', 'write', 2, 7, [('write',0,2)]),
    ('poll-input', 'owned-header-poll-profile-ready', 'read', 8, 7, [('read',0,8)]),
    ('poll-result', 'static-process-shell-bin-busybox-header-poll-ready', 'write', 2, 0, [('write',0,2)]),
    ('default-signal', 'static-process-shell-query-default-common', 'write', 32, 2,
     [('write',off,8) for off in range(0,32,8)]),
    ('quit-signal', 'static-process-shell-install-quit-ignore', 'read', 32, 6,
     [('read',off,8) for off in range(0,32,8)]),
    ('header-byte', 'static-process-shell-bin-busybox-header-read-profile-ready', 'write', 1, 6,
     [('write',0,1),('read',0,1)]),
    ('exec-path-byte', 'runtime-exec-path-byte', 'read', 1, 3, [('read',0,1)]),
    ('exec-vector-word', 'runtime-exec-vector-entry', 'read', 8, 13, [('read',0,8)]),
    ('exec-argument-byte', 'runtime-exec-argument-byte', 'read', 1, 3, [('read',0,1)]),
    ('tty-path', 'static-process-shell-dev-tty-open-miss-exact', 'read', 9, 6,
     [('read',0,8),('read',8,1)]),
    ('profile-path', 'static-process-shell-profile-open', 'read', 13, 6,
     [('read',0,8),('read',8,4),('read',12,1)]),
    ('open-path', 'static-process-shell-bin-busybox-open-profile-ready', 'read', 13, 6,
     [('read',0,8),('read',8,4),('read',12,1)]),
    ('reopen-path', 'static-process-shell-bin-busybox-reopen-profile-ready', 'read', 13, 6,
     [('read',0,8),('read',8,4),('read',12,1)]),
    ('root-path', 'static-process-shell-root-glob-open-profile-ready', 'read', 2, 6, [('read',0,2)]),
    ('bin-path', 'static-process-shell-empty-bin-glob-open-profile-ready', 'read', 6, 6,
     [('read',0,4),('read',4,2)]),
    ('newline', 'static-process-applet-newline-candidate-ready', 'read', 1, 6, [('read',0,1)]),
    ('prompt', 'static-process-shell-prompt', 'read', 4, 6, [('read',0,4)]),
    ('refill-prompt', 'static-process-shell-refill-prompt', 'read', 4, 6, [('read',0,4)]),
    ('pwd-output', 'static-process-shell-pwd-write-profile-ready', 'read', 2, 6, [('read',0,2)]),
    ('third-prompt', 'static-process-shell-third-prompt', 'read', 4, 6, [('read',0,4)]),
    ('fourth-prompt', 'static-process-shell-fourth-prompt-profile-ready', 'read', 4, 6, [('read',0,4)]),
    ('stat-executable-path', 'static-process-shell-bin-busybox-stat-path', 'read', 13, 6,
     [('read',0,8),('read',8,4),('read',12,1)]),
    ('stat-directory-path', 'static-process-shell-named-root-directory-stat-path', 'read', 5, 6,
     [('read',0,4),('read',4,1)]),
    ('namespace-byte', 'namespace-copy-byte', 'read', 1, 8, [('read',0,1)]),
    ('seconds', 'realtime-seconds', 'write', 8, 8, [('write',0,8)]),
    ('thread-local-base', 'child-thread-local', 'write', 8, 6, [('write',0,8)]),
    ('clear-registration', 'child-clear-registration-ready', 'write', 4, 8, [('write',0,4)]),
    ('wait-status', 'child-wait-matched', 'write', 4, 6, [('write',0,4)]),
    ('environment-word', 'runtime-environment-entry', 'read', 8, 13, [('read',0,8)]),
    ('environment-byte', 'runtime-environment-byte', 'read', 1, 3, [('read',0,1)]),
)
PRESERVED = {2,3,5,6,7,8,9,12,13,14,15}


def setup(body, call, length, candidate, name):
    count = b'\x49\xb9'+struct.pack('<Q',length)
    if name == 'poll-result':
        raw = bytes.fromhex('48 89 f8 48 81 c0 06 00 00 00 49 89 c0')+count
        aliases = {8:0,7:-6}
    elif candidate == 8:
        raw, aliases = count, {8:0}
    else:
        move = bytes([0x49 | (4 if candidate >= 8 else 0),0x89,0xc0|((candidate&7)<<3)])
        raw, aliases = move+count, {8:0,candidate:0}
        assert candidate in PRESERVED
    if name == 'header-byte':
        raw = bytes.fromhex('65 48 8b 34 25 40 01 00 00')+raw
    start = call-len(raw)
    assert body[start:call] == raw, ('candidate/count setup',name)
    return start,aliases


def relative_offset(bias, displacement, width, length):
    # Equality of effective target-word addresses is established by the exact
    # setup. The admitted candidate itself already has a nonwrapping extent.
    offset = bias+displacement
    assert 0 <= offset and offset+width <= length, 'scalar extent bounds'
    return offset


def register_effect(raw):
    """Exact small register-only subset; return writes and optional MOV alias."""
    if raw in [bytes.fromhex(s) for s in ['0f ae f0','0f ae f8']]:
        return set(),None
    rex,at = (raw[0],1) if 0x40 <= raw[0] <= 0x4f else (0,0)
    op = raw[at]
    if 0xb8 <= op <= 0xbf:
        assert len(raw) == at+(9 if rex&8 else 5)
        return {op-0xb8+(8 if rex&1 else 0)},None
    if op == 0x3d:
        assert len(raw) == at+5
        return set(),None
    modrm = raw[at+1]
    assert modrm>>6 == 3
    dst,src = (modrm&7)+(8 if rex&1 else 0),((modrm>>3)&7)+(8 if rex&4 else 0)
    if op == 0x89:
        assert rex&8 and len(raw) == at+2
        return {dst},(dst,src)
    if op in [0x31,0x09]:
        assert len(raw) == at+2
        return {dst},None
    if op == 0xc1:
        assert rex&8 and (modrm>>3)&7 in [4,5] and len(raw) == at+3
        return {dst},None
    raise AssertionError(('unsupported scalar interval register effect',raw.hex()))


def audit(build):
    census_bytes = (build/'extent-call-inventory.json').read_bytes()
    census = json.loads(census_bytes)
    provider,kernel = Elf(build/'provider.o'),Elf(build/'kernel.elf')
    verify_calls(provider,census)
    symbol = verify_calls(kernel,census)
    _,body = kernel.function(census['function'])
    graph_bytes,origin_bytes = [(build/name).read_bytes() for name in
        ['process-access-inventory.json','process-access-origins.json']]
    graph,origins = json.loads(graph_bytes),json.loads(origin_bytes)
    assert graph['passed'] and origins['passed']
    assert graph['function'] == origins['function'] == census['function']
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    nodes,roots,successors,predecessors = check_graph(graph,kernel,symbol,body)
    access_origins = {a['offset']:a for a in origins['accesses']}
    cases = []
    for name,label,direction,length,candidate,expected in CASES:
        matches = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1
        call = matches[0]['offset']
        assert matches[0]['target'] == 'runtime-'+('readable' if direction == 'read' else 'writable')+'-extent'
        start,aliases = setup(body,call,length,candidate,name)
        gate,success = call+14,call+20
        assert body[gate:gate+2] == b'\x0f\x82'
        prelude = [n for at,n in sorted(nodes.items()) if start <= at < gate+6]
        assert sum(n['size'] for n in prelude) == gate+6-start
        for left,right in zip(prelude,prelude[1:]):
            assert predecessors[right['offset']] == {left['offset']} and right['offset'] not in roots
        prefix = []
        at,previous = success,gate
        if name == 'header-byte':
            at = body.index(bytes.fromhex('88 06 0f ae f8 38 06'),success,success+256)
            prefix = preparation(nodes,successors,predecessors,roots,body,gate,at,6,320)
            previous = prefix[-1]['offset']
        bypass = reachable(roots,successors,(gate,success))
        assert all(n['offset'] not in bypass for n in prefix)
        local,accesses,other = [],[],[]
        while len(accesses) < len(expected):
            assert at in nodes and success <= at < success+1024
            row,raw = nodes[at],bytes.fromhex(nodes[at]['bytes'])
            assert predecessors[at] == {previous} and at not in roots
            assert at not in bypass, ('success-edge dominance',name,at)
            effect = {}
            if raw[:1] == b'\x65':
                effect = gs_effect(raw)
                assert effect['action'] == 'read' and effect['register_write'] in [0,1]
                if effect['register_write'] is not None:
                    aliases.pop(effect['register_write'],None)
            elif raw[:2] in [bytes.fromhex('0f 82'),bytes.fromhex('0f 83'),bytes.fromhex('0f 85')]:
                assert len(raw) == 6
                target = at+6+struct.unpack_from('<i',raw,2)[0]
                assert body[target:target+2] == bytes.fromhex('fa f4')
                effect = dict(kind='fatal-comparison-side-exit')
            elif raw == bytes.fromhex('0f 32'):
                assert name == 'thread-local-base' and local[-1]['bytes'] == '48b9000100c000000000'
                aliases.pop(0,None)
                aliases.pop(2,None)
                effect = dict(kind='fs-base-msr-read',register_writes=[0,2])
            else:
                try:
                    action,displacement,width,base,destination = scalar(raw)
                except (AssertionError,IndexError):
                    writes,move = register_effect(raw)
                    assert 4 not in writes, 'local interval cannot change RSP'
                    value = aliases.get(move[1]) if move else None
                    for register in writes:
                        aliases.pop(register,None)
                    if value is not None:
                        aliases[move[0]] = value
                    effect = dict(kind='register-only',writes=sorted(writes),move=move)
                else:
                    if base in aliases:
                        offset = relative_offset(aliases[base],displacement,width,length)
                        assert action == 'read' or direction == 'write'
                        assert (action,offset,width) == expected[len(accesses)]
                        origin = access_origins[at]
                        assert origin['bytes'] == row['bytes'] and origin['size'] == row['size']
                        accesses.append(dict(offset=at,action=action,candidate_offset=offset,
                            width=width,base_register=base,alias_bias=aliases[base],
                            bytes=row['bytes'],instruction=row['instruction'],byte_origins=origin['byte_origins']))
                    else:
                        assert name == 'wait-status' and (action,displacement,width,base,destination) == ('read',8216,8,15,0)
                        other.append(dict(row,action=action,width=width,base_register=base,displacement=displacement,
                            scope='Exposed additional read requiring separate owner/access proof; no register-based authority'))
                    if destination is not None:
                        aliases.pop(destination,None)
                    effect = dict(kind='scalar-memory',action=action,base_register=base)
            local.append(dict(row,effect=effect))
            previous,at = at,at+row['size']
        cases.append(dict(name=name,census_label=label,call_offset=call,
            candidate_setup_offset=start,candidate_register=candidate,length=length,direction=direction,
            carry_gate=gate,success_offset=success,end_offset=at,prelude=prelude,preparation=prefix,
            local_instructions=local,user_accesses=accesses,other_memory_obligations=other,
            success_edge_dominates=True))
    report = dict(passed=True,function=census['function'],provider_sha256=digest(provider.data),
        kernel_sha256=digest(kernel.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        transfer_regions=len(cases),user_access_instructions=sum(len(r['user_accesses']) for r in cases),cases=cases,
        scope='Selected scalar intervals only: exact setup, helper-preserved aliases including shifted poll output, raw scalar widths/directions/bounds, checked preparation and nonvacuous success-edge dominance. Repeated byte/vector probes establish only their per-invocation access interval. Other memory effects are explicit obligations; helper/root/callee/exception, captured-entry authority, complete owners and continuous lifetime remain conditional. No whole-caller, native transfer-observation or production claim')
    (build/'additional-scalar-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k != 'cases'}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
