"""GNU boundaries, backward admission cuts, native payload/page observations."""
from collections import Counter, defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build, baseline, output = map(Path,sys.argv[1:4])
metadata = Path(sys.argv[4]) if len(sys.argv)>4 else baseline
output.mkdir(parents=True,exist_ok=True)
audit = json.loads((build/'copy-transfer-audit.json').read_text())
graph = json.loads((build/'process-access-inventory.json').read_text())
assert audit['passed'] and audit['transfer_regions']==15 and audit['copy_instructions']==14
predecessors = defaultdict(set)
for e in graph['edges']:
    predecessors[e['target']].add(e['source'])
roots = {r['offset'] for r in graph['roots']}
for group in audit['copy_groups']:
    removed = {tuple(e) for e in group['admission_edges']}
    seen,pending = set(),[group['copy_offset']]
    while pending:
        at=pending.pop()
        if at in seen:
            continue
        assert at not in roots
        seen.add(at)
        pending.extend(p for p in predecessors[at] if (p,at) not in removed)
    assert seen
nodes = {n['offset']:n for n in graph['instructions']}
selected = {}
for case in audit['cases']:
    for row in case['prelude']+case['local_instructions']:
        selected[row['offset']] = nodes[row['offset']]
for contract in audit['sampler_contracts']:
    for row in contract['instructions']:
        selected[row['offset']] = nodes[row['offset']]
# Decode contiguous windows; a shared target is checked once independently.
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
        m=re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if m:
            at=int(m[1],16)-graph['function_address']
            assert at not in decoded and bytes.fromhex(m[2]).hex()==selected[at]['bytes']
            decoded[at]=m[3]
assert set(decoded)==set(selected)
for group in audit['copy_groups']:
    assert decoded[group['copy_offset']].startswith('rep movs')
observations=[]
for name in ['first-image-exit','source-user-extents']:
    path=build/name/'observations.json'
    if path.is_file():
        observations.append(path)
assert len(observations)==1
native=json.loads(observations[0].read_text())
observed=native['copy_transfer_observation']
assert native['passed'] and observed['passed'] and observed['audit_sha256']==digest((build/'copy-transfer-audit.json').read_bytes())
assert observed['kernel_sha256']==audit['kernel_sha256']
cases={r['name']:r for r in audit['cases']}
assert set(observed['counts'])==set(cases)
counts=Counter()
for name,row in observed['counts'].items():
    assert row['attempts']==row['admitted']+row['refused'] and row['completed']==row['admitted']
    counts.update(row)
for sample in observed['samples']:
    case=cases[sample['name']]
    assert sample['elements']*case['copy_width']==sample['size']
    assert sample['exact_payload_equal'] and sample['translation_observations_equal']
    assert sample['helper_registers_preserved'] and sample['captured_arguments_preserved']
    for endpoint,user,writable in [('source_map',case['admitted_direction']=='read',False),
                                  ('target_map',case['admitted_direction']=='write',True)]:
        assert sum(m['size'] for m in sample[endpoint])==sample['size']
        for mapping in sample[endpoint]:
            assert mapping['user']==user and (not writable or mapping['writable'])
            assert all(entry&1 for _,_,entry in mapping['path'])
            assert all(entry&4 for _,_,entry in mapping['path']) if user else not all(entry&4 for _,_,entry in mapping['path'])
            if writable:
                assert all(entry&2 for _,_,entry in mapping['path'])
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data=(build/name).read_bytes()
    assert data==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,sha256=digest(data),bytes=len(data)))
for name in ['static-elf-abi-fixture.elf','initramfs.cpio','extent-call-inventory.json']:
    assert (build/name).read_bytes()==(baseline/name).read_bytes(),name
assert (build/'kernel.elf').read_bytes()==(metadata/'kernel.elf').read_bytes()
for name in ['process-access-inventory.json','process-access-origins.json','process-byte-origins.json']:
    assert (build/name).read_bytes()==(metadata/name).read_bytes(),name
serial=(observations[0].parent/'serial.log').read_bytes()
assert serial==(baseline/observations[0].parent.name/'serial.log').read_bytes()
report=dict(passed=True,gnu_instructions=len(decoded),gnu_windows=len(windows),
    backward_admission_groups=len(audit['copy_groups']),native_counts=dict(counts),
    reached_cases=sorted(s['name'] for s in observed['samples']),
    unreached=observed['unreached'],kernel_comparisons=comparisons,
    serial_bytes=len(serial),serial_sha256=digest(serial),
    scope='Independent GNU boundaries, backward admission cuts, retained native count/permission/sample consistency and unchanged artifacts; payload equality and per-invocation checks rely on the frozen observer. Not continuous stability, complete owner/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='kernel_comparisons'}))
