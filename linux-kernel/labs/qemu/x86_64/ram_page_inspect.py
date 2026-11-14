"""Observe the real map, associations, source result and physical reclamation."""
import gdb
import json
import struct
import traceback
from pathlib import Path


def install(config):
    output = Path(config['output'])
    artifact = config['artifact']
    library = artifact['ram_page_pool_library']
    binding = artifact['ram_page_pool_runtime']
    state = dict(calls=[], syscalls=[], generation=0, finalized=False)
    current = None
    pending = None
    refusal = None
    cases = config['cases']
    registers = ('rbx', 'rbp', 'rdi', 'rsi', 'rdx', 'rcx',
                 'r8', 'r9', 'r10', 'r11', 'r12', 'r13', 'r14', 'r15')

    def reg(name): return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)
    def addr(name): return int(gdb.parse_and_eval("&'" + name + "'"))
    def memory(start, size): return bytes(gdb.selected_inferior().read_memory(start, size))
    def words(start, count): return struct.unpack('<' + 'Q' * count, memory(start, count * 8))
    def zero(start, size): assert memory(start, size) == bytes(size), (hex(start), size)
    def failed():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    controller = addr(binding['controller_owner']['symbol'])
    control = addr('topal_x86_systems_ram_page_control')
    arena = addr('topal_x86_systems_ram_page_arena')
    history = addr('topal_x86_systems_ram_page_history')
    owner_map = controller + binding['map_offset']
    associations = controller + binding['association_offset']
    payloads = controller + binding['payload_offset']
    export = addr(library['native_export'])

    def natural(pointer):
        pointer = int(pointer)
        cursor = words(arena+8, 1)[0]
        extents = [(arena+32, arena+cursor)]
        extents += [(v['address'], v['address']+v['bytes']) for v in binding['source_data']]
        assert any(lo <= pointer and pointer+16 <= hi for lo, hi in extents)
        sign, count = words(pointer, 2)
        assert sign == 0 and count <= 2
        assert any(lo <= pointer and pointer+16+count*4 <= hi for lo, hi in extents)
        return int.from_bytes(memory(pointer+16, count*4), 'little') if count else 0

    def check_frame():
        assert words(current['frame'], 2) == (current['action'], current['fact'])
        assert 0x108000 <= current['frame']-4096 < current['frame']+64 <= 0x180000

    class Before(gdb.Breakpoint):
        def __init__(self, index):
            super().__init__('*'+hex(cases[index]['returned']-2),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def stop(self):
            nonlocal pending
            try:
                self.enabled = False
                assert pending is None and current is None
                case = cases[self.index]
                assert reg('rax') == case['number']
                expected = None
                input_prefix = None
                admitted_page = None
                if case['number'] in (0, 1, 17, 18):
                    expected = 0
                    fd = reg('rdi') & 0xffffffff
                    if case['number'] in (1, 18) and case['expected'] > 0 and fd < 16:
                        slot = words(controller+64+fd*16, 1)[0]
                        if slot:
                            assert 1 <= slot <= 32
                            cell = controller+(320+(slot-1)*64 if slot <= 16 else 0x3d000+(slot-17)*64)
                            if words(cell, 1)[0] == 7:
                                identity = words(cell+32, 1)[0]
                                position = reg('r10') if case['number'] == 18 else words(cell+24, 1)[0]
                                if words(cell+16, 1)[0] & 0x400:
                                    objects = [controller+0x43100+i*128 for i in range(4)]
                                    position = next(words(v+16, 1)[0] for v in objects
                                                    if words(v, 2) == (1, identity))
                                resident = any(words(associations+i*32, 2) == (identity, position//4096)
                                               for i in range(8))
                                expected = int(not resident)
                                if expected:
                                    input_prefix = memory(reg('rsi'), case['expected'])
                                    admitted_page = (identity, position//4096)
                pending = dict(index=self.index, calls=len(state['calls']), expected=expected,
                               input_prefix=input_prefix, admitted_page=admitted_page)
                return False
            except BaseException:
                return failed()

    class UserReturned(gdb.Breakpoint):
        def __init__(self, index):
            super().__init__('*'+hex(cases[index]['returned']),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def stop(self):
            nonlocal pending
            try:
                self.enabled = False
                case = cases[self.index]
                if pending is None and case['number'] == 57 and case['expected'] > 0:
                    # Resuming the parent's saved fork continuation does not
                    # execute another syscall instruction or its Before trap.
                    pending = dict(index=self.index, calls=len(state['calls']), expected=None)
                assert pending is not None and pending['index'] == self.index and current is None
                assert reg('rax') == case['expected'] & ((1 << 64)-1), (case, reg('rax'))
                assert reg('r12') == 0x51ab
                count = len(state['calls'])-pending['calls']
                if pending['expected'] is not None:
                    assert count == pending['expected'], (case, pending, count)
                state['syscalls'].append(dict(index=self.index, number=case['number'],
                                             result=case['expected'], page_actions=count))
                pending = None
                if config.get('terminate_inactive') and case.get('tag') == 'positional-fork':
                    if case['expected'] == 0:
                        parent = next(i for i in range(self.index+1, len(cases))
                                      if cases[i].get('tag') == 'positional-fork')
                        UserReturned(parent)
                        gdb.execute('set $rax=24')
                    else:
                        assert words(controller+0x2000, 1) == (1,)
                        assert words(controller+56, 1) == (1,)
                        state['inactive_child_retained_at_parent_exit'] = True
                        gdb.execute('set $rax=60')
                        gdb.execute('set $rdi=35')
                    gdb.execute('set $rip=$rcx-2')
                    return False
                if self.index+1 < len(cases): arm(self.index+1)
                return False
            except BaseException:
                return failed()

    def arm(index):
        if not (cases[index]['number'] == 57 and cases[index]['expected'] > 0): Before(index)
        UserReturned(index)

    class Returned(gdb.Breakpoint):
        def __init__(self, kind):
            self.stack = reg('rsp')
            self.saved = [reg(v) for v in registers]
            self.kind = kind
            super().__init__('*'+hex(words(self.stack, 1)[0]), internal=True)

        def stop(self):
            nonlocal current
            try:
                self.enabled = False
                assert reg('rsp') == self.stack+8
                if self.kind == 'source':
                    check_frame()
                    assert reg('rax') in (0, 1)
                    current['source_result'] = reg('rax')
                    assert words(control, 8) == (0, 0, current['generation'], 0, 0, 0, 0, 0)
                    header = words(arena, 4)
                    assert header[0] == control and header[2] == current['generation']
                    assert 32 <= header[1] <= library['allocation']['numeric']['maximum_bytes']
                    current['arena_peak'] = header[1]
                    assert memory(associations, 256) == current['metadata_before']
                    assert memory(payloads, 32768) == current['payload_prepared']
                    current['map_after_source'] = list(memory(owner_map, 8))
                elif self.kind == 'owner':
                    assert [reg(v) for v in registers] == self.saved
                    assert current['source_entries'] == 1 and current['source_result'] == 1
                    selected = current['selected']
                    assert reg('rax') == selected
                    expected_map = bytearray(current['map_before'])
                    expected_map[selected] = 1-current['action']
                    assert memory(owner_map, 8) == bytes(expected_map)
                    expected = bytearray(current['metadata_before'])
                    expected[selected*32:selected*32+32] = (
                        struct.pack('<QQQQ', current['identity'], current['argument'], 0, 0)
                        if current['action'] == 0 else bytes(32))
                    assert memory(associations, 256) == bytes(expected)
                    assert memory(payloads, 32768) == current['payload_prepared']
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame']-4096, 4096); zero(current['frame'], 64)
                    assert words(history, 1) == (current['generation'],)
                    record = {k: v for k, v in current.items() if not isinstance(v, bytes)}
                    record.update(physical_cleanup=True, all_gprs_preserved=True,
                                  exact_association_publication=True)
                    state['calls'].append(record)
                    state['generation'] = current['generation']
                    current = None
                else:
                    zero(history, 8)
                    state['finalized'] = True
                return False
            except BaseException:
                return failed()

    class Enter(gdb.Breakpoint):
        def __init__(self, symbol, kind):
            self.kind = kind
            super().__init__('*'+symbol, internal=True)

        def stop(self):
            nonlocal current, refusal
            try:
                if self.kind == 'owner':
                    assert current is None and not state['finalized']
                    if config.get('fault'):
                        if config.get('release_fault') and reg('rdx') != 1: pass
                        else:
                            fault = config['fault']
                            if fault == 'foreign-owner': gdb.execute('set $rdi=$rdi+8')
                            elif fault == 'object-stride': gdb.execute('set $rsi=$rsi+1')
                            elif fault == 'interrupts-enabled': gdb.execute('set $eflags=$eflags|0x200')
                            elif fault == 'dirty-arena': gdb.selected_inferior().write_memory(arena+32768, b'\xa5')
                            elif fault == 'generation-exhausted': gdb.selected_inferior().write_memory(history, b'\xff'*8)
                            else: raise AssertionError(fault)
                            refusal = dict(map=memory(owner_map, 8), metadata=memory(associations, 256),
                                payload=memory(payloads, 32768), arena=memory(arena, 65536),
                                control=memory(control, 64), history=memory(history, 8),
                                source_entries=len(state['calls']))
                            return False
                    assert reg('rdi') == controller and not reg('eflags') & 0x200
                    obj = reg('rsi')
                    assert 0 <= obj-controller-0x43100 < 512 and (obj-controller-0x43100) % 128 == 0
                    assert words(obj, 1) == (1,)
                    action, argument = reg('rdx'), reg('rcx')
                    before = memory(owner_map, 8)
                    assert action in (0, 1) and all(v in (0, 1) for v in before)
                    if action == 0:
                        free = [i for i, v in enumerate(before) if v == 0]
                        selected = free[-1] if config.get('reverse') else free[0]
                        prepared = memory(payloads, 32768)
                        fact = 0
                        # Source allocation is entered only after input staging.
                        if pending is not None:
                            assert cases[pending['index']]['number'] in (1, 18)
                            assert pending['admitted_page'] == (words(obj+8, 1)[0], argument)
                            prefix = pending['input_prefix']
                            assert prefix is not None
                            zero_start = artifact['volatile_files']['transfer_staging_offset']
                            assert memory(controller+zero_start, len(prefix)) == prefix
                    else:
                        selected = fact = argument
                        assert selected < 8 and before[selected] == 1
                        prepared = bytearray(memory(payloads, 32768))
                        prepared[selected*4096:(selected+1)*4096] = bytes(4096)
                        prepared = bytes(prepared)
                    zero(arena, 65536); zero(control, 64)
                    assert words(history, 1) == (state['generation'],)
                    current = dict(action=action, argument=argument, fact=fact,
                        identity=words(obj+8, 1)[0], selected=selected,
                        generation=state['generation']+1, map_before=list(before),
                        metadata_before=memory(associations, 256), payload_prepared=prepared,
                        caller_return=words(reg('rsp'), 1)[0],
                        source_entries=0, source_result=None)
                elif self.kind == 'source':
                    assert current is not None and refusal is None
                    assert (reg('rdi'), reg('rsi'), reg('rdx'), reg('r8')) == (control, controller, current['generation'], 2)
                    current['frame'] = reg('rcx')
                    check_frame()
                    assert words(control, 8) == (export, controller, state['generation'], 0, owner_map, 8, 1, 0)
                    assert memory(owner_map, 8) == bytes(current['map_before'])
                    assert memory(associations, 256) == current['metadata_before']
                    assert memory(payloads, 32768) == current['payload_prepared']
                else:
                    assert current is None and not state['finalized']
                    zero(arena, 65536); zero(control, 64)
                    zero(owner_map, 8); zero(associations, 256); zero(payloads, 32768)
                Returned(self.kind)
                return False
            except BaseException:
                return failed()

    class Ordinary(gdb.Breakpoint):
        def stop(self):
            try:
                assert current is not None
                frame = gdb.newest_frame()
                assert tuple(natural(frame.read_var(v)) for v in ('operation', 'slot')) == (current['action'], current['fact'])
                view = int(frame.read_var('pool'))
                assert words(view, 8) == (control, 0, 8, 1, current['generation'], controller, export, 0)
                depth = current['frame']-reg('rsp')
                assert 0 <= depth <= library['maximum_stack_bytes']+32
                current['ordinary_depth'] = depth
                adapter = frame.older()
                native = adapter.older()
                caller = native.older()
                assert adapter.name() == library['native_export']
                assert native.name() == 'topal_x86_systems_ram_page_apply_v1'
                assert caller.name() == binding['controller_code']['symbol']
                assert caller.pc() == current['caller_return']
                current['source_caller_chain'] = [frame.name(), adapter.name(), native.name(), caller.name()]
                current['source_entries'] += 1
                return False
            except BaseException:
                return failed()

    class Fatal(gdb.Breakpoint):
        def stop(self):
            try:
                if config.get('bad_action'):
                    assert current is not None and current['source_entries'] == 1
                    assert current['source_result'] == (0 if config['bad_action'].startswith('false') else 1)
                    assert memory(associations, 256) == current['metadata_before']
                    assert memory(payloads, 32768) == current['payload_prepared']
                    assert list(memory(owner_map, 8)) == current['map_after_source']
                    assert current['map_after_source'] != current['map_before']
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame']-4096, 4096); zero(current['frame'], 64)
                    assert words(history, 1) == (current['generation'],)
                    assert all(reg(v) == 0 for v in ('rdi', 'rsi', 'rcx'))
                    state.update(result='expected-admitted-action-fatal', bad_action=config['bad_action'],
                        no_owner_resumption=True, source_stores_survive=True,
                        physical_cleanup=True, no_association_publication=True,
                        action=current['action'], facts=[current['action'], current['fact']],
                        actual_source_result=current['source_result'],
                        map_before=current['map_before'], map_after=current['map_after_source'],
                        generation=current['generation'],
                        released_payload_zero_before_source=current['action'] == 1)
                else:
                    assert refusal is not None and current is None
                    for name, start, count in [('map', owner_map, 8), ('metadata', associations, 256),
                        ('payload', payloads, 32768), ('arena', arena, 65536),
                        ('control', control, 64), ('history', history, 8)]:
                        assert memory(start, count) == refusal[name], name
                    assert len(state['calls']) == refusal['source_entries']
                    state.update(result='expected-pre-entry-refusal', fault=config['fault'],
                        release_before_erasure=bool(config.get('release_fault')),
                        no_source_entry=True, no_owner_resumption=True, storage_unchanged=True)
                (output/'observation.json').write_text(json.dumps(state, indent=2)+'\n')
                return True
            except BaseException:
                return failed()

    class Finished(gdb.Breakpoint):
        def stop(self):
            try:
                assert current is None and pending is None and state['finalized'] and state['calls']
                if config.get('terminate_inactive'):
                    assert state['inactive_child_retained_at_parent_exit']
                else:
                    assert len(state['syscalls']) == len(cases)
                zero(arena, 65536); zero(control, 64); zero(history, 8)
                zero(controller, binding['controller_owner']['bytes'])
                state['result'] = 'pass-controlled-inactive-terminal' if config.get('terminate_inactive') else 'pass'
                (output/'observation.json').write_text(json.dumps(state, indent=2)+'\n')
                return True
            except BaseException:
                return failed()

    Enter('topal_x86_systems_ram_page_apply_v1', 'owner')
    Enter(library['native_export'], 'source')
    Enter('topal_x86_systems_ram_page_finalize_v1', 'finalize')
    entry = next(v['symbol'] for v in library['functions'] if '.fn.apply.' in v['symbol'])
    Ordinary("'"+entry+"'", internal=True)
    Fatal('*'+library['imported_hooks'][1], internal=True)
    Finished('*topal_x86_systems_fatal', internal=True)
    if cases: arm(0)
