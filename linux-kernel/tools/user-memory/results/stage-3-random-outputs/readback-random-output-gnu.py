"""Independent GNU extraction effects, switched-frame operands and loop cuts."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from random_extract_effects import effects

build,gnu,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest=lambda raw:hashlib.sha256(raw).hexdigest()
audit_bytes=(build/'random-output-audit.json').read_bytes()
audit=json.loads(audit_bytes)
graph_bytes=(build/'process-access-inventory.json').read_bytes()
graph=json.loads(graph_bytes)
report=json.loads((gnu/'gnu-verification.json').read_text())
transcript=(gnu/'gnu-decoder-transcript.json').read_bytes()
assert report['passed'] and report['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
assert report['graph_sha256']==digest(graph_bytes) and report['decoder_transcript_sha256']==digest(transcript)
decoded={r['offset']:re.sub(r'\s+',' ',r['assembly'].split('#',1)[0].split('<',1)[0].strip()) for r in json.loads(transcript)}
registers={}
for index,(wide,narrow) in enumerate(zip(['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi'],
                                      ['eax','ecx','edx','ebx','esp','ebp','esi','edi'])):
    registers['%'+wide]=(index,8)
    registers['%'+narrow]=(index,4)
for index in range(8,16):
    registers['%r'+str(index)]=(index,8)
    registers['%r'+str(index)+'d']=(index,4)
def gnu_effects(text):
    mnemonic,_,operands=text.partition(' ')
    if mnemonic in ['je','jne','ret']: return set(),[],[],0
    assert mnemonic in ['mov','movl','add','sub','xor','or','rol','test','cmp'],text
    args=operands.split(',')
    assert len(args)==2,text
    destination=args[-1]
    reg=registers.get(destination)
    width=reg[1] if reg else registers.get(args[0],(None,4 if mnemonic=='movl' else None))[1]
    writes={reg[0]} if reg and mnemonic not in ['test','cmp'] else set()
    reads,stores=[],[]
    for i,arg in enumerate(args):
        if '(' not in arg: continue
        match=re.fullmatch(r'(-?0x[0-9a-f]+)?\(%(rsp|r10|r11)\)',arg)
        assert match and width in [4,8],text
        memory=(dict(rsp=4,r10=10,r11=11)[match[2]],int(match[1] or '0',16),width)
        if i==1 and mnemonic in ['mov','movl']: stores.append(memory)
        else: reads.append(memory)
    delta=0
    if destination=='%rsp':
        assert args[0]=='$0x80' and mnemonic in ['add','sub']
        delta=128 if mnemonic=='add' else -128
    return writes,reads,stores,delta
checked=set()
for row in audit['extract_contract']['instructions']:
    effect=effects(bytes.fromhex(row['bytes']))
    expected=(effect['register_writes'],effect['memory_reads'],effect['memory_writes'],effect.get('scratch_delta',0))
    assert gnu_effects(decoded[row['offset']])==expected,(row['offset'],decoded[row['offset']],expected)
    checked.add(row['offset'])
case=audit['cases'][0]
pre=[decoded[r['offset']] for r in case['prelude']]
assert pre[0]=='cmp $0x10,%rsi' and pre[1].startswith('ja ')
assert pre[2]=='test %rsi,%rsi' and pre[3].startswith('je ')
assert pre[-3:]==['mov %rdi,%rax','mov %rsi,%r9','mov %rax,%r8']
local=[decoded[r['offset']] for r in case['local_instructions']]
assert [s for s in local if s.startswith('push ')]==['push %r11','push %rcx','push %rdi','push %rsi']
assert [s for s in local if s.startswith('pop ')]==['pop %rsi','pop %rdi','pop %rcx','pop %r11']
for operand in ['mov %rsp,%gs:0x110','mov %gs:0x8,%rsp','mov %gs:0x110,%rsp','movq $0x0,%gs:0x110']:
    assert operand in local,operand
assert re.fullmatch(r'mov \(%r9,%r10(?:,1)?\),%al',decoded[case['load_offset']])
assert re.fullmatch(r'mov %al,\(%rdi,%r10(?:,1)?\)',decoded[case['start_offset']])
assert re.fullmatch(r'cmp \(%rdi,%r10(?:,1)?\),%al',decoded[case['readback_offset']])
checked.update(r['offset'] for r in case['prelude']+case['local_instructions'])
checked.update([case['call_offset'],case['call_offset']+5,case['carry_gate']])
pred=defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
cuts=[]
for name,edge in [('admission',(case['carry_gate'],case['success_offset'])),
    ('count-cap',(case['prelude'][1]['offset'],case['prelude'][2]['offset'])),
    ('positive-count',(case['prelude'][3]['offset'],case['prelude'][4]['offset']))]:
    seen,pending=set(),[r['offset'] for r in case['user_accesses']]
    while pending:
        at=pending.pop()
        if at in seen: continue
        assert at not in roots
        seen.add(at)
        pending.extend(p for p in pred[at] if (p,at)!=edge)
    cuts.append(dict(name=name,removed_edge=list(edge),predecessors=len(seen)))
result=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_transcript_sha256=digest(transcript),selected_instructions=len(checked),
    extract_effect_instructions=len(audit['extract_contract']['instructions']),
    exact_gnu_register_memory_stack_effects_equal=True,exact_count_candidate_stack_switch_and_loop_operands_equal=True,
    reverse_guard_cuts=cuts,
    scope='Independent register/memory/scratch effects for every selected extractor instruction, saved-word order, GS stack switch/restore, count/candidate and indexed byte-loop operands, and three reverse cuts using the preceding identical-kernel GNU transcript. Not new decoding, a second frame-dataflow algorithm, native owners/nonalias/lifetime/retired outer spills or production qualification')
(output/'readback.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(checked),extract_effects=result['extract_effect_instructions'],cuts=3)))
