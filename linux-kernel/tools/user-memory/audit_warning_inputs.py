"""Conditional variant-sensitive intervals for the three shell warning inputs.

TOPAL-SYSTEMS-USER-EXTENT-001;
TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001.
Shared helper/context preservation and entry-stable ownership remain obligations.
"""
import json
from pathlib import Path
import struct
import sys

from audit_fixed_transfers import check_graph, reachable, scalar
from audit_pipe_fixed_outputs import Instructions
from inspect_call_inventory import Elf, digest, verify_calls


VARIANTS=((0x00030101,4,[(0,4)]),(0x01030101,40,[(i,8) for i in range(0,40,8)]),
          (0x02030101,1,[(0,1)]))


def warning_variant(before,after,count):
    """Require the same retained selector and its exact admitted quantum."""
    assert before==after, 'warning selector changed across admission'
    matches=[(length,accesses) for state,length,accesses in VARIANTS if state==before]
    assert len(matches)==1, 'unqualified warning selector'
    length,accesses=matches[0]
    assert count==length, 'warning admission length disagrees with selector'
    return length,accesses


class Slice(Instructions):
    def branch(self,prefix,target=None):
        raw=bytes.fromhex(prefix)
        assert raw==b'\xe9' or len(raw)==2 and raw[0]==15 and 0x80<=raw[1]<=0x8f
        encoded=self.body[self.at:self.at+len(raw)+4]
        assert encoded[:len(raw)]==raw
        actual=self.at+len(encoded)+struct.unpack_from('<i',encoded,len(raw))[0]
        if target is not None: assert actual==target
        self.take(encoded.hex())
        return actual


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
    calls=[r for r in census['calls'] if 'static-process-shell-warning-candidate' in r['nearest_labels']]
    assert len(calls)==1 and calls[0]['target']=='runtime-readable-extent'
    call=calls[0]['offset']
    prefix=bytes.fromhex('65803c25a0010000010f85')
    start=body.rfind(prefix,max(0,call-1024),call)
    assert start>=0
    pre=Slice(body,nodes,start)
    pre.take('65803c25a001000001')
    pre.fatal('0f85')
    pre.take('65488b042568020000')
    pre.take('483d01010300')
    four=pre.branch('0f84')
    pre.take('483d01010301')
    forty=pre.branch('0f84')
    pre.take('483d01010302')
    pre.fatal('0f85')
    pre.take('41b901000000')
    join=pre.branch('e9')
    assert pre.at==four
    pre.take('41b904000000')
    pre.branch('e9',join)
    assert pre.at==forty
    pre.take('41b928000000')
    assert pre.at==join
    pre.take('4883ff02')
    pre.fatal('0f85')
    pre.take('4c39ca')
    pre.fatal('0f85')
    # Existing inline stack/disposition checks preserve RDX and GS:616.
    # Their numeric private reads are exposed, not converted into ownership.
    for raw,condition in [
        ('4881f900104000','0f82'),('4881f985c25800','0f83'),
        ('654c8b1425180000004d85d2','0f84'),('41f7c2ff0f0000','0f85'),
        ('0f20d8482500f0ffff4c39d0','0f85'),
        ('498b82182000004883e09f4d8d82004000004983c8074c39c0','0f85'),
        ('654c8b0425a80000004981f800104000','0f82'),('4981f885c25800','0f83'),
        ('6548813c25b000000000000004','0f85'),
        ('654c8b1c25e00000004981fb00104000','0f82'),('4981fb85c25800','0f83'),
        ('48b8ffffff7ffeffffff6548390425e8000000','0f85'),
        ('65803c25a101000001','0f85')]:
        # Match complete graph instruction boundaries even for joined encodings.
        end=pre.at+len(bytes.fromhex(raw))
        assert body[pre.at:end]==bytes.fromhex(raw)
        while pre.at<end: pre.take(nodes[pre.at]['bytes'])
        assert pre.at==end
        pre.fatal(condition)
    for raw in ['65488b342540010000','4889f0','4989d1','4989c0']: pre.take(raw)
    assert pre.at==call
    pre_offsets={r['offset'] for r in pre.rows}
    assert not roots & (pre_offsets-{start})
    assert all(predecessors[at]<=pre_offsets for at in pre_offsets-{start})
    assert predecessors[call]=={pre.rows[-1]['offset']} and call not in roots
    gate,success=call+14,call+20
    retirement=Slice(body,nodes,call+5)
    retirement.take('48c74424f800000000')
    assert retirement.at==gate
    failure=retirement.branch('0f82')
    assert predecessors[call+5]=={call} and predecessors[gate]=={call+5}
    assert not roots & {call+5,gate}
    dispatch=Slice(body,nodes,success)
    dispatch.take('65488b042568020000')
    dispatch.take('483d01010300')
    four=dispatch.branch('0f84')
    dispatch.take('483d01010301')
    forty=dispatch.branch('0f84')
    newline=dispatch.branch('e9')
    local={r['offset'] for r in dispatch.rows}
    assert not roots & local and all(predecessors[at]<=local|{gate} for at in local)
    by_origin={r['offset']:r for r in origins['accesses']}
    variants=[]
    for (state,length,expected),entry in zip(VARIANTS,[four,forty,newline]):
        assert warning_variant(state,state,length)==(length,expected)
        s=Slice(body,nodes,entry)
        if length==4: encodings=['448b06']
        elif length==40: encodings=['4c8b06','4c8b4e08','4c8b5610','4c8b5e18','4c8b6620']
        else: encodings=['448a06']
        accesses=[]
        for raw in encodings:
            at=s.at
            s.take(raw)
            action,offset,width,base,destination=scalar(bytes.fromhex(raw))
            assert action=='read' and base==6 and destination!=6
            assert 0<=offset and offset+width<=length
            origin=by_origin[at]
            assert origin['bytes']==raw and origin['size']==len(bytes.fromhex(raw))
            accesses.append(dict(offset=at,action=action,candidate_offset=offset,width=width,
                base_register=base,bytes=raw,byte_origins=origin['byte_origins']))
        assert [(r['candidate_offset'],r['width']) for r in accesses]==expected
        region={r['offset'] for r in s.rows}
        assert not roots & region and all(predecessors[at]<=region|local for at in region)
        assert not region & reachable(roots,successors,(gate,success))
        variants.append(dict(selector=state,admitted_bytes=length,entry=entry,
            local_end=s.at,access_instructions=accesses,local_instructions=s.rows))
    report=dict(passed=True,complete=False,function=census['function'],kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),linked_function_sha256=digest(body),
        census_sha256=digest(census_bytes),graph_sha256=digest(graph_bytes),origin_join_sha256=digest(origin_bytes),
        cases=[dict(name='terminal-warning-input',call_offset=call,carry_gate=gate,success_offset=success,
            failure_target=failure,candidate_register=6,admission_candidate_register=8,count_register=2,
            admitted_direction='read',selector_gs_offset=616,selector_bytes=8,
            selector_stability_required=True,prelude=pre.rows,dispatch=dispatch.rows,variants=variants,
            user_accesses=[r for v in variants for r in v['access_instructions']])],
        transfer_regions=1,variants=3,scalar_access_instructions=7,
        scope='Conditional warning count/selector binding, exact inline preparation preserving RDX and selector, captured candidate setup, helper result/retirement gate, post-helper selector dispatch and seven bounded scalar loads. Requires shared helper preservation of RSI/RDX and retained GS:616, actual private/input owners/nonalias and continuous entry/mapping stability. No complete output/staging-retirement/caller/fatal/exception or production qualification; this report alone does not extend the published index')
    (build/'warning-input-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,regions=1,variants=3,user_instructions=7,complete=False)))


if __name__=='__main__': audit(Path(sys.argv[1]).resolve())
