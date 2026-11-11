"""GNU re-decode, backward predecessor review and exact native comparisons."""
from collections import Counter, defaultdict
import json
from pathlib import Path
import re
import subprocess
import sys
sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build, baseline, output = map(Path, sys.argv[1:4])
output.mkdir(parents=True, exist_ok=True)
report = json.loads((build/'fixed-transfer-audit.json').read_text())
graph = json.loads((build/'process-access-inventory.json').read_text())
origins = json.loads((build/'process-access-origins.json').read_text())
assert report['passed'] and report['transfer_regions'] == 9 and report['scalar_access_instructions'] == 27
assert report['graph_sha256'] == digest((build/'process-access-inventory.json').read_bytes())
assert report['origin_join_sha256'] == digest((build/'process-access-origins.json').read_bytes())
predecessors = defaultdict(set)
for e in graph['edges']:
    predecessors[e['target']].add(e['source'])
nodes = {n['offset']: n for n in graph['instructions']}
roots = {r['offset'] for r in graph['roots']}
counts = Counter()
verified = []
for case in report['cases']:
    gate, success = case['success_edge']
    # A separate backward walk from every access must reach the success edge
    # through a unique predecessor chain, with no root or alternate entry.
    for access in case['access_instructions']:
        at, seen = access['offset'], set()
        while at != gate:
            assert at not in seen and at not in roots
            seen.add(at)
            assert len(predecessors[at]) == 1
            at = next(iter(predecessors[at]))
        assert success in seen
    command = ['objdump', '-d', '-M', 'intel', '--insn-width=16',
        '--start-address='+str(graph['function_address']+success),
        '--stop-address='+str(graph['function_address']+case['local_end']), str(build/'kernel.elf')]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    (output/(case['name']+'.stdout.log')).write_bytes(result.stdout)
    (output/(case['name']+'.stderr.log')).write_bytes(result.stderr)
    (output/(case['name']+'.command.json')).write_text(json.dumps(dict(command=command, exit_code=result.returncode), indent=2)+'\n')
    assert result.returncode == 0
    decoded = {}
    for line in result.stdout.decode().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$', line)
        if match:
            offset = int(match[1], 16)-graph['function_address']
            if success <= offset < case['local_end']:
                decoded[offset] = (bytes.fromhex(match[2]), match[3])
    assert set(decoded) == {n['offset'] for n in case['local_instructions']}
    for at, (raw, text) in decoded.items():
        assert raw.hex() == nodes[at]['bytes']
    for access in case['access_instructions']:
        text = decoded[access['offset']][1]
        match = re.search(r'(BYTE|WORD|DWORD|QWORD) PTR \[(rsi|r8|r10)(?:\+(0x[0-9a-f]+))?\]', text)
        assert match, text
        width = {'BYTE': 1, 'WORD': 2, 'DWORD': 4, 'QWORD': 8}[match[1]]
        offset = int(match[3], 16) if match[3] else 0
        action = 'write' if re.match(r'mov\s+(?:BYTE|WORD|DWORD|QWORD) PTR', text) else 'read'
        assert action == access['action'] and width == access['width'] and offset == access['candidate_offset']
        assert offset >= 0 and offset+width <= case['admitted_bytes']
        assert access['base_register'] == {'rsi': 6, 'r8': 8, 'r10': 10}[match[2]]
        counts[action] += 1
    verified.append(dict(name=case['name'], gnu_instructions=len(decoded),
        user_scalar_instructions=len(case['access_instructions']), backward_success_chain=True))
comparisons = []
for name in ['root.o', 'source.o', 'runtime.o', 'owner.o', 'controller.o', 'target.o', 'dispatch.o',
             'gateway.o', 'boundary.o', 'extent.o', 'extent-bridge.o', 'extent-callers.o', 'provider.o',
             'predecessor-provider.o', 'predecessor-gateway.o', 'kernel.elf', 'bzImage']:
    data = (build/name).read_bytes()
    assert data == (baseline/name).read_bytes(), name
    comparisons.append(dict(path=name, bytes=len(data), sha256=digest(data), unchanged=True))
for name in ['static-elf-abi-fixture.elf', 'initramfs.cpio', 'extent-call-inventory.json',
             'process-access-inventory.json', 'process-access-origins.json', 'process-byte-origins.json',
             'first-image-exit/serial.log']:
    assert (build/name).read_bytes() == (baseline/name).read_bytes(), name
observed = json.loads((build/'first-image-exit/observations.json').read_text())
assert observed['passed'] and observed['callback_observation']['passed']
assert sum(row['count'] for row in observed['callback_observation']['counts']) == 77
assert observed['extent_call_coverage']['reached_sites'] == 38
data = dict(passed=True, cases=verified, scalar_counts=dict(counts), kernel_comparisons=comparisons,
    native_extent_first_reach=38, native_callback_returns=77,
    serial_bytes=len((build/'first-image-exit/serial.log').read_bytes()),
    serial_sha256=digest((build/'first-image-exit/serial.log').read_bytes()),
    scope='Independent GNU local decoding, scalar widths/directions/ranges and backward unique-predecessor checks; unchanged artifact/native witness. Not complete root/callee/exception contracts, helper soundness, native execution of each transfer, stable lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(data, indent=2)+'\n')
print(json.dumps({k: v for k, v in data.items() if k not in ['cases', 'kernel_comparisons']}))
