"""Check selected scalar transfers against their actual extent success edges.

TOPAL-SYSTEMS-USER-EXTENT-001. Conditional on the shared helper contract and
the listed direct-control roots/callee returns; not a whole-kernel safety proof.
"""
from collections import defaultdict
import json
from pathlib import Path
import struct
import sys

from inspect_call_inventory import Elf, digest, verify_calls
from inventory_process_accesses import branch


# Each obligation names an existing census label and every scalar access in its
# local interval. Offsets here are relative to the admitted candidate, not code.
CASES = (
    ('echo', 'static-process-shell-echo-candidate-ready', 'read', 6, 6,
     [('read', 0, 4), ('read', 4, 2)]),
    ('exec-signal-reset', 'static-process-execve-signal-number-ready', 'read', 32, 6,
     [('read', n, 8) for n in range(0, 32, 8)]),
    ('sigchld-install', 'static-process-shell-sigchld-install', 'read', 32, 6,
     [('read', n, 8) for n in range(0, 32, 8)]),
    ('process-name', 'static-process-initial-process-name', 'write', 16, 6,
     [('write', 0, 8), ('write', 8, 8), ('read', 0, 8), ('read', 8, 8)]),
    ('missing-path', 'static-process-shell-root-directory-stat-profile-ready', 'read', 9, 6,
     [('read', 0, 8), ('read', 8, 1)]),
    ('mail-path', 'static-process-shell-mailpath-stat-miss', 'read', 21, 6,
     [('read', 0, 8), ('read', 8, 8), ('read', 16, 4), ('read', 20, 1)]),
    ('config-path', 'static-process-config-path-ready', 'read', 18, 6,
     [('read', 0, 8), ('read', 8, 8), ('read', 16, 2)]),
    ('stack-limit', 'static-process-stack-limit', 'write', 16, 10,
     [('write', 0, 8), ('write', 8, 8)]),
    ('descriptor-limit', 'fd-limit-publish', 'write', 16, 8,
     [('write', 0, 8), ('write', 8, 8)]),
)


def reachable(roots, successors, excluded=None):
    seen, pending = set(), list(roots)
    while pending:
        at = pending.pop()
        if at in seen:
            continue
        seen.add(at)
        pending.extend(target for target in successors.get(at, ())
                       if (at, target) != excluded)
    return seen


def check_graph(graph, kernel, symbol, body):
    """Reject omitted/added edges and roots before using the decoder graph."""
    nodes = {row['offset']: row for row in graph['instructions']}
    assert len(nodes) == len(graph['instructions'])
    occupied, expected = set(), set()
    terminals = {'ret', 'retf', 'iret', 'iretq', 'sysret', 'sysretq', 'hlt',
                 'ud2', 'int', 'int3', 'syscall', 'sysenter', 'sysexit'}
    for at, row in nodes.items():
        raw = bytes.fromhex(row['bytes'])
        assert 0 <= at and 0 < len(raw) == row['size'] <= 15
        assert raw == body[at:at + len(raw)], ('instruction bytes', at)
        span = set(range(at, at + len(raw)))
        assert not occupied & span
        occupied.update(span)
        name = row['mnemonic']
        assert name not in ['.byte', '.word', '<unknown>']
        if name in terminals:
            continue
        if name.startswith(('j', 'loop')) or name == 'call':
            target = branch(row)
            if target is not None:
                destination, address = target
                assert symbol['value'] + destination == address
                if 0 <= destination < len(body):
                    expected.add((at, destination, 'direct-transfer'))
            if name == 'jmp':
                continue
        expected.add((at, at + len(raw),
                      'call-return-assumption' if name == 'call' else 'fallthrough'))
    supplied = [(e['source'], e['target'], e['kind']) for e in graph['edges']]
    assert len(supplied) == len(set(supplied)) and set(supplied) == expected, 'graph edges'
    successors, predecessors = defaultdict(set), defaultdict(set)
    for source, target, _ in expected:
        assert target in nodes
        successors[source].add(target)
        predecessors[target].add(source)
    lstar = []
    for at in range(len(body) - 23):
        if body[at:at + 3] == bytes.fromhex('48 8d 05') and body[at + 7:at + 22] == bytes.fromhex(
                'b9 82 00 00 c0 48 89 c2 48 c1 ea 20 0f 30 b9'):
            lstar.append(at + 7 + struct.unpack_from('<i', body, at + 3)[0])
    assert len(lstar) == 1 and body[lstar[0]:lstar[0] + 3] == bytes.fromhex('0f 01 f8')
    roots = {0, lstar[0]}
    roots.update(s['value'] - symbol['value'] for s in kernel.symbols.values()
                 if s['section'] == symbol['section'] and s['info'] & 15 == 2
                 and symbol['value'] < s['value'] < symbol['value'] + len(body))
    assert roots == {r['offset'] for r in graph['roots']}, 'graph roots'
    assert reachable(roots, successors) == set(nodes), 'nonvacuous root closure'
    return nodes, roots, successors, predecessors


def scalar(raw):
    """Decode a deliberately small, fail-closed scalar x86 memory subset.

    No SIB, segments, address-size overrides, indexed operands or implicit
    accesses. A separate LLVM mnemonic is not used to establish these effects.
    """
    at, word, rex = 0, False, 0
    if raw[at] == 0x66:
        word, at = True, at + 1
    if 0x40 <= raw[at] <= 0x4f:
        rex, at = raw[at], at + 1
    op, at = raw[at], at + 1
    width = 8 if rex & 8 else (2 if word else 4)
    immediate, destination = 0, None
    if op == 0x0f:
        assert raw[at] == 0xb7
        at += 1
        width, action = 2, 'read'
    elif op in [0x8b, 0x39]:
        action = 'read'
    elif op == 0x89:
        action = 'write'
    elif op in [0x80, 0x81, 0x83]:
        action = 'read'
        immediate = 1 if op in [0x80, 0x83] else (2 if word else 4)
        if op == 0x80:
            width = 1
    elif op == 0xc7:
        action, immediate = 'write', 2 if word else 4
    else:
        raise AssertionError(('unsupported local scalar opcode', raw.hex()))
    modrm, at = raw[at], at + 1
    mod, reg, rm = modrm >> 6, (modrm >> 3) & 7, modrm & 7
    assert mod != 3 and rm != 4 and not (mod == 0 and rm == 5)
    base = rm + (8 if rex & 1 else 0)
    displacement = 0
    if mod in [1, 2]:
        size = 1 if mod == 1 else 4
        displacement = int.from_bytes(raw[at:at + size], 'little', signed=True)
        at += size
    if op in [0x80, 0x81, 0x83]:
        assert reg == 7, 'only CMP is admitted from this opcode group'
    if op == 0xc7:
        assert reg == 0
    if op in [0x8b, 0x0f]:
        destination = reg + (8 if rex & 4 else 0)
    assert len(raw) == at + immediate
    return action, displacement, width, base, destination


def audit(build):
    census_bytes = (build / 'extent-call-inventory.json').read_bytes()
    census = json.loads(census_bytes)
    provider, kernel = Elf(build / 'provider.o'), Elf(build / 'kernel.elf')
    verify_calls(provider, census)
    symbol = verify_calls(kernel, census)
    _, body = kernel.function(census['function'])
    graph_bytes = (build / 'process-access-inventory.json').read_bytes()
    graph = json.loads(graph_bytes)
    origin_bytes = (build / 'process-access-origins.json').read_bytes()
    origins = json.loads(origin_bytes)
    assert graph['passed'] and origins['passed']
    assert graph['function'] == origins['function'] == census['function']
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    nodes, roots, successors, predecessors = check_graph(graph, kernel, symbol, body)
    access_origins = {row['offset']: row for row in origins['accesses']}
    results = []
    for name, label, direction, length, candidate, obligation in CASES:
        calls = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(calls) == 1
        call = calls[0]['offset']
        assert calls[0]['target'] == 'runtime-' + ('readable' if direction == 'read' else 'writable') + '-extent'
        setup = call - (10 if candidate == 8 else 13)
        move = b'' if candidate == 8 else bytes.fromhex('49 89 f0' if candidate == 6 else '4d 89 d0')
        assert body[setup:call] == move + b'\x49\xb9' + struct.pack('<Q', length)
        gate, success = call + 14, call + 20
        assert body[gate:gate + 2] == b'\x0f\x82'
        count_setup = setup + len(move)
        if move:
            assert predecessors[count_setup] == {setup}, ('alternate candidate setup entry', name)
        assert predecessors[call] == {count_setup}, ('alternate count setup entry', name)
        assert predecessors[call + 5] == {call} and predecessors[gate] == {call + 5}
        assert not roots & {call, call + 5, gate}
        if move:
            assert count_setup not in roots
        assert success in successors[gate]
        bypass = reachable(roots, successors, (gate, success))
        aliases = {8} | ({candidate} if candidate in [2, 6, 7, 3, 5, 12, 13, 14, 15] else set())
        accesses, interval, at = [], [], success
        previous = gate
        while len(accesses) < len(obligation):
            assert at < success + 1024 and at in nodes
            row, raw = nodes[at], bytes.fromhex(nodes[at]['bytes'])
            assert predecessors[at] == {previous} and at not in roots, ('alternate local entry', name, at)
            assert at not in bypass, ('extent-success edge does not dominate access interval', name, at)
            if raw == bytes.fromhex('4d 89 c2'):
                assert 8 in aliases
                aliases.add(10)
            elif raw[:2] == b'\x48\xb8':
                assert len(raw) == 10
                aliases.discard(0)
            elif raw == bytes.fromhex('65 48 8b 0c 25 28 01 00 00'):
                aliases.discard(1)
            elif raw == bytes.fromhex('65 8a 04 25 b8 00 00 00'):
                aliases.discard(0)
            elif raw[:5] == bytes.fromhex('65 48 83 3c 25'):
                assert len(raw) == 10 and raw[-1] == 0
            elif raw[:1] == b'\x3c':
                assert len(raw) == 2
            elif raw == bytes.fromhex('0f ae f0'):
                pass
            elif raw[:1] == b'\x0f' and 0x80 <= raw[1] <= 0x8f:
                target = at + 6 + struct.unpack_from('<i', raw, 2)[0]
                assert body[target:target + 2] == bytes.fromhex('fa f4'), 'local mismatch must halt'
            else:
                action, offset, width, base, destination = scalar(raw)
                assert base in aliases, ('candidate alias lost', name, at)
                assert 0 <= offset and offset + width <= length, ('access bounds', name, at)
                assert action == 'read' or direction == 'write'
                assert at in access_origins, ('missing access origin', name, at)
                origin = access_origins[at]
                assert origin['bytes'] == row['bytes'] and origin['size'] == row['size']
                accesses.append(dict(offset=at, action=action, candidate_offset=offset,
                    width=width, base_register=base, bytes=row['bytes'],
                    instruction=row['instruction'], byte_origins=origin['byte_origins']))
                if destination is not None:
                    aliases.discard(destination)
            interval.append(dict(offset=at, bytes=row['bytes'], instruction=row['instruction']))
            previous, at = at, at + row['size']
        assert [(a['action'], a['candidate_offset'], a['width']) for a in accesses] == obligation
        results.append(dict(name=name, census_label=label, call_offset=call,
            candidate_setup_offset=setup, candidate_register=candidate,
            admitted_direction=direction, admitted_bytes=length,
            success_edge=[gate, success], local_end=at, access_instructions=accesses,
            candidate_aliases_at_end=sorted(aliases), local_instructions=interval,
            unreachable_nodes_without_success_edge=len(nodes) - len(bypass)))
    report = dict(passed=True, function=census['function'], kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data), linked_function_sha256=digest(body),
        graph_sha256=digest(graph_bytes), origin_join_sha256=digest(origin_bytes),
        census_sha256=digest(census_bytes), roots=graph['roots'], cases=results,
        locations={str(i): origins['locations'][i] for i in sorted({
            o['location'] for r in results for a in r['access_instructions'] for o in a['byte_origins']})},
        transfer_regions=len(results), scalar_access_instructions=sum(len(r['access_instructions']) for r in results),
        scope='Selected scalar intervals only: exact candidate/count setup, immediate CF result gate, nonvacuous success-edge dominance in the listed direct-control overapproximation, no alternate local entries, raw scalar effects, preserved aliases, direction and complete byte bounds. Writable x86 admission also permits readback. Conditional on decoder semantics outside these intervals, shared helper preservation/soundness, correct root/callee/exception contracts and stable owned mappings. No native transfer-execution, whole-syscall/full-access classification, lifetime or production qualification')
    (build / 'fixed-transfer-audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ['cases', 'roots', 'locations']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
