"""Reconcile saved native warning admissions, loads and staged register bytes."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
raw=(build/'source-user-extents/observations.json').read_bytes()
obs=json.loads(raw)
audit_bytes=(build/'warning-input-audit.json').read_bytes()
audit=json.loads(audit_bytes)
native=obs['warning_input_observation']
assert obs['passed'] and native['passed'] and native['audit_sha256']==digest(audit_bytes)
assert obs['extent_call_coverage']['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
case=audit['cases'][0]
records=native['records']
assert len(records)==3
payloads=[b'sh: ',b"can't access tty; job control turned off",b'\n']
events=[]
for record,variant,payload in zip(records,case['variants'],payloads):
    assert record['accepted'] and record['selector']==variant['selector']
    assert record['size']==variant['admitted_bytes']==len(payload)
    assert bytes.fromhex(record['payload_hex'])==payload
    assert all(record[k] for k in ['helper_registers_preserved','captured_arguments_preserved',
        'selector_preserved','complete_register_staging_equal','translation_observations_equal'])
    assert len(record['accesses'])==len(variant['access_instructions'])
    loaded=[]
    for actual,expected in zip(record['accesses'],variant['access_instructions']):
        assert all(actual[k]==expected[k] for k in ['offset','candidate_offset','width'])
        offset,width=expected['candidate_offset'],expected['width']
        assert bytes.fromhex(actual['payload_hex'])==payload[offset:offset+width]
        loaded.append(bytes.fromhex(actual['payload_hex']))
    assert b''.join(loaded)==payload
    events.append(dict(selector=record['selector'],candidate=record['candidate'],size=len(payload),
        scalar_loads=len(record['accesses']),payload_sha256=digest(payload)))
reached=next(r for r in obs['extent_call_coverage']['reached'] if r['offset']==case['call_offset'])
assert (reached['candidate'],reached['length'])==(records[0]['candidate'],records[0]['size'])
serial=(build/'source-user-extents/serial.log').read_bytes()
assert serial.count(b''.join(payloads))==1
program=(build/'source-user-extents/inspect.gdb').read_text()
observer=runpy.run_path('linux-kernel/labs/qemu/x86_64/user_extent_warning_observer.py')['PROGRAM']
assert observer in program
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(raw),serial_sha256=digest(serial),events=events,total_bytes=45,
    scalar_loads=7,exact_order_payload_and_first_reach_equal=True,saved_observer_program_present=True,
    scope='Independent saved-record selector order, admitted lengths, scalar operands/payloads and staged bytes, joined to first reach and actual serial warning. Register/helper/mapping assertions retain their native observer evidence; not another native run, refusal coverage, continuous lifetime, independent owners or complete warning/production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,variants=len(events),scalar_loads=7,bytes=45)))
