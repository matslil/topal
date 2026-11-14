"""Conditional initial/reusable random output loops and extraction stack shape.

TOPAL-SYSTEMS-USER-EXTENT-001;
TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001.
Numeric key/output/stack expressions do not grant owners or nonaliasing.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph,reachable
from audit_warning_inputs import Slice
from inspect_call_inventory import Elf,digest,verify_calls
from random_extract_effects import effects
from retired_stack_words import take as retired_words


def extract_contract(body,nodes,roots,successors,predecessors,entry):
    local=reachable([entry],successors)
    assert len(local)<2048 and not roots & (local-{entry})
    assert all(predecessors[at]<=local for at in local-{entry}), 'extraction alternate entry'
    decoded={at:effects(bytes.fromhex(nodes[at]['bytes'])) for at in local}
    assert body[entry+27:entry+33]==bytes.fromhex('4989fa4989f3')
    allocations=[at for at,e in decoded.items() if e.get('scratch_delta')==-128]
    releases=[at for at,e in decoded.items() if e.get('scratch_delta')==128]
    assert allocations==[entry+33] and len(releases)==2
    seen,pending,returns=set(),[(entry,0,False,0,0)],set()
    while pending:
        state=pending.pop()
        if state in seen: continue
        seen.add(state)
        assert len(seen)<8192
        at,depth,zero,dirty,aliases=state
        row=decoded[at]
        assert row['register_writes']<={0,1,2,4,8,10,11}
        if row.get('argument_alias'):
            destination,source=row['argument_alias']
            assert (at,destination,source) in [(entry+27,10,7),(entry+30,11,6)]
            aliases |= 1 if destination==10 else 2
        else: assert not row['register_writes']&{10,11}
        for base,offset,width in row['memory_reads']+row['memory_writes']:
            if base==4: assert depth==128 and 0<=offset and offset+width<=128
            else: assert aliases==3 and base in [10,11] and 0<=offset and offset+width<=32
        for base,offset,width in row['memory_writes']:
            if base!=4: continue
            mask=((1<<width)-1)<<offset
            cleared=row.get('immediate_store')==0 or row.get('stored_register')==0 and zero
            dirty=dirty&~mask if cleared else dirty|mask
        delta=row.get('scratch_delta')
        if delta==-128:
            assert depth==0 and aliases==3
            depth,dirty=128,(1<<128)-1
        elif delta==128:
            assert depth==128 and dirty==0, 'extraction retires uncleared scratch'
            depth=0
        else: assert 4 not in row['register_writes']
        if row.get('zero_register')==0: zero=True
        elif 0 in row['register_writes']: zero=False
        if row.get('control')=='return':
            assert depth==0
            returns.add(at)
        else:
            assert successors[at] and successors[at]<=local
            pending.extend((target,depth,zero,dirty,aliases) for target in successors[at])
    assert len(returns)==2
    return dict(entry=entry,instructions=[nodes[at] for at in sorted(local)],
        effects=[dict(offset=at,**dict(decoded[at],register_writes=sorted(decoded[at]['register_writes']))) for at in sorted(local)],
        allocations=allocations,releases=sorted(releases),normal_returns=sorted(returns),
        maximum_scratch_bytes=128,borrowed_stack_bytes_including_return=136,
        scratch_zero_before_every_release=True,argument_registers_preserved=[6,7],
        explored_frame_states=len(seen),
        scope='Closed explicit leaf effects and CFG frame/zero-store dataflow: 128-byte scratch is cleared before both releases, every return has entry RSP, RSI/RDI are unmodified, and key/output expressions fit 32 bytes. Actual argument/key/output/stack owners, nonalias, return-word retirement, exceptions and cryptographic semantics are separate obligations')


def random_iteration(length,completed):
    assert 1<=length<=16 and 0<=completed<length
    return dict(user_offset=completed,next_completed=completed+1)


def audit(build):
    census_bytes=(build/'extent-call-inventory.json').read_bytes()
    census=json.loads(census_bytes)
    provider,kernel=Elf(build/'provider.o'),Elf(build/'kernel.elf')
    verify_calls(provider,census)
    symbol=verify_calls(kernel,census)
    _,body=kernel.function(census['function'])
    graph_bytes,origin_bytes=[(build/name).read_bytes() for name in
        ['process-access-inventory.json','process-access-origins.json']]
    graph,origins=json.loads(graph_bytes),json.loads(origin_bytes)
    assert graph['passed'] and origins['passed']
    assert graph['function']==origins['function']==census['function']
    assert graph['provider_sha256']==origins['provider_sha256']==digest(provider.data)
    assert graph['kernel_sha256']==origins['kernel_sha256']==digest(kernel.data)
    assert graph['linked_function_sha256']==origins['linked_function_sha256']==digest(body)
    assert graph['extent_inventory_sha256']==digest(census_bytes)
    assert origins['process_graph_sha256']==digest(graph_bytes)
    nodes,roots,successors,predecessors=check_graph(graph,kernel,symbol,body)
    calls=[r for r in census['calls'] if 'static-process-getrandom' in r['nearest_labels']]
    assert len(calls)==1 and calls[0]['target']=='runtime-writable-extent'
    call=calls[0]['offset']
    prefix=bytes.fromhex('4883fe100f87')
    start=body.rfind(prefix,max(0,call-128),call)
    assert start>=0
    pre=Slice(body,nodes,start)
    pre.take('4883fe10')
    excessive=pre.branch('0f87')
    pre.take('4885f6')
    zero=pre.branch('0f84')
    pre.take('4881f900104000')
    pre.fatal('0f82')
    pre.take('4881f985c25800')
    pre.fatal('0f83')
    for raw in ['4889f8','4989f1','4989c0']: pre.take(raw)
    assert pre.at==call
    pre_offsets={r['offset'] for r in pre.rows}
    assert not roots & (pre_offsets-{start}) and call not in roots
    assert all(predecessors[at]<=pre_offsets for at in pre_offsets-{start})
    assert predecessors[call]=={pre.rows[-1]['offset']}
    gate,success=call+14,call+20
    assert body[call+5:gate]==bytes.fromhex('48c74424f800000000')
    assert predecessors[call+5]=={call} and predecessors[gate]=={call+5}
    assert not roots&{call+5,gate}
    gate_slice=Slice(body,nodes,gate)
    failure=gate_slice.branch('0f82')
    s=Slice(body,nodes,success)
    s.take('658a0425b8000000')
    s.take('3c01')
    initial=s.branch('0f84')
    s.take('3c02')
    s.fatal('0f82')
    s.take('3c03')
    s.fatal('0f87')
    s.take('6548833c251001000000')
    s.fatal('0f85')
    for raw in ['654889242510010000','65488b242508000000','4153','51','57','56',
                '65488b3c2500010000','4885ff']:
        s.take(raw)
    s.fatal('0f84')
    s.take('488db7e0000000')
    s.take('4881c7c0000000')
    extract_call=s.at
    encoded=body[s.at:s.at+5]
    assert encoded[0]==0xe8
    extract_entry=s.at+5+struct.unpack_from('<i',encoded,1)[0]
    s.take(encoded.hex())
    callee=extract_contract(body,nodes,roots,successors,predecessors,extract_entry)
    extract_return_word=retired_words(s,1)
    for raw in ['5e','5f','59','415b']:
        s.take(raw)
    caller_spills=retired_words(s,4)
    for raw in ['65488b242510010000','6548c704251001000000000000','85c0']:
        s.take(raw)
    s.fatal('0f84')
    s.take('654c8b0c2500010000')
    s.take('4981c1e0000000')
    join=s.branch('e9')
    assert s.at==initial
    s.take('654c8b0c2500010000')
    s.take('4981c1a8000000')
    assert s.at==join
    s.take('4531d2')
    loop=s.at
    s.take('4939f2')
    complete=s.branch('0f84')
    load=s.at
    s.take('438a0411')
    store=s.at
    s.take('42880417')
    readback=s.at
    s.take('423a0417')
    s.fatal('0f85')
    s.take('49ffc2')
    back=s.at
    s.branch('e9',loop)
    assert s.at==complete
    local={r['offset'] for r in s.rows}
    assert not roots&local
    assert all(predecessors[at]<=local|{gate} for at in local)
    assert not local&reachable(roots,successors,(gate,success))
    assert not local&reachable(roots,successors,(pre.rows[1]['offset'],pre.rows[2]['offset']))
    assert not local&reachable(roots,successors,(pre.rows[3]['offset'],pre.rows[4]['offset']))
    origin_by_offset={r['offset']:r for r in origins['accesses']}
    user=[]
    for at,action in [(store,'write'),(readback,'read')]:
        origin=origin_by_offset[at]
        assert origin['bytes']==nodes[at]['bytes']
        user.append(dict(offset=at,action=action,width=1,base_register=7,index_register=10,
            bytes=origin['bytes'],byte_origins=origin['byte_origins'],
            candidate_offsets='0 <= completed < admitted <= 16'))
    report=dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=[dict(name='bounded-random-output',call_offset=call,carry_gate=gate,success_offset=success,
            candidate_register=7,count_register=6,admitted_direction='write',minimum_bytes=1,maximum_bytes=16,
            prelude=pre.rows,local_instructions=s.rows,user_accesses=user,
            initial_output=initial,extract_call=extract_call,extract_entry=extract_entry,
            switched_stack_gs_offset=8,suspended_stack_gs_offset=272,outer_saved_words=4,
            selected_switched_stack_bytes=32+callee['borrowed_stack_bytes_including_return'],
            extract_return_word_retirement=extract_return_word,caller_spill_retirement=caller_spills,
            retired_outer_spill_erasure_verified=True,
            loop_offset=loop,load_offset=load,start_offset=store,readback_offset=readback,
            end_offset=complete,back_edge=[back,loop],zero_exit=zero,excessive_exit=excessive,failure_exit=failure,
            source=dict(captured_gs_field=256,initial_offset=168,reusable_offset=224,capacity=32))],
        extract_contract=callee,transfer_regions=1,user_access_instructions=2,
        scope='Conditional 1..16 count and success-edge dominance, exact initial/reusable preparation, suspended-stack save/switch/restore, closed extraction frame/effects, explicit returned-callee-word and four popped-word retirement, and scalar output/readback loop. Actual key/output/stack/user owners and nonalias, GS stack/reference stability, complete helpers/entry/fatal/exception/retirement/production remain open; this report alone does not extend the published index')
    (build/'random-output-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,regions=1,user_instructions=2,extract_instructions=len(callee['instructions']),
        switched_stack_bytes=168,complete=False)))


if __name__=='__main__': audit(Path(sys.argv[1]).resolve())
