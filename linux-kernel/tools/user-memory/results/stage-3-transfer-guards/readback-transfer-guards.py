"""Read-only native evidence review plus offline first-party artifact controls."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import shutil
import sys

build, serial_build, external_build, output = map(Path, sys.argv[1:5])
output.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf, digest
from audit_transfer_guards import audit

inventory = json.loads((build/'extent-call-inventory.json').read_text())
audit_record = json.loads((build/'transfer-guard-audit.json').read_text())
with contextlib.redirect_stdout(io.StringIO()):
    audit(build)
assert audit_record == json.loads((build/'transfer-guard-audit.json').read_text())
assert audit_record['passed'] and audit_record['call_count'] == len(inventory['calls']) == 91
assert len({r['refusal_offset'] for r in audit_record['carry_gates']}) == 14
assert [(r['name'], r['admitted_bytes']) for r in audit_record['fallback_local_reads']] == [
    ('named-condition', 14), ('exec-signal-reset', 32), ('missing-path', 9)]
assert sum(len(r['access_instructions']) for r in audit_record['fallback_local_reads']) == 7

comparisons = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o',
             'dispatch.o','gateway.o','boundary.o','extent.o','extent-bridge.o',
             'extent-callers.o','provider.o','predecessor-provider.o',
             'predecessor-gateway.o','kernel.elf','bzImage']:
    data = (build/name).read_bytes()
    assert data == (serial_build/name).read_bytes(), name
    assert data == (external_build/name).read_bytes(), name
    comparisons.append(dict(path=name, bytes=len(data), sha256=digest(data),
                            unchanged_from_serial_and_external=True))
image = (build/'static-elf-abi-fixture.elf').read_bytes()
assert digest(image) == '7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
assert image == (external_build/'static-elf-abi-fixture.elf').read_bytes()
serial = (build/'first-image-exit/serial.log').read_bytes()
assert serial == (external_build/'first-image-exit/serial.log').read_bytes()
observed = json.loads((build/'first-image-exit/observations.json').read_text())
assert observed['passed'] and observed['serial_delivered'] and observed['exit_selected']
coverage = observed['extent_call_coverage']
assert coverage['observation_passed'] and coverage['reached_sites'] == 38
assert coverage['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert coverage['inventory_sha256'] == digest((build/'extent-call-inventory.json').read_bytes())
current = {r['offset'] for r in coverage['reached']}
previous = set()
for directory in [
    'user-extent-source-poll-2268178-1794397912990685732',
    'user-extent-source-pipes-2271066-1794399038949272774',
    'user-extent-source-storage-2272711-1794399750942813740',
    serial_build.name,
]:
    directory = build.parent/directory
    assert inventory == json.loads((directory/'extent-call-inventory.json').read_text())
    data = json.loads((directory/'source-user-extents/observations.json').read_text())
    assert data['passed'] and data['extent_call_coverage']['observation_passed']
    assert data['extent_call_coverage']['kernel_sha256'] == coverage['kernel_sha256']
    previous.update(r['offset'] for r in data['extent_call_coverage']['reached'])
assert len(previous) == 88 and current <= previous
assert {r['offset'] for r in inventory['calls']} - previous == {88236, 94708, 101802}

# These controls alter copies of compiler-owned object data solely for the
# offline reader. No copy is loaded into a VM or changes a third-party image.
# Rebind the body hash so the reader must reject each semantic shape change,
# rather than stopping at the uninteresting content-digest mismatch.
elf = Elf(build/'provider.o')
symbol, body = elf.function(inventory['function'])
section = elf.sections[symbol['section']]
file_start = section[4] + symbol['value'] - section[3]
local = {r['name']:r for r in audit_record['fallback_local_reads']}
named = local['named-condition']
action = local['exec-signal-reset']
path = local['missing-path']
controls = [
    ('carry-condition', inventory['calls'][0]['offset']+15, bytes([0x83])),
    ('admitted-length', named['call_offset']-8, bytes([13])),
    ('loop-bound', named['local_end']-7, bytes([15])),
    ('action-read-offset', action['access_instructions'][-1]['offset']+3, bytes([32])),
    ('pathname-terminator-offset', path['access_instructions'][-1]['offset']+2, bytes([9])),
]
rejections = []
for name, offset, replacement in controls:
    work = output/'offline-controls'/name
    work.mkdir(parents=True, exist_ok=True)
    changed = bytearray(elf.data)
    assert changed[file_start+offset:file_start+offset+len(replacement)] != replacement
    changed[file_start+offset:file_start+offset+len(replacement)] = replacement
    (work/'provider.o').write_bytes(changed)
    meta = copy.deepcopy(inventory)
    meta['function_sha256'] = digest(changed[file_start:file_start+len(body)])
    (work/'extent-call-inventory.json').write_text(json.dumps(meta,indent=2)+'\n')
    shutil.copyfile(build/'extent-call-disassembly.stdout.log',work/'extent-call-disassembly.stdout.log')
    try:
        audit(work)
    except AssertionError as error:
        message = str(error)
    else:
        raise AssertionError(('incorrect artifact accepted', name))
    record = dict(name=name, body_offset=offset, replacement_hex=replacement.hex(),
        rejected=True, error=message,
        scope='Offline compiler-owned object byte shape with rebound function digest; unchanged decoder listing; no native execution')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    rejections.append(record)

report = dict(passed=True, carry_gates=91, refusal_targets=14, local_fallbacks=3,
    static_user_read_instructions=7, decoded_instruction_rows=audit_record['decoded_instruction_rows'],
    linear_decode_unknown_rows=len(audit_record['linear_decode_unknown_offsets']),
    decoded_indirect_transfers=len(audit_record['decoded_indirect_transfers']),
    kernel_comparisons=comparisons, pinned_image_bytes=len(image), pinned_image_sha256=digest(image),
    unchanged_pinned_image=True, unchanged_external_serial=True,
    serial_bytes=len(serial), serial_sha256=digest(serial), current_reached=len(current),
    previous_union=len(previous), union_reached=len(previous|current),
    union_unreached=[r for r in inventory['calls'] if r['offset'] not in previous|current],
    offline_rejections=rejections,
    scope='Selected exact result-gate/local-read artifact checks and unchanged BusyBox exit; not complete user-access dominance, native fallback reach, ownership/lifetime/mode or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','union_unreached','offline_rejections']}))
