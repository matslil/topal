"""Independent GNU operands and reverse cuts for the selected serial receiver."""
from collections import defaultdict
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
audit_bytes = (build/'serial-prefix-audit.json').read_bytes()
audit = json.loads(audit_bytes)
graph_bytes = (build/'process-access-inventory.json').read_bytes()
graph = json.loads(graph_bytes)
assert audit['passed'] and audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert audit['graph_sha256'] == digest(graph_bytes)
case = audit['cases'][0]
selected = {r['offset']:r for key in ['zero_guard','cap_prelude','local_instructions','post_copy_cleanup'] for r in case[key]}
nodes = {r['offset']:r for r in graph['instructions']}
for at in [case['call_offset'],case['call_offset']+5,case['carry_gate']]: selected[at] = nodes[at]
windows = []
for at,row in sorted(selected.items()):
    if windows and windows[-1][1] == at: windows[-1][1] = at+row['size']
    else: windows.append([at,at+row['size']])
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
        if not match: continue
        at = int(match[1],16)-graph['function_address']
        assert at in selected and at not in decoded
        assert bytes.fromhex(match[2]).hex() == selected[at]['bytes']
        decoded[at] = re.sub(r'\s+',' ',match[3].split('#',1)[0].strip())
assert set(decoded) == set(selected)
def exact(raw,expected):
    rows = [r for r in selected.values() if r['bytes'] == raw]
    assert rows and all(decoded[r['offset']] == expected for r in rows),(raw,expected,rows)
exact('4981f900000000','cmp r9,0x0')
exact('4981f900010000','cmp r9,0x100')
exact('49b90001000000000000','movabs r9,0x100')
exact('49bf0000000000000000','movabs r15,0x0')
exact('8806','mov BYTE PTR [rsi],al')
exact('4881c601000000','add rsi,0x1')
exact('4981c701000000','add r15,0x1')
exact('4d39cf','cmp r15,r9')
exact('4c89c7','mov rdi,r8')
exact('4c89f9','mov rcx,r15')
exact('4c89f8','mov rax,r15')
exact('66bafd03','mov dx,0x3fd')
exact('66baf803','mov dx,0x3f8')
exact('ec','in al,dx')
exact('a801','test al,0x1')
exact('fc','cld')
assert re.fullmatch(r'rep movs BYTE PTR (?:es:)?\[rdi\],BYTE PTR (?:ds:)?\[rsi\]',decoded[case['copy_offset']])
for at,text in decoded.items():
    raw = bytes.fromhex(selected[at]['bytes'])
    if raw[:1] == b'\xe8' or raw[:1] == b'\x0f' and 0x80 <= raw[1] <= 0x8f:
        width = 1 if raw[0] == 0xe8 else 2
        target = at+len(raw)+struct.unpack_from('<i',raw,width)[0]
        assert int(text.split()[1],16)-graph['function_address'] == target
pred = defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots = {r['offset'] for r in graph['roots']}
cuts = [(case['carry_gate'],case['success_offset']),
        (case['zero_guard'][1]['offset'],case['zero_guard'][2]['offset'])]
visited = []
for edge in cuts:
    seen,pending = set(),[case['copy_offset']]
    while pending:
        at = pending.pop()
        if at in seen: continue
        assert at not in roots
        seen.add(at)
        pending.extend(p for p in pred[at] if (p,at) != edge)
    visited.append(len(seen))
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_instructions=len(decoded),gnu_windows=len(windows),
    selected_zero_cap_receive_publication_operands_equal=True,reverse_zero_and_admission_cuts=True,
    reverse_predecessor_counts=visited,
    scope='Selected GNU byte/boundary/operand and reverse-cut readback only; not independent full x86 semantics, native execution, owners/nonalias/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(decoded),windows=len(windows))))
