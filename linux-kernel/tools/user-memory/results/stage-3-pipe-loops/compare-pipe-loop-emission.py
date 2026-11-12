"""Reconstruct the new complete process body from two guard insertions."""
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest
from inspect_callback_bindings import relocation_entries

old,new,output = map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
census = json.loads((old/'extent-call-inventory.json').read_text())
name = census['function']
before,after = Elf(old/'provider.o'),Elf(new/'provider.o')
old_symbol,old_body = before.function(name)
new_symbol,new_body = after.function(name)
graph = json.loads((old/'process-access-inventory.json').read_text())
audit = json.loads((new/'pipe-loop-audit.json').read_text())
assert audit['passed'] and len(audit['cases']) == 2
old_calls = [next(r['offset'] for r in census['calls'] if 'pipe-'+d+'-count-ready' in r['nearest_labels'])
    for d in ['read','write']]
assert len(new_body) == len(old_body)+52
def shifted(at): return at+26*sum(at>=call for call in old_calls)
def target_shifted(at): return shifted(at)-(26 if at in old_calls else 0)
relocations = relocation_entries(before,old_symbol['section'])
relative = []
expected = bytearray(old_body)
for row in graph['instructions']:
    at,size = row['offset'],row['size']
    raw = old_body[at:at+size]
    field = None
    if raw[:1] in [b'\xe8',b'\xe9'] and size==5: field = 1
    elif raw[:1]==b'\x0f' and 0x80<=raw[1]<=0x8f and size==6: field = 2
    elif size==7 and raw[0] in [0x48,0x4c] and raw[1]==0x8d and raw[2]&0xc7==5: field = 3
    if field is None or old_symbol['value']+at+field in relocations: continue
    target = at+size+struct.unpack_from('<i',raw,field)[0]
    assert 0<=target<=len(old_body),('unbound relative process target',at,target)
    value = target_shifted(target)-(shifted(at)+size)
    struct.pack_into('<i',expected,at+field,value)
    if value != struct.unpack_from('<i',raw,field)[0]:
        relative.append(dict(offset=at,field=field,old_target=target,new_target=target_shifted(target)))
insertions = []
for old_call,case in reversed(list(zip(old_calls,audit['cases']))):
    at = case['call_offset']-26
    assert at==shifted(old_call)-26
    guard = new_body[at:at+26]
    assert guard==b''.join(bytes.fromhex(r['bytes']) for r in case['count_guard'])
    expected[old_call:old_call] = guard
    insertions.append(dict(old_offset=old_call,new_offset=at,bytes=guard.hex()))
assert bytes(expected)==new_body,'Unexpected process-body byte change outside guards and derived relative displacements'
old_reloc = {field-old_symbol['value']:row for field,row in relocations.items()
    if 0<=field-old_symbol['value']<len(old_body)}
new_reloc = relocation_entries(after,new_symbol['section'])
new_reloc = {field-new_symbol['value']:row for field,row in new_reloc.items()
    if 0<=field-new_symbol['value']<len(new_body)}
assert set(new_reloc)=={shifted(at) for at in old_reloc}
for at,(symbol,kind,addend) in old_reloc.items():
    other,other_kind,other_addend = new_reloc[shifted(at)]
    assert (symbol['name'],kind,addend)==(other['name'],other_kind,other_addend)
unchanged_objects = []
for filename in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
                 'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o','predecessor-gateway.o']:
    data = (old/filename).read_bytes()
    assert data==(new/filename).read_bytes(),filename
    unchanged_objects.append(dict(path=filename,sha256=digest(data),bytes=len(data)))
report = dict(passed=True,function=name,old_provider_sha256=digest(before.data),new_provider_sha256=digest(after.data),
    old_function_sha256=digest(old_body),new_function_sha256=digest(new_body),
    old_function_bytes=len(old_body),new_function_bytes=len(new_body),insertions=list(reversed(insertions)),
    adjusted_relative_displacements=relative,process_relocations=len(old_reloc),
    unchanged_component_objects=unchanged_objects,
    scope='Complete provider process-body reconstruction by two exact guarded count insertions and mechanically derived direct/RIP-relative displacement changes, with equal shifted relocation identities and fourteen byte-identical components including predecessor provider/gateway. Not whole producer, native semantics or ownership/lifetime qualification')
(output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,body_bytes_added=52,adjusted_relative_displacements=len(relative),unchanged_objects=len(unchanged_objects))))
