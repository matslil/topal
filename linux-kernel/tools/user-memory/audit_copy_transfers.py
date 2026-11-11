"""Conditional candidate/count dataflow for selected native REP copies.

TOPAL-SYSTEMS-USER-EXTENT-001. Counterparty ownership and complete helper,
root/callee/exception and stable-mapping contracts remain separate obligations.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable
from inspect_call_inventory import Elf, digest, verify_calls


CASES = (
    ('header-command', 'static-process-shell-read-fstat-ready', 9),
    ('refill-command', 'static-process-shell-refill-command-read-facts-ready', 9),
    ('root-command', 'static-process-shell-root-test-command-read-profile-ready', 9),
    ('pwd-command', 'static-process-shell-pwd-command-read-profile-ready', 9),
    ('serial-command', 'owned-shell-serial-copy', 9),
    ('signal-action-input', 'process-signal-action', 10),
    ('signal-action-output', 'process-signal-action-publish', 26),
    ('signal-mask-input', 'process-signal-mask', 10),
    ('signal-mask-output', 'process-signal-mask-publish', 26),
    ('directory-output', 'namespace-directory-copy', 12),
    ('named-directory-output', 'named-directory-publish', 12),
    ('uname-output', 'process-observations-uname', 13),
    ('clock-time-output', 'process-time-gettime', 13),
    ('clock-resolution-output', 'process-time-getres', 13),
    ('sleep-interval-input', 'process-time-copy-interval', 13),
)
PRESERVED = {2, 3, 5, 6, 7, 8, 9, 12, 13, 14, 15}


def constant(number):
    assert 0 <= number < 1 << 64
    return ('constant', number)


def plus(value, amount):
    if value[0] == 'constant':
        return constant((value[1] + amount) % (1 << 64))
    if amount == 0:
        return value
    return ('add-word', value, amount)


def byte_count(count, width):
    assert width in [1, 8]
    if count[0] == 'constant':
        assert count[1] * width < 1 << 64, 'copy byte count overflow'
        return constant(count[1] * width)
    assert width == 1, 'dynamic scaled counts need a separate nonwrapping proof'
    return count


def sampler_writes(raw):
    """Raw effects of the small clock sampler's actual instruction subset."""
    if raw[:1] == b'\x65':
        assert len(raw) == 9 and raw[:5] == bytes.fromhex('65 48 8b 3c 25')
        return {7}
    if raw == b'\xc3' or raw[:1] in [b'\xe8', b'\xe9'] or raw[:1] == b'\x0f':
        assert raw == b'\xc3' or (len(raw) == 5 and raw[0] in [0xe8, 0xe9]) or (
            len(raw) == 6 and raw[0] == 0x0f and 0x80 <= raw[1] <= 0x8f)
        return set()  # Balanced nested CALL/RET is separately shape-checked.
    if raw[:1] == b'\xb9':
        assert len(raw) == 5
        return {1}
    if raw[:1] == b'\xa9':
        assert len(raw) == 5
        return set()
    if raw[:1] == b'\x31':
        assert len(raw) == 2 and raw[1] >> 6 == 3
        return {raw[1] & 7}
    rex, at = (raw[0], 1) if 0x40 <= raw[0] <= 0x4f else (0, 0)
    op = raw[at]
    if 0xb8 <= op <= 0xbf:
        assert rex & 8 and len(raw) == at+9
        return {op-0xb8+(8 if rex & 1 else 0)}
    modrm = raw[at+1]
    mod, reg, rm = modrm >> 6, (modrm >> 3) & 7, modrm & 7
    assert rm != 4  # No hidden stack/index addressing in this callee subset.
    base, operand = rm+(8 if rex & 1 else 0), reg+(8 if rex & 4 else 0)
    immediate = 1 if op in [0x80, 0x83, 0xc1, 0xf6, 0xc6] else (4 if op == 0x81 else 0)
    assert len(raw) == at+2+(1 if mod == 1 else 4 if mod == 2 else 0)+immediate
    if op == 0x8b:
        return {operand}
    if op == 0x89:
        return {base} if mod == 3 else set()
    if op in [0x39, 0x3b, 0x85, 0xf6, 0x80, 0x81]:
        assert op not in [0x80, 0x81] or reg == 7
        assert op != 0xf6 or reg == 0
        return set()
    if op == 0x83:
        assert mod == 3 and reg in [0, 1]
        return {base}
    if op == 0xc1:
        assert mod == 3 and reg == 5
        return {base}
    if op == 0xc6:
        assert mod != 3 and reg == 0
        return set()
    raise AssertionError(('unsupported clock sampler effect', raw.hex()))


def sampler_contract(body, nodes, successors, entry):
    # The wrapper has one direct nested call, compares its success witness,
    # and returns. Its nested leaf has no calls, stack changes or control-register
    # writes. Both branches to the common CLI/HLT remain nonreturning exits.
    assert body[entry:entry+10] == bytes.fromhex('65 48 8b 3c 25 b0 03 00 00 e8')
    nested = entry+14+struct.unpack_from('<i', body, entry+10)[0]
    assert body[entry+14:entry+27] == bytes.fromhex('48 81 f9 01 00 00 00 0f 85 b3 00 ff ff')
    assert body[entry+27:entry+28] == b'\xc3' and nested == entry+28
    seen, pending, effects = set(), [entry], []
    while pending:
        at = pending.pop()
        if at in seen:
            continue
        if body[at:at+2] == bytes.fromhex('fa f4'):
            continue
        assert entry <= at < entry+217
        seen.add(at)
        row = nodes[at]
        raw = bytes.fromhex(row['bytes'])
        writes = sampler_writes(raw)
        assert not writes & {3, 12}, 'clock sampler must preserve buffer/staging registers'
        if raw[:1] == b'\xe8':
            assert at == entry+9
        effects.append(dict(offset=at, bytes=raw.hex(), register_writes=sorted(writes)))
        pending.extend(successors.get(at, ()))
    return dict(entry=entry, nested_entry=nested, instructions=sorted(effects, key=lambda x:x['offset']),
        body_sha256=digest(body[entry:entry+217]), preserved_registers=[3, 12],
        scope='RBX/R12 raw register preservation and balanced nonrecursive CALL/RET shape only; clock-owner/MMIO/stack effects and stable mapping contracts remain separate')


def step(raw, registers, offset, address):
    """Interpret only the actual 64-bit, nonbranch subset used by these slices.

    This returns explicit memory effects rather than assuming a noncandidate
    operand is private. Unknown encodings fail; a mnemonic cannot grant effects.
    """
    if raw == b'\xfc':
        return dict(kind='clear-direction')
    if raw in [b'\xf3\xa4', b'\xf3\x48\xa5']:
        return dict(kind='copy', width=1 if len(raw) == 2 else 8)
    if len(raw) == 10 and 0x48 <= raw[0] <= 0x4f and 0xb8 <= raw[1] <= 0xbf:
        destination = raw[1] - 0xb8 + (8 if raw[0] & 1 else 0)
        registers[destination] = constant(int.from_bytes(raw[2:], 'little'))
        return dict(kind='register')
    if raw[:1] == b'\x65':
        assert len(raw) == 9 and raw[1] in [0x48, 0x4c] and raw[2] == 0x8b
        assert raw[3] & 0xc7 == 4 and raw[4] == 0x25
        destination = ((raw[3] >> 3) & 7) + (8 if raw[1] & 4 else 0)
        field = int.from_bytes(raw[5:], 'little')
        registers[destination] = ('captured-field', field)
        return dict(kind='captured-read', field=field, width=8)
    assert len(raw) >= 3 and raw[0] in [0x48, 0x49, 0x4c, 0x4d], ('unsupported copy-slice encoding', offset, raw.hex())
    rex, op, modrm = raw[:3]
    mod, reg, rm = modrm >> 6, (modrm >> 3) & 7, modrm & 7
    register = reg + (8 if rex & 4 else 0)
    base = rm + (8 if rex & 1 else 0)
    if mod == 3 and op == 0x89:
        assert len(raw) == 3
        registers[base] = registers[register]
        return dict(kind='register')
    if mod == 3 and op == 0x01:
        assert len(raw) == 3
        left, right = registers[base], registers[register]
        registers[base] = plus(left, right[1]) if right[0] == 'constant' else ('add-words', left, right)
        return dict(kind='register')
    if mod == 3 and op == 0x81:
        assert len(raw) == 7 and reg in [0, 7]
        if reg == 0:
            registers[base] = plus(registers[base], int.from_bytes(raw[3:], 'little', signed=True))
        return dict(kind='register')
    if mod == 0 and rm == 5 and op == 0x8d:
        assert len(raw) == 7 and not (rex & 1)
        registers[register] = constant(address + offset + len(raw) + struct.unpack_from('<i', raw, 3)[0])
        return dict(kind='register')
    if mod == 2 and rm == 4 and op == 0x89:
        assert len(raw) == 8 and raw[3] == 0x24 and rex & 1
        displacement = int.from_bytes(raw[4:], 'little', signed=True)
        return dict(kind='other-memory-write', address=plus(registers[12], displacement), width=8)
    raise AssertionError(('unsupported copy-slice encoding', offset, raw.hex()))


def audit(build):
    census_bytes = (build/'extent-call-inventory.json').read_bytes()
    census = json.loads(census_bytes)
    provider, kernel = Elf(build/'provider.o'), Elf(build/'kernel.elf')
    verify_calls(provider, census)
    symbol = verify_calls(kernel, census)
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
    nodes, roots, successors, predecessors = check_graph(graph, kernel, symbol, body)
    access_origins = {a['offset']: a for a in origins['accesses']}
    results, callee_contracts, local_edges = [], {}, set()
    for name, label, prelude_bytes in CASES:
        calls = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(calls) == 1
        call = calls[0]['offset']
        start, gate, success = call-prelude_bytes, call+14, call+20
        assert body[gate:gate+2] == b'\x0f\x82'
        registers = {i: ('entry-register', i) for i in range(16)}
        prelude, previous, at = [], None, start
        while at < call:
            row, raw = nodes[at], bytes.fromhex(nodes[at]['bytes'])
            if previous is not None:
                assert predecessors[at] == {previous} and at not in roots
            if raw[:2] == b'\x0f\x84':
                target = at+6+struct.unpack_from('<i', raw, 2)[0]
                assert not start <= target < call+20
                effect = dict(kind='optional-null-side-exit', target=target)
            else:
                effect = step(raw, registers, at, symbol['value'])
                assert effect['kind'] in ['register', 'captured-read']
            prelude.append(dict(row, effect=effect))
            previous, at = at, at+row['size']
        assert at == call and predecessors[call] == {previous} and call not in roots
        assert predecessors[call+5] == {call} and predecessors[gate] == {call+5}
        assert not roots & {call+5, gate}
        candidate, admitted = registers[8], registers[9]
        assert candidate[0] != 'constant' or candidate[1] != 0
        for register in set(registers)-PRESERVED:
            registers[register] = ('helper-clobbered', register)
        local, other, direction_clear, previous, at = [], [], False, gate, success
        while True:
            assert at < success+1024 and at in nodes
            row, raw = nodes[at], bytes.fromhex(nodes[at]['bytes'])
            assert at not in roots
            local_edges.add((previous, at))
            next_at = at+row['size']
            if raw[:1] == b'\xe8':
                assert name == 'clock-time-output' and at == success
                target = at+5+struct.unpack_from('<i', raw, 1)[0]
                callee_contracts[target] = sampler_contract(body, nodes, successors, target)
                for register in set(registers)-{3, 12}:
                    registers[register] = ('sampler-clobbered', register)
                effect = dict(kind='checked-sampler-call', target=target)
            elif raw[:1] == b'\xe9':
                assert name == 'clock-time-output'
                next_at = at+5+struct.unpack_from('<i', raw, 1)[0]
                effect = dict(kind='shared-output-jump', target=next_at)
            elif raw[:1] == b'\x0f' and 0x80 <= raw[1] <= 0x8f:
                target = at+6+struct.unpack_from('<i', raw, 2)[0]
                assert body[target:target+2] == bytes.fromhex('fa f4')
                effect = dict(kind='invariant-side-exit', target=target)
            elif raw == bytes.fromhex('49 f7 f2') or raw == bytes.fromhex('48 6b d2 0a'):
                assert name == 'clock-time-output'
                registers[2] = ('sample-arithmetic', at)
                if raw[1] == 0xf7:
                    registers[0] = ('sample-arithmetic', at)
                effect = dict(kind='sample-arithmetic')
            else:
                effect = step(raw, registers, at, symbol['value'])
            local.append(dict(row, effect=effect))
            if effect['kind'] == 'clear-direction':
                direction_clear = True
            elif effect['kind'] == 'other-memory-write':
                # Expose preparation as an outstanding owner obligation rather
                # than granting authority based on an R12 operand.
                other.append(dict(offset=at, **effect))
            elif effect['kind'] == 'copy':
                assert direction_clear, ('unknown direction flag', name)
                count = byte_count(registers[1], effect['width'])
                assert count == admitted, ('copy/admission count mismatch', name, count, admitted)
                direction = 'read' if calls[0]['target'] == 'runtime-readable-extent' else 'write'
                endpoint = 6 if direction == 'read' else 7
                assert registers[endpoint] == candidate, ('copy/admission candidate mismatch', name)
                counterpart = registers[7 if endpoint == 6 else 6]
                assert counterpart != candidate, 'both copy endpoints need separate admission/owner reasoning'
                origin = access_origins[at]
                assert origin['bytes'] == row['bytes'] and origin['size'] == row['size']
                results.append(dict(name=name, census_label=label, call_offset=call,
                    prelude_start=start, carry_gate=gate, success_offset=success,
                    copy_offset=at, copy_size=row['size'], copy_width=effect['width'],
                    admitted_direction=direction, candidate_expression=candidate,
                    admitted_byte_count=admitted, copy_element_count=registers[1],
                    user_endpoint_register=endpoint, counterpart_expression=counterpart,
                    other_memory_effects=other, prelude=prelude, local_instructions=local,
                    byte_origins=origin['byte_origins']))
                break
            previous, at = at, next_at
    groups = []
    for copy in sorted({r['copy_offset'] for r in results}):
        cases = [r for r in results if r['copy_offset'] == copy]
        excluded = {(r['carry_gate'], r['success_offset']) for r in cases}
        # Reuse the ordinary traversal with these complete admission-group edges
        # removed; all real caller paths to a shared copy must be covered.
        remaining = {source: targets-{target for a,target in excluded if a == source}
                     for source,targets in successors.items()}
        assert copy not in reachable(roots, remaining), ('uncovered copy caller', copy)
        groups.append(dict(copy_offset=copy, cases=[r['name'] for r in cases],
                           admission_edges=sorted(excluded), union_success_edges_dominate=True))
    for row in results:
        for node in row['local_instructions']:
            assert all((p, node['offset']) in local_edges for p in predecessors[node['offset']]), (
                'unaccounted local entry', node['offset'])
    report = dict(passed=True, function=census['function'], kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data), linked_function_sha256=digest(body),
        graph_sha256=digest(graph_bytes), origin_join_sha256=digest(origin_bytes),
        census_sha256=digest(census_bytes), transfer_regions=len(results), cases=results,
        copy_instructions=len(groups), copy_groups=groups, sampler_contracts=list(callee_contracts.values()),
        locations={str(i): origins['locations'][i] for i in sorted({
            origin['location'] for r in results for origin in r['byte_origins']})},
        scope='Selected REP intervals: raw candidate/count dataflow through specified preserved registers, exact byte-count equality without scaled-count overflow, CLD, accounted local entries and nonvacuous admission-group success-edge dominance. Shared clock output covers both callers and the intervening sampler RBX/R12 effects. Counterparty expressions and additional writes are exposed, not granted ownership. Shared helper soundness/preservation, complete roots/callees/exceptions, captured-entry stability, counterparty bounds/ownership and stable user mappings remain conditional. No complete syscall/access, native execution or production qualification')
    (build/'copy-transfer-audit.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ['cases', 'locations', 'copy_groups', 'sampler_contracts']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
