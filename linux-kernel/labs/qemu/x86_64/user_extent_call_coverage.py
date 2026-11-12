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
from user_extent_pipe_fixed_observer import PROGRAM as PIPE_FIXED_PROGRAM
from user_extent_pipe_loop_observer import PROGRAM as PIPE_LOOP_PROGRAM


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
    pipe_path = build / 'pipe-fixed-output-audit.json'
    pipe_bytes = pipe_path.read_bytes() if pipe_path.exists() else None
    pipes = json.loads(pipe_bytes) if pipe_bytes is not None else None
    if pipes is not None:
        assert pipes['passed'] and not pipes['complete']
        assert pipes['kernel_sha256'] == reader.digest(elf.data)
        assert pipes['graph_sha256'] == bindings['process_graph_sha256']
    stat_path = build / 'stat-output-audit.json'
    stat_bytes = stat_path.read_bytes() if stat_path.exists() else None
    stats = json.loads(stat_bytes) if stat_bytes is not None else None
    if stats is not None:
        assert stats['passed'] and not stats['complete']
        assert stats['kernel_sha256'] == reader.digest(elf.data)
        assert stats['graph_sha256'] == bindings['process_graph_sha256']
    serial_path = build / 'serial-prefix-audit.json'
    serial_bytes = serial_path.read_bytes() if serial_path.exists() else None
    serial = json.loads(serial_bytes) if serial_bytes is not None else None
    if serial is not None:
        assert serial['passed'] and not serial['complete']
        assert serial['kernel_sha256'] == reader.digest(elf.data)
        assert serial['graph_sha256'] == bindings['process_graph_sha256']
    loop_path = build / 'pipe-loop-audit.json'
    loop_bytes = loop_path.read_bytes() if loop_path.exists() else None
    loops = json.loads(loop_bytes) if loop_bytes is not None else None
    if loops is not None:
        assert loops['passed'] and not loops['complete']
        assert loops['kernel_sha256'] == reader.digest(elf.data)
        assert loops['graph_sha256'] == bindings['process_graph_sha256']
    file_path = build / 'file-input-audit.json'
    file_bytes = file_path.read_bytes() if file_path.exists() else None
    files = json.loads(file_bytes) if file_bytes is not None else None
    if files is not None:
        assert files['passed'] and not files['complete']
        assert files['kernel_sha256'] == reader.digest(elf.data)
        assert files['graph_sha256'] == bindings['process_graph_sha256']
    return dict(function=inventory['function'], address=symbol['value'], calls=inventory['calls'],
                callback_bindings=native_bindings,
                copy_transfer_audit=copies, copy_transfer_audit_sha256=reader.digest(copy_bytes),
                staged_transfer_audit=staged, staged_transfer_audit_sha256=reader.digest(staged_bytes),
                staging_copy_audit=staging,
                staging_copy_audit_sha256=reader.digest(staging_bytes) if staging_bytes is not None else None,
                pipe_fixed_output_audit=pipes,
                pipe_fixed_output_audit_sha256=reader.digest(pipe_bytes) if pipe_bytes is not None else None,
                stat_output_audit=stats,
                stat_output_audit_sha256=reader.digest(stat_bytes) if stat_bytes is not None else None,
                serial_prefix_audit=serial,
                serial_prefix_audit_sha256=reader.digest(serial_bytes) if serial_bytes is not None else None,
                pipe_loop_audit=loops,
                pipe_loop_audit_sha256=reader.digest(loop_bytes) if loop_bytes is not None else None,
                file_input_audit=files,
                file_input_audit_sha256=reader.digest(file_bytes) if file_bytes is not None else None,
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
        record = dict(offset=offset, target=self.row['target'],
            candidate=reg('r8'), length=reg('r9'), phase=memory(context+104,1)[0])
        if offset in extent_first_reach:
            # A source-scoped admission observer can record this same stopped
            # instruction first. Both read-only observations must agree.
            assert extent_first_reach[offset] == record
        else:
            extent_first_reach[offset] = record
        return False

def extent_coverage_complete():
    if extent_inventory is None:
        return
    for name, complete in [('callbacks',callback_coverage_complete),
                           ('copies',copy_observation_complete),
                           ('staged',staged_observation_complete),
                           ('pipe-fixed',pipe_fixed_observation_complete),
                           ('pipe-loops',pipe_loop_observation_complete)]:
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
    assert set(groups) <= {'callbacks','copies','staged','pipe-fixed','pipe-loops'}
    return groups

def install_extent_coverage():
    if extent_inventory is not None:
        for name, install in [('callbacks',install_callback_observation),
                              ('copies',install_copy_observation),
                              ('staged',install_staged_observation),
                              ('pipe-fixed',install_pipe_fixed_observation),
                              ('pipe-loops',install_pipe_loop_observation)]:
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
PROGRAM += PIPE_FIXED_PROGRAM
PROGRAM += PIPE_LOOP_PROGRAM
