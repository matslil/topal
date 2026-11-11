"""Re-decode every graph node with GNU GDB and review matching native evidence."""
import json
from pathlib import Path
import subprocess
import sys
sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf, digest

build, baseline, output = map(Path, sys.argv[1:4])
output.mkdir(parents=True, exist_ok=True)
graph = json.loads((build/'process-access-inventory.json').read_text())
assert graph['passed'] and graph['reached_instructions'] == 31016
assert graph['reached_bytes'] == 221656 and graph['unvisited_bytes'] == 10007
assert graph['extent_calls_in_graph'] == 90
assert [r['offset'] for r in graph['extent_calls_outside_graph']] == [88236]
assert {r['offset'] for r in graph['recovered_windows']} == {162020, 185049}
kernel = Elf(build/'kernel.elf')
symbol, body = kernel.function(graph['function'])
assert digest(kernel.data) == graph['kernel_sha256']
assert digest(body) == graph['linked_function_sha256']
assert symbol['value'] == graph['function_address'] and symbol['size'] == graph['function_bytes']
program = r'''
import gdb
import hashlib
import json
import re

graph = json.load(open(graph_path))
assert hashlib.sha256(open(kernel_path,'rb').read()).hexdigest() == graph['kernel_sha256']
arch = gdb.selected_inferior().architecture()
base = graph['function_address']
nodes = {row['offset']:row for row in graph['instructions']}
edges, external, indirect, terminals = [], [], [], []
transcript = []
for at, row in sorted(nodes.items()):
    decoded, = arch.disassemble(base+at,count=1)
    assert decoded['addr'] == base+at and decoded['length'] == row['size'], (at,decoded,row)
    assembly = decoded['asm']
    assert '(bad)' not in assembly and '.byte' not in assembly
    transcript.append(dict(offset=at,length=decoded['length'],assembly=assembly))
    parts = assembly.split()
    while parts[0].startswith(('rep','rex.')) or parts[0] in ['lock','data16','addr32','cs']:
        parts.pop(0)
    name = parts[0]
    if name in ['ret','retq','retf','iret','iretq','sysret','sysretq','hlt','ud2','int','int3','syscall','sysenter','sysexit']:
        terminals.append(at)
        continue
    if name.startswith(('j','loop')) or name in ['call','callq']:
        match = re.match(r'^\w+\s+0x([0-9a-f]+)', ' '.join(parts))
        if match:
            address = int(match[1],16)
            destination = address-base
            if 0 <= destination < graph['function_bytes']:
                edges.append((at,destination,'direct-transfer'))
            else:
                external.append((at,address))
        else:
            assert '*' in assembly and name in ['call','callq','jmp','jmpq'], (at,assembly)
            indirect.append(at)
        if name in ['jmp','jmpq']:
            continue
    edges.append((at,at+decoded['length'],'call-return-assumption' if name in ['call','callq'] else 'fallthrough'))
expected = [(e['source'],e['target'],e['kind']) for e in graph['edges']]
assert len(edges) == len(expected) and set(edges) == set(expected)
assert set(external) == {(r['offset'],r['address']) for r in graph['external_transfers']}
assert set(indirect) == {r['offset'] for r in graph['indirect_transfers']}
assert set(terminals) == {r['offset'] for r in graph['terminals']}
pending = [r['offset'] for r in graph['roots']]
outgoing = {}
for source,target,_ in edges:
    outgoing.setdefault(source,[]).append(target)
closure = set()
while pending:
    at = pending.pop()
    if at not in closure:
        assert at in nodes
        closure.add(at)
        pending.extend(outgoing.get(at,[]))
assert closure == set(nodes)
report = dict(passed=True,gnu_decoder_nodes=len(transcript),gnu_graph_edges=len(edges),
    external_transfers=len(external),indirect_transfers=len(indirect),terminals=len(terminals),
    exact_lengths_and_direct_edges_equal=True,root_closure_equal=True,
    graph_sha256=hashlib.sha256(open(graph_path,'rb').read()).hexdigest(),
    decoder_transcript_sha256=hashlib.sha256(json.dumps(transcript,sort_keys=True).encode()).hexdigest(),
    scope='GNU host-file instruction-length/branch/closure re-decode of every selected LLVM graph node; no VM, complete callee/exception graph, pointer classification or security-dominance proof')
json.dump(report,open(result_path,'w'),indent=2)
print(json.dumps(report))
'''
gdb_program = ('set pagination off\nset confirm off\nfile '+str((build/'kernel.elf').resolve())+'\npython\n'
    +'graph_path='+repr(str((build/'process-access-inventory.json').resolve()))+'\n'
    +'kernel_path='+repr(str((build/'kernel.elf').resolve()))+'\n'
    +'result_path='+repr(str((output/'gnu-verification.json').resolve()))+'\n'+program+'\nend\nquit\n')
(output/'verify-graph.gdb').write_text(gdb_program)
command = ['gdb','-nx','-batch','-x',str(output/'verify-graph.gdb')]
result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
(output/'gdb.stdout.log').write_bytes(result.stdout)
(output/'gdb.stderr.log').write_bytes(result.stderr)
(output/'gdb.command.json').write_text(json.dumps(dict(argv=command,exit_code=result.returncode),indent=2)+'\n')
assert result.returncode == 0, result.stderr.decode(errors='replace')+'\n'+result.stdout.decode(errors='replace')
gnu = json.loads((output/'gnu-verification.json').read_text())
assert gnu['passed']
comparisons = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data = (build/name).read_bytes()
    assert data == (baseline/name).read_bytes(), name
    comparisons.append(dict(path=name,bytes=len(data),sha256=digest(data),unchanged=True))
for name in ['static-elf-abi-fixture.elf','initramfs.cpio']:
    assert (build/name).read_bytes() == (baseline/name).read_bytes(),name
image = (build/'static-elf-abi-fixture.elf').read_bytes()
assert digest(image) == '7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
serial = (build/'first-image-exit/serial.log').read_bytes()
assert serial == (baseline/'first-image-exit/serial.log').read_bytes()
observed = json.loads((build/'first-image-exit/observations.json').read_text())
native_binding = json.loads((build/'process-access-coverage.json').read_text())
assert observed['passed'] and native_binding['passed'] and native_binding['native_reached_calls'] == 38
assert native_binding['graph_sha256'] == digest((build/'process-access-inventory.json').read_bytes())
assert native_binding['observation_sha256'] == digest((build/'first-image-exit/observations.json').read_bytes())
union = set()
for directory in [
    'user-extent-source-poll-2268178-1794397912990685732',
    'user-extent-source-pipes-2271066-1794399038949272774',
    'user-extent-source-storage-2272711-1794399750942813740',
    'user-extent-source-serial-2274533-1794400529999282660',
]:
    obs = json.loads((build.parent/directory/'source-user-extents/observations.json').read_text())
    assert obs['passed'] and obs['extent_call_coverage']['observation_passed']
    assert obs['extent_call_coverage']['kernel_sha256'] == graph['kernel_sha256']
    assert obs['extent_call_coverage']['inventory_sha256'] == graph['extent_inventory_sha256']
    union.update(r['offset'] for r in obs['extent_call_coverage']['reached'])
node_offsets = {r['offset'] for r in graph['instructions']}
assert len(union) == 88 and union <= node_offsets
census = json.loads((build/'extent-call-inventory.json').read_text())
assert {r['offset'] for r in census['calls'] if r['offset'] in node_offsets}-union == {94708,101802}
report = dict(passed=True,gnu_verification=gnu,kernel_comparisons=comparisons,
    pinned_image_sha256=digest(image),unchanged_pinned_image=True,unchanged_boot_resource=True,
    serial_bytes=len(serial),serial_sha256=digest(serial),unchanged_serial=True,
    native_calls_in_graph=38,preceding_union_calls_in_graph=88,
    graph_instructions=31016,graph_instruction_bytes=221656,unvisited_bytes=10007,
    access_instruction_count=graph['access_instruction_count'],access_kind_counts=graph['access_kind_counts'],
    recovered_offsets=[r['offset'] for r in graph['recovered_windows']],
    graph_calls=90,emitted_calls=91,emitted_call_outside_graph=88236,
    graph_calls_without_selected_native_reach=[94708,101802],
    scope='Selected linked process direct-control and address-bearing instruction inventory; complete all-kernel/callee/exception roots, user-pointer classification, admission dominance/lifetime and production qualification remain required')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','gnu_verification']}))
