"""Conditional volatile write/pwrite input copies through page and RTC lookup.

TOPAL-SYSTEMS-USER-EXTENT-001; TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001.
Normal-return register/stack shape does not establish owner, nonalias, retired
spill cleanup, exception behavior or continuous check-to-copy lifetime.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph,reachable
from audit_pipe_fixed_outputs import Instructions
from audit_staging_copies import Slice,clipped_prelude,page_lookup_contract
from file_input_effects import effects
from inspect_call_inventory import Elf,digest,verify_calls


def call_target(s):
    at=s.at
    raw=s.body[at:at+5]
    assert raw[0]==0xe8
    s.take(raw.hex())
    return at+5+struct.unpack_from('<i',raw,1)[0]


def leaf_contract(body,nodes,roots,successors,predecessors,entry):
    seen,pending,returns,fatal=set(),[entry],set(),set()
    details=[]
    while pending:
        at=pending.pop()
        if at in seen: continue
        if body[at:at+2]==bytes.fromhex('faf4'):
            fatal.add(at)
            continue
        assert at in nodes and len(seen)<512
        seen.add(at)
        row=nodes[at]
        raw=bytes.fromhex(row['bytes'])
        effect=effects(raw)
        assert not effect['register_writes'] & {4}, 'RTC leaf changes RSP'
        assert not effect.get('stack_pointer_operand'), 'RTC leaf exposes a stack alias'
        assert 'push' not in effect and 'pop' not in effect
        assert effect.get('control')!='call', 'RTC leaf has an unqualified nested call'
        assert all(base in [7,12,13] for base,_,_ in effect['memory_writes']), 'RTC leaf changes saved stack storage'
        if 12 in effect['register_writes']:
            assert at==entry and raw==bytes.fromhex('654c8b2425a0030000')
        if effect.get('control')=='return': returns.add(at)
        else: pending.extend(successors[at])
        details.append(dict(offset=at,register_writes=sorted(effect['register_writes']),
            memory_reads=effect['memory_reads'],memory_writes=effect['memory_writes'],
            repeated=effect.get('repeated'),io=effect.get('io')))
    assert len(returns)==2
    assert body[entry:entry+9]==bytes.fromhex('654c8b2425a0030000')
    assert not roots & (seen-{entry})
    assert all(predecessors[at] <= seen for at in seen-{entry}), 'RTC alternate entry'
    # Every DI write belongs to an exact bounded staging wipe. The only
    # R13-based store belongs to the seven-word BCD staging loop below.
    wipes=[]
    wipe_offsets=set()
    for at in sorted(seen):
        if body[at:at+20]==bytes.fromhex('4c89e74881c78005000048b80000000000000000'):
            wipe=Slice(body,nodes,at)
            wipe.wipe()
            wipes.append([at,wipe.at])
            wipe_offsets.update(p for p in seen if at<=p<wipe.at)
    assert len(wipes)==3
    bcd_prefix=bytes.fromhex('4d89e54981c58005000049be0700000000000000')
    bcd_start=body.find(bcd_prefix,entry,max(returns))
    assert bcd_start>=entry and body.find(bcd_prefix,bcd_start+1,max(returns))==-1
    bcd=Slice(body,nodes,bcd_start)
    bcd.take(bcd_prefix.hex())
    loop=bcd.at
    bcd.take('498b85000000004989c24981e20f0000004981fa09000000')
    error=bcd.branch('0f87')
    bcd.take('48c1e8044881f809000000')
    bcd.branch('0f87',error)
    bcd.take('4869c00a0000004c01d0')
    bcd_store=bcd.at
    bcd.take('498985000000004981c5080000004981c6ffffffff4981fe00000000')
    bcd.branch('0f85',loop)
    bcd_nodes=[nodes[at] for at in sorted(seen) if bcd_start<=at<bcd.at]
    assert sum(row['size'] for row in bcd_nodes)==bcd.at-bcd_start
    for left,right in zip(bcd_nodes,bcd_nodes[1:]):
        allowed={left['offset']}|({bcd.at-6} if right['offset']==loop else set())
        assert predecessors[right['offset']]==allowed and right['offset'] not in roots
    for row in details:
        if 7 in row['register_writes']: assert row['offset'] in wipe_offsets
        for base,offset,width in row['memory_writes']:
            assert (base==7 and row['offset'] in wipe_offsets) or (
                base==12 and width==8 and 1408<=offset and offset+width<=1472) or (
                base==13 and row['offset']==bcd_store and offset==0 and width==8)
    return dict(entry=entry,instructions=[nodes[at] for at in sorted(seen)],effects=sorted(details,key=lambda r:r['offset']),
        normal_returns=sorted(returns),fatal_boundaries=sorted(fatal),stack_bytes=8,
        reloaded_table=dict(register=12,captured_gs_field=928),
        staging_wipes=wipes,bcd_staging_loop=dict(start=bcd_start,body=loop,end=bcd.at,words=7),
        scope='Closed stackless normal-return RTC leaf effects; R12 reloads the captured table, and stack/register preservation still requires table/stack nonalias and captured-entry identity. Exposed table/staging/calendar/I/O effects grant no owners or fault/lifetime proof')


def clock_contract(body,nodes,roots,successors,predecessors,entry):
    s=Instructions(body,nodes,entry)
    s.take('4881fc00801000')
    s.fatal('0f82')
    s.take('4881fc00001800')
    s.fatal('0f87')
    for raw in ['9c','58','4881e000020000','4881f800000000']: s.take(raw)
    s.fatal('0f85')
    pushes=['53','55','57','56','52','4150','4151','4152','4153','4155','4156','4157']
    pops=['415f','415e','415d','415b','415a','4159','4158','5a','5e','5f','5d','5b']
    for raw in pushes: s.take(raw)
    leaf=leaf_contract(body,nodes,roots,successors,predecessors,call_target(s))
    for raw in pops: s.take(raw)
    s.take('c3')
    for left,right in zip(s.rows,s.rows[1:]):
        assert right['offset'] not in roots and predecessors[right['offset']]=={left['offset']}
    assert all(4 not in row['register_writes'] for row in leaf['effects'])
    return dict(entry=entry,instructions=s.rows,leaf=leaf,
        saved_registers=[3,5,7,6,2,8,9,10,11,13,14,15],spill_bytes=96,
        maximum_selected_stack_bytes=8+96+leaf['stack_bytes'],normal_return_stack_delta=8,
        retired_spill_erasure_verified=False,
        scope='Exact twelve-register save/reverse restore around the closed RTC leaf, RSP/IF checks and selected maximum stack cost. R12/captured table identity, actual stack/nonalias, retired spill cleanup, fatal/exception and continuous lifetime remain separate')


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
    assert graph['extent_inventory_sha256']==digest(census_bytes) and origins['process_graph_sha256']==digest(graph_bytes)
    nodes,roots,successors,predecessors=check_graph(graph,kernel,symbol,body)
    cases=[]
    clocks={}
    pages={}
    for name,label in [('volatile-write-input','file-write-end'),('volatile-pwrite-input','file-pwrite-end')]:
        matches=[r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches)==1 and matches[0]['target']=='runtime-readable-extent'
        call=matches[0]['offset']
        prelude=clipped_prelude()
        start=call-len(prelude)
        assert body[start:call]==prelude
        pre=[r for at,r in sorted(nodes.items()) if start<=at<call]
        assert sum(r['size'] for r in pre)==len(prelude)
        allowed={r['offset'] for r in pre}
        assert all(r['offset'] not in roots and predecessors[r['offset']]<=allowed for r in pre[1:]+[nodes[call]])
        gate,success=call+14,call+20
        assert body[call+5:gate]==bytes.fromhex('48c74424f800000000')
        assert body[gate:gate+2]==bytes.fromhex('0f82')
        assert predecessors[call+5]=={call} and predecessors[gate]=={call+5}
        assert not roots & {call+5,gate}
        s=Slice(body,nodes,success)
        s.take('4989da')
        page_entry=call_target(s)
        pages[page_entry]=page_lookup_contract(body,nodes,page_entry)
        resident=s.branch('0f83')
        s.take('4d89e34981c31830040049b9080000000000000048b90000000000000000')
        scan=s.at
        s.take('410fb683000000004881f801000000')
        s.branch('0f87',fatal=True)
        s.take('4881f800000000')
        advance=s.branch('0f85')
        s.take('4881c101000000')
        assert s.at==advance
        s.take('4981c3010000004981c1ffffffff4981f900000000')
        s.branch('0f85',scan)
        s.take('4881f900000000')
        no_space=s.branch('0f84')
        s.take('48be000000000000000048bf0000000000000000')
        at=s.at
        raw=body[at:at+5]
        assert raw[0]==0xe9
        clock=at+5+struct.unpack_from('<i',raw,1)[0]
        s.take(raw.hex())
        assert s.at==resident
        s.take('4d89e14981c1183004004901c9410fb681000000004881f801000000')
        s.branch('0f85',fatal=True)
        s.take('4889cf48c1e70c4c01e74881c7004004004c89de')
        assert s.at==clock
        clock_entry=call_target(s)
        clocks[clock_entry]=clock_contract(body,nodes,roots,successors,predecessors,clock_entry)
        clock_error=s.branch('0f82')
        s.take('4989c5574989f34c89c64c89e74881c7800500004889e9fc')
        copy=s.at
        s.take('f3a4')
        end=s.at
        s.take('5f')
        local=[r for at,r in sorted(nodes.items()) if success<=at<end]
        assert sum(r['size'] for r in local)==end-success
        local_offsets={r['offset'] for r in local}
        assert not roots & local_offsets
        assert all(predecessors[at]<=local_offsets|{gate} for at in local_offsets), 'file input alternate entry'
        assert copy not in reachable(roots,successors,(gate,success)), 'file input bypasses admission'
        assert no_space not in local_offsets and clock_error not in local_offsets
        origin=next(r for r in origins['accesses'] if r['offset']==copy)
        assert origin['bytes']=='f3a4'
        cases.append(dict(name=name,call_offset=call,carry_gate=gate,success_offset=success,
            prelude=pre,local_instructions=local,copy_offset=copy,copy_size=2,copy_width=1,
            admitted_direction='read',candidate_register=8,count_register=5,maximum_bytes=256,
            copy_live_stack_words=1,retain_transfer_records=True,
            counterpart=dict(kind='process-table-staging',base_register=12,offset=1408,capacity=256),
            page_lookup_entry=page_entry,clock_entry=clock_entry,resident_entry=resident,
            capacity_scan=scan,no_space_exit=no_space,clock_error_exit=clock_error,
            byte_origins=origin['byte_origins'],post_copy_pop=nodes[end]))
    report=dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=cases,transfer_regions=2,copy_instructions=2,
        copy_groups=[dict(copy_offset=r['copy_offset'],cases=[r['name']],admission_edges=[[r['carry_gate'],r['success_offset']]]) for r in cases],
        page_lookup_contracts=list(pages.values()),clock_contracts=list(clocks.values()),
        scope='Conditional first user-to-staging REP for write/pwrite: clipped byte count, saved RBP/R8 through exact page/clock preparation and normal-return frames, captured-table reload condition, one live DI word and explicit CLD. Actual table/staging/calendar/stack owners and nonalias, retired spill cleanup, continuous lifetime and complete helper/entry/callee/fatal/exception/post-copy/publication/production remain open; this report alone does not extend the published index')
    (build/'file-input-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,regions=2,copies=2,clock_leaves=len(clocks),complete=False)))


if __name__=='__main__': audit(Path(sys.argv[1]).resolve())
