"""Conditional byte bounds for the serial receiver's positive output prefix.

TOPAL-SYSTEMS-USER-EXTENT-001. Exact instructions do not establish private
owners/nonaliasing, uninterrupted mapping lifetime or full syscall behavior.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph,reachable
from audit_pipe_fixed_outputs import Instructions
from audit_staging_copies import Slice
from inspect_call_inventory import Elf,digest,verify_calls


def prefix_step(admitted,received):
    """One actual byte-receive transition; UART readiness chooses either exit.

    A waiting UART iteration does not call this transition or write memory.
    At the byte store, received is the exact staging-relative write index.
    """
    assert 0 < admitted <= 256 and 0 <= received < admitted
    following = received+1
    assert 1 <= following <= admitted
    return dict(store_index=received,received=following,may_receive_more=following<admitted)


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
    matches = [row for row in census['calls'] if 'fd-serial-count-ready' in row['nearest_labels']]
    assert len(matches) == 1 and matches[0]['target'] == 'runtime-writable-extent'
    call = matches[0]['offset']
    cap = call-23
    pre = Instructions(body,nodes,cap)
    for raw in ['4981f900010000','0f860a000000','49b90001000000000000']: pre.take(raw)
    assert pre.at == call and predecessors[call] == {cap+7,cap+13}
    assert predecessors[cap+7] == {cap} and predecessors[cap+13] == {cap+7}
    # The sole branch into the cap follows the existing zero-length exit.
    assert len(predecessors[cap]) == 1
    dispatch = next(iter(predecessors[cap]))
    zero = dispatch-13
    guard = Instructions(body,nodes,zero-7)
    guard.take('4981f900000000')
    raw = body[guard.at:guard.at+6]
    assert raw[:2] == bytes.fromhex('0f84')
    zero_target = guard.at+6+struct.unpack_from('<i',raw,2)[0]
    guard.take(raw.hex())
    guard.take('4881fb01000000')
    raw = body[guard.at:guard.at+6]
    assert guard.at == dispatch and raw[:2] == bytes.fromhex('0f84')
    assert guard.at+6+struct.unpack_from('<i',raw,2)[0] == cap
    guard.take(raw.hex())
    assert body[zero_target:zero_target+10] == bytes.fromhex('48b80000000000000000')
    for left,right in zip(guard.rows,guard.rows[1:]):
        assert predecessors[right['offset']] == {left['offset']}
    assert not roots & {row['offset'] for row in guard.rows[1:]}, 'serial entry bypasses zero comparison'
    gate,success = call+14,call+20
    assert body[call+5:gate] == bytes.fromhex('48c74424f800000000')
    assert body[gate:gate+2] == bytes.fromhex('0f82')
    assert predecessors[call+5] == {call} and predecessors[gate] == {call+5}
    assert cap not in reachable(roots,successors,(zero,zero+6)), 'zero count can enter serial receiver'
    s = Slice(body,nodes,success)
    s.wipe()
    s.take('4c89e64881c68005000049bf0000000000000000')
    wait = s.at
    s.take('66bafd03eca801')
    s.branch('0f84',wait)
    receive = s.at
    s.take('66baf803ec88064881c6010000004981c7010000004d39cf')
    full = s.branch('0f83')
    s.take('66bafd03eca801')
    s.branch('0f85',receive)
    assert s.at == full
    s.take('4c89e64881c6800500004c89c74c89f9')
    copy = s.at
    s.take('f3a4')
    end = s.at
    cleanup = Slice(body,nodes,end)
    cleanup.wipe()
    cleanup.take('4c89f8')
    local = {at for at in nodes if success <= at < end}
    assert sum(nodes[at]['size'] for at in local) == end-success
    allowed = local|{gate}
    assert not roots & (local|{call,call+5,gate,cap,cap+7,cap+13})
    assert all(predecessors[at] <= allowed for at in local), 'serial alternate local entry'
    bypass = reachable(roots,successors,(gate,success))
    assert not local & bypass, 'serial copy bypasses output admission'
    assert copy not in reachable(roots,successors,(zero,zero+6))
    origin = next(row for row in origins['accesses'] if row['offset'] == copy)
    assert origin['bytes'] == 'f3a4' and origin['size'] == 2
    row = dict(name='serial-prefix-output',call_offset=call,carry_gate=gate,success_offset=success,
        candidate_register=8,admitted_direction='write',admitted_maximum=256,
        copy_offset=copy,copy_size=2,copy_width=1,copy_count_kind='positive-prefix',
        copy_count_register=15,counterpart=dict(base_register=12,offset=1408,capacity=256),
        zero_guard=guard.rows,cap_prelude=pre.rows,zero_exit=zero_target,
        local_instructions=[nodes[at] for at in sorted(local)],
        wait_offset=wait,receive_offset=receive,publication_offset=full,
        post_copy_cleanup=[nodes[at] for at in sorted(nodes) if end <= at < cleanup.at],
        user_accesses=[dict(offset=copy,action='write',width=1,
            candidate_offsets='0 <= offset < received <= admitted <= 256',byte_origins=origin['byte_origins'])])
    report = dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=[row],transfer_regions=1,copy_instructions=1,
        copy_groups=[dict(copy_offset=copy,cases=[row['name']],admission_edges=[[gate,success]])],
        receive_invariant='At each byte store: RSI = R12+1408+received and 0 <= received < admitted <= 256. Each store increments received once. Publication has 1 <= received <= admitted; only that prefix is copied.',
        scope='Conditional exact bounded UART staging and positive output-prefix dataflow, zero/cap/admission success dominance, no local calls, explicit CLD and staging cleanup. UART availability may end a read early; no liveness guarantee. Complete actual entry/helper/owner/nonalias/lifetime/fatal/exception and production obligations remain separate; this report alone does not extend the published index')
    (build/'serial-prefix-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,transfer_regions=1,copy_instructions=1,call_offset=call,copy_offset=copy,
        admitted_maximum=256,complete=False)))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
