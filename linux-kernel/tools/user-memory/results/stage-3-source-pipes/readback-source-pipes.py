"""Independent source/ELF/stack/call-union readback for ordinary FIFO cases."""
import importlib.util
import json
from pathlib import Path
import struct
import sys

build, baseline, output = map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=True)
path=Path('linux-kernel/tools/user-memory/inspect_call_inventory.py')
spec=importlib.util.spec_from_file_location('reader',path)
reader=importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
digest=reader.digest
component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='pipes'
linked=reader.Elf(build/'source-user.elf')
assert digest(linked.data)==component['linked_sha256']
assert linked.symbols['topal_user_extent_pipe_cases']['value']==component['entry']
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
    matches=[p for p in headers if p[0]==1 and p[3]<=address and address+size<=p[3]+p[5]]
    assert len(matches)==1
    p=matches[0]
    assert p[1]==5
    offset=p[2]+address-p[3]
    return image[offset:offset+size]
assert loaded(text[3],len(data))==data
assert loaded(component['syscall'],15)==bytes.fromhex('4889f84889f74889d64889ca0f05c3')
assert digest((build/'pipe-cases.c').read_bytes())==component['pipe_source_sha256']
frames={}
for stem in ['source','startup','file','shell','command','pipe']:
    cost=0
    for line in (build/f'{stem}-cases.su').read_text().splitlines():
        _,size,kind=line.split('\t')
        assert kind=='static'
        cost+=int(size)+8
    frames[f'{stem}-cases']=cost
assert frames==component['stack_frame_costs']
assert 1024+128+frames['pipe-cases']==component['conservative_user_stack_bytes']<=2048
observation=json.loads((build/'source-user-extents/observations.json').read_text())
assert observation['passed'] and len(observation['executions'])==4
calls=[c for row in observation['executions'] for c in row['calls']]
assert len(calls)==144
assert all(row['pipe_retirement_verified'] for row in observation['executions'])
actions=[c for c in calls if c['number']!=12]
assert len(actions)==132
assert all(c['complete_fifo_state_verified'] and c['complete_table_verified'] for c in actions)
assert all(c['copy_window_verified'] for c in actions if c['number']!=3)
assert sum(c['result']==-14 for c in actions)==76
assert sum(c['result']==-11 for c in actions)==4
assert sum(c['number'] in [0,1] and c['result']>0 for c in actions)==16
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    raw=(build/name).read_bytes()
    assert raw==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(raw),sha256=digest(raw),unchanged=True))
assert image!=(baseline/'static-elf-abi-fixture.elf').read_bytes()
assert image[:4096]==(baseline/'static-elf-abi-fixture.elf').read_bytes()[:4096]
assert (build/'source-user-extents/serial.log').read_bytes()==(baseline/'source-user-extents/serial.log').read_bytes()
inventory=json.loads((build/'extent-call-inventory.json').read_text())
assert inventory==json.loads((baseline/'extent-call-inventory.json').read_text())
previous=json.loads((baseline/'source-user-extents/observations.json').read_text())['extent_call_coverage']
current=observation['extent_call_coverage']
assert previous['observation_passed'] and current['observation_passed']
assert previous['kernel_sha256']==current['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
old={r['offset'] for r in previous['reached']}
now={r['offset'] for r in current['reached']}
new_sites=[r for r in inventory['calls'] if r['offset'] in now-old]
pipe_sites=[r for r in inventory['calls'] if Path(r['source_file']).stem in [
    'systems_pipe_create','systems_pipe_transfer','systems_process_pipes']]
assert len(pipe_sites)==4 and {r['offset'] for r in pipe_sites}<=now-old
union=old|now
report=dict(passed=True,source_syscalls=len(calls),pipe_requests=len(actions),
    refused_buffer_requests=76,nonblocking_empty_reads=4,positive_byte_transfers=16,
    complete_fifo_table_checks=len(actions),exact_retirements=4,
    copy_window_checks=sum(c['number']!=3 for c in actions),
    linked_text_bytes=len(data),linked_text_sha256=digest(data),stack_frame_costs=frames,
    conservative_user_stack_bytes=component['conservative_user_stack_bytes'],
    kernel_comparisons=comparisons,application_image_changed=True,
    previous_reached=len(old),current_reached=len(now),union_reached=len(union),
    total_sites=len(inventory['calls']),new_sites=new_sites,
    union_unreached=[r for r in inventory['calls'] if r['offset'] not in union],
    scope='Selected ordinary nonblocking FIFO buffer/state/retirement cases and exact emitted-call union; not every FIFO semantic, every user access or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','new_sites','union_unreached']}))
