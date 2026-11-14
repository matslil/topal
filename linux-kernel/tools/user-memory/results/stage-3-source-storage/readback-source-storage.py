"""Independent ELF/stack and matching-kernel three-cohort call-union readback."""
import importlib.util
import json
from pathlib import Path
import struct
import sys

build,baseline,census,output=map(Path,sys.argv[1:5])
output.mkdir(parents=True,exist_ok=True)
path=Path('linux-kernel/tools/user-memory/inspect_call_inventory.py')
spec=importlib.util.spec_from_file_location('reader',path)
reader=importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
digest=reader.digest
component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='storage'
linked=reader.Elf(build/'source-user.elf')
assert digest(linked.data)==component['linked_sha256']
assert linked.symbols['topal_user_extent_storage_cases']['value']==component['entry']
strings=linked.section_bytes(int.from_bytes(linked.data[62:64],'little'))
text=next(s for s in linked.sections if strings[s[0]:strings.index(0,s[0])]==b'.text')
data=linked.data[text[4]:text[4]+text[5]]
assert text[3]==component['base']==0x570000 and len(data)==component['text_bytes']<=65536
image=(build/'static-elf-abi-fixture.elf').read_bytes()
assert digest(image)==component['image_sha256']
phoff=struct.unpack_from('<Q',image,32)[0]
width,count=struct.unpack_from('<2H',image,54)
assert width==56
headers=[struct.unpack_from('<II6Q',image,phoff+i*width) for i in range(count)]
def loaded(address,size):
    rows=[p for p in headers if p[0]==1 and p[3]<=address and address+size<=p[3]+p[5]]
    assert len(rows)==1 and rows[0][1]==5
    p=rows[0]
    at=p[2]+address-p[3]
    return image[at:at+size]
assert loaded(text[3],len(data))==data
assert loaded(component['syscall4'],18)==bytes.fromhex('4d89c24889f84889f74889d64889ca0f05c3')
assert digest((build/'storage-cases.c').read_bytes())==component['storage_source_sha256']
frames={}
for stem in ['source','startup','file','shell','command','storage']:
    cost=0
    for line in (build/f'{stem}-cases.su').read_text().splitlines():
        _,size,kind=line.split('\t')
        assert kind=='static'
        cost+=int(size)+8
    frames[f'{stem}-cases']=cost
assert frames==component['stack_frame_costs']
assert 1024+128+frames['storage-cases']==component['conservative_user_stack_bytes']<=2048
observed=json.loads((build/'source-user-extents/observations.json').read_text())
assert observed['passed'] and len(observed['executions'])==4
assert all(row['storage_retirement_verified'] for row in observed['executions'])
calls=[c for row in observed['executions'] for c in row['calls']]
assert len(calls)==208
actions=[c for c in calls if c['number']!=12]
assert len(actions)==196 and all(c['complete_storage_state_verified'] and c['complete_table_verified'] for c in actions)
assert sum(c['result']==-14 for c in actions)==112
assert sum(c['result']==-22 for c in actions)==4
windows=[c for c in actions if c['number'] not in [3,8]]
assert len(windows)==164 and all(c['copy_window_verified'] for c in windows)
assert sum(c.get('fresh_description_verified',False) for c in actions)==12
assert sum(c.get('positional_position_preserved',False) for c in actions)==20
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    raw=(build/name).read_bytes()
    assert raw==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(raw),sha256=digest(raw),unchanged=True))
assert image!=(baseline/'static-elf-abi-fixture.elf').read_bytes()
assert image[:4096]==(baseline/'static-elf-abi-fixture.elf').read_bytes()[:4096]
serial=(build/'source-user-extents/serial.log').read_bytes()
assert serial==(baseline/'source-user-extents/serial.log').read_bytes()
inventory=json.loads((build/'extent-call-inventory.json').read_text())
sets=[]
for directory in [census,baseline,build]:
    assert inventory==json.loads((directory/'extent-call-inventory.json').read_text())
    obs=json.loads((directory/'source-user-extents/observations.json').read_text())
    assert obs['passed'] and obs['extent_call_coverage']['observation_passed']
    assert obs['extent_call_coverage']['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
    sets.append({r['offset'] for r in obs['extent_call_coverage']['reached']})
previous=sets[0]|sets[1]
union=previous|sets[2]
added=[r for r in inventory['calls'] if r['offset'] in union-previous]
expected={'systems_volatile_transfer':4,'systems_positional_transfer':1,'systems_namespace_named':1}
assert {stem:sum(Path(r['source_file']).stem==stem for r in added) for stem in expected}==expected
assert len(added)==6
report=dict(passed=True,source_syscalls=len(calls),storage_requests=len(actions),
    buffer_refusals=112,signed_offset_refusals=4,copy_window_checks=len(windows),
    fresh_descriptions=12,positional_successes_preserving_position=20,exact_retirements=4,
    linked_text_bytes=len(data),linked_text_sha256=digest(data),stack_frame_costs=frames,
    conservative_user_stack_bytes=component['conservative_user_stack_bytes'],
    kernel_comparisons=comparisons,application_image_changed=True,
    serial_bytes=len(serial),serial_sha256=digest(serial),
    previous_union=len(previous),current_reached=len(sets[2]),union_reached=len(union),
    total_sites=len(inventory['calls']),new_sites=added,
    union_unreached=[r for r in inventory['calls'] if r['offset'] not in union],
    scope='Selected ordinary file/directory buffer/position/state/retirement checks; matching-kernel first-reach union, not complete filesystem/user-access or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','new_sites','union_unreached']}))
