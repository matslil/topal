"""Observe first reach of statically verified owned-extent helper call sites.

This observes existing ordinary application execution; it neither injects guest
state nor infers successful admission or complete load/store qualification.
"""
import importlib.util
import json
from pathlib import Path

from user_extent_callback_observer import PROGRAM as CALLBACK_PROGRAM
from user_extent_copy_observer import PROGRAM as COPY_PROGRAM
from user_extent_staged_observer import PROGRAM as STAGED_PROGRAM


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
    bindings = json.loads((build / 'callback-bindings.json').read_text())
    assert bindings['passed'] and bindings['kernel_sha256'] == reader.digest(elf.data)
    assert bindings['process_graph_sha256'] == reader.digest((build / 'process-access-inventory.json').read_bytes())
    native_bindings = dict(bindings)
    native_bindings['callbacks'] = [
        {key: row[key] for key in ['gs_slot', 'argument', 'entry_stack_offset', 'native_export', 'target', 'sections']}
        for row in bindings['callbacks']
    ]
    copy_bytes = (build / 'copy-transfer-audit.json').read_bytes()
    copies = json.loads(copy_bytes)
    assert copies['passed'] and copies['kernel_sha256'] == reader.digest(elf.data)
    assert copies['graph_sha256'] == bindings['process_graph_sha256']
    staged_bytes = (build / 'staged-transfer-audit.json').read_bytes()
    staged = json.loads(staged_bytes)
    assert staged['passed'] and staged['kernel_sha256'] == reader.digest(elf.data)
    assert staged['graph_sha256'] == bindings['process_graph_sha256']
    staging_path = build / 'staging-copy-audit.json'
    staging_bytes = staging_path.read_bytes() if staging_path.exists() else None
    staging = json.loads(staging_bytes) if staging_bytes is not None else None
    if staging is not None:
        assert staging['passed'] and staging['kernel_sha256'] == reader.digest(elf.data)
        assert staging['graph_sha256'] == bindings['process_graph_sha256']
    return dict(function=inventory['function'], address=symbol['value'], calls=inventory['calls'],
                callback_bindings=native_bindings,
                copy_transfer_audit=copies, copy_transfer_audit_sha256=reader.digest(copy_bytes),
                staged_transfer_audit=staged, staged_transfer_audit_sha256=reader.digest(staged_bytes),
                staging_copy_audit=staging,
                staging_copy_audit_sha256=reader.digest(staging_bytes) if staging_bytes is not None else None,
                callback_header_hex=(build / 'static-elf-abi-fixture.elf').read_bytes()[:64].hex(),
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
    for name, complete in [('callbacks',callback_coverage_complete),
                           ('copies',copy_observation_complete),
                           ('staged',staged_observation_complete)]:
        if name in selected_extent_observers():
            complete()
    calls = extent_inventory['calls']
    assert extent_first_reach and set(extent_first_reach) <= {row['offset'] for row in calls}
    observed['extent_call_coverage'] = dict(
        observation_passed=True, total_sites=len(calls), reached_sites=len(extent_first_reach),
        reached=[extent_first_reach[offset] for offset in sorted(extent_first_reach)],
        unreached=[row for row in calls if row['offset'] not in extent_first_reach],
        kernel_sha256=extent_inventory['kernel_sha256'],
        inventory_sha256=extent_inventory['inventory_sha256'],
        scope='First reach before helper invocation in this selected execution only; no successful-admission, return, every-length/error-path, direct-access/dominance or production qualification claim')

def selected_extent_observers():
    groups = extent_inventory.get('native_observers',['callbacks','copies','staged'])
    assert groups and len(groups) == len(set(groups))
    assert set(groups) <= {'callbacks','copies','staged'}
    return groups

def install_extent_coverage():
    if extent_inventory is not None:
        for name, install in [('callbacks',install_callback_observation),
                              ('copies',install_copy_observation),
                              ('staged',install_staged_observation)]:
            if name in selected_extent_observers():
                install()
        assert address(extent_inventory['function']) == extent_inventory['address']
        if not extent_inventory.get('source_scoped_staged'):
            for row in extent_inventory['calls']:
                ExtentFirstReach(row)
'''

PROGRAM += CALLBACK_PROGRAM
PROGRAM += COPY_PROGRAM
PROGRAM += STAGED_PROGRAM
