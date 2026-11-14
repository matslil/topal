"""Independent GNU operands/loop bounds, backward cuts and native consistency."""
from collections import Counter,defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build,baseline,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=True)
audit=json.loads((build/'staged-transfer-audit.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
assert audit['passed'] and audit['transfer_regions']==16 and audit['user_access_instructions']==208
nodes={n['offset']:n for n in graph['instructions']}
predecessors=defaultdict(set)
for edge in graph['edges']:
    predecessors[edge['target']].add(edge['source'])
roots={r['offset'] for r in graph['roots']}
selected={}
for case in audit['cases']:
    for access in case['user_accesses']:
        seen,pending=set(),[access['offset']]
        while pending:
            at=pending.pop()
            if at in seen:
                continue
            assert at not in roots
            seen.add(at)
            pending.extend(p for p in predecessors[at] if (p,at)!=(case['carry_gate'],case['success_offset']))
    for row in case['preparation']+case['transfer']:
        selected[row['offset']]=nodes[row['offset']]
    for at,node in nodes.items():
        if case['candidate_setup_offset']<=at<=case['carry_gate']:
            selected[at]=node
    for row in case['preparation']:
        effect=row.get('preparation_effect',{})
        if 'field' in effect:
            assert 0<=effect['field']<1<<31, 'selected GS displacements must be positive signed values'
windows=[]
for at,row in sorted(selected.items()):
    if windows and at==windows[-1][1]:
        windows[-1][1]=at+row['size']
    else:
        windows.append([at,at+row['size']])
decoded={}
for i,(start,end) in enumerate(windows):
    command=['objdump','-d','-M','intel','--insn-width=16',
        '--start-address='+str(graph['function_address']+start),
        '--stop-address='+str(graph['function_address']+end),str(build/'kernel.elf')]
    result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (output/f'{i}.stdout.log').write_bytes(result.stdout)
    (output/f'{i}.stderr.log').write_bytes(result.stderr)
    (output/f'{i}.command.json').write_text(json.dumps(dict(command=command,exit_code=result.returncode),indent=2)+'\n')
    assert result.returncode==0
    for line in result.stdout.decode().splitlines():
        match=re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if match:
            at=int(match[1],16)-graph['function_address']
            assert at not in decoded and bytes.fromhex(match[2]).hex()==selected[at]['bytes']
            decoded[at]=match[3]
assert set(decoded)==set(selected)
widths={'BYTE':1,'WORD':2,'DWORD':4,'QWORD':8}
registers={2:'rdx',6:'rsi',7:'rdi',9:'r9',10:'r10'}
operands=Counter()
for case in audit['cases']:
    core=case['transfer']
    if case['index_register'] is not None:
        index=registers[case['index_register']]
        assert re.fullmatch(r'inc\s+'+index,decoded[core[-3]['offset']])
        match=re.fullmatch(r'cmp\s+'+index+r',0x([0-9a-f]+)',decoded[core[-2]['offset']])
        assert match and int(match[1],16)==case['length']
        init=decoded[case['preparation'][-1]['offset']]
        assert re.fullmatch(r'xor\s+'+index+'d,'+index+'d',init)
    for access in case['user_accesses']:
        text=decoded[access['offset']]
        match=re.search(r'(BYTE|WORD|DWORD|QWORD) PTR \[([^]]+)\]',text)
        assert match,text
        width=widths[match[1]]
        terms=match[2].replace('*1','').split('+')
        action='write' if re.match(r'mov\s+(?:BYTE|WORD|DWORD|QWORD) PTR',text) else 'read'
        assert action==access['action'] and width==access['width']
        if case['index_register'] is not None:
            assert sorted(terms)==sorted([registers[case['user_register']],registers[case['index_register']]])
            # Independently enumerate the tested-after-increment loop's full
            # access set from the decoded constant and unit increment.
            indices=[]
            value=0
            while True:
                indices.append(value)
                value+=1
                if value==case['length']:
                    break
            assert indices==access['indices'] and all(0<=i and i+width<=case['length'] for i in indices)
        else:
            assert terms[0]==registers[case['user_register']] and len(terms)<=2
            offset=int(terms[1],16) if len(terms)==2 else 0
            assert offset==access['candidate_offset'] and 0<=offset and offset+width<=case['length']
            previous=next(core[i-1] for i,n in enumerate(core) if n['offset']==access['offset'])
            gs=re.search(r'gs:(?:\[)?(0x[0-9a-f]+)',decoded[previous['offset']])
            assert gs and int(gs[1],16)==case['counterpart']['offset']+offset
        operands[action]+=1
paths=[build/name/'observations.json' for name in ['first-image-exit','source-user-extents'] if (build/name/'observations.json').is_file()]
assert len(paths)==1
native=json.loads(paths[0].read_text())
observed=native['staged_transfer_observation']
assert native['passed'] and observed['passed'] and observed['kernel_sha256']==audit['kernel_sha256']
assert observed['audit_sha256']==digest((build/'staged-transfer-audit.json').read_bytes())
counts=Counter()
for name,row in observed['counts'].items():
    assert row['attempts']==row['admitted']+row['refused'] and row['completed']==row['admitted']
    counts.update(row)
cases={r['name']:r for r in audit['cases']}
for sample in observed['samples']:
    case=cases[sample['name']]
    assert sample['size']==case['length'] and sample['exact_payload_equal'] and sample['candidate_preserved']
    for field,user,writable in [('source_map',case['direction']=='read',False),('target_map',case['direction']=='write',True)]:
        assert sum(p['size'] for p in sample[field])==case['length']
        for page in sample[field]:
            assert page['user']==user and (not writable or page['writable'])
            assert all(entry&1 for _,_,entry in page['path'])
            assert all(entry&4 for _,_,entry in page['path']) if user else not all(entry&4 for _,_,entry in page['path'])
            assert not writable or all(entry&2 for _,_,entry in page['path'])
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data=(build/name).read_bytes()
    assert data==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,sha256=digest(data),bytes=len(data)))
for name in ['static-elf-abi-fixture.elf','initramfs.cpio','extent-call-inventory.json',
             'process-access-inventory.json','process-access-origins.json','process-byte-origins.json']:
    assert (build/name).read_bytes()==(baseline/name).read_bytes(),name
serial=(paths[0].parent/'serial.log').read_bytes()
assert serial==(baseline/paths[0].parent.name/'serial.log').read_bytes()
report=dict(passed=True,gnu_instructions=len(decoded),gnu_windows=len(windows),
    user_operand_counts=dict(operands),backward_access_checks=208,
    native_counts=dict(counts),reached_cases=sorted(s['name'] for s in observed['samples']),unreached=observed['unreached'],
    kernel_comparisons=comparisons,serial_bytes=len(serial),serial_sha256=digest(serial),
    scope='Independent GNU boundaries/operands/loop enumeration, backwards success-edge cuts, retained observation consistency and unchanged artifacts. Native payload/control comparisons rely on the frozen read-only observer; not continuous lifetime, complete owners or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='kernel_comparisons'}))
