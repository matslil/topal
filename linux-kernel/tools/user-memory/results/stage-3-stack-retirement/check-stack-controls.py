"""Parse-only ELF counterexamples for every emitted cleanup block."""
import copy
import json
from pathlib import Path
import struct
import subprocess
import sys

reader=Path('linux-kernel/tools/user-memory/audit_stack_retirement.py').resolve()
sys.path.insert(0,str(reader.parent))
from inspect_call_inventory import Elf,digest

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
load=lambda name:json.loads((build/name).read_text())
census,graph,origins,files,random=map(load,['extent-call-inventory.json','process-access-inventory.json',
    'process-access-origins.json','file-input-audit.json','random-output-audit.json'])
positive=subprocess.run([sys.executable,str(reader),str(build)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(output/'positive.log').write_bytes(positive.stdout)
assert positive.returncode==0,positive.stdout[-2000:]
expected=(build/'stack-retirement-audit.json').read_bytes()
audit=json.loads(expected)
_,body=Elf(build/'kernel.elf').function(census['function'])
controls=[]
for block in audit['blocks']:
    at=block['instructions'][-1]['offset']+4
    before=body[at:at+4]
    after=struct.pack('<i',struct.unpack('<i',before)[0]+8)
    controls.append((block['name']+'-wrong-slot',at,before,after))
first=audit['blocks'][0]['start']
controls.extend([('nonzero-immediate',first+8,b'\x00',b'\x01'),
    ('wrong-base',first+3,b'\x24',b'\x25'),('flag-changing-opcode',first+1,b'\xc7',b'\x81')])
results=[]
for name,offset,before,after in controls:
    work=output/name
    work.mkdir()
    assert len(before)==len(after) and body[offset:offset+len(before)]==before
    for filename in ['provider.o','kernel.elf']:
        elf=Elf(build/filename)
        symbol,function=elf.function(census['function'])
        section=elf.sections[symbol['section']]
        at=section[4]+symbol['value']-section[3]+offset
        data=bytearray(elf.data)
        assert data[at:at+len(before)]==before
        data[at:at+len(after)]=after
        (work/filename).write_bytes(data)
    provider,kernel=Elf(work/'provider.o'),Elf(work/'kernel.elf')
    _,supplied=provider.function(census['function'])
    _,linked=kernel.function(census['function'])
    cc,cg,co,cf,cr=map(copy.deepcopy,[census,graph,origins,files,random])
    cc['function_sha256']=digest(supplied)
    cb=(json.dumps(cc,indent=2)+'\n').encode()
    (work/'extent-call-inventory.json').write_bytes(cb)
    for row in cg['instructions']:
        if row['offset']<=offset<row['offset']+row['size']:
            row['bytes']=linked[row['offset']:row['offset']+row['size']].hex()
            break
    else: raise AssertionError(('absent control instruction',name))
    hashes=dict(provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),linked_function_sha256=digest(linked))
    cg.update(hashes,extent_inventory_sha256=digest(cb))
    gb=(json.dumps(cg,indent=2)+'\n').encode()
    (work/'process-access-inventory.json').write_bytes(gb)
    co.update(hashes,process_graph_sha256=digest(gb))
    for row in co['accesses']:
        if row['offset']<=offset<row['offset']+row['size']:
            row['bytes']=linked[row['offset']:row['offset']+row['size']].hex()
    ob=(json.dumps(co,indent=2)+'\n').encode()
    (work/'process-access-origins.json').write_bytes(ob)
    for filename,report in [('file-input-audit.json',cf),('random-output-audit.json',cr)]:
        report.update(hashes,graph_sha256=digest(gb),origin_join_sha256=digest(ob),census_sha256=digest(cb))
        (work/filename).write_text(json.dumps(report,indent=2)+'\n')
    result=subprocess.run([sys.executable,str(reader),str(work)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout[-2000:])
    results.append(dict(name=name,offset=offset,before=before.hex(),after=after.hex(),rejected=True,
        exit_code=result.returncode,kernel_sha256=digest(kernel.data),provider_sha256=digest(provider.data),log_sha256=digest(result.stdout)))
assert (build/'stack-retirement-audit.json').read_bytes()==expected
report=dict(passed=True,reader_sha256=digest(reader.read_bytes()),positive_report_sha256=digest(expected),controls=results,
    scope='Positive-backed offline first-party provider/kernel ELF mutations with rebound census/graph/origin/report identities. Changed artifacts are parsed only, never executed, booted or loaded into a VM. Not source authentication, actual owners/lifetime or production qualification')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
