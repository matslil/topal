"""Independent ELF, pollfd and private readiness-state readback."""
import hashlib
import json
from pathlib import Path
import struct
import sys

build,baseline,output,transfer_build=map(Path,sys.argv[1:5])
output.mkdir(parents=True,exist_ok=True)
def digest(data): return hashlib.sha256(data).hexdigest()
def sections(data):
    shoff=struct.unpack_from('<Q',data,40)[0]
    width,count,names=struct.unpack_from('<3H',data,58)
    if not count:
        assert shoff==0
        return [],{}
    assert width==64
    rows=[struct.unpack_from('<II4QII2Q',data,shoff+i*width) for i in range(count)]
    strings=data[rows[names][4]:rows[names][4]+rows[names][5]]
    named={strings[row[0]:strings.index(0,row[0])].decode():row for row in rows}
    return rows,named
def parse(path):
    data=path.read_bytes()
    assert data[:6]==b'\x7fELF\x02\x01'
    phoff=struct.unpack_from('<Q',data,32)[0]
    width,count=struct.unpack_from('<2H',data,54)
    assert width==56
    headers=[struct.unpack_from('<II6Q',data,phoff+i*width) for i in range(count)]
    return data,headers,sections(data)[1]
def virtual(data,headers,address,size):
    rows=[p for p in headers if p[0]==1 and p[3]<=address and address+size<=p[3]+p[5]]
    assert len(rows)==1
    p=rows[0]
    at=p[2]+address-p[3]
    return data[at:at+size],p[1]
def functions(path):
    data=path.read_bytes()
    rows,_=sections(data)
    result={}
    for table in rows:
        if table[1]!=2: continue
        strings=data[rows[table[6]][4]:rows[table[6]][4]+rows[table[6]][5]]
        assert table[9]==24
        for at in range(table[4],table[4]+table[5],24):
            name,info,_,section,value,size=struct.unpack_from('<IBBHQQ',data,at)
            if info&15!=2 or not size or not section: continue
            name=strings[name:strings.index(0,name)].decode()
            owner=rows[section]
            start=owner[4]+value-owner[3]
            assert name not in result
            result[name]=data[start:start+size]
    return result

component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='poll'
image,headers,_=parse(build/'static-elf-abi-fixture.elf')
linked,_,named=parse(build/'source-user.elf')
text=named['.text']
raw=linked[text[4]:text[4]+text[5]]
assert len(raw)==component['text_bytes'] and text[3]==component['base']
assert virtual(image,headers,text[3],len(raw))==(raw,5)
assert digest(image)==component['image_sha256'] and digest(linked)==component['linked_sha256']
adapter=bytes.fromhex('4d 89 c2 48 89 f8 48 89 f7 48 89 d6 48 89 ca 0f 05 c3')
assert virtual(image,headers,component['syscall4'],len(adapter))==(adapter,5)
bridge,_=virtual(image,headers,component['startup_call'],61)
prefix=bytes.fromhex('57 56 52 41 52 41 50 41 51 41 54 49 89 e4 48 83 e4 f0 48 89 c7 49 8b 74 24 30 49 8b 54 24 28 49 8b 4c 24 20 4d 8b 44 24 18 e8')
assert bridge[:len(prefix)]==prefix
assert component['startup_call']+len(prefix)+4+struct.unpack_from('<i',bridge,len(prefix))[0]==component['startup']
entrycode,_=virtual(image,headers,0x4a6100,0x9f00)
call_sites=[]
for at in range(7,len(entrycode)-11):
    if entrycode[at]==0xe8 and 0x4a6100+at+5+struct.unpack_from('<i',entrycode,at+1)[0]==component['startup_call']:
        assert entrycode[at-7:at]==bytes.fromhex('48 81 ec 00 04 00 00')
        assert entrycode[at+5:at+12]==bytes.fromhex('48 81 c4 00 04 00 00')
        call_sites.append(0x4a6100+at)
assert len(call_sites)==4
groups={}
for stem in ['source-cases','startup-cases','file-cases','shell-cases','command-cases','poll-cases']:
    cost=0
    for line in (build/(stem+'.su')).read_text().splitlines():
        _,size,kind=line.split('\t')
        assert kind=='static'
        cost+=int(size)+8
    groups[stem]=cost
assert groups==component['stack_frame_costs']
stack=1024+128+max(groups['source-cases'],groups['poll-cases'])
assert stack==component['conservative_user_stack_bytes']<=2048
assert digest((build/'poll-cases.c').read_bytes())==component['poll_source_sha256']
observation=json.loads((build/'source-user-extents/observations.json').read_text())
assert observation['passed'] and len(observation['executions'])==4
assert sum(len(r['calls']) for r in observation['executions'])==56
assert len(observation['poll_executions'])==len(observation['bridge_returns'])==4
assert all(not observation[k] for k in ['startup_executions','file_executions','shell_executions','command_executions'])
readonly=[]
for index,row in enumerate(observation['poll_executions']):
    calls=row['calls']
    assert [c['number'] for c in calls]==[12,12]+[7]*8+[0]*5+[12]
    assert [c['result'] for c in calls[2:-1]]==[-14]*5+[1,-14,1]+[-14]*4+[1]
    assert calls[5]['args'][0]==0x639ff9 and bytes.fromhex(calls[5]['input_hex'])==b'\xda'*7
    assert calls[7]['args'][0]==calls[9]['args'][0]==[0x5ffffc,0x638ffc,0x639ff8,0x638ff9][index]
    assert struct.unpack('<iHH',bytes.fromhex(calls[7]['input_hex']))==(3,1,0xffff)
    assert struct.unpack('<iHH',bytes.fromhex(calls[9]['input_hex']))==(3,1,0x5aa5)
    for c in calls:
        expected=dict(c['controls_before'])
        fields=bytearray.fromhex(expected['file'])
        if c['number']==7 and c['result']==1:
            source=bytes.fromhex(c['input_hex'])
            assert struct.unpack('<iH',source[:6])==(3,1)
            assert bytes.fromhex(c['output_hex'])==source[:6]+struct.pack('<H',1)
            fields[155]=1
        if c['number']==0 and c['result']==1:
            assert c['args'][1]==0x639fff and fields[155]==1
            assert bytes.fromhex(c['output_hex'])==b'\x7fELF'[index:index+1]
            fields[152]+=1; fields[153]+=1; fields[155]=0
        expected['file']=fields.hex()
        assert c['controls_after']==expected
        if c['number']==7 and component['base']<=c['args'][0]<component['base']+len(raw):
            assert c['result']==-14
            data,flags=virtual(image,headers,c['args'][0],8)
            assert flags==5 and data==bytes.fromhex(c['input_hex'])==struct.pack('<iHH',3,1,0xffff)
            readonly.append(dict(pointer=c['args'][0],input_hex=data.hex(),load_flags=flags))
assert len(readonly)==8
comparisons=[]
changed=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data=(build/name).read_bytes()
    same=data==(baseline/name).read_bytes()
    comparisons.append(dict(path=name,unchanged=same,bytes=len(data),sha256=digest(data)))
    if not same: changed.append(name)
assert changed==['provider.o','kernel.elf','bzImage']
before,after=functions(baseline/'provider.o'),functions(build/'provider.o')
assert set(before)==set(after)
changed_functions=[name for name in sorted(before) if before[name]!=after[name]]
assert changed_functions==['topal_x86_systems_static_image_process_enter']
serial=(build/'source-user-extents/serial.log').read_bytes()
assert serial==(baseline/'source-user-extents/serial.log').read_bytes()
transfer=(transfer_build/'source-abi.elf').read_bytes()
prior=Path('linux-kernel/.cache/x86_64/user-extent-source-2256205-1794392689887672216/static-elf-abi-fixture.elf').read_bytes()
assert transfer==prior
report=dict(passed=True,poll_callbacks=4,poll_syscalls=32,header_read_syscalls=20,callback_heap_syscalls=12,
    shared_runtime_syscalls=56,total_observed_syscalls=120,readonly_records=readonly,linked_text_bytes=len(raw),
    text_sha256=digest(raw),source_stack_bytes=stack,stack_frame_costs=groups,generated_header_calls=call_sites,
    comparisons=comparisons,unchanged_provider_functions=len(before)-1,changed_provider_functions=changed_functions,
    serial_bytes=len(serial),serial_sha256=digest(serial),preceding_transfer_fixture_unchanged=True,
    scope='Selected clean-header readiness/input/output/state qualification and native object comparison; external-profile compatibility and production qualification require separate evidence')
(output/'independent-readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['readonly_records','comparisons','generated_header_calls']}))
