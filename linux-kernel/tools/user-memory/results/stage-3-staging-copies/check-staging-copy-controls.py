"""Offline first-party artifact controls; changed artifacts are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest
from audit_fixed_transfers import reachable

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
census=json.loads((build/'extent-call-inventory.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
origins=json.loads((build/'process-access-origins.json').read_text())
audit=json.loads((build/'staging-copy-audit.json').read_text())
positive_expected=(build/'staging-copy-audit.json').read_bytes()
positive=subprocess.run([sys.executable,'linux-kernel/tools/user-memory/audit_staging_copies.py',str(build)],
    stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(output/'positive.log').write_bytes(positive.stdout)
assert positive.returncode==0,positive.stdout
assert (build/'staging-copy-audit.json').read_bytes()==positive_expected
cases={r['name']:r for r in audit['cases']}
_,body=Elf(build/'kernel.elf').function(census['function'])
console=cases['console-input']
descriptor=cases['descriptor-read-output']
cwd=cases['cwd-output']
sparse=cases['sparse-read-output']
wipe=body.index(bytes.fromhex('48 b9 c0 00 00 00 00 00 00 00'),console['success_offset'])
assert wipe<console['copy_offset']
controls=[('console-cap',console['prelude_start']+3,0x00,0x01),
    ('console-candidate',console['call_offset']-4,0xf0,0xf8),
    ('uart-retained-count',console['copy_offset']+7,0xce,0xc6),
    ('descriptor-retained-candidate',descriptor['copy_offset']-4,0xc7,0xcf),
    ('cwd-count-definition',cwd['count_initializers'][0]['offset']+2,None,0x06),
    ('sparse-count-cap',sparse['prelude_start']+6,0x00,0x01),
    ('lookup-register',audit['callee_contracts'][0]['entry']+2,0xe3,0xeb),
    ('staging-wipe-size',wipe+2,0xc0,0xc1),
    ('carry-polarity',console['carry_gate']+1,0x82,0x83),
    ('console-cap-exit',console['prelude_start']+9,None,
        struct.pack('<i',console['call_offset']-(console['prelude_start']+13)).hex()),
    ('console-zero-exit',console['prelude_start']+22,None,
        struct.pack('<i',console['call_offset']-(console['prelude_start']+26)).hex())]
results=[]
for name,offset,expected,replacement in controls:
    work=output/name
    work.mkdir()
    replacement=bytes.fromhex(replacement) if isinstance(replacement,str) else bytes([replacement])
    expected=body[offset:offset+len(replacement)] if expected is None else bytes([expected])
    for filename in ['provider.o','kernel.elf']:
        elf=Elf(build/filename)
        symbol,data=elf.function(census['function'])
        section=elf.sections[symbol['section']]
        at=section[4]+symbol['value']-section[3]+offset
        changed=bytearray(elf.data)
        assert changed[at:at+len(expected)]==data[offset:offset+len(expected)]==expected,(name,filename,offset)
        changed[at:at+len(replacement)]=replacement
        (work/filename).write_bytes(changed)
    provider,kernel=Elf(work/'provider.o'),Elf(work/'kernel.elf')
    _,original=provider.function(census['function'])
    _,linked=kernel.function(census['function'])
    cc,cg,co=copy.deepcopy(census),copy.deepcopy(graph),copy.deepcopy(origins)
    cc['function_sha256']=digest(original)
    census_bytes=(json.dumps(cc,indent=2)+'\n').encode()
    (work/'extent-call-inventory.json').write_bytes(census_bytes)
    for row in cg['instructions']:
        if row['offset']<=offset<row['offset']+row['size']:
            row['bytes']=linked[row['offset']:row['offset']+row['size']].hex()
            if name=='carry-polarity':
                row['mnemonic']='jae'
                row['instruction']=row['instruction'].replace('jb\t','jae\t')
            if name in ['console-cap-exit','console-zero-exit']:
                row['instruction']=re.sub(r'0x[0-9a-f]+',hex(kernel.function(census['function'])[0]['value']+console['call_offset']),row['instruction'],count=1)
                for edge in cg['edges']:
                    if edge['source']==row['offset'] and edge['kind']=='direct-transfer':
                        edge['target']=console['call_offset']
            break
    else: raise AssertionError(('control not in graph',name))
    if name in ['console-cap-exit','console-zero-exit']:
        successors={}
        for edge in cg['edges']: successors.setdefault(edge['source'],set()).add(edge['target'])
        closure=reachable({r['offset'] for r in cg['roots']},successors)
        cg['instructions']=[r for r in cg['instructions'] if r['offset'] in closure]
        cg['edges']=[r for r in cg['edges'] if r['source'] in closure]
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
    result=subprocess.run([sys.executable,'linux-kernel/tools/user-memory/audit_staging_copies.py',str(work)],
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout)
    if name in ['console-cap-exit','console-zero-exit']:
        assert b'console count refusal can reach admission' in result.stdout,result.stdout
    record=dict(name=name,rejected=True,offset=offset,expected_bytes=expected.hex(),changed_bytes=replacement.hex(),
        exit_code=result.returncode,scope='Offline first-party provider/kernel byte corruption with rebound outer identities; never executed, booted or loaded into a VM. Not source-provenance authentication')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    results.append(record)
(output/'controls.json').write_text(json.dumps(dict(passed=True,unchanged_positive_report_equal=True,rejections=results),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
