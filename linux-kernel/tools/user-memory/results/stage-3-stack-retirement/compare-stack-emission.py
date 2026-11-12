"""Reconstruct the complete changed process from exact retired-word stores."""
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest
from inspect_callback_bindings import relocation_entries

old,new,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
census=json.loads((old/'extent-call-inventory.json').read_text())
name=census['function']
before,after=Elf(old/'provider.o'),Elf(new/'provider.o')
old_symbol,old_body=before.function(name)
new_symbol,new_body=after.function(name)
graph=json.loads((old/'process-access-inventory.json').read_text())
linear_path=Path(__file__).with_name('baseline-linear-transcript.json')
if not linear_path.exists():
    linear_path=Path('linux-kernel/.cache/x86_64/stack-retirement-readbacks/baseline-linear/transcript.json')
linear_raw=linear_path.read_bytes()
linear=json.loads(linear_raw)
graph_nodes={row['offset']:row for row in graph['instructions']}
covered={at for row in graph['instructions'] for at in range(row['offset'],row['offset']+row['size'])}
extra=[]
for row in linear:
    at,size=row['offset'],row['size']
    if any(p in covered for p in range(at,at+size)):continue
    extra.append(dict(offset=at,size=size,bytes=old_body[at:at+size].hex(),gnu_assembly=row['assembly']))
files=json.loads((old/'file-input-audit.json').read_text())
random=json.loads((old/'random-output-audit.json').read_text())
clock,=files['clock_contracts']
rows=clock['instructions']
rtc=next(r for r in rows if r['bytes'].startswith('e8'))
calls=[at for at in range(len(old_body)-4) if old_body[at]==0xe8 and
    at+5+struct.unpack_from('<i',old_body,at+1)[0]==clock['entry']]
assert len(calls)==9
case,=random['cases']
extract=case['extract_call']
assert old_body[extract+5:extract+10]==bytes.fromhex('5e5f59415b')
insertions=[(rtc['offset']+5,1),(rows[-1]['offset'],12),
    (extract+5,1),(extract+10,4)]+[(at+5,1) for at in calls]
insertions.sort()
assert len(insertions)==13 and sum(words for _,words in insertions)==27
assert len(new_body)==len(old_body)+324
def shifted(at): return at+sum(12*words for offset,words in insertions if offset<=at)
def target_shifted(at): return at+sum(12*words for offset,words in insertions if offset<at)
relocations=relocation_entries(before,old_symbol['section'])
relative=[]
expected=bytearray(old_body)
for row in sorted(graph['instructions']+extra,key=lambda row:row['offset']):
    at,size=row['offset'],row['size']
    raw=old_body[at:at+size]
    field=None
    if raw[:1] in [b'\xe8',b'\xe9'] and size==5: field=1
    elif raw[:1]==b'\x0f' and 0x80<=raw[1]<=0x8f and size==6: field=2
    elif size==7 and raw[0] in [0x48,0x4c] and raw[1]==0x8d and raw[2]&0xc7==5: field=3
    if field is None or old_symbol['value']+at+field in relocations: continue
    if 'gnu_assembly' in row:
        mnemonic=row['gnu_assembly'].split()[0]
        assert (field in [1,2] and mnemonic.startswith(('j','call'))) or (field==3 and mnemonic.startswith('lea'))
    target=at+size+struct.unpack_from('<i',raw,field)[0]
    assert 0<=target<=len(old_body)
    value=target_shifted(target)-(shifted(at)+size)
    struct.pack_into('<i',expected,at+field,value)
    if value!=struct.unpack_from('<i',raw,field)[0]:
        relative.append(dict(offset=at,field=field,old_target=target,new_target=target_shifted(target),
            decoder='selected-graph' if at in graph_nodes else 'additional-gnu-linear'))
records=[]
for offset,words in reversed(insertions):
    # Independent encoding: a wide immediate-zero MOV to each negative RSP slot.
    raw=b''.join(bytes.fromhex('48c78424')+struct.pack('<i',-8*i)+bytes(4) for i in range(1,words+1))
    assert new_body[target_shifted(offset):shifted(offset)]==raw
    expected[offset:offset]=raw
    records.append(dict(old_offset=offset,new_offset=target_shifted(offset),words=words,bytes=raw.hex()))
if bytes(expected)!=new_body:
    differences=[at for at,(left,right) in enumerate(zip(expected,new_body)) if left!=right]
    (output/'differences.json').write_text(json.dumps(dict(count=len(differences),
        examples=[dict(offset=at,expected=expected[max(0,at-8):at+8].hex(),actual=new_body[max(0,at-8):at+8].hex())
            for at in differences[:32]]),indent=2)+'\n')
    raise AssertionError(('Unexpected process-body change outside exact cleanup stores and derived displacements',len(differences),differences[:32]))
old_reloc={field-old_symbol['value']:row for field,row in relocations.items()
    if 0<=field-old_symbol['value']<len(old_body)}
new_reloc={field-new_symbol['value']:row for field,row in relocation_entries(after,new_symbol['section']).items()
    if 0<=field-new_symbol['value']<len(new_body)}
assert set(new_reloc)=={shifted(at) for at in old_reloc}
for at,(symbol,kind,addend) in old_reloc.items():
    other,other_kind,other_addend=new_reloc[shifted(at)]
    assert (symbol['name'],kind,addend)==(other['name'],other_kind,other_addend)
names=lambda elf:{name for name,row in elf.symbols.items() if row['section'] and row['size'] and row['info']&15==2}
assert names(before)==names(after)
unchanged=[]
for function in sorted(names(before)-{name}):
    _,left=before.function(function)
    _,right=after.function(function)
    assert left==right,function
    unchanged.append(dict(name=function,sha256=digest(left),bytes=len(left)))
objects=[]
for filename in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
                 'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o','predecessor-gateway.o']:
    data=(old/filename).read_bytes()
    assert data==(new/filename).read_bytes(),filename
    objects.append(dict(path=filename,sha256=digest(data),bytes=len(data)))
report=dict(passed=True,function=name,old_provider_sha256=digest(before.data),new_provider_sha256=digest(after.data),
    old_function_sha256=digest(old_body),new_function_sha256=digest(new_body),old_function_bytes=len(old_body),new_function_bytes=len(new_body),
    insertions=list(reversed(records)),added_bytes=324,adjusted_relative_displacements=relative,
    process_relocations=len(old_reloc),unchanged_functions=unchanged,unchanged_component_objects=objects,
    baseline_linear_transcript_sha256=digest(linear_raw),additional_linear_instructions=len(extra),
    scope='Exact complete provider process-body reconstruction from thirteen cleanup blocks and derived displacements, shifted relocation identities, every other defined function and fourteen equal components including predecessor provider/gateway. Not complete source producer, actual owners/lifetime or production qualification')
(output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,added_bytes=324,blocks=13,unchanged_functions=len(unchanged),unchanged_objects=len(objects))))
