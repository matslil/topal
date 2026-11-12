"""Offline first-party ELF counterexamples; changed images are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_file_inputs.py').resolve()
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
expected_report = (build/'file-input-audit.json').read_bytes()
audit = json.loads(expected_report)
regular,positional = audit["cases"]
clock = audit["clock_contracts"][0]
leaf = clock["leaf"]
_,body = Elf(build/"kernel.elf").function(census["function"])
find = lambda rows,raw:next(r["offset"] for r in rows if r["bytes"]==raw)
controls=[]
for label,case in [("write",regular),("pwrite",positional)]:
 controls += [(label+"-candidate",find(case["local_instructions"],"4c89c6")+2,b"\xc6",b"\xce",None),
  (label+"-copy-count",find(case["local_instructions"],"4889e9"),b"\x48\x89\xe9",b"\x4c\x89\xc9",None),
  (label+"-counterpart",find(case["local_instructions"],"4881c780050000")+3,b"\x80",b"\x88",None),
  (label+"-count-cap",find(case["prelude"],"4881fd00010000")+4,b"\x01",b"\x02",None)]
controls += [
 ("live-counterpart-save",find(regular["local_instructions"],"57"),b"\x57",b"\x56",None),
 ("live-counterpart-restore",regular["post_copy_pop"]["offset"],b"\x5f",b"\x5e",None),
 ("clock-saved-count",clock["instructions"][10]["offset"],b"\x55",b"\x53",None),
 ("clock-restored-count",clock["instructions"][32]["offset"],b"\x5d",b"\x5b",None),
 ("clock-saved-candidate",clock["instructions"][14]["offset"]+1,b"\x50",b"\x51",None),
 ("clock-restored-candidate",clock["instructions"][28]["offset"]+1,b"\x58",b"\x59",None),
 ("rtc-table-binding",leaf["entry"]+5,b"\xa0",b"\xa8",None),
 ("rtc-stack-alias",find(leaf["instructions"],"4c89e7"),b"\x4c",b"\x48",None),
 ("rtc-stack-store",find(leaf["instructions"],"4989842480050000"),b"\x49",b"\x48",None),
 ("rtc-bcd-store-offset",find(leaf["instructions"],"49898500000000")+3,b"\x00",b"\x08",None),
 ("rtc-stack-destination",find(leaf["instructions"],"48b80000000000000000")+1,b"\xb8",b"\xbc",None),
 ("carry-polarity",regular["carry_gate"]+1,b"\x82",b"\x83",None),
 ("page-lookup-count",find(audit["page_lookup_contracts"][0]["instructions"],"498b8300000000")+2,b"\x83",b"\xab",None)]
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
    if name=="rtc-stack-alias": assert b"RTC leaf exposes a stack alias" in result.stdout
    results.append(dict(name=name,offset=offset,before=before.hex(),after=after.hex(),
        rejected=True,exit_code=result.returncode,kernel_sha256=digest(kernel.data),
        provider_sha256=digest(provider.data),log_sha256=digest(result.stdout)))
assert (build/'file-input-audit.json').read_bytes() == expected_report
(output/'controls.json').write_text(json.dumps(dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    positive_report_sha256=digest(expected_report),effects_reader_sha256=digest((reader.parent/"file_input_effects.py").read_bytes()),controls=results,
    scope='Positive-backed offline first-party kernel/provider ELF changes with rebound outer identities; no changed image is executed, booted or loaded into a VM. Not source-provenance authentication or native fault/owner/lifetime qualification'),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
