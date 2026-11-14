"""Independently summarize call-site reachability and unchanged executable bytes."""
import collections
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import sys

build, baseline, output = map(Path, sys.argv[1:4])
output.mkdir(parents=True, exist_ok=True)
path = Path('linux-kernel/tools/user-memory/inspect_call_inventory.py')
spec = importlib.util.spec_from_file_location('reader', path)
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
inventory = json.loads((build/'extent-call-inventory.json').read_text())
inspection = json.loads((build/'extent-call-inspection.json').read_text())
observed = json.loads((build/'source-user-extents/observations.json').read_text())
assert observed['passed'] and inspection['passed']
coverage = observed['extent_call_coverage']
assert coverage['observation_passed']
assert coverage['kernel_sha256'] == reader.digest((build/'kernel.elf').read_bytes())
assert coverage['inventory_sha256'] == reader.digest((build/'extent-call-inventory.json').read_bytes())
assert inspection['call_count'] == coverage['total_sites'] == len(inventory['calls']) == 91
calls = {r['offset']: r for r in inventory['calls']}
reached = {r['offset']: r for r in coverage['reached']}
assert len(reached) == coverage['reached_sites']
assert set(reached) | {r['offset'] for r in coverage['unreached']} == set(calls)
assert not set(reached) & {r['offset'] for r in coverage['unreached']}
assert all(row == calls[row['offset']] for row in coverage['unreached'])
assert all(row['target'] == calls[offset]['target'] for offset, row in reached.items())
groups = {}
for offset, row in calls.items():
    key = row['source_file']
    groups.setdefault(key, dict(total=0, reached=0))
    groups[key]['total'] += 1
    groups[key]['reached'] += offset in reached
comparisons = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage',
             'static-elf-abi-fixture.elf','initramfs.cpio']:
    data = (build/name).read_bytes()
    assert data == (baseline/name).read_bytes(), name
    comparisons.append(dict(path=name, bytes=len(data), sha256=reader.digest(data), unchanged=True))
serial = (build/'source-user-extents/serial.log').read_bytes()
assert serial == (baseline/'source-user-extents/serial.log').read_bytes()
count = sum(len(r['calls']) for r in observed['executions']+observed['poll_executions'])
assert count == 120
assert len(observed['executions']) == len(observed['poll_executions']) == 4
component = json.loads((build/'source-component.json').read_text())
previous_component = json.loads((baseline/'source-component.json').read_text())
metadata_differences = {k: [previous_component.get(k), component.get(k)]
                        for k in component.keys() | previous_component.keys()
                        if component.get(k) != previous_component.get(k)}
assert set(metadata_differences) == {'linked_sha256'}, metadata_differences
def named_sections(elf):
    names_index = int.from_bytes(elf.data[62:64], 'little')
    strings = elf.section_bytes(names_index)
    return {strings[s[0]:strings.index(0,s[0])].decode(): (s, elf.section_bytes(i))
            for i,s in enumerate(elf.sections) if s[1] != 8}
current_linked = reader.Elf(build/'source-user.elf')
previous_linked = reader.Elf(baseline/'source-user.elf')
assert reader.digest(current_linked.data) == component['linked_sha256']
assert reader.digest(previous_linked.data) == previous_component['linked_sha256']
now, before = named_sections(current_linked), named_sections(previous_linked)
assert set(now) == set(before)
linked_section_differences = []
for name in now:
    if now[name] != before[name]:
        linked_section_differences.append(name)
        assert name.startswith('.debug_'), name
    if now[name][0][2] & 2:
        assert now[name] == before[name], name
assert linked_section_differences
# Metadata-only negative inputs are rejected. No ELF or application bytes are
# modified, and these checks do not execute a guest.
negative = output/'negative-metadata'
negative.mkdir(exist_ok=True)
for name in ['provider.o','extent-call-disassembly.stdout.log']:
    target = negative/name
    if not target.exists():
        target.symlink_to((build/name).resolve())
results = []
for name in ['omitted-call', 'wrong-direction', 'wrong-function-hash']:
    case = copy.deepcopy(inventory)
    if name == 'omitted-call':
        case['calls'].pop(0)
    elif name == 'wrong-direction':
        case['calls'][0]['target'] = next(k for k in reader.EXTERNAL if k != case['calls'][0]['target'])
    else:
        case['function_sha256'] = '0'*64
    (negative/'extent-call-inventory.json').write_text(json.dumps(case))
    try:
        reader.inspect(negative)
    except AssertionError as error:
        results.append(dict(case=name, rejected=True, assertion=str(error)))
    else:
        raise AssertionError('invalid metadata accepted: '+name)
report = dict(passed=True, total_calls=len(calls), reached_calls=len(reached),
    unreached_calls=len(calls)-len(reached), calls_by_emitter=groups,
    reached_by_direction=dict(collections.Counter(r['target'] for r in reached.values())),
    source_syscalls=count, executable_comparisons=comparisons,
    source_component_metadata_differences=metadata_differences,
    source_linked_section_differences=linked_section_differences,
    source_linked_allocated_sections_unchanged=True,
    serial_bytes=len(serial), serial_sha256=reader.digest(serial),
    rejected_metadata=results,
    scope='Complete selected two-helper call census and first reach only; no complete direct-access inventory, dominance, admission result, all-error-path or production qualification')
(output/'readback.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['executable_comparisons']}))
