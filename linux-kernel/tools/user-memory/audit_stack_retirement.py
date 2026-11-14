"""Closed normal-return retirement of selected clock/random private stack words.

TOPAL-SYSTEMS-USER-EXTENT-001. This proves emitted layout and zero stores,
conditional on actual private stack ownership/nonalias and stable execution.
It does not establish those owners or exception/terminal reclamation.
"""
import json
from pathlib import Path
import struct
import sys

from audit_file_inputs import clock_contract
from audit_fixed_transfers import check_graph
from audit_pipe_fixed_outputs import Instructions
from inspect_call_inventory import Elf,digest,verify_calls
from retired_stack_words import take


def closed_block(body,nodes,roots,predecessors,start,words,previous):
    s=Instructions(body,nodes,start)
    block=take(s,words)
    last=previous
    for row in block['instructions']:
        at=row['offset']
        assert at not in roots and predecessors[at]=={last}, 'stack retirement alternate entry'
        last=at
    assert s.at in nodes and s.at not in roots and predecessors[s.at]=={last}, 'stack retirement bypass'
    block['previous_offset']=previous
    block['next_instruction']=nodes[s.at]
    return block


def audit(build):
    raw={name:(build/name).read_bytes() for name in [
        'extent-call-inventory.json','process-access-inventory.json',
        'process-access-origins.json','file-input-audit.json','random-output-audit.json']}
    census,graph,origins,files,random=[json.loads(value) for value in raw.values()]
    provider,kernel=Elf(build/'provider.o'),Elf(build/'kernel.elf')
    verify_calls(provider,census)
    symbol=verify_calls(kernel,census)
    _,body=kernel.function(census['function'])
    for report in [graph,origins,files,random]:
        assert report['passed'] and report['function']==census['function']
        assert report['provider_sha256']==digest(provider.data)
        assert report['kernel_sha256']==digest(kernel.data)
        assert report['linked_function_sha256']==digest(body)
    assert graph['extent_inventory_sha256']==digest(raw['extent-call-inventory.json'])
    assert origins['process_graph_sha256']==digest(raw['process-access-inventory.json'])
    for report in [files,random]:
        assert report['graph_sha256']==digest(raw['process-access-inventory.json'])
        assert report['origin_join_sha256']==digest(raw['process-access-origins.json'])
        assert report['census_sha256']==digest(raw['extent-call-inventory.json'])
    nodes,roots,successors,predecessors=check_graph(graph,kernel,symbol,body)
    assert len(files['clock_contracts'])==1
    clock=clock_contract(body,nodes,roots,successors,predecessors,files['clock_contracts'][0]['entry'])
    assert json.loads(json.dumps(clock))==files['clock_contracts'][0]
    blocks=[]
    for name,key in [('clock-leaf-return','leaf_return_word_retirement'),('clock-saved-registers','spill_retirement')]:
        row=clock[key]
        previous=next(r['offset'] for r in clock['instructions'] if r['offset']+r['size']==row['start'])
        blocks.append(dict(name=name,**closed_block(body,nodes,roots,predecessors,
            row['start'],row['words'],previous)))
    # Scan every byte as well as graph instructions: no omitted direct call to
    # this wrapper can silently inherit a caller's return-word cleanup claim.
    calls=[at for at in range(len(body)-4) if body[at]==0xe8 and
        at+5+struct.unpack_from('<i',body,at+1)[0]==clock['entry']]
    assert len(calls)==9
    for at in calls:
        assert nodes[at]['bytes']==body[at:at+5].hex() and nodes[at]['size']==5
        block=closed_block(body,nodes,roots,predecessors,at+5,1,at)
        assert block['next_instruction']['bytes'].startswith('0f82'), 'clock carry gate'
        blocks.append(dict(name='clock-caller-'+str(at),**block))
    assert len(random['cases'])==1
    case=random['cases'][0]
    previous=case['extract_call']
    assert nodes[previous]['bytes'].startswith('e8')
    for name,key in [('random-extract-return','extract_return_word_retirement'),('random-saved-registers','caller_spill_retirement')]:
        row=case[key]
        if key=='caller_spill_retirement':
            s=Instructions(body,nodes,case['extract_return_word_retirement']['end'])
            for pop in ['5e','5f','59','415b']:
                previous=s.at
                s.take(pop)
            assert s.at==row['start']
        block=closed_block(body,nodes,roots,predecessors,row['start'],row['words'],previous)
        assert {k:block[k] for k in row}==row
        blocks.append(dict(name=name,**block))
    assert len(blocks)==13 and sum(r['words'] for r in blocks)==27
    report=dict(passed=True,complete=False,function=census['function'],
        kernel_sha256=digest(kernel.data),provider_sha256=digest(provider.data),
        linked_function_sha256=digest(body),graph_sha256=digest(raw['process-access-inventory.json']),
        inputs={name:digest(value) for name,value in raw.items()},blocks=blocks,
        clock_direct_calls=calls,zero_store_instructions=27,added_code_bytes=324,
        maximum_clock_stack_bytes=112,maximum_random_stack_bytes=168,
        scope='Closed selected normal-return cleanup blocks and all nine direct process clock callers. Zero stores preserve all GPRs, RSP and flags and touch only their exact already-popped words. Actual owners/nonalias, continuous lifetime, complete caller/entry/helper/fatal/exception effects and production qualification remain open')
    (build/'stack-retirement-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,blocks=len(blocks),zero_stores=27,complete=False)))


if __name__=='__main__': audit(Path(sys.argv[1]).resolve())
