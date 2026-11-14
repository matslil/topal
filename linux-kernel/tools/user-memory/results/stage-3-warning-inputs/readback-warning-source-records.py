"""Join warning admission records to source-built ordinary syscall refusals."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

records,audit_path,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=False)
digest=lambda raw:hashlib.sha256(raw).hexdigest()
raw=(records/'observations.json').read_bytes()
obs=json.loads(raw)
audit_bytes=audit_path.read_bytes()
audit=json.loads(audit_bytes)
native=obs['warning_input_observation']
assert obs['passed'] and native['passed'] and native['audit_sha256']==digest(audit_bytes)
assert obs['extent_call_coverage']['kernel_sha256']==audit['kernel_sha256']
events=iter(native['records'])
payloads=[b'sh: ',b"can't access tty; job control turned off",b'\n']
qualified=[]
for payload,variant in zip(payloads,audit['cases'][0]['variants']):
    wrappers=[e for e in obs['shell_executions'] if e['number']==1 and e['args'][0]==2
        and e['args'][2]==len(payload) and bytes.fromhex(e['original_bytes'])==payload]
    assert len(wrappers)==1
    calls=[c for c in wrappers[0]['calls'] if c['number']==1]
    assert len(calls)==5
    bad=[0x63a000,(1<<64)-1,0x800000,0x63b000 if len(payload)==1 else 0x639fff]
    for index,call in enumerate(calls):
        event=next(events)
        assert call['args'][0]==2 and call['args'][1]==event['candidate']
        assert call['args'][2]==event['size']==len(payload)
        assert event['selector']==variant['selector'] and call['phase_before']==call['phase_after']==26
        if index<4:
            assert call['result']==-14 and not event['accepted'] and event['accesses']==[]
            assert event['candidate']==bad[index] and call['queue_before']==call['queue_after']
        else:
            assert call['result']==len(payload) and event['accepted']
            assert event['candidate']==0x638ff8 and bytes.fromhex(event['payload_hex'])==payload
            assert len(event['accesses'])==len(variant['access_instructions'])
            for actual,expected in zip(event['accesses'],variant['access_instructions']):
                assert all(actual[k]==expected[k] for k in ['offset','candidate_offset','width'])
                offset,width=actual['candidate_offset'],actual['width']
                assert bytes.fromhex(actual['payload_hex'])==payload[offset:offset+width]
        qualified.append(dict(selector=event['selector'],candidate=event['candidate'],size=event['size'],
            result=call['result'],accepted=event['accepted'],scalar_loads=len(event['accesses'])))
assert next(events,None) is None and len(qualified)==15
serial=(records/'serial.log').read_bytes()
assert serial.count(b''.join(payloads))==1
program=(records/'inspect.gdb').read_text()
for filename in ['user_extent_warning_observer.py','user_extent_shell_observer.py']:
    assert runpy.run_path(str(Path('linux-kernel/labs/qemu/x86_64')/filename))['PROGRAM'] in program
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(raw),serial_sha256=digest(serial),events=qualified,
    admissions=15,refusals=12,completions=3,copied_bytes=45,scalar_loads=7,
    source_syscall_and_selector_order_equal=True,saved_observer_programs_present=True,
    scope='Independent saved native warning records joined to source syscall candidates/counts/results, preserved phase and refused queue state, empty refused-access records and complete staged payloads. Protected-state and helper/mapping checks retain their live observer evidence; not another execution, continuous lifetime, independent owners or complete warning/production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,admissions=15,refusals=12,completions=3,bytes=45,scalar_loads=7)))
