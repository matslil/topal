"""Offline first-party ELF counterexamples; changed images are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_pipe_loops.py').resolve()
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
expected_report = (build/'pipe-loop-audit.json').read_bytes()
audit = json.loads(expected_report)
read,write = audit["cases"]
_,body = Elf(build/"kernel.elf").function(census["function"])
find = lambda case,raw:next(r["offset"] for r in case["local_instructions"] if r["bytes"]==raw)
controls = []
for label,case in [("read",read),("write",write)]:
 guard=case["count_guard"]
 controls += [(label+"-zero-count",guard[0]["offset"]+3,b"\x00",b"\x01",None),
  (label+"-count-cap",guard[2]["offset"]+3,b"\x00",b"\x01",None),
  (label+"-remaining-count",find(case,"4c89c9")+2,b"\xc9",b"\xc1",None),
  (label+"-ring-mask",find(case,"4981e7ff0f0000")+4,b"\x0f",b"\x1f",None),
  (label+"-remaining-decrement",find(case,"4881c1ffffffff")+3,b"\xff",b"\xfe",None)]
controls += [
 ("read-candidate",find(read,"4c89c7")+2,b"\xc7",b"\xcf",None),
 ("write-candidate",find(write,"4c89c6")+2,b"\xc6",b"\xce",None),
 ("read-stride",find(read,"4881c701000000")+3,b"\x01",b"\x02",None),
 ("write-stride",find(write,"4881c601000000")+3,b"\x01",b"\x02",None),
 ("ring-erasure",find(read,"c60600")+2,b"\x00",b"\x01",None),
 ("read-user-readback",find(read,"3807")+1,b"\x07",b"\x06",None),
 ("carry-polarity",read["carry_gate"]+1,b"\x82",b"\x83",None)]
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
assert (build/'pipe-loop-audit.json').read_bytes() == expected_report
(output/'controls.json').write_text(json.dumps(dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    positive_report_sha256=digest(expected_report),controls=results,
    scope='Positive-backed offline first-party kernel/provider ELF changes with rebound outer identities; no changed image is executed, booted or loaded into a VM. Not source-provenance authentication or native fault/owner/lifetime qualification'),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
