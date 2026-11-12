"""Replay complete saved metadata effects and join public calls to clock sites."""
import hashlib
import json
from pathlib import Path
import runpy
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest
from metadata_model import description,inode,transition,word

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
raw=(build/'source-user-extents/observations.json').read_bytes()
obs=json.loads(raw)
assert obs['passed'] and len(obs['executions'])==4
component=json.loads((build/'source-component.json').read_text())
image=(build/'static-elf-abi-fixture.elf').read_bytes()
linked=Elf(build/'source-user.elf')
strings=linked.section_bytes(struct.unpack_from('<H',linked.data,62)[0])
section,=[row for row in linked.sections if strings[row[0]:strings.index(0,row[0])]==b'.text']
text=linked.data[section[4]:section[4]+section[5]]
assert component['mode']=='metadata' and component['image_sha256']==digest(image)
assert component['linked_sha256']==digest(linked.data)
assert component['metadata_source_sha256']==digest(Path('linux-kernel/labs/qemu/x86_64/user_extent_metadata_cases.c').read_bytes())
assert component['base']==section[3] and component['text_bytes']==len(text)
assert linked.symbols['topal_user_extent_metadata_cases']['value']==component['entry']==struct.unpack_from('<Q',linked.data,24)[0]
assert component['conservative_user_stack_bytes']<=2048
frames={}
for stem in component['stack_frame_costs']:
    total=0
    for line in (build/(stem+'.su')).read_text().splitlines():
        columns=line.split('\t')
        assert len(columns)==3 and columns[2]=='static'
        total+=int(columns[1])+8
    frames[stem]=total
assert frames==component['stack_frame_costs']
assert component['conservative_user_stack_bytes']==1024+128+frames['metadata-cases']
adapter=bytes.fromhex('4d89c24889f84889f74889d64889ca0f05c3')
assert text[component['syscall4']-component['base']:component['syscall4']-component['base']+len(adapter)]==adapter
assert image[:7]==bytes.fromhex('7f454c46020101')
segments=[]
for at in range(struct.unpack_from('<H',image,56)[0]):
    phoff=struct.unpack_from('<Q',image,32)[0]
    entsize=struct.unpack_from('<H',image,54)[0]
    assert entsize==56
    kind,flags,offset,virtual,physical,filesz,memsz,align=struct.unpack_from('<II6Q',image,phoff+at*entsize)
    if kind==1 and virtual<=component['base'] and component['base']+len(text)<=virtual+filesz:
        assert flags&1 and not flags&2
        start=offset+component['base']-virtual
        assert image[start:start+len(text)]==text
        segments.append(dict(virtual=virtual,flags=flags,file_offset=offset))
assert len(segments)==1
observer=runpy.run_path('linux-kernel/labs/qemu/x86_64/user_extent_metadata_observer.py')['PROGRAM']
assert observer in (build/'source-user-extents/inspect.gdb').read_text()
expected_numbers=[12,12,90,257,5,5,18,8,5,77,5,91,91,5,86,5,82,87,5,257,5,18,8,5,91,86,82,87,5,3,5,3,87,12]
def apply(before,deltas):
    actual=bytearray(before)
    end=0
    for row in deltas:
        start=row['offset']
        old,new=bytes.fromhex(row['before']),bytes.fromhex(row['after'])
        assert old and len(old)==len(new) and end<=start and start+len(old)<=len(actual)
        assert actual[start:start+len(old)]==old and all(a!=b for a,b in zip(old,new))
        actual[start:start+len(new)]=new
        end=start+len(new)
    return bytes(actual)
clocks_by_syscall={}
events=[]
for iteration,row in enumerate(obs['executions']):
    assert [call['number'] for call in row['calls']]==expected_numbers
    assert row['metadata_retirement_verified']
    fs,table=bytes.fromhex(row['metadata_initial_filesystem']),bytes.fromhex(row['metadata_initial_table'])
    initial_table=table
    initial_identity=word(fs,0)
    actions=[call for call in row['calls'] if call['number']!=12]
    assert [call['result'] for call in actions if call['result']<0]==[-14,-9,-17,-14,-2]
    for call in actions:
        n,args,result=call['number'],call['args'],call['result']
        clocks=call['metadata_clocks']
        mutation=result>=0 and n in [257,18,77,91,82,86,87]
        assert len(clocks)==int(mutation) and all(clock['carry']==0 for clock in clocks)
        seconds=clocks[0]['seconds'] if clocks else None
        paths=[bytes.fromhex(p) if p is not None else None for p in call['metadata_paths']]
        pointers=([args[1]] if n==257 else [args[0]] if n in [90,87] else args[:2] if n in [82,86] else [])
        assert len(pointers)==len(paths)
        for pointer,path in zip(pointers,paths):
            if path is None:
                assert pointer==0x63a000
            else:
                at=pointer-component['base']
                assert 0<=at and at+len(path)<len(text) and text[at:at+len(path)+1]==path+b'\0'
        if n==18:
            at=args[1]-component['base']
            assert 0<=at and at+args[2]<=len(text)
            assert call['metadata_payload']==text[at:at+args[2]].hex()
        expected_fs,expected_table=transition(fs,table,n,args,result,paths,seconds,bytes.fromhex(call['metadata_payload']))
        assert apply(fs,call['metadata_filesystem_delta'])==expected_fs
        assert apply(table,call['metadata_table_delta'])==expected_table
        assert call['metadata_filesystem_sha256']==digest(expected_fs)
        assert call['metadata_table_sha256']==digest(expected_table)
        if n==5:
            assert result==0 and call['stat_private_state_preserved'] and call['stat_output_window_verified']
            identity=word(table,description(table,args[0])+32)
            at=inode(fs,identity)
            expected=bytearray(144)
            fields={0:2,8:identity,16:word(fs,at+56),24:word(fs,at+24),48:word(fs,at+16),56:4096,
                64:sum(word(fs,1024+i*32)==identity for i in range(8))*8,
                72:word(fs,at+32),88:word(fs,at+40),104:word(fs,at+48)}
            for offset,value in fields.items():struct.pack_into('<Q',expected,offset,value)
            assert call['stat_output_hex']==expected.hex()
        for clock in clocks:clocks_by_syscall.setdefault(str(n),set()).add(clock['call_offset'])
        events.append(dict(iteration=iteration,number=n,result=result,clocks=clocks,
            filesystem_sha256=digest(expected_fs),table_sha256=digest(expected_table)))
        fs,table=expected_fs,expected_table
    assert table==initial_table and word(fs,0)==initial_identity+1
    assert fs[64:128]==bytes(64) and fs[256:]==bytes(len(fs)-256)
audit=(build/'stack-retirement-audit.json').read_bytes()
clock_sites={row['previous_offset']:row for row in json.loads(audit)['blocks'] if row['name'].startswith('clock-caller-')}
assert len(clock_sites)==9
for event in events:
    for clock in event['clocks']:
        assert clock['call_offset'] in clock_sites, 'unlisted metadata clock site'
        assert clock_sites[clock['call_offset']]['name']==clock['block']
native=obs['stack_retirement_observation']
assert native['passed'] and native['audit_sha256']==digest(audit)
report=dict(passed=True,kernel_sha256=digest((build/'kernel.elf').read_bytes()),
    observation_sha256=digest(raw),source_component_sha256=digest((build/'source-component.json').read_bytes()),
    metadata_source_sha256=component['metadata_source_sha256'],metadata_model_sha256=digest(Path('linux-kernel/tools/user-memory/metadata_model.py').read_bytes()),
    iterations=4,syscalls=136,modeled_calls=len(events),stat_outputs=44,clock_returns=sum(len(row['clocks']) for row in events),
    clocks_by_syscall={n:sorted(sites) for n,sites in clocks_by_syscall.items()},events=events,
    loaded_source_text_sha256=digest(text),source_text_bytes=len(text),source_load_segments=segments,
    user_stack_bound=component['conservative_user_stack_bytes'],ordinary_syscall_adapter_equal=True,
    compiled_stack_usage_verified=True,
    frozen_observer_and_model_present=True,complete_saved_delta_replay_equal=True,
    scope='Independent saved-record replay of complete filesystem/table transitions and all stat records, actual public call/clock joins, source ELF/load-byte and ordinary ABI adapter binding. No new native run, cryptographic authentication, independent kernel owner/nonalias/lifetime proof or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=136,modeled_calls=len(events),stat_outputs=44,clock_returns=report['clock_returns'])))
