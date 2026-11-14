"""Explain the complete process-body change, including internal fixups."""
import json
from pathlib import Path
import struct
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest
from inventory_process_accesses import decoded_rows

before,after,output=map(Path,sys.argv[1:4])
old,new=Elf(before/'provider.o'),Elf(after/'provider.o')
function='topal_x86_systems_static_image_process_enter'
osym,ob=old.function(function)
nsym,nb=new.function(function)
og=json.loads((before/'process-access-inventory.json').read_text())
ng=json.loads((after/'process-access-inventory.json').read_text())
on,nn=og['instructions'],ng['instructions']
assert len(on)==len(nn)
allowed={
    '65488b342540010000':'4c89c6',
    '65488b0c2548010000':'4c89c9',
    '654c8b342548010000':'4d89ce',
    '65488b3c2540010000':'4c89c7',
}
changes=[]
for a,b in zip(on,nn,strict=True):
    if a['size']!=b['size']:
        assert (a['size'],b['size'])==(9,3)
        assert allowed[a['bytes']]==b['bytes']
        changes.append(dict(old_offset=a['offset'],new_offset=b['offset'],before=a['bytes'],after=b['bytes']))
assert len(changes)==6 and len(ob)-len(nb)==36
unchanged_functions=[]
old_functions={name:s for name,s in old.symbols.items() if s['section'] and s['info']&15==2 and s['size']}
new_functions={name:s for name,s in new.symbols.items() if s['section'] and s['info']&15==2 and s['size']}
assert set(old_functions)==set(new_functions)
for name,old_symbol in old_functions.items():
    if name==function: continue
    new_symbol=new_functions[name]
    _,old_body=old.function(name)
    _,new_body=new.function(name)
    assert old_body==new_body,('other provider function changed',name)
    def function_relocations(elf,symbol,body):
        return {at-symbol['value']:row for at,row in elf.relocations(symbol['section']).items()
            if symbol['value']<=at<symbol['value']+len(body)}
    assert function_relocations(old,old_symbol,old_body)==function_relocations(new,new_symbol,new_body),('other provider relocations changed',name)
    unchanged_functions.append(name)
def shifted(at):
    assert not any(c['old_offset']<at<c['old_offset']+9 for c in changes)
    return at-6*sum(c['old_offset']<at for c in changes)
assert all(shifted(a['offset'])==b['offset'] for a,b in zip(on,nn,strict=True))
oreloc={at-osym['value']:row for at,row in old.relocations(osym['section']).items()
    if osym['value']<=at<osym['value']+len(ob)}
nreloc={at-nsym['value']:row for at,row in new.relocations(nsym['section']).items()
    if nsym['value']<=at<nsym['value']+len(nb)}
assert len(oreloc)==len(nreloc)==19
assert {shifted(at):row for at,row in oreloc.items()}==nreloc
old_mask,new_mask=bytearray(ob),bytearray(nb)
fixups=[]
change_offsets={c['old_offset'] for c in changes}
def check_fixup(a,b):
    at,bt=a['offset'],b['offset']
    ar,br=ob[at:at+a['size']],nb[bt:bt+b['size']]
    if at in change_offsets or ar==br:
        return
    control=(len(ar)==5 and ar[0] in [0xe8,0xe9]) or (len(ar)==6 and ar[0]==0x0f and 0x80<=ar[1]<=0x8f)
    rip=(len(ar)==7 and ar[0] in [0x48,0x4c] and ar[1]==0x8d and ar[2]&0xc7==5)
    assert (control or rip) and ar[:-4]==br[:-4],('unexpected native change',at,ar.hex(),br.hex())
    assert at+len(ar)-4 not in oreloc
    target=at+len(ar)+struct.unpack_from('<i',ar,len(ar)-4)[0]
    actual=bt+len(br)+struct.unpack_from('<i',br,len(br)-4)[0]
    assert 0<=target<len(ob) and actual==shifted(target)
    old_mask[at+len(ar)-4:at+len(ar)]=bytes(4)
    new_mask[bt+len(br)-4:bt+len(br)]=bytes(4)
    fixups.append(dict(old_offset=at,new_offset=bt,old_target=target,new_target=actual))
for a,b in zip(on,nn,strict=True): check_fixup(a,b)
# The complete body also contains unreachable emitted blocks. Verify their
# differing PC-relative instructions rather than ignoring their displacement
# bytes or asserting that the direct-control graph covers them.
olk,nlk=Elf(before/'kernel.elf'),Elf(after/'kernel.elf')
ols,olb=olk.function(function)
nls,nlb=nlk.function(function)
ol=decoded_rows((before/'process-access-disassembly.stdout.log').read_text(),ols,olb)
nl=decoded_rows((after/'process-access-disassembly.stdout.log').read_text(),nls,nlb)
visited={r['offset'] for r in on}
unvisited_fixups_before=len(fixups)
for at,a in sorted(ol.items()):
    if at in visited: continue
    bt=shifted(at)
    assert bt in nl and a['size']==nl[bt]['size'],('unvisited instruction mapping',at,bt)
    check_fixup(a,nl[bt])
unvisited_fixups=len(fixups)-unvisited_fixups_before
left,right=bytearray(),bytearray()
old_at=new_at=0
for c in changes:
    left.extend(old_mask[old_at:c['old_offset']])
    right.extend(new_mask[new_at:c['new_offset']])
    old_at,new_at=c['old_offset']+9,c['new_offset']+3
left.extend(old_mask[old_at:])
right.extend(new_mask[new_at:])
assert left==right,'unexplained process-body bytes, including unvisited data'
expected={(shifted(e['source']),shifted(e['target']),e['kind']) for e in og['edges']}
assert expected=={(e['source'],e['target'],e['kind']) for e in ng['edges']}
assert {shifted(r['offset']) for r in og['roots']}=={r['offset'] for r in ng['roots']}
oc=json.loads((before/'extent-call-inventory.json').read_text())
nc=json.loads((after/'extent-call-inventory.json').read_text())
assert len(oc['calls'])==len(nc['calls'])==91
for a,b in zip(oc['calls'],nc['calls'],strict=True):
    assert shifted(a['offset'])==b['offset'] and a['target']==b['target'] and a['nearest_labels']==b['nearest_labels']
same=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
    'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o',
    'predecessor-provider.o','predecessor-gateway.o']:
    assert (before/name).read_bytes()==(after/name).read_bytes(),name
    same.append(name)
report=dict(passed=True,old_function_bytes=len(ob),new_function_bytes=len(nb),rewrites=changes,
    checked_internal_fixups=fixups,matching_external_relocations=19,
    unvisited_internal_fixups=unvisited_fixups,
    unchanged_other_provider_functions=sorted(unchanged_functions),
    equal_remaining_body_bytes=True,mapped_graph_nodes=len(on),mapped_graph_edges=len(expected),
    mapped_helper_calls=91,unchanged_artifacts=same,
    old_provider_sha256=digest(old.data),new_provider_sha256=digest(new.data),
    scope='Complete relocatable process-body difference and mapped direct graph/census: exactly six GS reloads become preserved-register moves; other bytes agree after checked PC-relative fixups. Earlier native observations are not relabeled as execution of the new kernel; complete owners/lifetime/production remain open')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['checked_internal_fixups','unchanged_artifacts']}))
