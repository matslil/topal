set pagination off
set confirm off
file /home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-application-2279338-1794402953201096759/kernel.elf
python
graph_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-application-2279338-1794402953201096759/process-access-inventory.json'
kernel_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-application-2279338-1794402953201096759/kernel.elf'
result_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/process-accesses-readback/gnu-verification.json'

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

end
quit
