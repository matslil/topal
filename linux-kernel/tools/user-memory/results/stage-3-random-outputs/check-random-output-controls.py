"""Offline first-party ELF counterexamples; changed images are never executed."""
import copy
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

reader = Path('linux-kernel/tools/user-memory/audit_random_outputs.py').resolve()
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
expected_report = (build/'random-output-audit.json').read_bytes()
audit = json.loads(expected_report)

case=audit['cases'][0]
callee=audit['extract_contract']
_,body=Elf(build/'kernel.elf').function(census['function'])
find=lambda rows,raw:next(r['offset'] for r in rows if r['bytes']==raw)
pre,local=case['prelude'],case['local_instructions']
def control(name,offset,before,after,target=None):
    return name,offset,bytes.fromhex(before),bytes.fromhex(after),target
controls=[
 control('count-cap',pre[0]['offset']+3,'10','11'),
 control('admission-candidate',pre[-1]['offset']+2,'c0','c8'),
 control('admission-count',pre[-2]['offset']+2,'f1','f9'),
 control('initial-source',find(local,'4981c1a8000000')+3,'a8','b0'),
 control('reusable-source',find(local,'4981c1e0000000')+3,'e0','e8'),
 control('suspended-stack-save',find(local,'654889242510010000')+5,'10','18'),
 control('suspended-stack-restore',find(local,'65488b242510010000')+5,'10','18'),
 control('switched-stack-field',find(local,'65488b242508000000')+5,'08','10'),
 control('saved-count',find(local,'56'),'56','57'),
 control('restored-count',find(local,'5e'),'5e','5f'),
 control('key-offset',find(local,'4881c7c0000000')+3,'c0','c8'),
 control('loop-index-reset',find(local,'4531d2')+2,'d2','db'),
 control('loop-count',find(local,'4939f2')+2,'f2','fa'),
 control('private-load-index',case['load_offset']+3,'11','19'),
 control('user-store-base',case['start_offset']+3,'17','16'),
 control('user-readback-base',case['readback_offset']+3,'17','16'),
 control('loop-increment',find(local,'49ffc2')+2,'c2','c3'),
 control('carry-polarity',case['carry_gate']+1,'82','83'),
 control('extract-allocation',callee['allocations'][0]+3,'80','88'),
 control('extract-stack-alias',callee['entry']+27+2,'fa','e2')]
success_zero=body.find(bytes.fromhex('31c0488904244889442408'),callee['entry'],callee['releases'][0])
assert success_zero>=callee['entry']
controls.append(control('extract-zero-register',success_zero+1,'c0','c9'))
last_zero=next(r['offset'] for r in callee['instructions']
    if success_zero<r['offset']<callee['releases'][0] and r['bytes']=='4889442478')
controls.append(control('extract-uncleared-final-word',last_zero+4,'78','70'))
conditional=next(r for r in callee['instructions'] if r['bytes'].startswith('0f85'))
at,target=conditional['offset'],callee['releases'][0]
controls.append(('extract-wipe-bypass',at+2,body[at+2:at+6],struct.pack('<i',target-at-6),target))

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
assert (build/'random-output-audit.json').read_bytes() == expected_report
(output/'controls.json').write_text(json.dumps(dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    positive_report_sha256=digest(expected_report),effects_reader_sha256=digest((reader.parent/'random_extract_effects.py').read_bytes()),controls=results,
    scope='Positive-backed offline first-party kernel/provider ELF changes with rebound outer identities; no changed image is executed, booted or loaded into a VM. Not source-provenance authentication or native fault/owner/lifetime qualification'),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
