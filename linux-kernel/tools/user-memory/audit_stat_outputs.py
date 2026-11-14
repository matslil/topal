"""Conditional shared stat-output and nested saved-register contracts.

TOPAL-SYSTEMS-USER-EXTENT-001. Numeric/private expressions and byte consistency
do not grant actual owners, nonaliasing, continuous lifetime or whole callers.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph,reachable
from audit_pipe_fixed_outputs import Instructions,lookup_contract
from audit_staging_copies import Slice
from inspect_call_inventory import Elf,digest,verify_calls
from stat_instruction_effects import effects


def closure(entry,body,nodes,successors,stop=None):
    seen,pending,calls,fatal,returns = set(),[entry],set(),set(),set()
    while pending:
        at = pending.pop()
        if at in seen or at == stop: continue
        assert len(seen) < 2048 and at in nodes
        if body[at:at+2] == bytes.fromhex('fa f4'):
            fatal.add(at)
            continue
        seen.add(at)
        row = nodes[at]
        raw = bytes.fromhex(row['bytes'])
        effect = effects(raw)
        if effect.get('control') == 'return':
            returns.add(at)
        elif effect.get('control') == 'call':
            target = at+5+struct.unpack_from('<i',raw,1)[0]
            assert target in successors[at] and at+5 in successors[at]
            calls.add(target)
            pending.append(at+5)
        else:
            pending.extend(successors[at])
    return dict(entry=entry,instructions=[nodes[at] for at in sorted(seen)],
                callees=sorted(calls),fatal_boundaries=sorted(fatal),returns=sorted(returns))


def metadata_contract(summary,body,nodes,roots,successors,predecessors):
    entry = summary['entry']
    assert not summary['callees'] and not summary['fatal_boundaries']
    assert len(summary['returns']) == 2
    normal,last = summary['returns']
    assert last == normal+11
    push = ['51','52','56','57','4150','4151','4152','4153']
    pop = ['415b','415a','4159','4158','5f','5e','5a','59']
    p = Instructions(body,nodes,entry)
    for raw in push: p.take(raw)
    push_end = p.at
    p.take('4989f8')
    suffix = bytes.fromhex(''.join(pop))
    restore = body.find(suffix,p.at,normal)
    assert restore >= p.at and body.find(suffix,restore+1,normal) == -1
    s = Instructions(body,nodes,restore)
    for raw in pop: s.take(raw)
    pop_end = s.at
    stores = []
    for offset in range(-64,0,8):
        stores.append(s.at)
        s.take('48c78424'+struct.pack('<i',offset).hex()+'00000000')
    for offset in range(-64,0,8):
        s.take('4881bc24'+struct.pack('<i',offset).hex()+'00000000')
        raw = body[s.at:s.at+6]
        assert raw[:2] == bytes.fromhex('0f85')
        assert s.at+6+struct.unpack_from('<i',raw,2)[0] == normal+1
        s.take(raw.hex())
    assert s.at == normal
    s.take('c3')
    s.take('48b80000000000000000')
    s.take('c3')
    assert s.at == last+1
    local = {row['offset'] for row in summary['instructions']}
    assert local == {at for at in nodes if entry <= at < s.at}
    assert not roots & (local-{entry})
    assert all(predecessors[at] <= local for at in local-{entry}), 'metadata alternate entry'
    assert all(successors[at] <= local for at in local)
    prefix = [r for r in summary['instructions'] if entry <= r['offset'] <= push_end]
    for left,right in zip(prefix,prefix[1:]):
        assert predecessors[right['offset']] == {left['offset']}
    restoring = [r for r in summary['instructions'] if restore <= r['offset'] < pop_end]
    for left,right in zip(restoring,restoring[1:]):
        assert predecessors[right['offset']] == {left['offset']}
    without_restore = {at:targets-{restore} for at,targets in successors.items() if at in local}
    assert not set(summary['returns']) & reachable({entry},without_restore), 'metadata return bypasses restoration'
    body_effects = []
    for row in summary['instructions']:
        at = row['offset']
        effect = effects(bytes.fromhex(row['bytes']))
        if push_end <= at < restore:
            assert not effect['register_writes'] & {3,4,5,12,13,14,15}
            assert not effect['memory_writes'], 'metadata body changes saved storage'
            assert 'push' not in effect and 'pop' not in effect and effect.get('control') not in ['return','call']
            body_effects.append(dict(offset=at,register_writes=sorted(effect['register_writes']),
                memory_reads=effect['memory_reads'],repeated=effect.get('repeated')))
        elif at >= pop_end:
            assert not effect['register_writes'] & {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}
            assert not effect['memory_writes'] or at in stores
    return dict(kind='saved-metadata-validator',entry=entry,instructions=summary['instructions'],
        normal_returns=summary['returns'],restore_offset=restore,saved_registers=[1,2,6,7,8,9,10,11],
        preserved_registers=[r for r in range(1,16) if r != 4],normal_return_stack_delta=8,
        stack_bytes=72,spill_bytes=64,
        restored_spill_erasure_offsets=stores,body_effects=body_effects,
        scope='Closed normal-return saved-register/DF shape and 64-byte spill retirement; actual stack/metadata owners, memory fault/exception behavior and continuous lifetime remain separate')


def wrapper_contract(summary,inner,body,nodes,roots,predecessors):
    assert len(summary['callees']) == 1 and len(summary['returns']) == 1
    entry = summary['entry']
    s = Instructions(body,nodes,entry)
    for raw in ['57','4c89e7','4881c700300400']: s.take(raw)
    raw = body[s.at:s.at+5]
    assert raw[0] == 0xe8 and s.at+5+struct.unpack_from('<i',raw,1)[0] == inner['entry']
    s.take(raw.hex())
    s.take('5f')
    for offset in [-16,-8]: s.take('48c78424'+struct.pack('<i',offset).hex()+'00000000')
    fatal = []
    for offset in [-16,-8]:
        s.take('4881bc24'+struct.pack('<i',offset).hex()+'00000000')
        fatal.append(s.fatal('0f85'))
    s.take('4881f801000000')
    fatal.append(s.fatal('0f85'))
    s.take('c3')
    assert s.rows == summary['instructions'] and set(fatal) == set(summary['fatal_boundaries'])
    assert summary['returns'] == [s.at-1]
    assert not roots & {r['offset'] for r in s.rows[1:]}
    for left,right in zip(s.rows,s.rows[1:]):
        assert predecessors[right['offset']] == {left['offset']}
    return dict(kind='metadata-check-wrapper',entry=entry,instructions=s.rows,
        preserved_registers=[r for r in range(1,16) if r != 4],normal_return_stack_delta=8,
        stack_bytes=8+8+inner['stack_bytes'],
        scope='Exact DI save/restore, nested Boolean check and two retired wrapper words; actual stack owner, fatal and exception contracts remain separate')


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
    calls = []
    for label in ['fd-stat','namespace-stat-object']:
        matches = [r for r in census['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1 and matches[0]['target'] == 'runtime-writable-extent'
        calls.append(matches[0]['offset'])
    publication = bytes.fromhex('4c89e64881c6800500004c89c748b99000000000000000f3a4')
    start = body.find(publication,calls[0]+20,calls[0]+1024)
    assert start >= 0 and body.find(publication,start+1,calls[0]+1024) == -1
    copy = start+len(publication)-2
    preparations = [closure(call+20,body,nodes,successors,copy) for call in calls]
    summaries,contracts,active = {},{},set()
    def qualify(entry):
        assert entry not in active, 'recursive stat callee'
        if entry in contracts: return contracts[entry]
        active.add(entry)
        summary = closure(entry,body,nodes,successors)
        summaries[entry] = summary
        assert summary['returns']
        nested = [qualify(target) for target in summary['callees']]
        if body[entry:entry+4] == bytes.fromhex('51525657'):
            contract = metadata_contract(summary,body,nodes,roots,successors,predecessors)
        elif body[entry:entry+4] == bytes.fromhex('574c89e7'):
            assert len(nested) == 1 and nested[0]['kind'] == 'saved-metadata-validator'
            contract = wrapper_contract(summary,nested[0],body,nodes,roots,predecessors)
        else:
            other = []
            assert all(r['kind'] == 'metadata-check-wrapper' for r in nested)
            for row in summary['instructions']:
                effect = effects(bytes.fromhex(row['bytes']))
                assert not effect['register_writes'] - {0,1,10,11}
                assert 'push' not in effect and 'pop' not in effect
                assert not effect.get('repeated')
                assert all(base == 12 and 1408 <= offset and offset+width <= 1408+144
                    for base,offset,width in effect['memory_writes'])
                if effect['memory_reads'] or effect['memory_writes']:
                    other.append(dict(offset=row['offset'],reads=effect['memory_reads'],writes=effect['memory_writes']))
            contract = dict(kind='frameless-private-preparation',entry=entry,instructions=summary['instructions'],
                preserved_registers=[8,9,12],normal_return_stack_delta=8,
                stack_bytes=8+max([c['stack_bytes'] for c in nested],default=0),
                normal_returns=summary['returns'],other_memory_effects=other,
                scope='Explicit register/DF effects plus separately checked nested returns; exposed metadata reads/staging writes grant no owners')
        active.remove(entry)
        contracts[entry] = contract
        return contract
    for prep in preparations:
        assert not prep['returns']
        for target in prep['callees']: qualify(target)
    assert len(contracts) == 6
    # Reuse the exact reviewed pipe leaf, in addition to the generic effects.
    pipe = [entry for entry in contracts if body[entry:entry+7] == bytes.fromhex('4d8b9e20000000')]
    assert len(pipe) == 1
    lookup_contract(body,nodes,pipe[0],roots,successors,predecessors)
    local = {r['offset'] for prep in preparations for r in prep['instructions']}
    gates = {call+14 for call in calls}
    assert not roots & local
    assert all(predecessors[at] <= local|gates for at in local), 'stat alternate preparation entry'
    results = []
    for number,(call,prep) in enumerate(zip(calls,preparations)):
        prelude = ['4989f0','49b99000000000000000'] if number == 0 else ['654c8b042548010000','49b99000000000000000']
        at = call-sum(len(bytes.fromhex(raw)) for raw in prelude)
        s = Instructions(body,nodes,at)
        for raw in prelude: s.take(raw)
        assert s.at == call
        previous = at
        for row in s.rows[1:]+[nodes[call]]:
            assert row['offset'] not in roots and predecessors[row['offset']] == {previous}
            previous = row['offset']
        gate,success = call+14,call+20
        assert predecessors[call+5] == {call} and predecessors[gate] == {call+5}
        assert not roots & {call+5,gate}
        assert body[call+5:gate] == bytes.fromhex('48c74424f800000000')
        assert body[gate:gate+2] == bytes.fromhex('0f82') and success in successors[gate]
        wipe = Slice(body,nodes,success)
        wipe.wipe()
        for row in prep['instructions']:
            effect = effects(bytes.fromhex(row['bytes']))
            assert not effect['register_writes'] & {3,4,5,8,9,12,13,14}
            assert 'push' not in effect and 'pop' not in effect
            if effect.get('repeated'):
                assert effect['repeated'] in ['fill','scan'] and success <= row['offset'] < wipe.at
            else:
                assert all(base == 12 and 1408 <= offset and offset+width <= 1408+144
                    for base,offset,width in effect['memory_writes'])
        results.append(dict(name='descriptor-stat-output' if number == 0 else 'namespace-stat-output',
            call_offset=call,carry_gate=gate,success_offset=success,prelude=s.rows,
            candidate_register=8,admitted_direction='write',admitted_bytes=144,
            copy_offset=copy,copy_size=2,copy_width=1,user_endpoint_register=7,
            counterpart=dict(base_register=12,offset=1408,capacity=256),
            local_instructions=prep['instructions'],callee_entries=prep['callees'],
            maximum_nested_callee_stack_bytes=max(contracts[t]['stack_bytes'] for t in prep['callees'])))
    excluded = {(r['carry_gate'],r['success_offset']) for r in results}
    remaining = {source:targets-{target for a,target in excluded if a == source} for source,targets in successors.items()}
    assert copy not in reachable(roots,remaining), 'uncovered stat copy caller'
    publication_rows = [nodes[at] for at in sorted(nodes) if start <= at <= copy]
    assert sum(row['size'] for row in publication_rows) == len(publication)
    for left,right in zip(publication_rows,publication_rows[1:]):
        assert predecessors[right['offset']] == {left['offset']} and right['offset'] not in roots
    origin = next(row for row in origins['accesses'] if row['offset'] == copy)
    assert origin['bytes'] == 'f3a4' and origin['size'] == 2
    report = dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=results,transfer_regions=2,copy_instructions=1,byte_origins=origin['byte_origins'],
        copy_groups=[dict(copy_offset=copy,cases=[r['name'] for r in results],
            admission_edges=sorted(excluded),union_success_edges_dominate=True)],
        callee_contracts=[contracts[at] for at in sorted(contracts)],
        scope='Conditional shared 144-byte stat output only: exact admission/publication/wipe shapes, candidate/count/DF and restricted preparation effects, nested normal-return save/restore and graph entry/union-success checks. Additional reads/writes/stack are exposed, not granted ownership. Complete helper/entry/callee/fatal/exception/owner/nonalias/lifetime/native/all-caller/production qualification remains open; this report alone does not extend the published index')
    (build/'stat-output-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['cases','copy_groups','callee_contracts','byte_origins']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
