"""Independent ELF and runtime readback for source-built file and startup tests."""
import hashlib
import json
from pathlib import Path
import struct
import sys

build = Path(sys.argv[1])
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=True)
def digest(data): return hashlib.sha256(data).hexdigest()
def elf(path):
    data = path.read_bytes()
    assert data[:6] == b'\x7fELF\x02\x01'
    phoff, shoff = struct.unpack_from('<QQ', data, 32)
    phsize, phnum, shsize, shnum, shstr = struct.unpack_from('<HHHHH', data, 54)
    assert phsize == 56
    loads = [struct.unpack_from('<IIQQQQQQ', data, phoff+i*phsize)
             for i in range(phnum)]
    if not shnum:
        assert shoff == 0
        return data, loads, {}
    assert shsize == 64
    sections = [struct.unpack_from('<IIQQQQIIQQ', data, shoff+i*shsize)
                for i in range(shnum)]
    strings = data[sections[shstr][4]:sections[shstr][4]+sections[shstr][5]]
    named = {}
    for section in sections:
        at = section[0]
        name = strings[at:strings.index(0, at)].decode()
        named[name] = section
    return data, loads, named

component = json.loads((build/'source-component.json').read_text())
linked, _, sections = elf(build/'source-user.elf')
image, loads, _ = elf(build/'static-elf-abi-fixture.elf')
text = sections['.text']
raw = linked[text[4]:text[4]+text[5]]
assert text[3] == component['base'] and len(raw) == component['text_bytes']
assert digest(linked) == component['linked_sha256']
assert digest(image) == component['image_sha256']
def virtual(data, headers, address, size):
    matching = [p for p in headers if p[0] == 1 and p[3] <= address
                and address+size <= p[3]+p[5]]
    assert len(matching) == 1
    p = matching[0]
    at = p[2]+address-p[3]
    return data[at:at+size], p
actual, load = virtual(image, loads, component['base'], len(raw))
assert actual == raw and load[1] == 5
adapter3 = bytes.fromhex('48 89 f8 48 89 f7 48 89 d6 48 89 ca 0f 05 c3')
adapter4 = bytes.fromhex('4d 89 c2')+adapter3
for key, expected in [('syscall',adapter3),('syscall4',adapter4)]:
    data, _ = virtual(image, loads, component[key], len(expected))
    assert data == expected
bridge, _ = virtual(image, loads, component['startup_call'], 61)
prefix = bytes.fromhex('57 56 52 41 52 41 50 41 51 41 54 49 89 e4 48 83 e4 f0 48 89 c7 49 8b 74 24 30 49 8b 54 24 28 49 8b 4c 24 20 4d 8b 44 24 18 e8')
suffix = bytes.fromhex('4c 89 e4 41 5c 41 59 41 58 41 5a 5a 5e 5f c3')
assert len(bridge) == len(prefix)+4+len(suffix)
assert bridge[:len(prefix)] == prefix and bridge[-len(suffix):] == suffix
destination = component['startup_call']+len(prefix)+4+struct.unpack_from('<i',bridge,len(prefix))[0]
assert destination == component['startup']
startup, _ = virtual(image, loads, 0x4a6100, 0x9f00)
target = component['startup_call']
sites = []
for at in range(7,len(startup)-11):
    if startup[at] == 0xe8 and 0x4a6100+at+5+struct.unpack_from('<i',startup,at+1)[0] == target:
        assert startup[at-7:at] == bytes.fromhex('48 81 ec 00 04 00 00')
        assert startup[at+5:at+12] == bytes.fromhex('48 81 c4 00 04 00 00')
        sites.append(0x4a6100+at)
assert len(sites) == 68
frames = []
groups = {}
for stem in ['source-cases','startup-cases','file-cases','shell-cases','command-cases']:
    groups[stem] = 0
    for line in (build/(stem+'.su')).read_text().splitlines():
        name,size,kind = line.split('\t')
        assert kind == 'static'
        frames.append(dict(function=name,bytes=int(size)))
        groups[stem] += int(size)+8
assert groups == component['stack_frame_costs']
stack = 1024+128+max(groups['source-cases'],groups['startup-cases']+max(groups['file-cases'],groups['shell-cases'],groups['command-cases']))
assert stack == component['conservative_user_stack_bytes'] <= 2048
observation = json.loads((build/'source-user-extents/observations.json').read_text())
assert observation['passed']
assert len(observation['executions']) == 4
assert [r['number'] for r in observation['startup_executions']] == [302,318,318,157,262]*4
assert len(observation['bridge_returns']) == 83
startup_calls = [c for row in observation['startup_executions'] for c in row['calls']]
shared_calls = [c for row in observation['executions'] for c in row['calls']]
assert len(startup_calls) == 200 and len(shared_calls) == 56
file_calls = [c for row in observation['file_executions'] for c in row['calls']]
assert len(file_calls) == 176
baseline = Path('linux-kernel/.cache/x86_64/user-extent-source-poll-2263956-1794395971230199800')
comparison = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o',
             'dispatch.o','gateway.o','boundary.o','extent.o','extent-bridge.o',
             'extent-callers.o','provider.o','predecessor-provider.o','predecessor-gateway.o',
             'kernel.elf','bzImage']:
    left, right = (build/name).read_bytes(), (baseline/name).read_bytes()
    assert left == right, name
    comparison.append(dict(path=name,bytes=len(left),sha256=digest(left)))
serial = (build/'source-user-extents/serial.log').read_bytes()
assert serial == (baseline/'source-user-extents/serial.log').read_bytes()
# Independently reconstruct the entire native records, including padding.
def stat_record(ino,links,mode,rdev=0,size=0,blocks=0):
    return struct.pack('<18Q',0,ino,links,mode,0,rdev,size,4096,blocks,*([0]*9))
def directory_record(ino,offset,length,kind,name):
    data=struct.pack('<QQHB',ino,offset,length,kind)+name+b'\0'
    assert len(data)<=length
    return data+bytes(length-len(data))
expected_stats=[stat_record(1,1,0x2180,rdev=0x4400),stat_record(2,1,0x8124),
                stat_record(3,3,0x41ed),stat_record(4,2,0x41ed)]
stats=[bytes.fromhex(row['output_after']) for row in observation['file_executions'] if row['number']==5]
assert stats==expected_stats
for row in observation['file_executions']:
    if row['number']==262 and 'output_after' in row:
        path=bytes.fromhex(row['path'])
        expected=stat_record(4,2,0x41ed) if path==b'/bin\0' else stat_record(5,1,0x816d,size=2024544,blocks=3955)
        assert bytes.fromhex(row['output_after'])==expected
root_dir=(directory_record(3,1,24,4,b'.')+directory_record(3,2,24,4,b'..')+
          directory_record(4,3,24,4,b'bin'))
bin_dir=(directory_record(4,1,24,4,b'.')+directory_record(3,2,24,4,b'..')+
         directory_record(5,3,32,8,b'busybox'))
directories=[bytes.fromhex(row['output_after']) for row in observation['file_executions'] if row['number']==217 and 'output_after' in row]
assert directories==[root_dir,bin_dir]
for row in observation['file_executions']:
    if row['number']==217 and 'output_after' in row:
        call=row['calls'][-2]
        assert call['args'][1:3]==[0x639fb0,512]
        assert call['args'][1]+80==0x63a000 and call['result']<=80


shell_calls=[c for row in observation['shell_executions'] for c in row['calls']]
command_calls=[c for row in observation['command_executions'] for c in row['calls']]
assert len(observation['shell_executions'])==38 and len(command_calls)==65
for row in observation['shell_executions']:
    n=row['number']
    actions=row['calls'][2:-1]
    if n==13:
        last=actions[-1]
        if row['input']:
            pointer=last['args'][1]
            data,p=virtual(image,loads,pointer,32)
            expected=struct.pack('<QQQQ',0x4a6580 if row['args'][0]==17 else 1,0x4000000,0x4a6580,0xfffffffe7fffffff)
            assert p[1]==5 and data==expected==bytes.fromhex(last['input_hex'])
        else: assert bytes.fromhex(row['output_after'])==bytes(32)
    if n==79:
        assert actions[-1]['args'][0]==0x639ffe and actions[-1]['result']==2
        assert bytes.fromhex(row['output_after'])==b'/\0'
    if n==0:
        assert row['args'][0]==3 and actions[-1]['args'][1]==0x639fff
commands=[b'echo topal\n',b'cd /\n',
    b'echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read; exec 3<&-; echo busybox-close\n',
    b'exec 3</bin/busybox;pwd\n',b'exec /bin/busybox true\n']
assert len(commands[2])==294
for row,command in zip(observation['command_executions'],commands,strict=True):
    assert bytes.fromhex(row['delivered_hex'])==command
    actions=row['calls'][2:-1]
    assert [call['result'] for call in actions]==[0]+[-14]*5+[1,0,-14,len(command)-1]
    expected=bytearray.fromhex(row['queue_before'])
    assert expected==bytes(512)
    if row['args'][2]==23:
        expected[:23]=command
        struct.pack_into('<Q',expected,264,23)
        struct.pack_into('<Q',expected,272,6)
    for call in actions[1:6]: assert bytes.fromhex(call['queue_after'])==expected
    if row['args'][2]==23:
        expected[:256]=command[1:]+bytes(256-len(command)+1)
        struct.pack_into('<Q',expected,264,22)
    else: struct.pack_into('<Q',expected,256,1)
    for call in actions[6:9]: assert bytes.fromhex(call['queue_after'])==expected
    if row['args'][2]==23:
        expected[:256]=bytes(256)
        struct.pack_into('<Q',expected,264,0)
    else: struct.pack_into('<Q',expected,256,0)
    assert bytes.fromhex(actions[-1]['queue_after'])==expected

report = dict(passed=True,linked_text_bytes=len(raw),text_sha256=digest(raw),
              executable_load_flags=load[1],adapters={'syscall3':adapter3.hex(),'syscall4':adapter4.hex(),
              'startup_bridge':bridge.hex(),'source_target':destination,'call_sites':sites},
              user_stack_bytes=stack,static_frames=frames,
              shared_invocations=4,shared_syscalls=len(shared_calls),
              startup_invocations=20,startup_syscalls=len(startup_calls),
              bridge_returns=83,shell_invocations=38,shell_syscalls=len(shell_calls),command_invocations=5,command_syscalls=len(command_calls),file_invocations=20,file_syscalls=len(file_calls),stack_frame_costs=groups,kernel_comparisons=comparison,
              serial_bytes=len(serial),serial_sha256=digest(serial),
              scope='Selected native shell/command/file/startup/shared calls and separate ELF byte/queue models; no complete caller or production qualification')
(output/'independent-readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['adapters','static_frames','kernel_comparisons']}))
