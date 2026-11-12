"""Inspect only: changed first-party ELF files are never executed or booted."""
import copy
import json
from pathlib import Path
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_pipe_fixed_outputs.py').resolve()
sys.path.insert(0, str(reader.parent))
from inspect_call_inventory import Elf, digest

build, output = map(Path, sys.argv[1:3])
output.mkdir(parents=True, exist_ok=False)
census = json.loads((build/'extent-call-inventory.json').read_text())
graph = json.loads((build/'process-access-inventory.json').read_text())
origins = json.loads((build/'process-access-origins.json').read_text())
positive = subprocess.run([sys.executable, str(reader), str(build)],
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
(output/'positive.log').write_bytes(positive.stdout)
assert positive.returncode == 0, positive.stdout
expected = (build/'pipe-fixed-output-audit.json').read_bytes()
report = json.loads(expected)
pair, count = report['cases']
leaf = report['callee_contracts'][0]
_, body = Elf(build/'kernel.elf').function(census['function'])
pair_rows = pair['local_instructions']
move = next(r['offset'] for r in pair_rows if r['bytes'] == '4d89e6')
controls = [
    ('pair-count', pair['call_offset']-8, b'\x08', b'\x07'),
    ('pair-candidate', pair['candidate_setup_offset']+5, b'\x38', b'\x40'),
    ('pair-preparation-r8-write', move+2, b'\xe6', b'\xe0'),
    ('pair-output-base', pair['access_instructions'][0]['offset']+2, b'\x80', b'\x81'),
    ('pair-readback-base', pair['access_instructions'][1]['offset']+2, b'\x98', b'\x99'),
    ('pair-carry-polarity', pair['success_edge'][0]+1, b'\x82', b'\x83'),
    ('count-output-base', count['access_instructions'][0]['offset']+2, b'\x00', b'\x01'),
    ('count-length', count['call_offset']-8, b'\x04', b'\x05'),
    ('count-source-field', count['local_instructions'][1]['offset']+3, b'\x28', b'\x20'),
    ('lookup-r8-write', leaf['entry']+20+2, b'\xe2', b'\xe0'),
    ('lookup-search-bound', leaf['entry']+30+2, b'\x04', b'\x05'),
    ('lookup-ring-limit', leaf['entry']+115+3, b'\x00', b'\x01'),
]
results = []
for name, offset, before, after in controls:
    assert len(before) == len(after) and body[offset:offset+len(before)] == before, (name,offset)
    work = output/name
    work.mkdir()
    for filename in ['provider.o','kernel.elf']:
        elf = Elf(build/filename)
        symbol, function = elf.function(census['function'])
        section = elf.sections[symbol['section']]
        at = section[4]+symbol['value']-section[3]+offset
        data = bytearray(elf.data)
        assert data[at:at+len(before)] == before
        data[at:at+len(after)] = after
        (work/filename).write_bytes(data)
    provider, kernel = Elf(work/'provider.o'), Elf(work/'kernel.elf')
    _, supplied = provider.function(census['function'])
    _, linked = kernel.function(census['function'])
    cc, cg, co = copy.deepcopy(census), copy.deepcopy(graph), copy.deepcopy(origins)
    cc['function_sha256'] = digest(supplied)
    cb = (json.dumps(cc,indent=2)+'\n').encode()
    (work/'extent-call-inventory.json').write_bytes(cb)
    for row in cg['instructions']:
        if row['offset'] <= offset < row['offset']+row['size']:
            row['bytes'] = linked[row['offset']:row['offset']+row['size']].hex()
            if name == 'pair-carry-polarity':
                row['mnemonic'] = 'jae'
                row['instruction'] = row['instruction'].replace('jb\t','jae\t')
            break
    else:
        raise AssertionError(('control is absent from actual graph',name))
    cg.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
              linked_function_sha256=digest(linked),extent_inventory_sha256=digest(cb))
    gb = (json.dumps(cg,indent=2)+'\n').encode()
    (work/'process-access-inventory.json').write_bytes(gb)
    co.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
              linked_function_sha256=digest(linked),process_graph_sha256=digest(gb))
    for access in co['accesses']:
        if access['offset'] <= offset < access['offset']+access['size']:
            access['bytes'] = linked[access['offset']:access['offset']+access['size']].hex()
    (work/'process-access-origins.json').write_text(json.dumps(co,indent=2)+'\n')
    result = subprocess.run([sys.executable,str(reader),str(work)],
                            stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode != 0 and b'AssertionError' in result.stdout, (name,result.stdout)
    assert b'pipe instruction contract' in result.stdout or name == 'pair-carry-polarity', (name,result.stdout)
    results.append(dict(name=name,offset=offset,original_bytes=before.hex(),changed_bytes=after.hex(),
        rejected=True,exit_code=result.returncode,kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),log_sha256=digest(result.stdout)))
assert (build/'pipe-fixed-output-audit.json').read_bytes() == expected
record = dict(passed=True,positive_report_sha256=digest(expected),reader_sha256=digest(reader.read_bytes()),
    controls=results,scope='Offline first-party ELF/metadata counterexamples with outer hashes rebound. These changed files are never executed, booted or loaded in a VM. No provenance authentication, native memory safety or lifetime claim')
(output/'controls.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results),output=str(output/'controls.json'))))
