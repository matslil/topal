"""Reconcile saved source calls, observed copies and scoped stat field rules."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import runpy
import struct
import sys

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
obs_bytes = (build/'source-user-extents/observations.json').read_bytes()
obs = json.loads(obs_bytes)
assert obs['passed'] and len(obs['executions']) == 4
copies = obs['copy_transfer_observation']
assert copies['passed'] and copies['source_syscall_scoped'] and copies['audit_kind'] == 'staging-and-stat'
audit_bytes = (build/'stat-output-audit.json').read_bytes()
audit = json.loads(audit_bytes)
assert audit['passed'] and not audit['complete']
assert copies['kernel_sha256'] == audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert copies['stat_audit_sha256'] == digest(audit_bytes)
completed,refused = iter(copies['source_syscall_completions']),iter(copies['source_syscall_refusals'])
events,errors,skipped,variants = [],[],[],Counter()
def record(fields):
    raw = bytearray(144)
    for offset,value in fields.items(): struct.pack_into('<Q',raw,offset,value)
    return bytes(raw)
def check_fields(payload,variant,volatile_index):
    q = struct.unpack('<18Q',payload)
    assert payload[28:40] == bytes(12) and payload[80:88] == bytes(8)
    assert payload[96:104] == bytes(8) and payload[112:144] == bytes(32)
    assert q[7] == 4096
    if variant == 'terminal':
        assert payload == record({0:1,8:1,16:1,24:0x2180,40:0x4400,56:4096})
    elif variant == 'immutable-file':
        assert payload == record({0:1,8:5,16:1,24:0o100755,48:2024544,56:4096,64:3955})
    elif variant in ['directory-root','directory-bin']:
        inode,links = (3,4) if variant == 'directory-root' else (4,2)
        assert payload == record({0:1,8:inode,16:links,24:0o40755,56:4096})
    elif variant == 'directory-tmp':
        assert (q[0],q[1],q[2],q[5],q[6],q[8]) == (2,6,2,0,0,0)
        assert q[3]&0o170000 == 0o40000
    elif variant in ['pipe-reader','pipe-writer']:
        assert (q[0],q[2],q[3],q[5],q[6],q[8]) == (2,1,0o10600,0,0,0)
        assert q[1] > 0 and q[9] == q[11] == q[13] > 0
    else:
        assert variant == 'volatile-file' and 0 <= volatile_index < 5
        assert q[0] == 2 and q[1] >= 7 and q[5] == 0
        assert q[3]&0o170000 == 0o100000 and q[3]&~0o100600 == 0
        assert (q[2],q[6],q[8]) == [(0,0,0),(0,0,0),(1,4,8),(1,4,8),(0,4,8)][volatile_index]
    # Dynamic mode/time/inode fields retain the live observer's private-data
    # oracle. This separate pass checks source order, constants and relations;
    # it is not an independent reconstruction of every private metadata read.

for execution in obs['executions']:
    assert len(execution['calls']) == 38
    volatile,pipe = [],[]
    for call in execution['calls']:
        if call['number'] not in [5,262]: continue
        n,args,result = call['number'],call['args'],call['result']
        pointer = args[1 if n == 5 else 2]
        name = 'descriptor-stat-output' if n == 5 or args[1] == 0 else 'namespace-stat-output'
        assert call['stat_pointer'] == pointer and call['phase_before'] == call['phase_after']
        assert call['stat_private_state_preserved'] and call['stat_output_window_verified']
        variant = call['stat_variant']
        if result == 0:
            event = next(completed)
            assert (event['name'],event['number'],event['args'],event['candidate'],event['size'],event['result']) == (name,n,args,pointer,144,0)
            assert all(event[key] for key in ['helper_registers_preserved','captured_arguments_preserved','translation_observations_equal'])
            payload = bytes.fromhex(event['payload_hex'])
            assert payload.hex() == call['stat_output_hex'] == call['stat_expected_hex']
            check_fields(payload,variant,len(volatile))
            if variant == 'volatile-file': volatile.append(payload)
            if variant in ['pipe-reader','pipe-writer']: pipe.append(payload)
            variants[variant] += 1
            events.append(dict(name=name,variant=variant,number=n,pointer=pointer,payload_sha256=digest(payload)))
        elif result == -14:
            event = next(refused)
            assert (event['name'],event['number'],event['args'],event['candidate'],event['size'],event['result']) == (name,n,args,pointer,144,-14)
            assert event['phase_before'] == event['phase_after'] == call['phase_before']
            errors.append(dict(name=name,number=n,pointer=pointer,result=result))
        else:
            assert result in [-9,-2] and call['stat_expected_hex'] is None
            skipped.append(dict(number=n,args=args,result=result))
    assert len(volatile) == 5 and volatile[0] == volatile[1] and volatile[2] == volatile[3]
    assert volatile[0][8:16] != volatile[2][8:16] and volatile[2][8:16] == volatile[4][8:16]
    assert len(pipe) == 2 and pipe[0] == pipe[1]
assert next(completed,None) is None and next(refused,None) is None
assert (len(events),len(errors),len(skipped),sum(variants.values())) == (60,20,8,60)
assert copies['counts']['descriptor-stat-output'] == dict(attempts=52,admitted=40,refused=12,completed=40,bytes=5760)
assert copies['counts']['namespace-stat-output'] == dict(attempts=28,admitted=20,refused=8,completed=20,bytes=2880)
assert all(not row['attempts'] for name,row in copies['counts'].items() if name not in ['descriptor-stat-output','namespace-stat-output'])
assert len(variants) == 8
for name in ['user_extent_stat_observer.py','user_extent_copy_observer.py']:
    program = runpy.run_path(str(Path('linux-kernel/labs/qemu/x86_64')/name))['PROGRAM']
    assert program in (build/'source-user-extents/inspect.gdb').read_text()
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(obs_bytes),source_syscalls=152,stat_queries=88,
    completed_copies=events,refused_output_admissions=errors,lookup_errors_without_registered_output_events=skipped,
    variants=dict(variants),saved_observer_programs_present=True,
    scope='Independent saved-source/event order and selected stat constants/relations, with runtime payload/private-oracle correspondence. Dynamic mode/time/inode fields retain the live observer oracle. Not a second native run, independent reconstruction of all private reads, complete owners/nonalias/lifetime/callee/exception or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=152,queries=88,completed=60,refused=20,lookup_errors=8,variants=len(variants))))
