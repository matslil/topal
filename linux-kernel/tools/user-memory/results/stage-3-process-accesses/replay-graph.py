"""Replay the frozen linked graph with LLVM/GNU decoders and no VM execution."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
work = Path(sys.argv[1]).resolve()
tool = Path(sys.argv[2]).resolve()
assert tool.is_file()
work.mkdir(parents=True, exist_ok=False)
digest = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((root/'source-identity.json').read_text())
for name in ['inspect_call_inventory.py', 'inventory_process_accesses.py']:
    row = identity['selected_inputs']['linux-kernel/tools/user-memory/'+name]
    data = gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data) == row['sha256']
    (work/name).write_bytes(data)
for name, row in json.loads((root/'artifact-identity.json').read_text()).items():
    compressed = (root/row['snapshot']).read_bytes()
    assert digest(compressed) == row['snapshot_sha256']
    data = gzip.decompress(compressed)
    assert len(data) == row['bytes'] and digest(data) == row['sha256']
    (work/name).write_bytes(data)
expected = json.loads((work/'process-access-inventory.json').read_text())
census = (root/'extent-call-inventory.json').read_bytes()
assert digest(census) == expected['extent_inventory_sha256']
(work/'extent-call-inventory.json').write_bytes(census)
commands = [
    [sys.executable, str(work/'inventory_process_accesses.py'), str(work), str(tool)],
]
(work/'first-image-exit').mkdir()
(work/'first-image-exit/observations.json').write_bytes((root/'guest/observations.json').read_bytes())
commands.append([sys.executable,str(work/'inventory_process_accesses.py'),'--verify-coverage',str(work)])
for index, command in enumerate(commands):
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (work/f'{index}.log').write_bytes(result.stdout)
    assert result.returncode == 0, result.stdout.decode(errors='replace')
assert json.loads((work/'process-access-inventory.json').read_text()) == expected
assert json.loads((work/'process-access-coverage.json').read_text()) == json.loads(
    (root/'process-access-coverage.json').read_text())
gnu = (root/'gnu/verify-graph.gdb').read_text().splitlines()
replacements = {'file ': 'file '+str(work/'kernel.elf'),
    'graph_path=': 'graph_path='+repr(str(work/'process-access-inventory.json')),
    'kernel_path=': 'kernel_path='+repr(str(work/'kernel.elf')),
    'result_path=': 'result_path='+repr(str(work/'gnu-verification.json'))}
for prefix, replacement in replacements.items():
    matches = [index for index,line in enumerate(gnu) if line.startswith(prefix)]
    assert len(matches) == 1
    gnu[matches[0]] = replacement
(work/'verify-graph.gdb').write_text('\n'.join(gnu)+'\n')
result = subprocess.run(['gdb','-nx','-batch','-x',str(work/'verify-graph.gdb')],
                        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(work/'gnu.log').write_bytes(result.stdout)
assert result.returncode == 0, result.stdout.decode(errors='replace')
assert json.loads((work/'gnu-verification.json').read_text()) == json.loads(
    (root/'gnu/gnu-verification.json').read_text())
report = dict(passed=True, exact_llvm_graph_equal=True, exact_gnu_report_equal=True,
    exact_observation_binding_equal=True, instructions=expected['reached_instructions'],
    scope='Offline archived first-party linked graph and recorded observation binding; no kernel rebuild, new VM/native execution or complete user-access qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
