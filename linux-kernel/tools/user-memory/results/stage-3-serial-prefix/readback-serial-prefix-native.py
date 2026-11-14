"""Reconcile saved UART/syscall observations and aggregate prefix-copy counts."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
raw = (build/'source-user-extents/observations.json').read_bytes()
obs = json.loads(raw)
audit_bytes = (build/'serial-prefix-audit.json').read_bytes()
audit = json.loads(audit_bytes)
copies = obs['copy_transfer_observation']
assert obs['passed'] and audit['passed'] and copies['passed']
assert copies['kernel_sha256'] == audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert copies['serial_prefix_audit_sha256'] == digest(audit_bytes)
counts = dict(attempts=0,admitted=0,refused=0,completed=0,bytes=0)
calls,short,early = [],[],[]
assert len(obs['executions']) == 4
for execution in obs['executions']:
    received = bytearray()
    for call in execution['calls']:
        if call['number'] != 0: continue
        args,result = call['args'],call['result']
        assert call['native_input_verified'] and call['complete_table_verified'] and call['copy_window_verified']
        assert call['phase_before'] == call['phase_after']
        payload = bytes(call['uart_bytes'])
        if 'extent_call' not in call:
            assert result in [-14,-9,0] and not payload
            early.append(dict(args=args,result=result))
            continue
        admission = call['extent_call']
        assert args[0] == 0 and admission == dict(candidate=args[1],length=min(args[2],256))
        assert 0 < admission['length'] <= 256
        counts['attempts'] += 1
        if result < 0:
            assert result == -14 and not payload
            counts['refused'] += 1
        else:
            assert 0 < result == len(payload) <= admission['length']
            assert payload.hex() == call['output_hex']
            start = len(received)
            assert payload == b'Uabcdefg12345678'[start:start+result]
            received.extend(payload)
            counts['admitted'] += 1
            counts['completed'] += 1
            counts['bytes'] += result
            if result < admission['length']: short.append(dict(args=args,admission=admission,result=result))
        calls.append(dict(args=args,admission=admission,result=result,uart_payload_hex=payload.hex()))
    assert bytes(received) == b'Uabcdefg12345678'
assert counts == copies['counts']['serial-prefix-output']
assert counts == dict(attempts=28,admitted=12,refused=16,completed=12,bytes=64)
assert len(short) == 4 and all(row['args'][2] == 512 and row['admission']['length'] == 256 and row['result'] == 8 for row in short)
sample = next(s for s in copies['samples'] if s['name'] == 'serial-prefix-output')
assert sample['size'] == sample['elements'] == sample['admitted_size'] == 1
assert sample['copy_count_kind'] == 'positive-prefix'
program = (build/'source-user-extents/inspect.gdb').read_text()
for path in ['user_extent_copy_observer.py','user_extent_serial_observer.py']:
    selected = runpy.run_path(str(Path('linux-kernel/labs/qemu/x86_64')/path))['PROGRAM']
    assert selected in program
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(raw),source_syscalls=sum(len(e['calls']) for e in obs['executions']),
    serial_admission_records=len(calls),counts=counts,short_prefixes=short,
    early_results_without_output_admission=early,saved_observer_programs_present=True,
    scope='Independent saved syscall/UART payload/admission/return reconstruction and aggregate REP-count correspondence. Runtime boundary translation/payload checks remain live-observer evidence; not a complete per-copy saved event transcript, second native run, actual owners/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=report['source_syscalls'],admissions=len(calls),copies=12,refusals=16,short_prefixes=len(short))))
