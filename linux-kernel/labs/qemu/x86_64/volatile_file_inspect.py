"""GDB-side independent ownership/zeroing checks at actual user returns."""
import gdb
import hashlib
import json
from pathlib import Path
import traceback

BASE, SIZE, OBJECTS, PAGES, BUFFER = 0x43000, 0x9000, 256, 1024, 4096


def install(config):
    page_map_offset = config.get('page_map_offset')
    assert page_map_offset in (None, 24)
    records = []
    previous = None
    corrupted = False
    terminated_live_child = False
    table = int(gdb.parse_and_eval("&'topal_x86_static_process_descriptor_table'"))

    def memory(address, size):
        return bytes(gdb.selected_inferior().read_memory(address, size))

    def word(data, offset):
        return int.from_bytes(data[offset:offset + 8], 'little')

    def inspect_store():
        store = memory(table + BASE, SIZE)
        assert word(store, 0) >= 7
        assert not any(store[8:24])
        assert not any(store[32 if page_map_offset is not None else 24:OBJECTS])
        assert not any(store[OBJECTS + 512:PAGES])
        assert not any(store[PAGES + 256:BUFFER])
        objects = {}
        for index in range(4):
            value = store[OBJECTS + index * 128:OBJECTS + (index + 1) * 128]
            if not word(value, 0):
                assert not any(value)
                continue
            assert word(value, 0) == 1
            identity, size = word(value, 8), word(value, 16)
            assert 7 <= identity < word(store, 0) and identity not in objects
            assert size <= (1 << 63) - 1 and word(value, 24) <= 0o7777
            assert word(value, 32) > 0 and word(value, 40) > 0 and word(value, 48) > 0
            assert not any(value[56:64]) and word(value, 64) in (0, 1) and not any(value[72:])
            objects[identity] = size
        associations = set()
        for index in range(8):
            value = store[PAGES + index * 32:PAGES + (index + 1) * 32]
            payload = store[BUFFER + index * 4096:BUFFER + (index + 1) * 4096]
            owner, page = word(value, 0), word(value, 8)
            if page_map_offset is not None:
                assert store[page_map_offset + index] == int(owner != 0), 'allocation/association mismatch'
            if owner == 0:
                assert not any(value) and not any(payload)
                continue
            assert owner in objects and not any(value[16:])
            assert (owner, page) not in associations
            associations.add((owner, page))
            remaining = objects[owner] - page * 4096
            assert remaining > 0
            if remaining < 4096:
                assert not any(payload[remaining:])
        assert not any(memory(table + 1408, 256)), 'retained transfer staging'
        assert not any(memory(table + 4096, 4096)), 'retained copied name'
        return store, objects, associations

    class Checked(gdb.Breakpoint):
        def stop(self):
            try:
                return self.inspect()
            except BaseException:
                Path(config['failed']).write_text(traceback.format_exc())
                return True

    class Returned(Checked):
        def __init__(self, index):
            super().__init__('*' + hex(config['calls'][index]['returned']),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def inspect(self):
            nonlocal previous, corrupted, terminated_live_child
            case = config['calls'][self.index]
            actual = int(gdb.parse_and_eval('$rax')) & ((1 << 64) - 1)
            assert actual == case['expected'] & ((1 << 64) - 1), (self.index, case, actual)
            assert int(gdb.parse_and_eval('$r12')) == 0x51ab
            store, objects, associations = inspect_store()
            if case['expected'] < 0 and previous is not None:
                assert store == previous, ('failure changed filesystem state', self.index, case)
            records.append(dict(index=self.index, number=case['number'], result=case['expected'],
                                objects=objects, resident_pages=len(associations),
                                store_sha256=hashlib.sha256(store).hexdigest()))
            previous = store
            Path(config['result'] + '.progress').write_text(json.dumps(dict(completed=len(records), last=records[-1])))
            self.enabled = False
            if config.get('corrupt') and not corrupted and case['number'] == 1 and case['expected'] == 8:
                page = next(index for index in range(8) if word(store, PAGES + index * 32) in objects)
                address = table + BASE + BUFFER + page * 4096
                gdb.selected_inferior().write_memory(address, bytes([memory(address, 1)[0] ^ 128]))
                corrupted = True
            if config.get('terminate_inactive') and case['number'] == 57:
                if case['expected'] == 0:
                    # Controlled native request: suspend the live child at yield.
                    parent = next(index for index in range(self.index + 1, len(config['calls']))
                                  if config['calls'][index]['number'] == 57)
                    Returned(parent)
                    gdb.execute('set $rax=24')
                    gdb.execute('set $rip=$rcx-2')
                else:
                    assert word(memory(table, 0x2100), 0x2000) == 1
                    assert word(memory(table, 64), 56) == 1
                    assert len(objects) == len(associations) == 1
                    terminated_live_child = True
                    gdb.execute('set $rax=60')
                    gdb.execute('set $rdi=35')
                    gdb.execute('set $rip=$rcx-2')
                return False
            if self.index + 1 < len(config['calls']):
                Returned(self.index + 1)
            else:
                if config.get('retained_for_exit'):
                    assert len(objects) == len(associations) == 1
                else:
                    assert not objects and not associations
            return False

    class Terminal(Checked):
        def inspect(self):
            if config.get('terminate_inactive'):
                assert terminated_live_child and records[-1]['number'] == 57
            else:
                assert len(records) == len(config['calls'])
            assert not any(memory(table, BASE + SIZE)), 'retained process/filesystem backing'
            Path(config['result']).write_text(json.dumps(dict(
                result='pass-controlled-inactive-file-terminal' if terminated_live_child else 'pass', calls=records,
                controlled_child_yield_and_parent_exit=terminated_live_child,
                complete_store_erased=True, complete_process_backing_erased=True), indent=2, sort_keys=True))
            self.enabled = False
            return True

    class Rejected(Checked):
        def inspect(self):
            actual = int(gdb.parse_and_eval('$rax')) & ((1 << 64) - 1)
            expected = int(gdb.parse_and_eval('$r11')) & ((1 << 64) - 1)
            assert corrupted and actual != expected and int(gdb.parse_and_eval('$eflags')) & 64 == 0
            assert memory(int(gdb.parse_and_eval('$rip')), 2) == bytes.fromhex('75fe')
            Path(config['result']).write_text(json.dumps(dict(result='expected-wrong-file-data-rejection',
                calls=records, actual=actual, expected=expected, controlled_payload_corruption=True,
                stopped_at_independent_user_comparison=True), indent=2, sort_keys=True))
            self.enabled = False
            return True

    Returned(0)
    Terminal('topal_x86_systems_static_image_process_restore_entry',
             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
    if config.get('corrupt'):
        Rejected('*' + hex(config['rejection_pc']), type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
