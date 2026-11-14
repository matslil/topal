"""Independent GNU file disassembly and reverse admission cuts; no execution."""
from collections import defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build, output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
audit_bytes = (build/'pipe-fixed-output-audit.json').read_bytes()
audit = json.loads(audit_bytes)
graph_bytes = (build/'process-access-inventory.json').read_bytes()
graph = json.loads(graph_bytes)
assert audit['passed'] and audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert audit['graph_sha256'] == digest(graph_bytes)
nodes = {r['offset']:r for r in graph['instructions']}
predecessors = defaultdict(set)
for edge in graph['edges']:
    predecessors[edge['target']].add(edge['source'])
roots = {r['offset'] for r in graph['roots']}
selected = {}
for case in audit['cases']:
    for at,row in nodes.items():
        if case['candidate_setup_offset'] <= at < case['local_end']:
            selected[at] = row
for leaf in audit['callee_contracts']:
    selected.update((r['offset'],r) for r in leaf['instructions'])
windows = []
for at,row in sorted(selected.items()):
    if windows and windows[-1][1] == at:
        windows[-1][1] = at+row['size']
    else:
        windows.append([at,at+row['size']])
decoded = {}
for index,(start,end) in enumerate(windows):
    command = ['objdump','-d','-M','intel','--insn-width=16',
        '--start-address='+str(graph['function_address']+start),
        '--stop-address='+str(graph['function_address']+end),str(build/'kernel.elf')]
    result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (output/f'{index}.stdout.log').write_bytes(result.stdout)
    (output/f'{index}.stderr.log').write_bytes(result.stderr)
    (output/f'{index}.command.json').write_text(json.dumps(dict(command=command,exit_code=result.returncode),indent=2)+'\n')
    assert result.returncode == 0
    for line in result.stdout.decode().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if not match:
            continue
        at = int(match[1],16)-graph['function_address']
        assert at in selected and at not in decoded
        assert bytes.fromhex(match[2]).hex() == selected[at]['bytes']
        decoded[at] = match[3].split('#',1)[0].strip()
assert set(decoded) == set(selected)

def family(name):
    for group in [('rax','eax','ax','al','ah'),('rcx','ecx','cx','cl','ch')]:
        if name in group:
            return group[0]
    match = re.fullmatch(r'(r(?:[89]|1[0-5]))(?:d|w|b)?',name)
    return match[1] if match else name

def effects(rows,allowed,memory_writes):
    registers,stores = set(),[]
    for row in rows:
        text = decoded[row['offset']]
        mnemonic,_,operands = text.partition(' ')
        if mnemonic in ['cmp','je','jne','jbe','jb','ja','jae','jmp','ret']:
            continue
        assert mnemonic in ['mov','movabs','add','sub','and','or','shl','shr'],text
        destination = operands.split(',',1)[0].strip()
        if re.fullmatch(r'[a-z0-9]+',destination):
            register = family(destination)
            assert register in allowed,(row,text)
            registers.add(register)
        else:
            assert memory_writes and mnemonic == 'mov',(row,text)
            stores.append(row['offset'])
    return dict(register_writes=sorted(registers),memory_store_offsets=stores)

cuts,operands = [],[]
for case in audit['cases']:
    gate,success = case['success_edge']
    for access in case['access_instructions']:
        pending,seen = [access['offset']],set()
        while pending:
            at = pending.pop()
            if at in seen:
                continue
            assert at not in roots,'GNU readback: access bypasses admission'
            seen.add(at)
            pending.extend(p for p in predecessors[at] if (p,at) != (gate,success))
        cuts.append(dict(offset=access['offset'],predecessors_visited=len(seen)))
        text = decoded[access['offset']]
        expected = {('write',8):r'mov\s+QWORD PTR \[r8(?:\+0x0)?\],rax',
                    ('read',8):r'mov\s+r11,QWORD PTR \[r8(?:\+0x0)?\]',
                    ('write',4):r'mov\s+DWORD PTR \[r8(?:\+0x0)?\],eax'}[(access['action'],access['width'])]
        assert re.fullmatch(expected,text),text
        operands.append(dict(offset=access['offset'],assembly=text))
pair,count = audit['cases']
pair_effects = effects(pair['local_instructions'],{'rax','rcx','r11','r13','r14'},True)
assert decoded[count['local_instructions'][0]['offset']].startswith('call ')
count_effects = effects(count['local_instructions'][1:],{'rax'},True)
leaf_effects = effects(audit['callee_contracts'][0]['instructions'],{'rax','rcx','r10','r11'},False)
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_instructions=len(decoded),gnu_windows=len(windows),
    output_operands=operands,backward_success_edge_cuts=cuts,
    pair_effects=pair_effects,count_effects=count_effects,lookup_effects=leaf_effects,
    scope='Independent selected GNU file disassembly and reverse graph cuts. Exact restricted operand vocabulary supports explicit normal-return register effects only. No VM execution, actual owners, nonaliasing, continuous lifetime or complete exception/production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(decoded),windows=len(windows),reverse_cuts=len(cuts))))
