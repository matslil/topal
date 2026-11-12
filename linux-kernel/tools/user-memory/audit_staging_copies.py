"""Check preserved admission values through bounded staging and exact REP copies.

TOPAL-SYSTEMS-USER-EXTENT-001. Numeric staging expressions and closed register
effects do not establish counterpart ownership or continuous mapping lifetime.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable
from inspect_call_inventory import Elf, digest, verify_calls


CASES = (
    ('console-input','runtime-console-payload','console','read'),
    ('descriptor-pread-output','fd-pread-count-ready','pread','write'),
    ('descriptor-read-output','fd-read-count-ready','read','write'),
    ('cwd-output','namespace-cwd-length','cwd','write'),
    ('sparse-read-output','file-read-end','sparse','write'),
    ('sparse-pread-output','file-pread-end','sparse','write'),
)


def bounded_count_initializers(nodes, predecessors, at, roots):
    """Every reaching R15 definition in this selected cwd prelude is 2 or 5."""
    assert at not in roots, ('unbounded cwd count entry',at)
    seen,pending,definitions = set(),list(predecessors[at]),[]
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        assert current in nodes and len(seen) < 64
        seen.add(current)
        raw = bytes.fromhex(nodes[current]['bytes'])
        if raw[:2] == bytes.fromhex('49 bf'):
            value = int.from_bytes(raw[2:],'little')
            assert len(raw) == 10 and value in [2,5]
            definitions.append(dict(offset=current,value=value))
            continue
        assert current not in roots, ('unbounded cwd count entry',current)
        assert ((len(raw) == 6 and raw[0] == 0x0f and 0x80 <= raw[1] <= 0x8f) or
                (len(raw) == 5 and raw[0] == 0xe9) or raw == bytes.fromhex('4c 39 fe') or
                (len(raw) == 7 and raw[:3] == bytes.fromhex('48 81 f8'))), ('unproved cwd count definition',current)
        assert predecessors[current]
        pending.extend(predecessors[current])
    assert {r['value'] for r in definitions} == {2,5}
    return sorted(definitions,key=lambda r:r['offset'])


def clipped_prelude():
    # Unsigned min with 256, then min with page remainder, then min with R13.
    # Later selections cannot increase the bound, even for arbitrary R13 bits.
    return bytes.fromhex('4c 89 cd 48 81 fd 00 01 00 00 0f 86 0a 00 00 00 '
        '48 bd 00 01 00 00 00 00 00 00 4c 89 f8 48 81 e0 ff 0f 00 00 '
        '48 b9 00 10 00 00 00 00 00 00 48 29 c1 48 39 cd 0f 86 03 00 00 00 '
        '48 89 cd 4c 39 ed 0f 86 03 00 00 00 4c 89 ed 49 89 e9')


class Slice:
    def __init__(self,body,nodes,start):
        self.body,self.nodes,self.start,self.at = body,nodes,start,start
        self.effects = []

    def take(self,hexadecimal):
        raw = bytes.fromhex(hexadecimal)
        assert self.body[self.at:self.at+len(raw)] == raw, ('staging slice encoding',self.at,raw.hex())
        self.at += len(raw)

    def branch(self,opcode,target=None,fatal=False):
        at = self.at
        assert self.body[at:at+2] == bytes.fromhex(opcode)
        actual = at+6+struct.unpack_from('<i',self.body,at+2)[0]
        if target is not None:
            assert actual == target
        if fatal:
            assert self.body[actual:actual+2] == bytes.fromhex('fa f4')
        self.at += 6
        return actual

    def wipe(self,console=False):
        if console:
            self.take('65 4c 8b 24 25 88 03 00 00 49 81 c4 00 98 00 00')
            self.take('4c 89 e7 48 b8 00 00 00 00 00 00 00 00 48 b9 c0 00 00 00 00 00 00 00 '
                'fc f3 48 ab 4c 89 e7 48 b9 c0 00 00 00 00 00 00 00 f3 48 af')
            size,offset = 1536,0x9800
        else:
            self.take('4c 89 e7 48 81 c7 80 05 00 00 48 b8 00 00 00 00 00 00 00 00 '
                '48 b9 20 00 00 00 00 00 00 00 fc f3 48 ab '
                '4c 89 e7 48 81 c7 80 05 00 00 48 b9 20 00 00 00 00 00 00 00 f3 48 af')
            size,offset = 256,0x580
        self.branch('0f 85',fatal=True)
        self.effects.append(dict(kind='staging-wipe-and-readback',bytes=size,offset=offset,
            authority='Unqualified owner/alias obligation; exact numeric shape only'))


def page_lookup_contract(body,nodes,entry):
    expected = bytes.fromhex('4d 89 e3 49 81 c3 00 34 04 00 48 b9 00 00 00 00 00 00 00 00 '
        '49 8b 83 00 00 00 00 49 3b 82 08 00 00 00 0f 85 14 00 00 00 '
        '4c 89 f8 48 c1 e8 0c 49 3b 83 08 00 00 00 0f 84 27 00 00 00 '
        '49 81 c3 20 00 00 00 48 81 c1 01 00 00 00 48 81 f9 08 00 00 00 '
        '0f 82 bd ff ff ff 49 bb 00 00 00 00 00 00 00 00 f9 c3 f8 c3')
    assert body[entry:entry+len(expected)] == expected, 'page lookup register/loop contract'
    rows = [n for at,n in sorted(nodes.items()) if entry <= at < entry+len(expected)]
    assert sum(n['size'] for n in rows) == len(expected)
    return dict(entry=entry,bytes=len(expected),sha256=digest(expected),instructions=rows,
        register_writes=[0,1,11],preserved_candidate_register=8,preserved_count_register=5,
        stack_bytes=8,recursive=False,found_slot_range=[0,7],
        scope='Exact closed lookup leaf: bounded slot loop, CF outcome and register/stack shape. Association reads still require owner, format and lifetime proof')


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
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    assert graph['function'] == origins['function'] == census['function']
    nodes,roots,successors,predecessors = check_graph(graph,kernel,symbol,body)
    access_origins = {r['offset']:r for r in origins['accesses']}
    cases,contracts = [],{}
    for name,label,kind,direction in CASES:
        matches = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1
        call = matches[0]['offset']
        assert matches[0]['target'] == 'runtime-'+('readable' if direction == 'read' else 'writable')+'-extent'
        gate,success = call+14,call+20
        assert body[gate:gate+2] == bytes.fromhex('0f 82')
        assert predecessors[call+5] == {call} and predecessors[gate] == {call+5}
        assert not roots & {call,call+5,gate}
        count_definitions = []
        if kind == 'console':
            start = call-32
            p = Slice(body,nodes,start)
            p.take('48 81 fa 00 01 00 00')
            too_long = p.branch('0f 87')
            p.take('48 81 fa 00 00 00 00')
            empty = p.branch('0f 84')
            for rejected in [too_long,empty]:
                assert rejected in nodes and call not in reachable({rejected},successors), (
                    'console count refusal can reach admission',rejected)
            p.take('49 89 f0 49 89 d1')
            assert p.at == call
            maximum,count_register = 256,9
        elif kind in ['read','pread']:
            start = call-23
            p = Slice(body,nodes,start)
            p.take('49 81 f9 00 01 00 00')
            p.branch('0f 86',call)
            p.take('49 b9 00 01 00 00 00 00 00 00')
            maximum,count_register = 256,9
        elif kind == 'sparse':
            raw = clipped_prelude()
            start = call-len(raw)
            assert body[start:call] == raw, 'clipped nonincreasing count'
            maximum,count_register = 256,5
        else:
            start = call-3
            assert body[start:call] == bytes.fromhex('4d 89 f9')
            count_definitions = bounded_count_initializers(nodes,predecessors,start,roots)
            maximum,count_register = 5,15
        prelude = [r for at,r in sorted(nodes.items()) if start <= at < call]
        assert sum(r['size'] for r in prelude) == call-start
        allowed_prelude = {r['offset'] for r in prelude}
        for row in prelude[1:]+[nodes[call]]:
            assert row['offset'] not in roots and predecessors[row['offset']] <= allowed_prelude
        s = Slice(body,nodes,success)
        s.wipe(kind == 'console')
        if kind == 'console':
            s.take('4c 89 c6 4c 89 c9 4c 89 e7 fc')
            counterpart = dict(kind='process-snapshot-staging',offset=0x9800,capacity=1536)
        elif kind in ['read','pread']:
            s.take('49 8b b4 24 10 00 00 00 4c 01 fe 4c 89 e7 48 81 c7 80 05 00 00 4c 89 c9')
            if kind == 'pread': s.take('fc')
            s.take('f3 a4')
            s.effects.append(dict(kind='image-to-staging-copy',count='admitted R9',
                authority='Separate image/staging owner and nonalias obligation'))
            s.take('4c 89 e6 48 81 c6 80 05 00 00 4c 89 c7 4c 89 c9')
            if kind == 'pread': s.take('fc')
            counterpart = dict(kind='process-table-staging',offset=0x580,capacity=256)
        elif kind == 'cwd':
            s.take('48 b8 2f 62 69 6e 00 00 00 00 49 81 ff 05 00 00 00')
            branch = s.branch('0f 84')
            s.take('48 b8 2f 00 00 00 00 00 00 00')
            assert branch == s.at
            s.take('49 8b 8c 24 28 00 00 00 48 81 f9 06 00 00 00')
            branch = s.branch('0f 85')
            s.take('48 b8 2f 74 6d 70 00 00 00 00')
            assert branch == s.at
            s.take('49 89 84 24 80 05 00 00 4c 89 c7 4c 89 e6 48 81 c6 80 05 00 00 4c 89 f9 fc')
            s.effects.append(dict(kind='cwd-staging-store',width=8,offset=0x580,authority='Separate table/staging owner obligation'))
            counterpart = dict(kind='process-table-staging',offset=0x580,capacity=256)
        else:
            s.take('49 89 da')
            at = s.at
            assert body[at] == 0xe8
            entry = at+5+struct.unpack_from('<i',body,at+1)[0]
            contracts[entry] = page_lookup_contract(body,nodes,entry)
            s.at += 5
            absent = s.branch('0f 82')
            s.take('48 89 ce 48 c1 e6 0c 4c 01 e6 48 81 c6 00 40 04 00 '
                '4c 89 f8 48 81 e0 ff 0f 00 00 48 01 c6 4c 89 e7 48 81 c7 80 05 00 00 48 89 e9 fc f3 a4')
            s.effects.append(dict(kind='resident-page-to-staging-copy',count='admitted RBP',
                authority='Separate page/table/staging owner and nonalias obligation'))
            assert absent == s.at
            s.take('4c 89 c7 4c 89 e6 48 81 c6 80 05 00 00 48 89 e9 fc')
            counterpart = dict(kind='process-table-staging',offset=0x580,capacity=256)
        copy = s.at
        s.take('f3 a4')
        if kind == 'console':
            assert body[s.at:s.at+6] == bytes.fromhex('4c 89 e6 4d 89 ce'), 'UART count must retain admitted R9'
        local = [r for at,r in sorted(nodes.items()) if success <= at < s.at]
        assert sum(r['size'] for r in local) == s.at-success
        allowed = {r['offset'] for r in local}|{gate}
        assert all(r['offset'] not in roots and predecessors[r['offset']] <= allowed for r in local)
        assert copy not in reachable(roots,successors,(gate,success)), ('staging success dominance',name)
        origin = access_origins[copy]
        assert origin['bytes'] == 'f3a4'
        cases.append(dict(name=name,census_label=label,call_offset=call,prelude_start=start,
            carry_gate=gate,success_offset=success,copy_offset=copy,copy_size=2,copy_width=1,
            admitted_direction=direction,user_endpoint_register=6 if direction=='read' else 7,
            candidate_register=8,count_register=count_register,maximum_bytes=maximum,
            counterpart=counterpart,count_initializers=count_definitions,other_memory_effects=s.effects,
            prelude=prelude,local_instructions=local,byte_origins=origin['byte_origins']))
    groups = [dict(copy_offset=r['copy_offset'],cases=[r['name']],
        admission_edges=[[r['carry_gate'],r['success_offset']]],union_success_edges_dominate=True) for r in cases]
    report = dict(passed=True,function=census['function'],provider_sha256=digest(provider.data),
        kernel_sha256=digest(kernel.data),linked_function_sha256=digest(body),census_sha256=digest(census_bytes),
        graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),cases=cases,
        transfer_regions=len(cases),copy_instructions=len(groups),copy_groups=groups,callee_contracts=list(contracts.values()),
        scope='Selected exact-byte staging paths: preserved admitted R8 candidate and equal byte count through wipes, private preparation/copy and closed page lookup; explicit maxima, CLD/no intervening DF change, incoming-edge closure and success dominance. Extra memory effects are exposed rather than granted owners. Complete helper/root/callee/exception, owner/nonalias and continuous-lifetime obligations remain; no whole-caller or production claim')
    (build/'staging-copy-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['cases','copy_groups','callee_contracts']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
