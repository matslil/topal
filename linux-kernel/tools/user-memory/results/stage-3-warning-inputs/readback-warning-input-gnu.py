"""Independent warning selector/register/scalar readback of saved GNU decoding."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import sys

build,gnu,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
audit_bytes=(build/'warning-input-audit.json').read_bytes()
audit=json.loads(audit_bytes)
graph_bytes=(build/'process-access-inventory.json').read_bytes()
graph=json.loads(graph_bytes)
report=json.loads((gnu/'gnu-verification.json').read_text())
transcript=(gnu/'gnu-decoder-transcript.json').read_bytes()
assert report['passed'] and report['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
assert report['graph_sha256']==digest(graph_bytes) and report['decoder_transcript_sha256']==digest(transcript)
decoded={r['offset']:re.sub(r'\s+',' ',r['assembly'].split('#',1)[0].split('<',1)[0].strip()) for r in json.loads(transcript)}
case=audit['cases'][0]
pre=case['prelude']
assert decoded[pre[2]['offset']]=='mov %gs:0x268,%rax'
comparisons=[decoded[r['offset']] for r in pre if decoded[r['offset']].startswith('cmp $0x') and decoded[r['offset']].endswith(',%rax')]
assert comparisons[:3]==['cmp $0x30101,%rax','cmp $0x1030101,%rax','cmp $0x2030101,%rax']
assert [decoded[r['offset']] for r in pre if decoded[r['offset']].endswith(',%r9d')]==[
    'mov $0x1,%r9d','mov $0x4,%r9d','mov $0x28,%r9d']
assert [decoded[r['offset']] for r in pre[-4:]]==[
    'mov %gs:0x140,%rsi','mov %rsi,%rax','mov %rdx,%r9','mov %rax,%r8']
assert 'cmp %r9,%rdx' in [decoded[r['offset']] for r in pre]
for row in pre:
    mnemonic,_,operands=decoded[row['offset']].partition(' ')
    if mnemonic.startswith('j') or mnemonic in ['cmp','cmpb','cmpq','test','testb']: continue
    assert mnemonic in ['mov','movabs','lea','and','or'],decoded[row['offset']]
    destination=operands.rsplit(',',1)[-1]
    assert destination in ['%rax','%r8','%r9','%r9d','%r10','%r11','%rsi']
assert decoded[case['dispatch'][0]['offset']]=='mov %gs:0x268,%rax'
assert decoded[case['call_offset']].startswith('call ')
assert decoded[case['call_offset']+5]=='movq $0x0,-0x8(%rsp)'
assert decoded[case['carry_gate']].startswith('jb ')
pred=defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
checked={r['offset'] for r in pre+case['dispatch']}
checked.update([case['call_offset'],case['call_offset']+5,case['carry_gate']])
operands=[]
for variant in case['variants']:
    for i,row in enumerate(variant['access_instructions']):
        address=(hex(row['candidate_offset']) if row['candidate_offset'] else '')+'(%rsi)'
        destination='%r8d' if variant['admitted_bytes']==4 else '%r8b' if variant['admitted_bytes']==1 else '%r'+str(8+i)
        expected='mov '+address+','+destination
        assert decoded[row['offset']]==expected
        operands.append(dict(offset=row['offset'],assembly=expected,selector=variant['selector'],
            admitted_bytes=variant['admitted_bytes'],candidate_offset=row['candidate_offset'],width=row['width']))
        checked.add(row['offset'])
seen,pending=set(),[r['offset'] for r in case['user_accesses']]
edge=(case['carry_gate'],case['success_offset'])
while pending:
    at=pending.pop()
    if at in seen: continue
    assert at not in roots
    seen.add(at)
    pending.extend(p for p in pred[at] if (p,at)!=edge)
result=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_transcript_sha256=digest(transcript),selected_instructions=len(checked),
    exact_selector_count_candidate_setup_equal=True,preparation_has_no_rdx_or_selector_writes=True,
    scalar_operands=operands,reverse_admission_cut=dict(removed_edge=list(edge),predecessors=len(seen)),
    scope='Independent warning selector/count/candidate operands, no RDX/selector writes in the selected inline prelude, scalar addresses/widths and reverse success-edge cut using the prior identical-kernel GNU transcript. Not new decoding, native execution, helper preservation, actual owners/lifetime or complete warning/production qualification')
(output/'readback.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(checked),scalar_loads=len(operands),variants=3)))
