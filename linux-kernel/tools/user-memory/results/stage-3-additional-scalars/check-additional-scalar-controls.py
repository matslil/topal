"""Offline first-party artifact controls; changed artifacts are never executed."""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
inventory=json.loads((build/'extent-call-inventory.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
origins=json.loads((build/'process-access-origins.json').read_text())
controls=[('shifted-final-byte',33606,0x06,0x07),('shifted-candidate',33566,0x06,0x07),
 ('path-final-byte',56116,0x0c,0x0d),('byte-candidate',44452,0x03,0x06),
 ('header-reload',41464,0x40,0x48),('signal-final-word',36062,0x18,0x20),
 ('clear-store-width',194651,0x45,0x4d),('carry-polarity',27876,0x82,0x83)]
results=[]
for name,offset,expected,replacement in controls:
    work=output/name
    work.mkdir()
    for filename in ['provider.o','kernel.elf']:
        elf=Elf(build/filename)
        symbol,body=elf.function(inventory['function'])
        section=elf.sections[symbol['section']]
        at=section[4]+symbol['value']-section[3]+offset
        changed=bytearray(elf.data)
        assert changed[at]==body[offset]==expected,(name,filename,offset)
        changed[at]=replacement
        (work/filename).write_bytes(changed)
    provider,kernel=Elf(work/'provider.o'),Elf(work/'kernel.elf')
    _,original=provider.function(inventory['function'])
    _,linked=kernel.function(inventory['function'])
    census,cg,co=copy.deepcopy(inventory),copy.deepcopy(graph),copy.deepcopy(origins)
    census['function_sha256']=digest(original)
    census_bytes=(json.dumps(census,indent=2)+'\n').encode()
    (work/'extent-call-inventory.json').write_bytes(census_bytes)
    for row in cg['instructions']:
        if row['offset']<=offset<row['offset']+row['size']:
            row['bytes']=linked[row['offset']:row['offset']+row['size']].hex()
            if name=='carry-polarity':
                row['mnemonic']='jae'
                row['instruction']=row['instruction'].replace('jb\t','jae\t')
            break
    else:
        raise AssertionError(('control not in graph',name))
    cg.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
        linked_function_sha256=digest(linked),extent_inventory_sha256=digest(census_bytes))
    graph_bytes=(json.dumps(cg,indent=2)+'\n').encode()
    (work/'process-access-inventory.json').write_bytes(graph_bytes)
    co.update(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
        linked_function_sha256=digest(linked),process_graph_sha256=digest(graph_bytes))
    for access in co['accesses']:
        if access['offset']<=offset<access['offset']+access['size']:
            access['bytes']=linked[access['offset']:access['offset']+access['size']].hex()
    (work/'process-access-origins.json').write_text(json.dumps(co,indent=2)+'\n')
    result=subprocess.run([sys.executable,'linux-kernel/tools/user-memory/audit_additional_scalars.py',str(work)],
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout)
    record=dict(name=name,rejected=True,offset=offset,expected_byte=expected,changed_byte=replacement,
        exit_code=result.returncode,scope='Offline first-party provider/kernel byte corruption with rebound outer census/graph/join identities; never compiled, booted, loaded into a VM or used to alter guest state; not source-provenance authentication')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    results.append(record)
(output/'controls.json').write_text(json.dumps(dict(passed=True,rejections=results),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
