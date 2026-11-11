"""Check the unchanged default observer cohort against its preceding run."""
import hashlib
import json
from pathlib import Path
import sys

build, baseline, output = map(Path, sys.argv[1:4])
digest = lambda data: hashlib.sha256(data).hexdigest()
native = json.loads((build / 'first-image-exit/observations.json').read_text())
old = json.loads((baseline / 'first-image-exit/observations.json').read_text())
assert native['passed'] and native == old
assert 'source_syscall_refusals' not in native['staged_transfer_observation']
rows = []
for name in ['root.o', 'source.o', 'runtime.o', 'owner.o', 'controller.o',
             'target.o', 'dispatch.o', 'gateway.o', 'boundary.o', 'extent.o',
             'extent-bridge.o', 'extent-callers.o', 'provider.o',
             'predecessor-provider.o', 'predecessor-gateway.o', 'kernel.elf', 'bzImage',
             'static-elf-abi-fixture.elf', 'initramfs.cpio', 'extent-call-inventory.json',
             'process-access-inventory.json', 'process-access-origins.json',
             'process-byte-origins.json', 'fixed-transfer-audit.json',
             'copy-transfer-audit.json', 'staged-transfer-audit.json']:
    data = (build / name).read_bytes()
    assert data == (baseline / name).read_bytes(), name
    rows.append(dict(path=name, bytes=len(data), sha256=digest(data)))
bindings = json.loads((build / 'callback-bindings.json').read_text())
previous = json.loads((baseline / 'callback-bindings.json').read_text())
for record, directory in [(bindings,build),(previous,baseline)]:
    assert record['baseline_provenance_sha256'] == digest((directory / 'baseline/provenance.json').read_bytes())
assert {k:v for k,v in bindings.items() if k != 'baseline_provenance_sha256'} == {
    k:v for k,v in previous.items() if k != 'baseline_provenance_sha256'}
serial = (build / 'first-image-exit/serial.log').read_bytes()
assert serial == (baseline / 'first-image-exit/serial.log').read_bytes()
report = dict(passed=True, exact_observations_equal_prior=True, comparisons=rows,
    observations_sha256=digest((build / 'first-image-exit/observations.json').read_bytes()),
    serial_bytes=len(serial), serial_sha256=digest(serial),
    scope='The default all-observer pinned BusyBox cohort retains exactly the previous native observations, executable artifacts, census/graph/access metadata and output bytes. Callback diagnostic provenance is revalidated separately; not full owner/lifetime or production qualification')
output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k:v for k,v in report.items() if k != 'comparisons'}))
