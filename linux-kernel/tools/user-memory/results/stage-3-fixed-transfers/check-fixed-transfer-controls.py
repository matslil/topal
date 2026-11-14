"""Offline metadata corruption controls, with unchanged first-party ELF inputs."""
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

build, output = map(Path, sys.argv[1:3])
output.mkdir(parents=True, exist_ok=True)
graph = json.loads((build/'process-access-inventory.json').read_text())
origins = json.loads((build/'process-access-origins.json').read_text())
results = []
for name in ['missing-edge', 'extra-edge', 'missing-root', 'instruction-bytes', 'missing-access-origin']:
    work = output/name
    work.mkdir()
    for file in ['provider.o', 'kernel.elf', 'extent-call-inventory.json']:
        shutil.copyfile(build/file, work/file)
    changed, changed_origins = copy.deepcopy(graph), copy.deepcopy(origins)
    if name == 'missing-edge':
        changed['edges'].remove(next(e for e in changed['edges'] if e['source'] == 93535 and e['target'] == 93541))
    elif name == 'extra-edge':
        changed['edges'].append(dict(source=133237, target=93541, kind='direct-transfer'))
    elif name == 'missing-root':
        changed['roots'].pop()
    elif name == 'instruction-bytes':
        changed['instructions'][0]['bytes'] = '4150'
    else:
        changed_origins['accesses'].remove(next(a for a in changed_origins['accesses'] if a['offset'] == 93541))
    data = (json.dumps(changed, indent=2)+'\n').encode()
    (work/'process-access-inventory.json').write_bytes(data)
    changed_origins['process_graph_sha256'] = hashlib.sha256(data).hexdigest()
    (work/'process-access-origins.json').write_text(json.dumps(changed_origins, indent=2)+'\n')
    command = [sys.executable, 'linux-kernel/tools/user-memory/audit_fixed_transfers.py', str(work)]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode != 0 and b'AssertionError' in result.stdout, (name, result.stdout)
    assert (build/'kernel.elf').read_bytes() == (work/'kernel.elf').read_bytes()
    record = dict(name=name, rejected=True, exit_code=result.returncode,
        scope='Diagnostic graph/join JSON mutation only, including updated outer graph digest; unchanged first-party ELF, no VM or guest-state mutation')
    (work/'rejection.json').write_text(json.dumps(record, indent=2)+'\n')
    results.append(record)
report = dict(passed=True, rejections=results,
    scope='Consistency controls, not authentication against replacing an entire producer or native admission/lifetime evidence')
(output/'controls.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(dict(passed=True, rejected=len(results))))
