"""Independent GNU instruction boundaries, operands and reverse admission cuts."""
from collections import defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
audit=json.loads((build/'staging-copy-audit.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
nodes={r['offset']:r for r in graph['instructions']}
pred=defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
selected={}
for case in audit['cases']:
    seen,pending=set(),[case['copy_offset']]
    while pending:
        at=pending.pop()
        if at in seen: continue
        assert at not in roots
        seen.add(at)
        pending.extend(p for p in pred[at] if (p,at)!=(case['carry_gate'],case['success_offset']))
    for row in case['prelude']+case['local_instructions']:
        selected[row['offset']]=nodes[row['offset']]
    if case['name']=='console-input':
        at=case['copy_offset']+2
        selected[at]=nodes[at]
        selected[at+nodes[at]['size']]=nodes[at+nodes[at]['size']]
    for row in case['count_initializers']: selected[row['offset']]=nodes[row['offset']]
for leaf in audit['callee_contracts']:
    for row in leaf['instructions']: selected[row['offset']]=nodes[row['offset']]
windows=[]
for at,row in sorted(selected.items()):
    if windows and windows[-1][1]==at: windows[-1][1]=at+row['size']
    else: windows.append([at,at+row['size']])
decoded={}
for index,(start,end) in enumerate(windows):
    command=['objdump','-d','-M','intel','--insn-width=16',
        '--start-address='+str(graph['function_address']+start),
        '--stop-address='+str(graph['function_address']+end),str(build/'kernel.elf')]
    result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (output/f'{index}.stdout.log').write_bytes(result.stdout)
    (output/f'{index}.stderr.log').write_bytes(result.stderr)
    (output/f'{index}.command.json').write_text(json.dumps(dict(command=command,exit_code=result.returncode),indent=2)+'\n')
    assert result.returncode==0
    for line in result.stdout.decode().splitlines():
        match=re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if match:
            at=int(match[1],16)-graph['function_address']
            assert at not in decoded and bytes.fromhex(match[2]).hex()==selected[at]['bytes']
            decoded[at]=match[3]
assert set(decoded)==set(selected)
transfers=[]
for case in audit['cases']:
    text=decoded[case['copy_offset']]
    assert re.fullmatch(r'rep movs\s+BYTE PTR (?:es:)?\[rdi\],BYTE PTR (?:ds:)?\[rsi\]',text),text
    assert any(decoded[r['offset']]=='cld' for r in case['local_instructions'])
    operands=[decoded[r['offset']] for r in case['local_instructions']]
    endpoint='rsi' if case['admitted_direction']=='read' else 'rdi'
    assert any(re.fullmatch(r'mov\s+'+endpoint+',r8',text) for text in operands)
    count={5:'rbp',9:'r9',15:'r15'}[case['count_register']]
    assert any(re.fullmatch(r'mov\s+rcx,'+count,text) for text in operands)
    if case['name']=='console-input':
        assert re.fullmatch(r'mov\s+r14,r9',decoded[case['copy_offset']+5])
        assert re.fullmatch(r'cmp\s+rdx,0x100',decoded[case['prelude_start']])
    if case['name'].startswith('descriptor-'):
        assert re.fullmatch(r'cmp\s+r9,0x100',decoded[case['prelude_start']])
    if case['name'].startswith('sparse-'):
        assert re.fullmatch(r'cmp\s+rbp,0x100',decoded[case['prelude_start']+3])
    for row in case['count_initializers']:
        assert re.fullmatch(r'movabs\s+r15,0x'+format(row['value'],'x'),decoded[row['offset']])
    transfers.append(dict(name=case['name'],copy_offset=case['copy_offset'],user_endpoint=endpoint,
        count_register=count,maximum_bytes=case['maximum_bytes'],reverse_success_cut=True))
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],gnu_instructions=len(decoded),
    gnu_windows=len(windows),selected_copies=transfers,backward_success_edge_cuts=len(transfers),
    scope='Separate GNU byte boundaries and operands with reverse graph cuts. Conditional local intervals only; complete counterpart ownership and continuous mapping stability remain open')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
