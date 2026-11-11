"""Separate ELF, native-request and unchanged-kernel readback for exec cases."""
import hashlib
import json
from pathlib import Path
import struct
import sys

build, baseline, output = map(Path,sys.argv[1:4])
transfer_build=Path(sys.argv[4])
output.mkdir(parents=True,exist_ok=True)
def digest(data): return hashlib.sha256(data).hexdigest()
def parse(path):
    data=path.read_bytes()
    assert data[:6]==b'\x7fELF\x02\x01'
    phoff,shoff=struct.unpack_from('<QQ',data,32)
    phsize,phnum,shsize,shnum,shstr=struct.unpack_from('<5H',data,54)
    assert phsize==56
    headers=[struct.unpack_from('<II6Q',data,phoff+i*phsize) for i in range(phnum)]
    sections={}
    if shnum:
        assert shsize==64
        rows=[struct.unpack_from('<II4QII2Q',data,shoff+i*shsize) for i in range(shnum)]
        strings=data[rows[shstr][4]:rows[shstr][4]+rows[shstr][5]]
        for row in rows:
            at=row[0]
            sections[strings[at:strings.index(0,at)].decode()]=row
    else: assert shoff==0
    return data,headers,sections
def virtual(data,headers,address,size):
    rows=[p for p in headers if p[0]==1 and p[3]<=address and address+size<=p[3]+p[5]]
    assert len(rows)==1
    row=rows[0]
    start=row[2]+address-row[3]
    return data[start:start+size],row[1]

component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='exec'
image,headers,_=parse(build/'static-elf-abi-fixture.elf')
linked,_,sections=parse(build/'source-user.elf')
text=sections['.text']
raw=linked[text[4]:text[4]+text[5]]
assert text[3]==component['base'] and len(raw)==component['text_bytes']
assert digest(image)==component['image_sha256'] and digest(linked)==component['linked_sha256']
assert virtual(image,headers,component['base'],len(raw))==(raw,5)
adapter=bytes.fromhex('48 89 f8 48 89 f7 48 89 d6 48 89 ca 0f 05 c3')
assert virtual(image,headers,component['syscall'],len(adapter))==(adapter,5)
entry=struct.unpack_from('<Q',linked,24)[0]
assert entry==component['entry']
expected=(bytes.fromhex('53 55 41 54 41 55 41 56 41 57 41 54 49 89 e4 48 83 e4 f0 48 b8')+
          struct.pack('<Q',entry)+bytes.fromhex('ff d0 4c 89 e4 41 5c 48 85 c0 75 fe'))
call,_=virtual(image,headers,0x4e0000,len(expected))
assert len(expected)==len(call) and call==expected
groups={}
for stem in ['source-cases','startup-cases','file-cases','shell-cases','command-cases','exec-cases']:
    total=0
    for line in (build/(stem+'.su')).read_text().splitlines():
        name,size,kind=line.split('\t')
        assert kind=='static'
        total+=int(size)+8
    groups[stem]=total
assert groups==component['stack_frame_costs']
stack=1024+128+groups['exec-cases']
assert stack==component['conservative_user_stack_bytes']<=2048
source=(build/'exec-cases.c').read_bytes()
assert digest(source)==component['exec_source_sha256']
observed=json.loads((build/'source-user-extents/observations.json').read_text())
assert observed['passed'] and len(observed['executions'])==4
assert all(not observed[key] for key in ['startup_executions','file_executions','shell_executions','command_executions','bridge_returns'])
readonly_inputs=[]
def prefix(row,key): return bytes.fromhex(row[key+'_prefix_hex'])
def qwords(data): return list(struct.unpack('<'+'Q'*(len(data)//8),data[:len(data)//8*8]))
def readonly_string(pointer,expected):
    assert virtual(image,headers,pointer,len(expected))==(expected,5)
for execution in observed['executions']:
    calls=execution['calls']
    assert len(calls)==32 and [c['number'] for c in calls]==[12,12]+[59]*29+[12]
    assert calls[0]['result']==calls[-1]['result']==0x619000 and calls[1]['result']==0x63a000
    rows=calls[2:-1]
    errors=[-14]*5+[-2,-36]+[-14]*10+[-22,-7,-7]+[-14]*6+[-7]*3
    assert [r['result'] for r in rows]==errors
    assert all(r['private_staging_zero'] and r['protected_state_unchanged'] and r['phase_after']==r['phase_before'] for r in rows)
    assert prefix(rows[4],'pathname')==b'/'
    assert prefix(rows[5],'pathname').split(b'\0')[0]==b'/dev/tty'
    assert prefix(rows[6],'pathname')==b'x'*256
    assert len(prefix(rows[11],'argv'))==4
    assert prefix(rows[12],'pathname').startswith(b'/bin/busybox\0')
    vector=qwords(prefix(rows[12],'argv'))[:3]
    assert vector[2]==0
    readonly_string(vector[0],b'/bin/busybox\0')
    readonly_string(vector[1],b'true\0')
    assert qwords(prefix(rows[13],'argv'))[:3]==[0x638ff8,0x636fff,0]
    readonly_pointer=rows[14]['args'][1]
    assert virtual(image,headers,readonly_pointer,24)==(prefix(rows[14],'argv')[:24],5)
    assert qwords(prefix(rows[14],'argv'))[:3]==vector
    readonly_inputs.append(dict(vector=readonly_pointer,pathname=vector[0],argument=vector[1],load_flags=5))
    assert qwords(prefix(rows[15],'argv'))[:3]==[0x638ff8,0x639fff,0]
    assert len(prefix(rows[16],'argv'))==16 and qwords(prefix(rows[16],'argv'))==vector[:2]
    assert qwords(prefix(rows[18],'argv'))[-1]==(1<<64)-1
    assert len(prefix(rows[23],'environment'))==4
    assert qwords(prefix(rows[24],'environment'))[1]==0x639fff
    assert len(prefix(rows[25],'environment'))==16
    environment=qwords(prefix(rows[26],'environment'))
    assert len(environment)==17 and len(set(environment[:16]))==1 and environment[-1]==(1<<64)-1
    readonly_string(environment[0],b'A=one\0')
    assert qwords(prefix(rows[27],'environment'))[:2]==[0x635000,0]
    assert qwords(prefix(rows[28],'environment'))[:3]==[0x635000,0x635300,0]
comparison=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o',
             'provider.o','predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data=(build/name).read_bytes()
    assert data==(baseline/name).read_bytes(),name
    comparison.append(dict(path=name,bytes=len(data),sha256=digest(data)))
serial=(build/'source-user-extents/serial.log').read_bytes()
assert serial==(baseline/'source-user-extents/serial.log').read_bytes()
transfer=(transfer_build/'source-abi.elf').read_bytes()
assert transfer==(baseline/'static-elf-abi-fixture.elf').read_bytes()
report=dict(passed=True,source_invocations=4,exec_syscalls=116,total_observed_syscalls=128,
            source_text_bytes=len(raw),source_text_sha256=digest(raw),
            source_stack_bytes=stack,stack_frame_costs=groups,readonly_inputs=readonly_inputs,
            runtime_call=call.hex(),kernel_comparisons=comparison,serial_bytes=len(serial),serial_sha256=digest(serial),
            preceding_transfer_fixture_unchanged=True,transfer_fixture_sha256=digest(transfer),
            scope='Selected ordinary exec argument/environment refusal and subsequent existing successful replacement; not complete caller/production qualification or a full native memory transcript')
(output/'independent-readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['readonly_inputs','kernel_comparisons']}))
