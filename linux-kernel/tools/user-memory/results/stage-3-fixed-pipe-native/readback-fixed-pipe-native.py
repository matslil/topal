"""Reconcile native fixed outputs/refusals with ordinary source syscall order."""
import hashlib
import json
from pathlib import Path
import runpy
import struct
import sys

build, output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
obs_bytes = (build/'source-user-extents/observations.json').read_bytes()
obs = json.loads(obs_bytes)
assert obs['passed'] and len(obs['executions']) == 4
fixed = obs['pipe_fixed_output_observation']
audit_bytes = (build/'pipe-fixed-output-audit.json').read_bytes()
audit = json.loads(audit_bytes)
assert fixed['passed'] and audit['passed'] and not audit['complete']
assert fixed['kernel_sha256'] == audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert fixed['audit_sha256'] == digest(audit_bytes)
cases = {r['name']:r for r in audit['cases']}
records = iter(fixed['records'])
refusals = iter(fixed['refusals'])
matched,denied = [],[]
calls = sum((row['calls'] for row in obs['executions']),[])
mask = 0x000ffffffffff000

def translation(record):
    at,remaining = record['candidate'],record['size']
    for mapping in record['mapping']:
        assert mapping['pointer'] == at and 0 < mapping['size'] <= min(remaining,4096-(at&4095))
        assert mapping['user'] and mapping['writable'] and mapping['leaf_shift'] == 12
        assert [r[1] for r in mapping['path']] == [39,30,21,12]
        table = record['mode'][1]&mask
        for actual,shift,entry in mapping['path']:
            assert actual == table and entry&7 == 7
            assert not (shift in [30,21] and entry&128)
            table = entry&mask
        assert mapping['physical'] == table+(at&4095)
        for private in record['private_exclusions']:
            assert private['size'] == sum(m['size'] for m in private['mapping'])
            for other in private['mapping']:
                assert not other['user'] and other['writable']
                assert other['physical'] == other['pointer']
                assert mapping['physical']+mapping['size'] <= other['physical'] or other['physical']+other['size'] <= mapping['physical']
        tables = {r[0] for r in mapping['path']}
        assert all(mapping['physical']+mapping['size'] <= table or table+4096 <= mapping['physical'] for table in tables)
        at,remaining = at+mapping['size'],remaining-mapping['size']
    assert remaining == 0

for execution in obs['executions']:
    count = 0
    for call in execution['calls']:
        n,result,args = call['number'],call['result'],call['args']
        if n in [22,293,16]:
            name = 'pipe-pair-output' if n in [22,293] else 'pipe-readable-count-output'
            pointer,width = (args[0],8) if n in [22,293] else (args[2],4)
            if result < 0:
                assert result == -14
                event = next(refusals)
                assert (event['name'],event['number'],event['args'],event['candidate'],event['size']) == (name,n,args,pointer,width)
                assert event['result'] == result and event['phase_before'] == event['phase_after'] == call['phase_before'] == call['phase_after']
                assert event['helper_registers_preserved'] and event['captured_arguments_preserved']
                denied.append(dict(name=name,number=n,pointer=pointer,bytes=width))
            else:
                assert result == 0
                event = next(records)
                assert (event['name'],event['number'],event['args'],event['candidate'],event['size']) == (name,n,args,pointer,width)
                assert event['helper_registers_preserved'] and event['mode'][0]&(1<<16)
                translation(event)
                expected = struct.pack('<II',*call['pair']) if n in [22,293] else struct.pack('<I',count)
                expected_accesses = cases[name]['access_instructions']
                assert len(event['accesses']) == len(expected_accesses)
                for access,contract in zip(event['accesses'],expected_accesses):
                    assert (access['offset'],access['action'],access['bytes']) == (contract['offset'],contract['action'],width)
                    assert bytes.fromhex(access['payload_hex']) == expected
                    assert all(access[k] for k in ['exact_payload_equal','boundary_mapping_equal',
                        'candidate_count_preserved','captured_arguments_preserved'])
                matched.append(dict(name=name,number=n,pointer=pointer,bytes=width,
                                    payload_hex=expected.hex(),accesses=len(event['accesses'])))
        if n == 1 and result > 0:
            count += result
        elif n == 0 and result > 0:
            count -= result
        assert 0 <= count <= 4096
    assert count == 0 and execution['pipe_retirement_verified']
assert next(records,None) is None and next(refusals,None) is None
assert (len(calls),len(matched),len(denied),sum(r['accesses'] for r in matched)) == (144,24,40,28)
assert fixed['counts'] == {
    'pipe-pair-output':dict(attempts=24,admitted=4,refused=20,completed=4),
    'pipe-readable-count-output':dict(attempts=40,admitted=20,refused=20,completed=20)}
program = runpy.run_path('linux-kernel/labs/qemu/x86_64/user_extent_pipe_fixed_observer.py')['PROGRAM']
assert program in (build/'source-user-extents/inspect.gdb').read_text()
baseline = Path('linux-kernel/.cache/x86_64/user-extent-source-pipes-2367848-1794444753502668964')
identical = []
for name in ['kernel.elf','provider.o','extent-call-inventory.json','process-access-inventory.json',
             'process-access-origins.json','pipe-fixed-output-audit.json',
             'static-elf-abi-fixture.elf','initramfs.cpio']:
    data = (build/name).read_bytes()
    assert data == (baseline/name).read_bytes(),name
    identical.append(dict(path=name,bytes=len(data),sha256=digest(data)))
report = dict(passed=True,kernel_sha256=fixed['kernel_sha256'],observation_sha256=digest(obs_bytes),
    audit_sha256=digest(audit_bytes),source_syscalls=len(calls),completed_transfers=matched,
    refused_transfers=denied,user_access_observations=28,byte_identical_artifacts=identical,
    observer_program_present_in_saved_gdb=True,
    scope='Independent saved-native/source-syscall correspondence and queue-count/payload/translation/exclusion readback. These are the same native run, not a second execution or complete owner/nonalias/continuous-lifetime/entry/callee/exception/production proof')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=len(calls),completed=len(matched),refused=len(denied),accesses=28,identical_artifacts=len(identical))))
