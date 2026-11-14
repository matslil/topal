"""Offline first-party ELF counterexamples; changed images are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_stat_outputs.py').resolve()
sys.path.insert(0,str(reader.parent))
from inspect_call_inventory import Elf,digest

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
census = json.loads((build/'extent-call-inventory.json').read_text())
graph = json.loads((build/'process-access-inventory.json').read_text())
origins = json.loads((build/'process-access-origins.json').read_text())
positive = subprocess.run([sys.executable,str(reader),str(build)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(output/'positive.log').write_bytes(positive.stdout)
assert positive.returncode == 0,positive.stdout[-2000:]
expected_report = (build/'stat-output-audit.json').read_bytes()
audit = json.loads(expected_report)
fd,namespace = audit['cases']
metadata = next(r for r in audit['callee_contracts'] if r['kind'] == 'saved-metadata-validator')
wrapper = next(r for r in audit['callee_contracts'] if r['kind'] == 'metadata-check-wrapper')
_,body = Elf(build/'kernel.elf').function(census['function'])
copy_at = fd['copy_offset']
body_rows = [r for r in metadata['instructions'] if metadata['entry']+12 <= r['offset'] < metadata['restore_offset']]
load = next(r['offset'] for r in body_rows if r['bytes'] == '498b8000000000')
constant = next(r['offset'] for r in body_rows if r['bytes'].startswith('48b8'))
success_jumps = [r['offset'] for r in body_rows if r['bytes'].startswith('e9') and
    r['offset']+5+struct.unpack_from('<i',bytes.fromhex(r['bytes']),1)[0] == metadata['restore_offset']]
assert len(success_jumps) == 1
jump = success_jumps[0]
controls = [
    ('descriptor-candidate',fd['call_offset']-11,b'\xf0',b'\xf8',None),
    ('namespace-captured-field',namespace['prelude'][0]['offset']+5,b'\x48',b'\x40',None),
    ('descriptor-admitted-length',fd['call_offset']-8,b'\x90',b'\x8f',None),
    ('namespace-admitted-length',namespace['call_offset']-8,b'\x90',b'\x8f',None),
    ('copy-candidate',copy_at-11,b'\xc7',b'\xcf',None),
    ('copy-length',copy_at-8,b'\x90',b'\x91',None),
    ('metadata-saved-candidate',metadata['entry']+5,b'\x50',b'\x51',None),
    ('metadata-restored-candidate',metadata['restore_offset']+7,b'\x58',b'\x59',None),
    ('metadata-body-store',load+1,b'\x8b',b'\x89',None),
    ('metadata-body-stack-change',constant+1,b'\xb8',b'\xbc',None),
    ('metadata-restoration-bypass',jump+1,body[jump+1:jump+5],
        struct.pack('<i',metadata['normal_returns'][0]-(jump+5)),metadata['normal_returns'][0]),
    ('wrapper-restored-argument',wrapper['entry']+16,b'\x5f',b'\x5e',None),
    ('wrapper-retirement-window',wrapper['entry']+21,b'\xf0',b'\xe8',None),
    ('carry-polarity',fd['carry_gate']+1,b'\x82',b'\x83',None),
]
results = []
for name,offset,before,after,target in controls:
    assert len(before) == len(after) and body[offset:offset+len(before)] == before,(name,offset)
    work = output/name
    work.mkdir()
    for filename in ['provider.o','kernel.elf']:
        elf = Elf(build/filename)
        symbol,function = elf.function(census['function'])
        section = elf.sections[symbol['section']]
        at = section[4]+symbol['value']-section[3]+offset
        data = bytearray(elf.data)
        assert data[at:at+len(before)] == before
        data[at:at+len(after)] = after
        (work/filename).write_bytes(data)
    provider,kernel = Elf(work/'provider.o'),Elf(work/'kernel.elf')
    _,supplied = provider.function(census['function'])
    symbol,linked = kernel.function(census['function'])
    cc,cg,co = copy.deepcopy(census),copy.deepcopy(graph),copy.deepcopy(origins)
    cc['function_sha256'] = digest(supplied)
    cb = (json.dumps(cc,indent=2)+'\n').encode()
    (work/'extent-call-inventory.json').write_bytes(cb)
    for row in cg['instructions']:
        if row['offset'] <= offset < row['offset']+row['size']:
            row['bytes'] = linked[row['offset']:row['offset']+row['size']].hex()
            if name == 'carry-polarity':
                row['mnemonic'] = 'jae'
                row['instruction'] = row['instruction'].replace('jb\t','jae\t')
            if target is not None:
                row['instruction'] = re.sub(r'0x[0-9a-f]+',hex(symbol['value']+target),row['instruction'],count=1)
                for edge in cg['edges']:
                    if edge['source'] == row['offset'] and edge['kind'] == 'direct-transfer': edge['target'] = target
            break
    else: raise AssertionError(('absent control node',name))
    cg.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
        linked_function_sha256=digest(linked),extent_inventory_sha256=digest(cb))
    gb = (json.dumps(cg,indent=2)+'\n').encode()
    (work/'process-access-inventory.json').write_bytes(gb)
    co.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
        linked_function_sha256=digest(linked),process_graph_sha256=digest(gb))
    for row in co['accesses']:
        if row['offset'] <= offset < row['offset']+row['size']:
            row['bytes'] = linked[row['offset']:row['offset']+row['size']].hex()
    (work/'process-access-origins.json').write_text(json.dumps(co,indent=2)+'\n')
    result = subprocess.run([sys.executable,str(reader),str(work)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode != 0 and b'AssertionError' in result.stdout,(name,result.stdout[-2000:])
    if name == 'metadata-restoration-bypass': assert b'metadata return bypasses restoration' in result.stdout
    results.append(dict(name=name,offset=offset,before=before.hex(),after=after.hex(),
        rejected=True,exit_code=result.returncode,kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),log_sha256=digest(result.stdout)))
assert (build/'stat-output-audit.json').read_bytes() == expected_report
(output/'controls.json').write_text(json.dumps(dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    effects_reader_sha256=digest((reader.parent/'stat_instruction_effects.py').read_bytes()),
    positive_report_sha256=digest(expected_report),controls=results,
    scope='Positive-backed offline first-party kernel/provider ELF changes with rebound outer identities and a coherent changed branch for restoration bypass; no changed image is executed, booted or loaded into a VM. Not source-provenance authentication or native fault/owner/lifetime qualification'),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
