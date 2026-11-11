"""Separate ELF and set-based signal-contract readback of native observations."""
import hashlib
import json
from pathlib import Path
import struct
import sys

build,baseline,output,transfer_build=map(Path,sys.argv[1:5])
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
    at=row[2]+address-row[3]
    return data[at:at+size],row[1]

component=json.loads((build/'source-component.json').read_text())
assert component['mode']=='signals'
image,headers,_=parse(build/'static-elf-abi-fixture.elf')
linked,_,sections=parse(build/'source-user.elf')
text=sections['.text']
raw=linked[text[4]:text[4]+text[5]]
assert text[3]==component['base'] and len(raw)==component['text_bytes']
assert digest(image)==component['image_sha256'] and digest(linked)==component['linked_sha256']
assert virtual(image,headers,component['base'],len(raw))==(raw,5)
adapter=bytes.fromhex('4d 89 c2 48 89 f8 48 89 f7 48 89 d6 48 89 ca 0f 05 c3')
assert virtual(image,headers,component['syscall4'],len(adapter))==(adapter,5)
entry=struct.unpack_from('<Q',linked,24)[0]
assert entry==component['entry']
call=(bytes.fromhex('53 55 41 54 41 55 41 56 41 57 41 54 49 89 e4 48 83 e4 f0 48 b8')+
      struct.pack('<Q',entry)+bytes.fromhex('ff d0 4c 89 e4 41 5c 48 85 c0 75 fe'))
assert virtual(image,headers,0x4e0000,len(call))==(call,5)
groups={}
for stem in ['source-cases','startup-cases','file-cases','shell-cases','command-cases','signal-cases']:
    total=0
    for line in (build/(stem+'.su')).read_text().splitlines():
        _,size,kind=line.split('\t')
        assert kind=='static'
        total+=int(size)+8
    groups[stem]=total
assert groups==component['stack_frame_costs']
stack=1024+128+groups['signal-cases']
assert stack==component['conservative_user_stack_bytes']<=2048
assert digest((build/'signal-cases.c').read_bytes())==component['signal_source_sha256']
observation=json.loads((build/'source-user-extents/observations.json').read_text())
assert observation['passed'] and len(observation['executions'])==4
assert all(not observation[key] for key in ['startup_executions','file_executions','shell_executions','command_executions','bridge_returns'])

# Use sets of signal/flag numbers rather than the native service's bitwise
# update instructions. Both implement the already specified UAPI encoding.
def signals(value): return {n for n in range(1,65) if value>>(n-1)&1}
def encode(values): return sum(2**(n-1) for n in values)
def flags(value): return sum(2**n for n in [0,1,2,11,26,27,28,30,31] if value>>n&1)
unblockable={9,19}
readonly=[]
output_faults=[]
overlaps=[]
for invocation in observation['executions']:
    calls=invocation['calls']
    assert [c['number'] for c in calls]==[12,12]+[13]*23+[14]*23+[12]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    rows=calls[2:-1]
    assert sum(c['result']==-14 for c in rows)==22
    assert sum(c['result']==-22 for c in rows)==6
    for c in rows:
        n=c['number']
        selector,new,old,size=c['args']
        width=32 if n==13 else 8
        source=bytes.fromhex(c['input_hex'])
        before=c['state_before']
        expected=dict(blocked=before['blocked'],action10=before['action10'][:])
        error=0
        committed=False
        payload=None
        if size!=8: error=-22
        elif new and len(source)!=width: error=-14
        elif n==13:
            if selector<1 or selector>64 or (new and selector in unblockable): error=-22
            else:
                assert selector==10
                payload=struct.pack('<4Q',*before['action10'])
                if new:
                    handler,attributes,restorer,mask=struct.unpack('<4Q',source)
                    expected['action10']=[handler,flags(attributes),restorer,encode(signals(mask)-unblockable)]
                    committed=True
        else:
            payload=struct.pack('<Q',before['blocked'])
            if new:
                if selector not in [0,1,2]: error=-22
                else:
                    requested=signals(struct.unpack('<Q',source)[0])-unblockable
                    current=signals(before['blocked'])
                    chosen=[current|requested,current-requested,requested][selector]
                    expected['blocked']=encode(chosen)
                    committed=True
        if error==0 and old:
            if 0x5f7000<=old and old+width<=0x63a000:
                assert bytes.fromhex(c['output_hex'])==payload
            else: error=-14
        assert c['result']==error and c['committed']==committed
        assert c['state_after']==expected
        assert c['phase_before']==c['phase_after'] and c['complete_table_verified'] and c['private_staging_zero']
        if component['base']<=new<component['base']+len(raw):
            assert virtual(image,headers,new,width)==(source,5)
            readonly.append(dict(number=n,pointer=new,input_hex=source.hex(),load_flags=5))
        if committed and error==-14:
            assert expected!=before and old==0x400000
            assert len(bytes.fromhex(c['old_output_prefix_before']))==width
            output_faults.append(dict(number=n,before=before,after=expected))
        if new and new==old and error==0:
            assert error==0 and payload==bytes.fromhex(c['output_hex'])
            overlaps.append(dict(number=n,pointer=new,old_payload=payload.hex(),new_state=expected))
    assert rows[-1]['state_after']==rows[0]['state_before']
assert len(readonly)==36 and len(output_faults)==8 and len(overlaps)==8
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
previous=Path('linux-kernel/.cache/x86_64/user-extent-source-2256205-1794392689887672216/static-elf-abi-fixture.elf').read_bytes()
assert transfer==previous
report=dict(passed=True,source_invocations=4,action_syscalls=92,mask_syscalls=92,heap_syscalls=12,
    total_observed_syscalls=196,source_text_bytes=len(raw),source_text_sha256=digest(raw),
    source_stack_bytes=stack,stack_frame_costs=groups,readonly_inputs=readonly,
    output_fault_commits=output_faults,overlap_transfers=overlaps,kernel_comparisons=comparison,
    serial_bytes=len(serial),serial_sha256=digest(serial),preceding_transfer_fixture_unchanged=True,
    transfer_fixture_sha256=digest(transfer),runtime_call=call.hex(),
    scope='Selected source-issued action/mask transfers with a separate set-based oracle; complete-table equality is checked by the retained native observer, not reconstructed from a full table transcript; no production qualification')
(output/'independent-readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['readonly_inputs','output_fault_commits','overlap_transfers','kernel_comparisons']}))
