"""Actual persistent field loan, ordinary source, caller and cleanup oracle."""
import gdb
import json
import struct
import traceback
from pathlib import Path


def install(config):
    output = Path(config['output'])
    artifact = config['artifact']
    library = artifact['file_position_library']
    binding = artifact['file_position_runtime']
    framed = binding['interface'] in ('borrowed-byte-region-framed-action-v1',
                                     'borrowed-byte-region-instance-action-v1')
    export_name = library['native_export']
    prefix = ('topal.region-instance.' + library['instance'] + '.' if library.get('instance')
              else 'topal.framed-region.' if framed else 'topal.region.action.')
    if library.get('instance'):
        assert binding['instance'] == library['instance'] == 'file-position'
    state = {'calls': [], 'generation': 0, 'finalized': False, 'finalizer_entries': 0, 'scope': 'actual persistent field loans; source/owner/cleanup observations'}
    current = None
    pending = None
    refusal = None
    cases = config.get('cases', [])
    state['syscalls'] = []
    registers = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')

    def reg(name): return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)
    def addr(name): return int(gdb.parse_and_eval("&'" + name + "'"))
    def memory(address, size): return bytes(gdb.selected_inferior().read_memory(address, size))
    def words(address, count): return struct.unpack('<' + 'Q' * count, memory(address, count * 8))
    def zero(address, size): assert memory(address, size) == bytes(size), (hex(address), size)
    def natural(value):
        address = int(value)
        sign, count = words(address, 2)
        assert sign == 0 and count <= 3, (sign, count)
        return int.from_bytes(memory(address + 16, count * 4), 'little') if count else 0
    def failed():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    control = addr('topal_x86_systems_file_position_control')
    arena = addr('topal_x86_systems_file_position_arena')
    history = addr('topal_x86_systems_file_position_history')
    controller = addr('topal_x86_static_process_descriptor_table')
    export = addr(export_name)

    def check_scalar_frame():
        if framed:
            assert binding['scalar_count'] == library['scalar_count'] == 2
            address = current['frame_sp']
            assert words(address, 2) == (current['base'], current['transferred'])
            # The actual caller owns this mapped stack extent above its
            # retired reserve, disjoint from every private invocation object.
            assert 0x108000 <= address - 4096 < address < address + 32 <= 0x180000
            for start, size in ((control, 64), (arena, 65536),
                                (controller, binding['controller_owner']['bytes'])):
                assert address + 32 <= start or start + size <= address

    def positions():
        result = {}
        for bank in (320, 0x3d000):
            for offset in range(0, 1024, 64):
                cell = controller + bank + offset
                kind, references = words(cell, 2)
                if kind in (3, 7) and references:
                    result[cell] = words(cell + 24, 1)[0]
        return result

    class Before(gdb.Breakpoint):
        def __init__(self, index):
            super().__init__('*' + hex(cases[index]['returned'] - 2),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def stop(self):
            nonlocal pending
            try:
                self.enabled = False
                assert pending is None and current is None
                case = cases[self.index]
                assert reg('rax') == case['number']
                fd = reg('rdi') & 0xffffffff
                pending = dict(index=self.index, calls=len(state['calls']), positions=positions(),
                               description=None, base=None)
                if fd < 16:
                    slot = words(controller + 64 + fd * 16, 1)[0]
                    if slot:
                        assert 1 <= slot <= 32
                        cell = controller + (320 + (slot-1)*64 if slot <= 16 else 0x3d000+(slot-17)*64)
                        if cell in pending['positions']:
                            pending['description'] = cell
                            base = pending['positions'][cell]
                            if case['number'] == 1 and words(cell+16, 1)[0] & 0x400:
                                identity = words(cell+32, 1)[0]
                                sizes = [words(controller+0x43000+256+i*128+16, 1)[0]
                                         for i in range(4)
                                         if words(controller+0x43000+256+i*128, 2) == (1, identity)]
                                assert len(sizes) == 1
                                base = sizes[0]
                            pending['base'] = base
                return False
            except BaseException:
                return failed()

    def arm(index):
        UserReturned(index)
        if cases[index]['number'] in (0, 1, 17, 18):
            Before(index)

    class UserReturned(gdb.Breakpoint):
        def __init__(self, index):
            super().__init__('*' + hex(cases[index]['returned']),
                             type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            self.index = index

        def stop(self):
            nonlocal pending
            try:
                self.enabled = False
                case = cases[self.index]
                assert current is None
                assert reg('rax') == case['expected'] & ((1 << 64)-1), (self.index, case, reg('rax'))
                assert reg('r12') == 0x51ab
                entry = dict(index=self.index, number=case['number'], result=case['expected'])
                if case['number'] in (0, 1, 17, 18):
                    assert pending is not None and pending['index'] == self.index
                    expected = int(case['number'] in (0, 1) and case['expected'] > 0
                                   and pending['description'] is not None)
                    assert len(state['calls']) - pending['calls'] == expected, (case, pending)
                    after = dict(pending['positions'])
                    if expected:
                        call = state['calls'][-1]
                        assert (call['description'], call['base'], call['transferred']) == (
                            pending['description'], pending['base'], case['expected']), (case, call, pending)
                        after[call['description']] = pending['base'] + case['expected']
                    assert positions() == after, (case, pending, positions())
                    entry.update(source_invocations=expected, all_position_fields_checked=True)
                    pending = None
                state['syscalls'].append(entry)
                if config.get('terminate_inactive') and case.get('tag') == 'positional-fork':
                    if case['expected'] == 0:
                        parent = next(i for i in range(self.index+1, len(cases))
                                      if cases[i].get('tag') == 'positional-fork')
                        UserReturned(parent)
                        gdb.execute('set $rax=24')
                        gdb.execute('set $rip=$rcx-2')
                    else:
                        assert words(controller+0x2000, 1) == (1,)
                        assert words(controller+56, 1) == (1,)
                        state['inactive_child_retained_at_parent_exit'] = True
                        gdb.execute('set $rax=60')
                        gdb.execute('set $rdi=35')
                        gdb.execute('set $rip=$rcx-2')
                    return False
                if self.index+1 < len(cases):
                    arm(self.index+1)
                return False
            except BaseException:
                return failed()

    class Returned(gdb.Breakpoint):
        def __init__(self, kind):
            # Observe the actual saved native return address. GDB's inferred
            # finish scope is ambiguous for leaf calls inside an internal
            # same-symbol service subroutine without a conventional frame.
            pc = words(reg('rsp'), 1)[0]
            super().__init__('*' + hex(pc), internal=True)
            self.kind = kind
            self.saved = [reg(name) for name in registers]
            self.stack = reg('rsp')
        def stop(self):
            nonlocal current
            try:
                # A non-stopping Python breakpoint must explicitly retire its
                # per-invocation observation before this call site is reused.
                self.enabled = False
                actual = [reg(name) for name in registers]
                assert actual == self.saved, (self.kind, 'callee registers', self.saved, actual)
                assert reg('rsp') == self.stack + 8, (self.kind, 'stack')
                if self.kind == 'source':
                    check_scalar_frame()
                    assert reg('rax') == (0 if config.get('false_action') else 1)
                    assert words(control, 8) == (0, 0, current['generation'], 0, 0, 0, 0, 0)
                    header = words(arena, 4)
                    assert header[0] == control and header[2] == current['generation']
                    assert 32 <= header[1] <= library['allocation']['numeric']['maximum_bytes']
                    assert header[3] > 0
                    current['observed_arena_bytes'] = header[1]
                    current['source_result'] = reg('rax')
                    if config.get('wrong_codec'):
                        actual = memory(current['description'], 64)
                        assert actual != current['expected']
                        assert actual[:24] == current['expected'][:24] and actual[32:] == current['expected'][32:]
                        state.update(result='expected-wrong-source-byte-rejection', actual_field=list(actual[24:32]),
                                     expected_field=list(current['expected'][24:32]), before_owner_resumption=True)
                        (output/'observation.json').write_text(json.dumps(state, indent=2)+'\n')
                        return True
                    assert memory(current['description'], 64) == current['expected']
                elif self.kind == 'owner':
                    assert reg('rax') == 1
                    assert current['entries'] == current['codecs'] == 1
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame_sp'] - 4096, 4096)
                    zero(current['frame_sp'], 32)
                    assert words(history, 1) == (current['generation'],)
                    assert memory(current['description'], 64) == current['expected']
                    assert all(reg(name) == 0 for name in ('rcx', 'rdx', 'rsi', 'rdi', 'r8', 'r9', 'r10', 'r11'))
                    current['field_after'] = list(current['expected'][24:32])
                    current['caller_preserved'] = current['temporary_cleanup'] = True
                    current.pop('expected')
                    state['generation'] = current['generation']
                    state['calls'].append(current)
                    current = None
                else:
                    zero(history, 8)
                    state['finalized'] = True
                    assert state['calls']
                return False
            except BaseException:
                return failed()

    class Enter(gdb.Breakpoint):
        def __init__(self, symbol, kind):
            super().__init__('*' + symbol, internal=True)
            self.kind = kind
        def stop(self):
            nonlocal current, refusal
            try:
                if self.kind == 'owner':
                    assert current is None and not state['finalized']
                    if config.get('fault'):
                        description = reg('rsi')
                        fault = config['fault']
                        if fault == 'owner-prefix':
                            gdb.execute('set $rdi=$rdi+8')
                        elif fault == 'field-stride':
                            gdb.execute('set $rsi=$rsi+1')
                        elif fault == 'interrupts-enabled':
                            gdb.execute('set $eflags=$eflags|0x200')
                        elif fault == 'dirty-arena':
                            gdb.selected_inferior().write_memory(arena+32768, b'\xa5')
                        elif fault == 'generation-exhausted':
                            gdb.selected_inferior().write_memory(history, b'\xff'*8)
                        else:
                            raise AssertionError(fault)
                        refusal = dict(description=description, field=memory(description, 64),
                            arena=memory(arena, 65536), control=memory(control, 64), history=memory(history, 8))
                        return False
                    assert reg('rdi') == controller and not reg('eflags') & 0x200
                    description = reg('rsi')
                    relative = description - controller
                    assert any(bank <= relative < bank + 1024 and (relative-bank) % 64 == 0 for bank in (320, 0x3d000))
                    before = memory(description, 64)
                    kind, references = words(description, 2)
                    assert kind in (3, 7) and references > 0
                    base, count = reg('rdx'), reg('rcx')
                    assert 1 <= count <= 256 and base + count <= (1 << 63) - 1
                    zero(arena, 65536); zero(control, 64)
                    assert words(history, 1) == (state['generation'],)
                    expected = bytearray(before)
                    expected[24:32] = (base + count).to_bytes(8, 'little')
                    current = dict(generation=state['generation']+1, description=description,
                        kind=kind, references=references, base=base, transferred=count,
                        old_position=int.from_bytes(before[24:32], 'little'), expected=bytes(expected),
                        frame_sp=reg('rsp')-80, return_pc=words(reg('rsp'), 1)[0], entries=0, codecs=0)
                    # Charge the immutable caller's live count push as well as
                    # the wrapper's actual AND-RSP adjustment and call reserve.
                    caller_save = 8 if kind == 3 else 0
                    alignment = current['frame_sp'] % 16 + 16 + caller_save
                    assert alignment <= binding['caller_alignment_bytes']
                    assert current['frame_sp'] % 16 == (8 if kind == 3 else 0)
                    current['caller_alignment_and_count_save_bytes'] = alignment
                    caller = binding['controller_code']
                    assert caller['address'] <= current['return_pc'] < caller['address'] + caller['bytes']
                elif self.kind == 'source':
                    assert current is not None
                    assert (reg('rdi'), reg('rsi'), reg('rdx')) == (
                        control, controller, current['generation'])
                    if framed:
                        assert (reg('rcx'), reg('r8')) == (current['frame_sp'], 2)
                        check_scalar_frame()
                        current['scalar_transport'] = dict(
                            interface=binding['interface'], address=reg('rcx'), words=2,
                            frame_bytes=16, owned_staging_bytes=32,
                            mapped_owned_extent=True, outside_arena_and_retired_stack=True)
                    else:
                        assert (reg('rcx'), reg('r8')) == (current['base'], current['transferred'])
                    assert reg('r12') == current['frame_sp']
                else:
                    state['finalizer_entries'] += 1
                    assert current is None
                    zero(arena, 65536); zero(control, 64)
                    assert words(history, 1) == (state['generation'],)
                    zero(controller, 8192)
                    zero(controller + 0x3d000, 4096)
                Returned(self.kind)
                return False
            except BaseException:
                return failed()

    class Ordinary(gdb.Breakpoint):
        def __init__(self, symbol, codec):
            super().__init__(symbol, internal=True)
            self.codec = codec
        def stop(self):
            try:
                assert current is not None
                check_scalar_frame()
                frame = gdb.newest_frame()
                if self.codec:
                    assert frame.find_sal().symtab.fullname().endswith('support/word-bytes.t')
                    assert natural(frame.read_var('value')) == current['base'] + current['transferred']
                    view = frame.read_var('region')
                    current['codecs'] += 1
                    chain = []
                    value = frame
                    for _ in range(5):
                        assert value is not None
                        chain.append(value.name()); value = value.older()
                    assert chain[2] == export_name, chain
                    assert chain[3] == 'topal_x86_systems_file_position_apply_v1', chain
                    assert chain[4] == binding['controller_code']['symbol'], chain
                    caller = frame.older().older().older().older()
                    assert caller.pc() == current['return_pc']
                    current['helper_chain'] = chain
                else:
                    assert natural(frame.read_var('base')) == current['base']
                    assert natural(frame.read_var('transferred')) == current['transferred']
                    view = frame.read_var('field')
                    current['entries'] += 1
                assert words(int(view), 8) == (control, 0, 8, 1, current['generation'], controller, export, 0)
                assert words(control, 8) == (export, controller, current['generation'], 1,
                    current['description']+24, 8, 1, 1)
                return False
            except BaseException:
                return failed()

    class Fatal(gdb.Breakpoint):
        def stop(self):
            try:
                if config.get('fault'):
                    assert refusal is not None and current is None and not state['calls']
                    assert memory(refusal['description'], 64) == refusal['field']
                    assert memory(arena, 65536) == refusal['arena']
                    assert memory(control, 64) == refusal['control']
                    assert memory(history, 8) == refusal['history']
                    state.update(result='expected-pre-entry-refusal', fault=config['fault'],
                                 no_source_entry=True, no_owner_resumption=True, captured_storage_unchanged=True)
                elif config.get('false_action'):
                    assert current is not None and current['source_result'] == 0
                    assert current['entries'] == current['codecs'] == 1
                    assert memory(current['description'], 64) == current['expected']
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame_sp']-4096, 4096); zero(current['frame_sp'], 32)
                    assert words(history, 1) == (current['generation'],)
                    state.update(result='expected-false-action-fatal', no_owner_resumption=True,
                                 temporary_cleanup=True, successful_stores_survive=True)
                else:
                    raise AssertionError('unexpected native action fatal disposition')
                (output/'observation.json').write_text(json.dumps(state, indent=2)+'\n')
                return True
            except BaseException:
                return failed()

    class Finished(gdb.Breakpoint):
        def stop(self):
            try:
                assert current is None and state['finalized'] and state['calls'], (state, current, words(history, 1))
                assert pending is None
                if config.get('terminate_inactive'):
                    assert state['inactive_child_retained_at_parent_exit']
                else:
                    assert len(state['syscalls']) == len(cases)
                zero(arena, 65536); zero(control, 64); zero(history, 8)
                zero(controller, binding['controller_owner']['bytes'])
                result = 'pass-controlled-inactive-terminal' if config.get('terminate_inactive') else 'pass'
                (output/'observation.json').write_text(json.dumps(dict(state, result=result), indent=2)+'\n')
                return True
            except BaseException:
                return failed()

    Finished('*topal_x86_systems_fatal', internal=True)
    Enter('topal_x86_systems_file_position_apply_v1', 'owner')
    Enter(export_name, 'source')
    Enter('topal_x86_systems_file_position_finalize_v1', 'finalize')
    entry = next(f['symbol'] for f in library['functions'] if f['symbol'].startswith(prefix) and 'fn.advance.' in f['symbol'])
    codec = next(f['symbol'] for f in library['functions'] if f['symbol'].startswith(prefix) and 'store_2du64_2dle' in f['symbol'])
    Ordinary(entry, False)
    Ordinary(codec, True)
    Fatal('*' + library['imported_hooks'][1], internal=True)
    if cases:
        arm(0)
