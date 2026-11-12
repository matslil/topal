"""Independent GNU operands and complete predecessor closure of zero blocks."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import sys

build,gnu,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest=lambda raw:hashlib.sha256(raw).hexdigest()
audit_bytes=(build/'stack-retirement-audit.json').read_bytes()
audit=json.loads(audit_bytes)
graph_bytes=(build/'process-access-inventory.json').read_bytes()
graph=json.loads(graph_bytes)
transcript=(gnu/'gnu-decoder-transcript.json').read_bytes()
report=json.loads((gnu/'gnu-verification.json').read_text())
assert report['passed'] and report['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
assert report['graph_sha256']==audit['graph_sha256']==digest(graph_bytes)
assert report['decoder_transcript_sha256']==digest(transcript)
decoded={r['offset']:re.sub(r'\s+',' ',r['assembly'].split('#',1)[0].split('<',1)[0].strip()) for r in json.loads(transcript)}
pred=defaultdict(set)
for edge in graph['edges']:pred[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
blocks=[]
selected=set()
for block in audit['blocks']:
    previous=block['previous_offset']
    for index,row in enumerate(block['instructions'],1):
        at=row['offset']
        assert decoded[at]=='movq $0x0,-0x'+format(8*index,'x')+'(%rsp)',(at,decoded[at])
        assert pred[at]=={previous} and at not in roots
        previous=at
        selected.add(at)
    assert pred[block['end']]=={previous} and block['end'] not in roots
    assert block['relative_start']==-8*block['words'] and block['relative_end']==0
    blocks.append(dict(name=block['name'],stores=block['words'],
        negative_offsets=[-8*i for i in range(1,block['words']+1)],
        predecessor_chain_closed=True,register_writes=[],flags_unchanged=True))
assert len(blocks)==13 and len(selected)==27
result=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_transcript_sha256=digest(transcript),
    blocks=blocks,selected_instructions=len(selected),exact_zero_store_operands_equal=True,
    scope='Fresh complete GNU graph re-decode plus independent qword/immediate-zero/negative-RSP operand and predecessor-chain checks. MOV memory-immediate has no GPR/RSP/flag writes. Actual owners/nonalias, frame/lifetime, full caller and production qualification remain separate')
(output/'readback.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(passed=True,blocks=13,zero_stores=27)))
