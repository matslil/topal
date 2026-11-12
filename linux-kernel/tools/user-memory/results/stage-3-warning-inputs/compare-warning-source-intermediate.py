"""Compare executable sections and load bytes of first-party source intermediates."""
import hashlib
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf

old,new,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
a,b=map(Elf,[old,new])
digest=lambda raw:hashlib.sha256(raw).hexdigest()
assert a.data[:64]==b.data[:64] and a.sections==b.sections
strings=a.section_bytes(struct.unpack_from('<H',a.data,62)[0])
changed=[]
unchanged_allocated=[]
for i,section in enumerate(a.sections):
    name=strings[section[0]:strings.index(0,section[0])].decode()
    x,y=a.section_bytes(i),b.section_bytes(i)
    if section[2]&2:
        assert x==y
        unchanged_allocated.append(dict(section=name,sha256=digest(x),bytes=len(x)))
    if x!=y:
        assert not section[2]&2
        changed.append(dict(section=name,flags=section[2],bytes=len(x),
            changed_bytes=sum(u!=v for u,v in zip(x,y)),before_sha256=digest(x),after_sha256=digest(y)))
assert {r['section'] for r in changed}=={'.debug_info','.debug_line','.debug_line_str'}
offset=struct.unpack_from('<Q',a.data,32)[0]
width,count=struct.unpack_from('<HH',a.data,54)
assert width==56 and count>0
assert a.data[offset:offset+width*count]==b.data[offset:offset+width*count]
loads=[]
for i in range(count):
    kind,flags,start,virtual,physical,size,memsize,alignment=struct.unpack_from('<II6Q',a.data,offset+i*width)
    if kind!=1: continue
    assert a.data[start:start+size]==b.data[start:start+size]
    loads.append(dict(offset=start,virtual=virtual,bytes=size,sha256=digest(a.data[start:start+size])))
assert loads
report=dict(passed=True,old_sha256=digest(a.data),new_sha256=digest(b.data),section_headers_equal=True,
    elf_and_program_headers_equal=True,unchanged_allocated_sections=unchanged_allocated,
    unchanged_load_segments=loads,changed_sections=changed,
    scope='Exact ELF/header/allocated-section/load-byte comparison of the recorded first-party source intermediates. Only three nonallocated DWARF sections differ; no complete clean producer, debug-content reproducibility or kernel owner/lifetime qualification')
(output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,changed_debug_sections=3,unchanged_allocated_sections=len(unchanged_allocated),unchanged_load_segments=len(loads))))
