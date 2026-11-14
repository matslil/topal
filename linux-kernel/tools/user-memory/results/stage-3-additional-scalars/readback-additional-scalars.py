"""Independent GNU operands, reverse admission cuts and unchanged artifacts."""
from collections import Counter,defaultdict
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

build,baseline,output = map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=True)
digest = lambda data:hashlib.sha256(data).hexdigest()
audit = json.loads((build/'additional-scalar-audit.json').read_text())
graph = json.loads((build/'process-access-inventory.json').read_text())
assert audit['passed'] and audit['transfer_regions'] == 33 and audit['user_access_instructions'] == 51
nodes = {n['offset']:n for n in graph['instructions']}
predecessors = defaultdict(set)
for edge in graph['edges']:
    predecessors[edge['target']].add(edge['source'])
roots = {r['offset'] for r in graph['roots']}
selected = {}
for case in audit['cases']:
    for access in case['user_accesses']:
        seen,pending = set(),[access['offset']]
        while pending:
            at = pending.pop()
            if at in seen:
                continue
            assert at not in roots
            seen.add(at)
            pending.extend(p for p in predecessors[at]
                if (p,at) != (case['carry_gate'],case['success_offset']))
    for node in case['prelude']+case['preparation']+case['local_instructions']:
        selected[node['offset']] = nodes[node['offset']]
windows = []
for at,node in sorted(selected.items()):
    if windows and at == windows[-1][1]:
        windows[-1][1] = at+node['size']
    else:
        windows.append([at,at+node['size']])
decoded = {}
for i,(start,end) in enumerate(windows):
    command = ['objdump','-d','-M','intel','--insn-width=16',
        '--start-address='+str(graph['function_address']+start),
        '--stop-address='+str(graph['function_address']+end),str(build/'kernel.elf')]
    result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    for suffix,data in [('stdout.log',result.stdout),('stderr.log',result.stderr),
        ('command.json',(json.dumps(dict(command=command,exit_code=result.returncode),indent=2)+'\n').encode())]:
        (output/(str(i)+'.'+suffix)).write_bytes(data)
    assert result.returncode == 0
    for line in result.stdout.decode().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if match:
            at = int(match[1],16)-graph['function_address']
            assert at not in decoded and bytes.fromhex(match[2]).hex() == selected[at]['bytes']
            decoded[at] = match[3]
assert set(decoded) == set(selected)
registers = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15']
widths = {'BYTE':1,'WORD':2,'DWORD':4,'QWORD':8}
actions = Counter()
for case in audit['cases']:
    assert re.fullmatch(r'movabs\s+r9,0x'+format(case['length'],'x'),decoded[case['call_offset']-10])
    if case['name'] == 'poll-result':
        assert re.fullmatch(r'add\s+rax,0x6',decoded[case['candidate_setup_offset']+3])
    for access in case['user_accesses']:
        text = decoded[access['offset']]
        match = re.search(r'(BYTE|WORD|DWORD|QWORD) PTR \[([^]]+)\]',text)
        assert match and widths[match[1]] == access['width'],text
        terms = match[2].split('+')
        assert terms[0] == registers[access['base_register']] and len(terms) <= 2
        displacement = int(terms[1],16) if len(terms) == 2 else 0
        assert displacement+access['alias_bias'] == access['candidate_offset']
        assert 0 <= access['candidate_offset'] and access['candidate_offset']+access['width'] <= case['length']
        action = 'write' if re.match(r'mov\s+(BYTE|WORD|DWORD|QWORD) PTR',text) else 'read'
        assert action == access['action'] and (action == 'read' or case['direction'] == 'write')
        actions[action] += 1
comparisons = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
    'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
    'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data = (build/name).read_bytes()
    assert data == (baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(data),sha256=digest(data)))
for name in ['static-elf-abi-fixture.elf','initramfs.cpio','extent-call-inventory.json',
    'process-access-inventory.json','process-access-origins.json','process-byte-origins.json',
    'fixed-transfer-audit.json','copy-transfer-audit.json','staged-transfer-audit.json']:
    assert (build/name).read_bytes() == (baseline/name).read_bytes(),name
native = json.loads((build/'first-image-exit/observations.json').read_text())
assert native['passed'] and native == json.loads((baseline/'first-image-exit/observations.json').read_text())
serial = (build/'first-image-exit/serial.log').read_bytes()
assert serial == (baseline/'first-image-exit/serial.log').read_bytes()
report = dict(passed=True,gnu_instructions=len(decoded),gnu_windows=len(windows),
    scalar_user_accesses=51,user_operand_counts=dict(actions),backward_success_edge_cuts=51,
    kernel_comparisons=comparisons,preceding_three_access_audits_equal=True,
    exact_pinned_observations_equal=True,serial_bytes=len(serial),serial_sha256=digest(serial),
    current_native_first_reach_of_selected_sites=len({r['call_offset'] for r in audit['cases']} &
        {r['offset'] for r in native['extent_call_coverage']['reached']}),
    scope='Separate GNU raw boundaries/operands and reverse root cuts for selected intervals, plus unchanged artifact/observation identity. Current native first reach is not new scalar-transfer observation; complete owners, lifetime, other accesses and production qualification remain open')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k != 'kernel_comparisons'}))
