"""Independent selected GNU boundaries, effects, saved-frame names and cuts."""
from collections import defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest
from stat_instruction_effects import effects

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
audit_bytes = (build/'stat-output-audit.json').read_bytes()
audit = json.loads(audit_bytes)
graph_bytes = (build/'process-access-inventory.json').read_bytes()
graph = json.loads(graph_bytes)
assert audit['passed'] and audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert audit['graph_sha256'] == digest(graph_bytes)
nodes = {r['offset']:r for r in graph['instructions']}
selected = {}
for case in audit['cases']:
    selected.update((r['offset'],r) for r in case['prelude']+case['local_instructions'])
    for at in [case['call_offset'],case['call_offset']+5,case['carry_gate'],case['copy_offset']]:
        selected[at] = nodes[at]
for contract in audit['callee_contracts']:
    selected.update((r['offset'],r) for r in contract['instructions'])
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
        decoded[at] = match[3].split('#',1)[0].strip()
assert set(decoded) == set(selected)
registers = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi']+['r'+str(n) for n in range(8,16)]
def native_effects(text):
    mnemonic,_,operands = text.partition(' ')
    if mnemonic in ['rep','repz','repe','repnz','repne']:
        mnemonic,_,operands = operands.strip().partition(' ')
        assert mnemonic in ['stos','scas','cmps','movs']
        expected = {'stos':({1,7},0,1),'scas':({1,7},1,0),
                    'cmps':({1,6,7},2,0),'movs':({1,6,7},1,1)}[mnemonic]
        assert operands.count('PTR') == expected[1]+expected[2]
        assert ('BYTE PTR' if mnemonic == 'movs' else 'QWORD PTR') in operands
        return expected
    if mnemonic in ['push','pop']:
        assert operands.strip() in registers
        return ({registers.index(operands.strip())} if mnemonic == 'pop' else set(),0,0)
    if mnemonic in ['cld','ret','call','jmp','je','jne','jb','jbe','ja','jae','jl']:
        return set(),0,0
    assert mnemonic in ['mov','movabs','cmp','add','sub','and','or','shl','shr'],text
    destination = operands.split(',',1)[0].strip()
    if mnemonic == 'cmp': return set(),operands.count('PTR'),0
    if destination in registers:
        return {registers.index(destination)},operands.count('PTR'),0
    assert mnemonic == 'mov' and destination.startswith('QWORD PTR '),text
    return set(),0,1
for at,text in decoded.items():
    effect = effects(bytes.fromhex(selected[at]['bytes']))
    assert native_effects(text) == (effect['register_writes'],len(effect['memory_reads']),len(effect['memory_writes'])),(at,text,effect)
    if effect.get('control') in ['jump','conditional','call']:
        operand = text.split(None,1)[1].split()[0]
        target = int(operand,16)-graph['function_address']
        raw = bytes.fromhex(selected[at]['bytes'])
        width = 2 if effect['control'] == 'conditional' else 1
        import struct
        assert target == at+len(raw)+struct.unpack_from('<i',raw,width)[0]
metadata = next(c for c in audit['callee_contracts'] if c['kind'] == 'saved-metadata-validator')
push_names = [decoded[r['offset']].split()[-1] for r in metadata['instructions'][:8]]
assert push_names == ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']
restoring = [r for r in metadata['instructions'] if r['offset'] >= metadata['restore_offset']][:8]
assert [decoded[r['offset']].split()[-1] for r in restoring] == list(reversed(push_names))
pred = defaultdict(set)
for edge in graph['edges']: pred[edge['target']].add(edge['source'])
roots = {r['offset'] for r in graph['roots']}
group = audit['copy_groups'][0]
excluded = {tuple(edge) for edge in group['admission_edges']}
seen,pending = set(),[group['copy_offset']]
while pending:
    at = pending.pop()
    if at in seen: continue
    assert at not in roots
    seen.add(at)
    pending.extend(p for p in pred[at] if (p,at) not in excluded)
copy_text = decoded[group['copy_offset']]
assert re.fullmatch(r'rep movs\s+BYTE PTR (?:es:)?\[rdi\],BYTE PTR (?:ds:)?\[rsi\]',copy_text),copy_text
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    graph_sha256=digest(graph_bytes),gnu_instructions=len(decoded),gnu_windows=len(windows),
    raw_and_gnu_register_and_memory_effect_counts_equal=True,saved_register_order=push_names,
    reverse_admission_group_cut=True,reverse_cut_predecessors=len(seen),
    scope='Independent selected GNU instruction/effect/saved-frame readback and reverse shared-copy cut; not native execution, complete owners/nonalias/lifetime/callee/exception or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,instructions=len(decoded),windows=len(windows),saved_registers=len(push_names))))
