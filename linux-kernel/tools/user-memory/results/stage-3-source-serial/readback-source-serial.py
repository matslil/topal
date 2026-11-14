"""Independent serial dialogue, ELF/stack and matching-kernel reachability audit."""
import importlib.util
import json
from pathlib import Path
import struct
import sys

build,baseline,pipes,poll,output=map(Path,sys.argv[1:6])
output.mkdir(parents=True,exist_ok=True)
path=Path('linux-kernel/tools/user-memory/inspect_call_inventory.py')
spec=importlib.util.spec_from_file_location('reader',path)
reader=importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
digest=reader.digest
component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='serial'
linked=reader.Elf(build/'source-user.elf')
assert digest(linked.data)==component['linked_sha256']
assert linked.symbols['topal_user_extent_serial_cases']['value']==component['entry']
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
assert loaded(component['syscall'],15)==bytes.fromhex('4889f84889f74889d64889ca0f05c3')
assert digest((build/'serial-cases.c').read_bytes())==component['serial_source_sha256']
frames={}
for stem in ['source','startup','file','shell','command','serial']:
    cost=0
    for line in (build/f'{stem}-cases.su').read_text().splitlines():
        _,size,kind=line.split('\t')
        assert kind=='static'
        cost+=int(size)+8
    frames[f'{stem}-cases']=cost
assert frames==component['stack_frame_costs']
assert 1024+128+frames['serial-cases']==component['conservative_user_stack_bytes']<=2048
observed=json.loads((build/'source-user-extents/observations.json').read_text())
assert observed['passed'] and len(observed['executions'])==4
assert all(row['serial_input_verified'] for row in observed['executions'])
calls=[c for row in observed['executions'] for c in row['calls']]
assert 64<=len(calls)<=116
actions=[c for c in calls if c['number']!=12]
assert all(c['complete_table_verified'] and c['copy_window_verified'] and c['native_input_verified'] for c in actions)
assert sum(c['result']==-14 for c in actions)==20 and sum(c['result']==-9 for c in actions)==8
assert all(not c['uart_bytes'] for c in actions if c['result']<=0)
positive=[c for c in actions if c['number']==0 and c['result']>0]
assert sum(c['result'] for c in positive)==sum(len(c['uart_bytes']) for c in actions)==64
assert all(bytes(c['uart_bytes']).hex()==c['output_hex'] for c in positive)
large=[c for c in positive if c['args'][2]==512]
assert len(large)==4 and all(c['extent_call']==dict(candidate=0x639f00,length=256) for c in large)
dialogue=json.loads((build/'source-user-extents/serial-dialogue.json').read_text())
expected=[(b'TOPAL_SOURCE_SERIAL_A\n',b'Uabcdefg'),(b'TOPAL_SOURCE_SERIAL_B\n',b'12345678')]*4
assert [(bytes.fromhex(r['marker_hex']),bytes.fromhex(r['payload_hex'])) for r in dialogue]==expected
serial=(build/'source-user-extents/serial.log').read_bytes()
for row,(marker,_) in zip(dialogue,expected):
    at=row['serial_offset']
    assert serial[at:at+len(marker)]==marker
stripped=serial
for marker in {m for m,_ in expected}:
    assert serial.count(marker)==4
    stripped=stripped.replace(marker,b'')
assert stripped==(baseline/'source-user-extents/serial.log').read_bytes()
sites=json.loads((build/'serial-native-sites.json').read_text())
elf=reader.Elf(build/'kernel.elf')
symbol,body=elf.function('topal_x86_systems_static_image_process_enter')
at=sites['byte_received_offset']
assert body[at-5:at+2]==bytes.fromhex('66baf803ec8806')
assert sites['byte_received']==symbol['value']+at
assert sites['kernel_sha256']==digest(elf.data)
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    raw=(build/name).read_bytes()
    assert raw==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(raw),sha256=digest(raw),unchanged=True))
assert image!=(baseline/'static-elf-abi-fixture.elf').read_bytes()
assert image[:4096]==(baseline/'static-elf-abi-fixture.elf').read_bytes()[:4096]
inventory=json.loads((build/'extent-call-inventory.json').read_text())
sets=[]
for directory in [poll,pipes,baseline,build]:
    assert inventory==json.loads((directory/'extent-call-inventory.json').read_text())
    obs=json.loads((directory/'source-user-extents/observations.json').read_text())
    assert obs['passed'] and obs['extent_call_coverage']['observation_passed']
    assert obs['extent_call_coverage']['kernel_sha256']==digest(elf.data)
    sets.append({r['offset'] for r in obs['extent_call_coverage']['reached']})
previous=set.union(*sets[:3])
union=set.union(*sets)
added=[r for r in inventory['calls'] if r['offset'] in union-previous]
assert len(added)==1 and 'fd-serial-count-ready' in added[0]['nearest_labels']
report=dict(passed=True,source_syscalls=len(calls),observed_requests=len(actions),
    buffer_refusals=20,descriptor_refusals=8,positive_reads=len(positive),native_input_bytes=64,
    returned_input_bytes=64,large_request_admissions=4,serial_dialogue_messages=len(dialogue),
    linked_text_bytes=len(data),linked_text_sha256=digest(data),stack_frame_costs=frames,
    conservative_user_stack_bytes=component['conservative_user_stack_bytes'],
    kernel_comparisons=comparisons,application_image_changed=True,
    serial_bytes=len(serial),serial_sha256=digest(serial),
    serial_without_markers_bytes=len(stripped),serial_without_markers_sha256=digest(stripped),
    previous_union=len(previous),current_reached=len(sets[3]),union_reached=len(union),
    total_sites=len(inventory['calls']),new_sites=added,
    union_unreached=[r for r in inventory['calls'] if r['offset'] not in union],
    scope='Selected ordinary serial descriptor/copy/UART-consumption checks and matching-kernel first reach; not full tty/driver, user-access or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','new_sites','union_unreached']}))
