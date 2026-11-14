"""Conditional exact-byte checks for fixed pipe outputs and their lookup leaf.

TOPAL-SYSTEMS-USER-EXTENT-001. These checks do not create memory ownership,
prove continuous lifetime, or qualify the complete caller or production kernel.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable, scalar
from inspect_call_inventory import Elf, digest, verify_calls


class Instructions:
    """Match each reviewed instruction boundary as well as its exact bytes."""
    def __init__(self, body, nodes, start):
        self.body, self.nodes, self.start, self.at = body, nodes, start, start
        self.rows = []

    def take(self, encoding):
        raw = bytes.fromhex(encoding)
        row = self.nodes[self.at]
        assert row['size'] == len(raw) and row['bytes'] == raw.hex(), (
            'pipe instruction contract', self.at, raw.hex())
        assert self.body[self.at:self.at + len(raw)] == raw
        self.rows.append(row)
        self.at += len(raw)

    def fatal(self, opcode):
        prefix = bytes.fromhex(opcode)
        assert prefix == b'\xe9' or (len(prefix) == 2 and prefix[0] == 0x0f
                                    and 0x80 <= prefix[1] <= 0x8f)
        size = len(prefix) + 4
        raw = self.body[self.at:self.at + size]
        assert raw[:len(prefix)] == prefix
        target = self.at + size + struct.unpack_from('<i', raw, len(prefix))[0]
        assert 0 <= target < len(self.body) - 1 and target in self.nodes
        assert self.body[target:target + 2] == bytes.fromhex('fa f4'), 'pipe fatal boundary'
        self.take(raw.hex())
        return target


def pair_preparation(s):
    """Closed publishing sequence: R8/R9 are never written; no nested call.

    Stores through R12/R13/R14/RAX still require actual owner and nonalias
    proofs. Numeric expressions alone cannot make them private memory.
    """
    s.take('4d8bac2480050000')
    s.take('4d89bd08000000')
    for index in range(2):
        s.take('4d8b9c24' + ('98050000' if index == 0 else 'a0050000'))
        s.take('4d899d' + ('10000000' if index == 0 else '18000000'))
        for raw in ['4d89e6', '4981c640010000', '4981fb10000000',
                    '0f8607000000', '4981c6c0ca0300', '4c89d8',
                    '4881c0ffffffff', '48c1e006', '4901c6']:
            s.take(raw)
        s.take('48b8' + struct.pack('<Q', 5 + index).hex())
        for raw in ['49898600000000', '48b80100000000000000',
                    '49898608000000', '4889d8', '4881e000080000']:
            s.take(raw)
        s.take('4881c8' + struct.pack('<I', index).hex())
        s.take('49898610000000')
        s.take('4d89be20000000')
        s.take('498b8424' + ('88050000' if index == 0 else '90050000'))
        for raw in ['48c1e004', '4c01e0', '4881c040000000', '4c899800000000',
                    '4889d9', '48c1e913', '4881e101000000', '48898808000000']:
            s.take(raw)
    for offset in [72, 80, 88, 96]:
        s.take('4989ad' + struct.pack('<I', offset).hex())
    for raw in ['48b88011000000000000', '49898540000000',
                '48b80100000000000000', '49898500000000', '4d89bc2400e20300',
                '498b842488050000', '498b8c2490050000', '48c1e120', '4809c8']:
        s.take(raw)


def lookup_contract(body, nodes, entry, roots, successors, predecessors):
    """Exact bounded leaf, preserving R8/R9 on normal return.

    RAX/RCX/R10/R11 are the only explicit register destinations. No stores,
    nested calls or stack adjustments occur; RET consumes the caller's one
    return word. Entry arguments, that word's owner and fatal/exception effects
    remain separate obligations.
    """
    s = Instructions(body, nodes, entry)
    fatal = []
    for raw in ['4d8b9e20000000', '4981fb00000000']:
        s.take(raw)
    fatal.append(s.fatal('0f84'))
    for raw in ['4d89e2', '4981c200e00300', '48b90400000000000000',
                '498b8200000000', '4881f801000000', '0f8510000000',
                '498b8208000000', '4c39d8', '0f8420000000',
                '4981c280000000', '4881c1ffffffff', '4881f900000000',
                '0f85c1ffffff']:
        s.take(raw)
    fatal.append(s.fatal('e9'))
    s.take('498b8220000000')
    s.take('4881f800100000')
    fatal.append(s.fatal('0f83'))
    s.take('498b8228000000')
    s.take('4881f800100000')
    fatal.append(s.fatal('0f87'))
    for raw in ['4c89f0', '4c29e0', '4881f800d00300', '0f8217000000',
                '4881c00030fcff', '48c1e806', '4881c011000000', 'e912000000',
                '4881c0c0feffff', '48c1e806', '4881c001000000',
                '4d8b9e00000000', '4981fb05000000', '0f8419000000',
                '4981fb06000000']:
        s.take(raw)
    fatal.append(s.fatal('0f85'))
    for raw in ['498b8a18000000', 'e907000000', '498b8a10000000', '4839c8']:
        s.take(raw)
    fatal.append(s.fatal('0f85'))
    s.take('c3')
    local = {r['offset'] for r in s.rows}
    assert not roots & (local - {entry}), 'lookup alternate root'
    assert all(predecessors[at] <= local for at in local - {entry}), 'lookup alternate entry'
    assert all(successors[at] <= local | set(fatal) for at in local), 'lookup exit'
    assert len(set(fatal)) == 1
    return dict(entry=entry, end=s.at, instructions=s.rows,
                normal_return_offset=s.at - 1, fatal_boundaries=sorted(set(fatal)),
                register_writes=[0, 1, 10, 11], preserved_registers=[8, 9],
                borrowed_call_stack_bytes=8, memory_writes=0,
                maximum_object_records=4,
                scope='Exact normal-return register/control shape; private reads, entry/return-word ownership, fatal and exception effects remain unqualified')


def check_local(nodes, roots, successors, predecessors, gate, success, rows, accesses):
    local = {r['offset'] for r in rows}
    assert rows[0]['offset'] == success and success in successors[gate]
    assert not roots & local, 'pipe alternate local root'
    assert all(predecessors[at] <= local | {gate} for at in local), 'pipe alternate local entry'
    bypass = reachable(roots, successors, (gate, success))
    assert not local & bypass, 'pipe success-edge dominance'
    for row, direction, width in accesses:
        action, offset, actual_width, base, _ = scalar(bytes.fromhex(row['bytes']))
        assert (action, offset, actual_width, base) == (direction, 0, width, 8), 'pipe output endpoint'
        assert row['offset'] in local


def audit(build):
    census_bytes = (build / 'extent-call-inventory.json').read_bytes()
    census = json.loads(census_bytes)
    provider, kernel = Elf(build / 'provider.o'), Elf(build / 'kernel.elf')
    verify_calls(provider, census)
    symbol = verify_calls(kernel, census)
    _, body = kernel.function(census['function'])
    graph_bytes, origin_bytes = [(build / name).read_bytes() for name in
        ['process-access-inventory.json', 'process-access-origins.json']]
    graph, origins = json.loads(graph_bytes), json.loads(origin_bytes)
    assert graph['passed'] and origins['passed']
    assert graph['function'] == origins['function'] == census['function']
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    nodes, roots, successors, predecessors = check_graph(graph, kernel, symbol, body)
    access_origins = {r['offset']: r for r in origins['accesses']}
    cases, contracts = [], []
    for name, label, width, prelude in [
            ('pipe-pair-output', 'pipe-descriptions-ready', 8,
             ['654c8b042538010000', '49b90800000000000000']),
            ('pipe-readable-count-output', 'pipe-ioctl', 4,
             ['4989d0', '49b90400000000000000'])]:
        matches = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1 and matches[0]['target'] == 'runtime-writable-extent'
        call = matches[0]['offset']
        gate, success = call + 14, call + 20
        start = call - sum(len(bytes.fromhex(raw)) for raw in prelude)
        s = Instructions(body, nodes, start)
        for raw in prelude:
            s.take(raw)
        assert s.at == call
        previous = start
        for row in s.rows[1:] + [nodes[call]]:
            assert row['offset'] not in roots and predecessors[row['offset']] == {previous}
            previous = row['offset']
        assert predecessors[call + 5] == {call} and predecessors[gate] == {call + 5}
        assert not roots & {call + 5, gate}
        result = Instructions(body, nodes, call + 5)
        result.take('48c74424f800000000')
        assert body[gate:gate + 2] == bytes.fromhex('0f82')
        s = Instructions(body, nodes, success)
        if width == 8:
            pair_preparation(s)
            store = s.at
            s.take('49898000000000')
            readback = s.at
            s.take('4d8b9800000000')
            s.take('4c39d8')
            fatal = s.fatal('0f85')
            accesses = [(nodes[store], 'write', 8), (nodes[readback], 'read', 8)]
            effects = dict(register_writes=[0, 1, 11, 13, 14], nested_calls=0,
                           private_store_bases=[0, 12, 13, 14], fatal_boundaries=[fatal])
        else:
            raw = body[s.at:s.at + 5]
            assert len(raw) == 5 and raw[0] == 0xe8, 'pipe lookup call'
            entry = s.at + 5 + struct.unpack_from('<i', raw, 1)[0]
            contracts.append(lookup_contract(body, nodes, entry, roots, successors, predecessors))
            s.take(raw.hex())
            s.take('498b8228000000')
            store = s.at
            s.take('418900')
            accesses = [(nodes[store], 'write', 4)]
            effects = dict(register_writes=[0, 1, 10, 11], nested_calls=1,
                           lookup_entry=entry, borrowed_call_stack_bytes=8)
        check_local(nodes, roots, successors, predecessors, gate, success, s.rows, accesses)
        selected = []
        for row, direction, size in accesses:
            origin = access_origins[row['offset']]
            assert origin['bytes'] == row['bytes'] and origin['size'] == row['size']
            selected.append(dict(offset=row['offset'], action=direction, width=size,
                candidate_offset=0, base_register=8, bytes=row['bytes'],
                instruction=row['instruction'], byte_origins=origin['byte_origins']))
        cases.append(dict(name=name, census_label=label, call_offset=call,
            candidate_register=8, admitted_direction='write', admitted_bytes=width,
            candidate_setup_offset=start, success_edge=[gate, success], local_end=s.at,
            local_instructions=s.rows, access_instructions=selected, additional_effects=effects))
    report = dict(passed=True, complete=False, function=census['function'],
        kernel_sha256=digest(kernel.data), provider_sha256=digest(provider.data),
        linked_function_sha256=digest(body), census_sha256=digest(census_bytes),
        graph_sha256=digest(graph_bytes), origin_join_sha256=digest(origin_bytes),
        cases=cases, callee_contracts=contracts, transfer_regions=len(cases),
        scalar_access_instructions=sum(len(r['access_instructions']) for r in cases),
        scope='Two conditional local fixed-output intervals only: exact instruction contracts, R8/R9 preservation, writable admission and bounded readback, closed normal-return lookup, incoming-entry and success-edge checks. Private/table/stack/output owners and nonalias, continuous lifetime, helper/root/exception/fatal contracts, native transfers, other callers and production qualification remain open. This report alone does not extend the published qualified-site index')
    (build / 'pipe-fixed-output-audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ['cases', 'callee_contracts']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
