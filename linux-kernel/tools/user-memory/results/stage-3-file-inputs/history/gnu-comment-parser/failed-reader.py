"""Independent saved GNU RTC effects, saved-frame order and input-copy cuts."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from file_input_effects import effects

build,gnu,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
audit_bytes=(build/'file-input-audit.json').read_bytes()
audit=json.loads(audit_bytes)
graph_bytes=(build/'process-access-inventory.json').read_bytes()
graph=json.loads(graph_bytes)
gnu_report=json.loads((gnu/'gnu-verification.json').read_text())
transcript=(gnu/'gnu-decoder-transcript.json').read_bytes()
assert gnu_report['passed'] and gnu_report['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
assert gnu_report['graph_sha256']==digest(graph_bytes) and gnu_report['decoder_transcript_sha256']==digest(transcript)
decoded={r['offset']:re.sub(r'\s+',' ',r['assembly'].split('<',1)[0].strip()) for r in json.loads(transcript)}
registers=['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi']+['r'+str(i) for i in range(8,16)]
aliases={name:i for i,name in enumerate(registers)}
aliases.update(eax=0,ax=0,al=0,ecx=1,edx=2,ebx=3,esp=4,ebp=5,esi=6,edi=7)
def register(text):
    assert text.startswith('%') and text[1:] in aliases,text
    return aliases[text[1:]]
def gnu_effects(text):
    mnemonic,_,operands=text.partition(' ')
    if mnemonic in ['rep','repz']:
        action,_,arguments=operands.partition(' ')
        assert action in ['stos','scas'] and '(%rdi)' in arguments
        return {1,7},(1 if action=='scas' else 0),(1 if action=='stos' else 0)
    if mnemonic in ['ret','cld','clc','stc'] or mnemonic.startswith('j'): return set(),0,0
    args=operands.split(',')
    reads=sum('(' in a or '%gs:' in a for a in args)
    if mnemonic in ['cmp','cmpb']: return set(),reads,0
    if mnemonic=='out':
        assert args==['%al','$0x70']
        return set(),0,0
    if mnemonic=='in':
        assert args==['$0x71','%al']
        return {0},0,0
    if mnemonic=='lea': return {register(args[-1])},0,0
    assert mnemonic in ['mov','movb','movabs','movzbl','add','sub','and','shr','shl','imul'],text
    if args[-1].startswith('%') and '(' not in args[-1]: return {register(args[-1])},reads,0
    assert mnemonic in ['mov','movb'] and '(' in args[-1],text
    return set(),reads-1,1
checked=set()
for clock in audit['clock_contracts']:
    leaf=clock['leaf']
    for row in leaf['instructions']:
        at=row['offset']
        effect=effects(bytes.fromhex(row['bytes']))
        assert gnu_effects(decoded[at])==(effect['register_writes'],len(effect['memory_reads']),len(effect['memory_writes'])),(at,decoded[at],effect)
        checked.add(at)
    pushes=clock['instructions'][9:21]
    pops=clock['instructions'][22:34]
    names=['rbx','rbp','rdi','rsi','rdx','r8','r9','r10','r11','r13','r14','r15']
    assert [decoded[r['offset']] for r in pushes]==['push %'+name for name in names]
    assert [decoded[r['offset']] for r in pops]==['pop %'+name for name in reversed(names)]
    assert decoded[clock['instructions'][4]['offset']] in ['pushf','pushfq']
    assert decoded[clock['instructions'][5]['offset']]=='pop %rax'
    checked.update(r['offset'] for r in clock['instructions'])
pred=defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
cuts=[]
for case in audit['cases']:
    rows=case['local_instructions']
    # The complete saved word and input/counterpart/count setup is checked
    # without depending on GNU's optional segment spelling for REP MOVSB.
    start=next(i for i,r in enumerate(rows) if r['bytes']=='57')
    prefix=[decoded[r['offset']] for r in rows[start:start+7]]
    assert prefix==['push %rdi','mov %rsi,%r11','mov %r8,%rsi','mov %r12,%rdi','add $0x580,%rdi','mov %rbp,%rcx','cld']
    assert decoded[case['post_copy_pop']['offset']]=='pop %rdi'
    assert re.fullmatch(r'rep movsb (?:%ds:)?\(%rsi\),(?:%es:)?\(%rdi\)',decoded[case['copy_offset']])
    checked.update(r['offset'] for r in case['prelude']+rows)
    seen,pending=set(),[case['copy_offset']]
    edge=(case['carry_gate'],case['success_offset'])
    while pending:
        at=pending.pop()
        if at in seen: continue
        assert at not in roots
        seen.add(at)
        pending.extend(p for p in pred[at] if (p,at)!=edge)
    cuts.append(dict(name=case['name'],removed_edge=list(edge),predecessors=len(seen)))
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_transcript_sha256=digest(transcript),selected_instructions=len(checked),
    rtc_effect_instructions=sum(len(c['leaf']['instructions']) for c in audit['clock_contracts']),
    twelve_register_save_reverse_restore_equal=True,exact_copy_setup_and_live_word_equal=True,
    reverse_admission_cuts=cuts,
    scope='Independent selected GNU RTC register/memory effects, saved-register order, live-word/copy operands and reverse input-admission cuts, bound to the preceding byte-identical full GNU graph. Not new native/GNU execution, complete owners/nonalias/lifetime/retired-spill cleanup or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(checked),rtc_effect_instructions=report['rtc_effect_instructions'],reverse_cuts=len(cuts))))
