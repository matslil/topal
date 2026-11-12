"""Check every defined provider function against the preceding provider."""
import json
from pathlib import Path
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest

old,new,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
before,after=Elf(old/'provider.o'),Elf(new/'provider.o')
names=lambda elf:{name for name,row in elf.symbols.items() if row['section'] and row['size'] and row['info']&15==2}
assert names(before)==names(after)
changed,unchanged=[],[]
for name in sorted(names(before)):
    _,left=before.function(name)
    _,right=after.function(name)
    row=dict(name=name,old_sha256=digest(left),new_sha256=digest(right),old_bytes=len(left),new_bytes=len(right))
    (unchanged if left==right else changed).append(row)
assert [r['name'] for r in changed]==['topal_x86_systems_static_image_process_enter']
assert changed[0]['new_bytes']==changed[0]['old_bytes']+52
report=dict(passed=True,old_provider_sha256=digest(before.data),new_provider_sha256=digest(after.data),
    functions=len(changed)+len(unchanged),changed_functions=changed,unchanged_functions=unchanged,
    scope='Complete defined provider function-byte comparison in relocatable objects; only the separately reconstructed process function changes. Not a clean producer, all object metadata/relocations or native ownership/lifetime qualification')
(output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,functions=report['functions'],changed=len(changed),unchanged=len(unchanged))))
