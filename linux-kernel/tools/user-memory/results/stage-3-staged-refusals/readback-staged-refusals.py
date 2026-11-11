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
# Entropy differs between boots. Permit only the eight measured getrandom
# results, their exact propagated words, and the private 32-byte key field.
entropy = {}
for before, after in zip(old['startup_executions'], native['startup_executions'], strict=True):
    if before['number'] == 318:
        a, b = bytes.fromhex(before['output_after']), bytes.fromhex(after['output_after'])
        assert len(a) == len(b) == 8 and a not in entropy
        entropy[a] = b
assert len(entropy) == 8
entropy_words = {int.from_bytes(a,'little'):int.from_bytes(b,'little') for a,b in entropy.items()}
differences = []
def compare(a, b, path):
    assert type(a) is type(b), path
    if isinstance(a, dict):
        assert set(a) == set(b), path
        for key in a:
            compare(a[key], b[key], path+[key])
    elif isinstance(a, list):
        assert len(a) == len(b), path
        for i, (left, right) in enumerate(zip(a,b,strict=True)):
            compare(left, right, path+[i])
    elif a != b:
        if path[0] == 'startup_executions' and path[-1] == 'output_after':
            assert old['startup_executions'][path[1]]['number'] == 318
            assert entropy[bytes.fromhex(a)] == bytes.fromhex(b)
        elif ((path[0] == 'startup_executions' and path[-2:] == ['args',3]) or
              (path[0] == 'bridge_returns' and path[-2:] == ['registers','r10'])):
            assert a in entropy_words and entropy_words[a] == b, path
        elif path[0] in ['shell_executions','command_executions'] and path[-1] == 'protected_before':
            left, right = bytes.fromhex(a), bytes.fromhex(b)
            assert len(left) == len(right)
            # protected_controls starts at GS+152. The process-owned random
            # key occupies GS+192..224 (completion emitter's extract input).
            # Each run's native EFAULT assertions still compare all 424 bytes
            # without masking; this exemption applies only across two boots.
            assert len(left) == 424 and left[:40] == right[:40] and left[72:] == right[72:], path
        else:
            raise AssertionError(('unexpected observation difference',path))
        differences.append(path)
for key, value in old.items():
    compare(value, native[key], [key])
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
for name in ['static-elf-abi-fixture.elf', 'initramfs.cpio']:
    assert (build / name).read_bytes() == (baseline / name).read_bytes(), name
component = json.loads((build / 'source-component.json').read_text())
previous_component = json.loads((baseline / 'source-component.json').read_text())
assert component['linked_sha256'] == digest((build / 'source-user.elf').read_bytes())
assert previous_component['linked_sha256'] == digest((baseline / 'source-user.elf').read_bytes())
assert set(component)-set(previous_component) == {'pipe_source_sha256','serial_source_sha256','storage_source_sha256'}
assert all(component[k] is None for k in set(component)-set(previous_component))
assert all(component[k] == v for k,v in previous_component.items() if k != 'linked_sha256')
for name in ['extent-call-inventory.json', 'process-access-inventory.json',
             'process-access-origins.json', 'process-byte-origins.json',
             'fixed-transfer-audit.json',
             'copy-transfer-audit.json', 'staged-transfer-audit.json']:
    assert (build / name).read_bytes() == (common / name).read_bytes(), name
bindings = json.loads((build / 'callback-bindings.json').read_text())
previous_bindings = json.loads((common / 'callback-bindings.json').read_text())
for row, directory in [(bindings,build),(previous_bindings,common)]:
    assert row['baseline_provenance_sha256'] == digest((directory / 'baseline/provenance.json').read_bytes())
assert {k:v for k,v in bindings.items() if k != 'baseline_provenance_sha256'} == {
    k:v for k,v in previous_bindings.items() if k != 'baseline_provenance_sha256'}
serial = (build / 'source-user-extents/serial.log').read_bytes()
assert serial == (baseline / 'source-user-extents/serial.log').read_bytes()
report = dict(passed=True, syscall_calls=len(calls),
    syscall_results=dict(Counter(str(c['result']) for c in calls)),
    existing_observation_sections_compared=list(old),
    entropy_word_pairs=[dict(before=a.hex(),after=b.hex()) for a,b in entropy.items()],
    entropy_differences=len(differences), staged_totals=dict(totals),
    diagnostic_metadata_differences=['source-component linked ELF debug identity and three unused null mode fields',
        'callback baseline provenance identity; remaining callback report equal'],
    staged_counts=staged['counts'], kernel_comparisons=comparisons,
    serial_bytes=len(serial), serial_sha256=digest(serial),
    callback_observation_in_this_run='callback_observation' in native,
    helper_sites_reached=native['extent_call_coverage']['reached_sites'],
    rep_observation_in_this_run='copy_transfer_observation' in native,
    scope='Existing source-fixture observations agree outside the measured entropy words, their exact propagated values and private random-key field. Native per-refusal checks retain that entire field. Adds paired staged transfers and source-syscall/refusal association with unchanged executable artifacts; not complete ownership, continuous lifetime, every path or production qualification.')
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons', 'staged_counts', 'syscall_results']}))
