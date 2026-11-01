"""GDB-side independent ownership/zeroing checks at actual user returns."""
import gdb
import hashlib
import json
from pathlib import Path
import traceback

BASE, SIZE, OBJECTS, PAGES, BUFFER = 0x43000, 0x9000, 256, 1024, 4096


def install(config):
    records = []
    previous = None
    corrupted = False
    terminated_live_child = False
    positional_corrupted = False
    position_watch = None
    position_snapshot = None
    position_checks = 0
    table = int(gdb.parse_and_eval("&'topal_x86_static_process_descriptor_table'"))

    def memory(address, size):
        return bytes(gdb.selected_inferior().read_memory(address, size))

    def word(data, offset):
        return int.from_bytes(data[offset:offset + 8], 'little')

    def inspect_store():
        store = memory(table + BASE, SIZE)
        assert word(store, 0) >= 7
        assert not any(store[24:OBJECTS]) and not any(store[OBJECTS + 512:PAGES])
        assert word(store,8)==word(store,16)
        assert not any(store[PAGES + 256:1536]) and not any(store[1536+1024:3072])
        assert not any(store[3072:BUFFER])
        objects,links,names = {},{},{}
        descriptions={}
        for bank in (320,0x3d000):
            for offset in range(0,1024,64):
                data=memory(table+bank+offset,64)
                if word(data,0)!=7:continue
                identity=word(data,32)
                assert 1<=word(data,8)<=32 and word(data,24)<1<<63
                descriptions[identity]=descriptions.get(identity,0)+1
        for index in range(4):
            value=store[OBJECTS+index*128:OBJECTS+(index+1)*128]
            name=store[1536+index*256:1536+(index+1)*256]
            if not word(value,0):
                assert not any(value) and not any(name)
                continue
            assert word(value,0)==1
            identity,size=word(value,8),word(value,16)
            assert 7<=identity<word(store,0) and identity not in objects
            assert size<(1<<63) and word(value,24)<=0o7777
            assert all(word(value,at)>0 for at in (32,40,48))
            linked,length=word(value,56),word(value,72)
            assert linked in (0,1) and word(value,64) in (0,1) and not any(value[80:])
            if linked:
                assert 1<=length<=255 and word(value,64)==0
                actual=name[:length]
                assert 0 not in actual and 47 not in actual and not any(name[length:])
                assert actual not in names
                names[actual]=identity
            else:
                assert length==0 and not any(name) and descriptions.get(identity,0)>0
            objects[identity]=size;links[identity]=linked
        assert set(descriptions)<=set(objects)
        associations = set()
        for index in range(8):
            value = store[PAGES + index * 32:PAGES + (index + 1) * 32]
            payload = store[BUFFER + index * 4096:BUFFER + (index + 1) * 4096]
            owner, page = word(value, 0), word(value, 8)
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

    def description_positions():
        positions={}
        for bank in (320,0x3d000):
            for offset in range(0,1024,64):
                data=memory(table+bank+offset,64)
                if word(data,0) in (3,7) and word(data,8)>0:
                    positions[table+bank+offset+24]=word(data,24)
        return positions

    class PositionWritten(Checked):
        def inspect(self):
            raise AssertionError("positional transfer changed a shared description position during native service")

    class BeforePositional(Checked):
        def __init__(self,index):
            super().__init__("*"+hex(config["calls"][index]["returned"]-2),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        def inspect(self):
            nonlocal position_watch,position_snapshot
            assert position_watch is None and position_snapshot is None
            position_snapshot=description_positions()
            fd=int(gdb.parse_and_eval("$rdi"))&0xffff_ffff
            if fd<16:
                slot=word(memory(table+64+fd*16,16),0)
                if slot:
                    assert 1<=slot<=32
                    cell=table+(320+(slot-1)*64 if slot<=16 else 0x3d000+(slot-17)*64)
                    if cell+24 in position_snapshot:
                        position_watch=PositionWritten("*(unsigned long long*)"+hex(cell+24),type=gdb.BP_WATCHPOINT,wp_class=gdb.WP_WRITE,internal=True)
            self.enabled=False
            return False

    def arm(index):
        Returned(index)
        if config["calls"][index]["number"] in (17,18):BeforePositional(index)

    class Returned(Checked):
        def __init__(self, index):
            super().__init__('*' + hex(config['calls'][index]['returned']),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def inspect(self):
            nonlocal previous, corrupted, terminated_live_child, positional_corrupted, position_watch, position_snapshot, position_checks
            case = config['calls'][self.index]
            actual = int(gdb.parse_and_eval('$rax')) & ((1 << 64) - 1)
            assert actual == case['expected'] & ((1 << 64) - 1), (self.index, case, actual)
            assert int(gdb.parse_and_eval('$r12')) == 0x51ab
            positional=case["number"] in (17,18)
            if positional:
                assert position_snapshot is not None
                if position_watch is not None:
                    position_watch.enabled=False
                    position_watch.delete()
                    position_watch=None
                assert description_positions()==position_snapshot, "positional return changed a live description position"
                position_snapshot=None
                position_checks+=1
            store, objects, associations = inspect_store()
            if case['expected'] < 0 and previous is not None:
                assert store == previous, ('failure changed filesystem state', self.index, case)
            records.append(dict(index=self.index, number=case['number'], result=case['expected'],
                                objects=objects, resident_pages=len(associations), positional=positional,
                                all_live_description_positions_preserved=True if positional else None,
                                store_sha256=hashlib.sha256(store).hexdigest()))
            previous = store
            Path(config['result'] + '.progress').write_text(json.dumps(dict(completed=len(records), last=records[-1])))
            self.enabled = False
            if config.get('corrupt') and not corrupted and case.get('tag') == 'positional-content-write':
                page = next(index for index in range(8) if word(store, PAGES + index * 32) in objects)
                address = table + BASE + BUFFER + page * 4096
                gdb.selected_inferior().write_memory(address, bytes([memory(address, 1)[0] ^ 128]))
                corrupted = True
                positional_corrupted = True
            if config.get('terminate_inactive') and case.get('tag') == 'positional-fork':
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
                arm(self.index + 1)
            else:
                if config.get('retained_for_exit'):
                    assert len(objects) == len(associations) == 2
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
                result='pass-controlled-inactive-positional-terminal' if terminated_live_child else 'pass', calls=records,
                controlled_child_yield_and_parent_exit=terminated_live_child,
                complete_store_erased=True, complete_process_backing_erased=True,
                positional_invocations_with_unchanged_descriptions=position_checks), indent=2, sort_keys=True))
            self.enabled = False
            return True

    class Rejected(Checked):
        def inspect(self):
            actual = int(gdb.parse_and_eval('$rax')) & ((1 << 64) - 1)
            expected = int(gdb.parse_and_eval('$r11')) & ((1 << 64) - 1)
            assert corrupted and positional_corrupted and actual != expected and int(gdb.parse_and_eval('$eflags')) & 64 == 0
            assert memory(int(gdb.parse_and_eval('$rip')), 2) == bytes.fromhex('75fe')
            Path(config['result']).write_text(json.dumps(dict(result='expected-wrong-positional-data-rejection',
                calls=records, actual=actual, expected=expected, controlled_payload_corruption=True, positional_content_write_corrupted=True,
                stopped_at_independent_user_comparison=True), indent=2, sort_keys=True))
            self.enabled = False
            return True

    arm(0)
    Terminal('topal_x86_systems_static_image_process_restore_entry',
             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
    if config.get('corrupt'):
        Rejected('*' + hex(config['rejection_pc']), type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
