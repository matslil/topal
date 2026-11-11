"""Read back ordinary refusal execution and unchanged artifact identities."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys

build, baseline, common, output = map(Path, sys.argv[1:5])
digest = lambda data: hashlib.sha256(data).hexdigest()
native_path = build / 'source-user-extents/observations.json'
native = json.loads(native_path.read_text())
old = json.loads((baseline / 'source-user-extents/observations.json').read_text())
assert native['passed'] and old['passed']
# Compare every existing assertion's retained inputs/results, rather than only
# counting successes. New observation sections are additional evidence.
assert all(native[key] == value for key, value in old.items())
keys = ['executions', 'startup_executions', 'file_executions',
        'shell_executions', 'command_executions', 'poll_executions']
assert [len(native[key]) for key in keys] == [4, 20, 20, 38, 5, 0]
calls = [call for key in keys for row in native[key] for call in row['calls']]
assert len(calls) == 796 and sum(c['result'] == -14 for c in calls) == 393
staged = native['staged_transfer_observation']
audit = json.loads((build / 'staged-transfer-audit.json').read_text())
assert staged['passed'] and staged['kernel_sha256'] == digest((build / 'kernel.elf').read_bytes())
assert staged['audit_sha256'] == digest((build / 'staged-transfer-audit.json').read_bytes())
assert audit['passed'] and audit['transfer_regions'] == 16
assert set(staged['counts']) == {row['name'] for row in audit['cases']}
totals = Counter()
for name, counts in staged['counts'].items():
    assert counts['attempts'] == counts['admitted'] + counts['refused']
    assert counts['admitted'] == counts['completed'] and counts['completed'] > 0
    assert counts['refused'] > 0, name
    totals.update(counts)
assert not staged['unreached']
events = staged['source_syscall_refusals']
assert staged['source_syscall_scoped'] and len(events) == totals['refused']
assert Counter(e['name'] for e in events) == Counter({n:c['refused'] for n,c in staged['counts'].items()})
for event in events:
    index = 0 if event['number'] == 63 else (2 if event['number'] == 262 else 1)
    assert event['candidate'] == event['args'][index]
    assert event['result'] == -14 and event['phase_before'] == event['phase_after']
    assert event['counterpart_unchanged'] and event['helper_registers_preserved'] and event['captured_arguments_preserved']

comparisons = []
for name in ['root.o', 'source.o', 'runtime.o', 'owner.o', 'controller.o',
             'target.o', 'dispatch.o', 'gateway.o', 'boundary.o', 'extent.o',
             'extent-bridge.o', 'extent-callers.o', 'provider.o',
             'predecessor-provider.o', 'predecessor-gateway.o', 'kernel.elf', 'bzImage']:
    data = (build / name).read_bytes()
    assert data == (baseline / name).read_bytes() == (common / name).read_bytes(), name
    comparisons.append(dict(path=name, bytes=len(data), sha256=digest(data)))
for name in ['static-elf-abi-fixture.elf', 'initramfs.cpio', 'source-component.json']:
    assert (build / name).read_bytes() == (baseline / name).read_bytes(), name
for name in ['extent-call-inventory.json', 'process-access-inventory.json',
             'process-access-origins.json', 'process-byte-origins.json',
             'callback-bindings.json', 'fixed-transfer-audit.json',
             'copy-transfer-audit.json', 'staged-transfer-audit.json']:
    assert (build / name).read_bytes() == (common / name).read_bytes(), name
serial = (build / 'source-user-extents/serial.log').read_bytes()
assert serial == (baseline / 'source-user-extents/serial.log').read_bytes()
report = dict(passed=True, syscall_calls=len(calls),
    syscall_results=dict(Counter(str(c['result']) for c in calls)),
    existing_observation_sections_equal=list(old), staged_totals=dict(totals),
    staged_counts=staged['counts'], kernel_comparisons=comparisons,
    serial_bytes=len(serial), serial_sha256=digest(serial),
    callback_observation_in_this_run='callback_observation' in native,
    helper_sites_reached=native['extent_call_coverage']['reached_sites'],
    rep_observation_in_this_run='copy_transfer_observation' in native,
    scope='Exact pre-existing source-fixture observations plus paired staged admission/transfer counts, source-adapter syscall/refusal association and unchanged artifacts. Not complete ownership, continuous lifetime, every path or production qualification.')
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons', 'staged_counts', 'syscall_results']}))
