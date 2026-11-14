"""Independent GNU operand projection and reverse guard cuts for pipe loops."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

build,gnu,output = map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
audit_bytes = (build/'pipe-loop-audit.json').read_bytes()
audit = json.loads(audit_bytes)
graph_bytes = (build/'process-access-inventory.json').read_bytes()
graph = json.loads(graph_bytes)
transcript_bytes = (gnu/'gnu-decoder-transcript.json').read_bytes()
gnu_report = json.loads((gnu/'gnu-verification.json').read_text())
assert gnu_report['passed'] and gnu_report['graph_sha256'] == digest(graph_bytes)
assert gnu_report['kernel_sha256'] == audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert gnu_report['decoder_transcript_sha256'] == digest(transcript_bytes)
decoded = {row['offset']:re.sub(r'\s+',' ',row['assembly'].split('<',1)[0].strip())
    for row in json.loads(transcript_bytes)}
expected = {
    '4981f900000000':'cmp $0x0,%r9','4981f900100000':'cmp $0x1000,%r9',
    '4d89eb':'mov %r13,%r11','4d29e3':'sub %r12,%r11',
    '4981c30020fcff':'add $0xfffffffffffc2000,%r11','49c1eb07':'shr $0x7,%r11',
    '49c1e30c':'shl $0xc,%r11','4d01e3':'add %r12,%r11','4981c300f00300':'add $0x3f000,%r11',
    '4d8bbd20000000':'mov 0x20(%r13),%r15','498b8528000000':'mov 0x28(%r13),%rax',
    '4901c7':'add %rax,%r15','4981e7ff0f0000':'and $0xfff,%r15',
    '4c89c7':'mov %r8,%rdi','4c89c6':'mov %r8,%rsi','4c89c9':'mov %r9,%rcx',
    '4c89de':'mov %r11,%rsi','4c89df':'mov %r11,%rdi',
    '4c01fe':'add %r15,%rsi','4c01ff':'add %r15,%rdi',
    '8a06':'mov (%rsi),%al','8807':'mov %al,(%rdi)','3807':'cmp %al,(%rdi)',
    'c60600':'movb $0x0,(%rsi)','803e00':'cmpb $0x0,(%rsi)',
    '4881c701000000':'add $0x1,%rdi','4881c601000000':'add $0x1,%rsi',
    '4981c701000000':'add $0x1,%r15','4881c1ffffffff':'add $0xffffffffffffffff,%rcx',
    '4881f900000000':'cmp $0x0,%rcx'}
selected = {}
for case in audit['cases']:
    for row in case['count_guard']+case['local_instructions']:
        at,raw = row['offset'],bytes.fromhex(row['bytes'])
        selected[at] = row
        if raw[:1] == b'\x0f':
            assert raw[1] in [0x84,0x85,0x87]
            mnemonic = {0x84:'je',0x85:'jne',0x87:'ja'}[raw[1]]
            parts = decoded[at].split()
            assert parts[0] == mnemonic
            target = at+6+struct.unpack_from('<i',raw,2)[0]
            assert int(parts[1],16) == graph['function_address']+target
        else:
            assert raw.hex() in expected and decoded[at] == expected[raw.hex()],(at,decoded[at])
pred = defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots = {r['offset'] for r in graph['roots']}
cuts = []
for case in audit['cases']:
    guards = case['count_guard']
    edges = [(case['carry_gate'],case['success_offset']),
             (guards[1]['offset'],guards[2]['offset']),(guards[3]['offset'],case['call_offset'])]
    for edge in edges:
        seen,pending = set(),[case['start_offset']]
        while pending:
            at = pending.pop()
            if at in seen: continue
            assert at not in roots
            seen.add(at)
            pending.extend(p for p in pred[at] if (p,at) != edge)
        cuts.append(dict(name=case['name'],removed_edge=list(edge),predecessors=len(seen)))
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_transcript_sha256=digest(transcript_bytes),
    selected_instructions=len(selected),exact_operands_and_direct_targets_equal=True,
    reverse_guard_cuts=cuts,
    scope='Independent selected GNU operands/targets and six reverse guard/admission cuts, bound to complete GNU graph readback. Not complete x86 semantics, native execution, actual private owners/nonalias/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(selected),reverse_cuts=len(cuts))))
