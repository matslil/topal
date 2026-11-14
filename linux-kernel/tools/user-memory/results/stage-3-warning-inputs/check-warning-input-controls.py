"""Offline first-party ELF counterexamples; changed images are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_warning_inputs.py').resolve()
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
expected_report = (build/'warning-input-audit.json').read_bytes()
audit = json.loads(expected_report)
case = audit["cases"][0]
_,body = Elf(build/"kernel.elf").function(census["function"])
find = lambda rows,raw:next(r["offset"] for r in rows if r["bytes"]==raw)
pre,dispatch = case["prelude"],case["dispatch"]
controls=[
 ("one-byte-count",find(pre,"41b901000000")+2,b"\x01",b"\x02",None),
 ("four-byte-count",find(pre,"41b904000000")+2,b"\x04",b"\x05",None),
 ("forty-byte-count",find(pre,"41b928000000")+2,b"\x28",b"\x29",None),
 ("requested-count-binding",find(pre,"4c39ca")+2,b"\xca",b"\xcb",None),
 ("captured-candidate",find(pre,"65488b342540010000")+5,b"\x40",b"\x48",None),
 ("admission-candidate",find(pre,"4989c0")+2,b"\xc0",b"\xc8",None),
 ("admission-count",find(pre,"4989d1")+2,b"\xd1",b"\xd9",None),
 ("selector-reload",dispatch[0]["offset"]+5,b"\x68",b"\x70",None),
 ("carry-polarity",case["carry_gate"]+1,b"\x82",b"\x83",None),
 ("four-byte-base",case["variants"][0]["entry"]+2,b"\x06",b"\x07",None),
 ("four-byte-width",case["variants"][0]["entry"],b"\x44",b"\x4c",None),
 ("forty-byte-last-offset",case["variants"][1]["access_instructions"][-1]["offset"]+3,b"\x20",b"\x28",None),
 ("newline-base",case["variants"][2]["entry"]+2,b"\x06",b"\x07",None),
 ("selector-dispatch",dispatch[1]["offset"]+2,b"\x01",b"\x00",None)]
branch_at=dispatch[2]['offset']
target=case['variants'][2]['entry']
controls.append(('wrong-variant-edge',branch_at+2,
    body[branch_at+2:branch_at+6],struct.pack('<i',target-branch_at-6),target))
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
    results.append(dict(name=name,offset=offset,before=before.hex(),after=after.hex(),
        rejected=True,exit_code=result.returncode,kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),log_sha256=digest(result.stdout)))
assert (build/'warning-input-audit.json').read_bytes() == expected_report
(output/'controls.json').write_text(json.dumps(dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    positive_report_sha256=digest(expected_report),controls=results,
    scope='Positive-backed offline first-party kernel/provider ELF changes with rebound outer identities; no changed image is executed, booted or loaded into a VM. Not source-provenance authentication or native fault/owner/lifetime qualification'),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
