"""Conditional guarded circular byte transfers at owned user admissions.

TOPAL-SYSTEMS-USER-EXTENT-001; TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001.
Numeric table/ring expressions grant no owner, nonalias or lifetime authority.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph,reachable
from audit_pipe_fixed_outputs import Instructions
from inspect_call_inventory import Elf,digest,verify_calls


def loop_state(length,completed,cursor):
    assert 1 <= length <= 4096 and 0 <= completed < length and 0 <= cursor < 4096
    return dict(user_offset=completed,ring_offset=(cursor+completed)&4095,
        remaining=length-completed,next_completed=completed+1)


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
    assert graph['provider_sha256'] == origins['provider_sha256'] == digest(provider.data)
    assert graph['kernel_sha256'] == origins['kernel_sha256'] == digest(kernel.data)
    assert graph['linked_function_sha256'] == origins['linked_function_sha256'] == digest(body)
    assert graph['extent_inventory_sha256'] == digest(census_bytes)
    assert origins['process_graph_sha256'] == digest(graph_bytes)
    nodes,roots,successors,predecessors = check_graph(graph,kernel,symbol,body)
    origin_by_offset = {row['offset']:row for row in origins['accesses']}
    results = []
    for direction in ['read','write']:
        matches = [row for row in census['calls'] if 'pipe-'+direction+'-count-ready' in row['nearest_labels']]
        assert len(matches) == 1
        row, = matches
        assert row['target'] == 'runtime-'+('writable' if direction == 'read' else 'readable')+'-extent'
        call = row['offset']
        gate,success = call+14,call+20
        guard = Instructions(body,nodes,call-26)
        guard.take('4981f900000000')
        guard.fatal('0f84')
        guard.take('4981f900100000')
        guard.fatal('0f87')
        assert guard.at == call
        assert body[call+5:gate] == bytes.fromhex('48c74424f800000000')
        assert body[gate:gate+2] == bytes.fromhex('0f82')
        chain = guard.rows+[nodes[call],nodes[call+5],nodes[gate]]
        for left,right in zip(chain,chain[1:]):
            assert predecessors[right['offset']] == {left['offset']} and right['offset'] not in roots
        s = Instructions(body,nodes,success)
        for raw in ['4d89eb','4d29e3','4981c30020fcff','49c1eb07','49c1e30c','4d01e3','4981c300f00300',
                    '4d8bbd20000000']:
            s.take(raw)
        if direction == 'read':
            s.take('4c89c7')
        else:
            for raw in ['498b8528000000','4901c7','4981e7ff0f0000','4c89c6']:
                s.take(raw)
        s.take('4c89c9')
        start = s.at
        s.take('4c89de' if direction == 'read' else '4c89df')
        s.take('4c01fe' if direction == 'read' else '4c01ff')
        load = s.at
        s.take('8a06')
        store = s.at
        s.take('8807')
        readback = s.at
        s.take('3807')
        s.fatal('0f85')
        if direction == 'read':
            s.take('c60600')
            s.take('803e00')
            s.fatal('0f85')
        for raw in ['4881c701000000' if direction == 'read' else '4881c601000000',
                    '4981c701000000','4981e7ff0f0000','4881c1ffffffff','4881f900000000']:
            s.take(raw)
        back = s.at
        s.take('0f85'+struct.pack('<i',start-(back+6)).hex())
        end = s.at
        local = {r['offset'] for r in s.rows}
        assert sum(nodes[at]['size'] for at in local) == end-success
        assert not roots & local
        assert all(predecessors[at] <= local|{gate} for at in local), 'pipe alternate transfer entry'
        assert not local & reachable(roots,successors,(gate,success)), 'pipe loop bypasses admission'
        assert not local & reachable(roots,successors,(guard.rows[1]['offset'],guard.rows[2]['offset'])), 'pipe loop bypasses positive count'
        assert not local & reachable(roots,successors,(guard.rows[3]['offset'],call)), 'pipe loop bypasses count cap'
        accesses = [(store,'write'),(readback,'read')] if direction == 'read' else [(load,'read')]
        user = []
        for at,action in accesses:
            origin = origin_by_offset[at]
            user.append(dict(offset=at,action=action,width=1,bytes=origin['bytes'],
                instruction=origin['instruction'],byte_origins=origin['byte_origins'],
                base_register=7 if direction == 'read' else 6,
                candidate_offsets='0 <= completed < admitted <= 4096'))
        results.append(dict(name='pipe-'+direction+'-loop',call_offset=call,carry_gate=gate,
            success_offset=success,count_guard=guard.rows,length_minimum=1,length_maximum=4096,
            candidate_register=8,admitted_direction='write' if direction == 'read' else 'read',
            start_offset=start,end_offset=end,back_edge=[back,start],
            local_instructions=s.rows,user_accesses=user,
            counterpart=dict(object_register=13,ring_register=11,process_register=12,
                object_array_offset=0x3e000,object_stride=128,objects=4,ring_array_offset=0x3f000,ring_capacity=4096),
            loop_invariant='At iteration completed: 0 <= completed < admitted <= 4096, RCX = admitted-completed, user cursor = candidate+completed, ring cursor = (initial_cursor+completed)&4095; R8/R9 and captured arguments are unchanged'))
    report = dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=results,transfer_regions=2,user_access_instructions=3,
        scope='Conditional exact positive/capped count guards, admission/direction, closed circular byte-loop instructions, user offset bounds and direct-graph guard/success dominance. Ring/index/value expressions expose private access obligations; actual owners/nonalias/lifetime, helper/entry/callee/return/fatal/exception, syscall effects and production qualification remain separate. This report alone does not extend the published index')
    (build/'pipe-loop-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,regions=2,user_access_instructions=3,complete=False)))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
