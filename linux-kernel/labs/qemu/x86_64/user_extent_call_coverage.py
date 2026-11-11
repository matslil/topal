"""Observe first reach of statically verified owned-extent helper call sites.

This observes existing ordinary application execution; it neither injects guest
state nor infers successful admission or complete load/store qualification.
"""
import importlib.util
import json
from pathlib import Path


def linked_inventory(build):
    path = Path(__file__).resolve().parents[3] / 'tools/user-memory/inspect_call_inventory.py'
    spec = importlib.util.spec_from_file_location('extent_call_inventory', path)
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    inventory_path = build / 'extent-call-inventory.json'
    inventory = json.loads(inventory_path.read_text())
    inspection = json.loads((build / 'extent-call-inspection.json').read_text())
    assert inspection['passed'] and inspection['inventory_sha256'] == reader.digest(inventory_path.read_bytes())
    assert inspection['provider_sha256'] == reader.digest((build / 'provider.o').read_bytes())
    elf = reader.Elf(build / 'kernel.elf')
    assert elf.kind == 2
    symbol = reader.verify_calls(elf, inventory)
    return dict(function=inventory['function'], address=symbol['value'], calls=inventory['calls'],
                kernel_sha256=reader.digest(elf.data), inventory_sha256=inspection['inventory_sha256'])


PROGRAM = r'''
extent_first_reach = {}

class ExtentFirstReach(Checked):
    def __init__(self, row):
        super().__init__('*'+hex(extent_inventory['address']+row['offset']), internal=True)
        self.row = row

    def observe(self):
        self.enabled = False
        assert reg('cs') & 3 == 0 and reg('cr0') & (1<<16)
        offset = self.row['offset']
        assert offset not in extent_first_reach
        extent_first_reach[offset] = dict(offset=offset, target=self.row['target'],
            candidate=reg('r8'), length=reg('r9'), phase=memory(context+104,1)[0])
        return False

def extent_coverage_complete():
    if extent_inventory is None:
        return
    calls = extent_inventory['calls']
    assert extent_first_reach and set(extent_first_reach) <= {row['offset'] for row in calls}
    observed['extent_call_coverage'] = dict(
        observation_passed=True, total_sites=len(calls), reached_sites=len(extent_first_reach),
        reached=[extent_first_reach[offset] for offset in sorted(extent_first_reach)],
        unreached=[row for row in calls if row['offset'] not in extent_first_reach],
        kernel_sha256=extent_inventory['kernel_sha256'],
        inventory_sha256=extent_inventory['inventory_sha256'],
        scope='First reach before helper invocation in this selected execution only; no successful-admission, return, every-length/error-path, direct-access/dominance or production qualification claim')

def install_extent_coverage():
    if extent_inventory is not None:
        assert address(extent_inventory['function']) == extent_inventory['address']
        for row in extent_inventory['calls']:
            ExtentFirstReach(row)
'''
