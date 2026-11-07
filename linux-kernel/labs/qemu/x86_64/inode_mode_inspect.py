"""Actual inode bytes, source facts, machine ABI and complete reclamation."""
import gdb
import hashlib
import json
from pathlib import Path
import re
import struct
import traceback


def install(config):
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['inspector_sha256']
    output = Path(config['output'])
    artifact = config['artifact']
    library, binding = artifact['inode_mode_library'], artifact['inode_mode_runtime']
    state = dict(calls=[], syscalls=[], generation=0, finalized=False)
    current = None
    refusal = None
    cases = config['cases']
    registers = ('rbx', 'rbp', 'rdi', 'rsi', 'rdx', 'rcx', 'r8', 'r9',
                 'r10', 'r11', 'r12', 'r13', 'r14', 'r15')

    def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1 << 64)-1)
    def addr(name): return int(gdb.parse_and_eval("&'"+name+"'"))
    def memory(start, size): return bytes(gdb.selected_inferior().read_memory(start, size))
    def words(start, count): return struct.unpack('<'+'Q'*count, memory(start, count*8))
    def zero(start, size): assert memory(start, size) == bytes(size), (hex(start), size)
    def vectors():
        return [[int(gdb.parse_and_eval('$xmm%d.v2_int64[%d]' % (i, j)))
                 for j in range(2)] for i in range(16)]
    def failed():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    controller = addr(binding['controller_owner']['symbol'])
    arena = addr('topal_x86_systems_inode_mode_arena')
    control = addr('topal_x86_systems_inode_mode_control')
    history = addr('topal_x86_systems_inode_mode_history')
    export = addr(library['native_export'])
    instructions = {}

    def natural(pointer):
        pointer = int(pointer)
        cursor = words(arena+8, 1)[0]
        extents = [(arena+32, arena+cursor)]
        extents += [(d['address'], d['address']+d['bytes']) for d in binding['source_data']]
        assert any(lo <= pointer and pointer+16 <= hi for lo, hi in extents)
        sign, count = words(pointer, 2)
        assert sign == 0 and count <= 2
        assert any(lo <= pointer and pointer+16+count*4 <= hi for lo, hi in extents)
        return int.from_bytes(memory(pointer+16, count*4), 'little') if count else 0

    def check_frame():
        assert words(current['frame'], 2) == (current['permissions'], current['seconds'])
        assert 0x108000 <= current['frame']-4096 < current['frame']+64 <= 0x180000

    def check_owner():
        expected = bytearray(current['owner_before'])
        offset = current['view']-controller
        mode = (current['before'][0] & ~0o7777) | current['permissions']
        if config.get('bad_action') == 'wrong-mode': mode = 0
        expected[offset:offset+8] = mode.to_bytes(8, 'little')
        if config.get('bad_action') != 'false-after-mode':
            expected[offset+24:offset+32] = current['seconds'].to_bytes(8, 'little')
        assert memory(controller, binding['controller_owner']['bytes']) == bytes(expected)

    class Payload(gdb.Breakpoint):
        def __init__(self, field):
            super().__init__('*(unsigned long long*)'+hex(current['view']+field*8),
                type=gdb.BP_WATCHPOINT, wp_class=gdb.WP_ACCESS, internal=True)
            self.field = field

        def stop(self):
            try:
                assert self.field in (0, 3), ('preserved timestamp accessed', self.field)
                instruction = instructions.get(reg('rip'))
                assert instruction is not None, ('unexpected payload access', hex(reg('rip')))
                if re.match(r'mov\s+BYTE PTR \[.+\],', instruction):
                    current['stores'] += 1
                else:
                    assert self.field == 0 and re.match(r'movzx\s+\w+,\s*BYTE PTR \[.+\]', instruction)
                    current['loads'] += 1
                return False
            except BaseException:
                return failed()

    class Returned(gdb.Breakpoint):
        def __init__(self, kind):
            self.kind, self.stack = kind, reg('rsp')
            self.saved = [reg(r) for r in registers]
            self.simd = vectors()
            super().__init__('*'+hex(words(self.stack, 1)[0]), internal=True)

        def stop(self):
            nonlocal current
            try:
                self.enabled = False
                assert reg('rsp') == self.stack+8
                assert vectors() == self.simd
                if self.kind == 'source':
                    assert [reg(r) for r in ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')] == [self.saved[registers.index(r)] for r in ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')]
                    for watch in current['watches']: watch.delete()
                    current['watches'] = []
                    check_frame()
                    expected = int(config.get('bad_action') != 'false-after-mode')
                    assert reg('rax') == expected and current['ordinary_entries'] == 1
                    assert (current['loads'], current['stores']) == (8, 16 if expected else 8)
                    current['source_result'] = reg('rax')
                    assert words(control, 8) == (0, 0, current['generation'], 0, 0, 0, 0, 0)
                    header = words(arena, 4)
                    assert header[0] == control and header[2] == current['generation']
                    assert 32 <= header[1] <= library['allocation']['numeric']['maximum_bytes']
                    current['arena_peak'] = header[1]
                    check_owner()
                    current['source_returned'] = True
                elif self.kind == 'owner':
                    assert [reg(r) for r in registers] == self.saved
                    assert reg('rax') == 1 and current['source_returned']
                    check_owner()
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame']-4096, 4096); zero(current['frame'], 64)
                    assert words(history, 1) == (current['generation'],)
                    state['generation'] = current['generation']
                    record = {k: v for k, v in current.items() if k not in ('owner_before', 'watches')}
                    record.update(complete_physical_cleanup=True, actual_owner_exact_delta=True,
                        all_caller_gprs_and_simd_preserved=True)
                    state['calls'].append(record)
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
                        fault = config['fault']
                        if fault == 'foreign-owner': gdb.execute('set $rdi=$rdi+8')
                        elif fault == 'record-stride': gdb.execute('set $rsi=$rsi+1')
                        elif fault == 'interrupts-enabled': gdb.execute('set $eflags=$eflags|0x200')
                        elif fault == 'permissions-range': gdb.execute('set $rcx=4096')
                        elif fault == 'dirty-arena': gdb.selected_inferior().write_memory(arena+32768, b'\xa5')
                        elif fault == 'generation-exhausted': gdb.selected_inferior().write_memory(history, b'\xff'*8)
                        elif fault == 'bad-type': gdb.selected_inferior().write_memory(reg('rsi')+24, bytes(8))
                        else: raise AssertionError(fault)
                        refusal = [memory(controller, binding['controller_owner']['bytes']),
                            memory(arena, 65536), memory(control, 64), memory(history, 8)]
                        return False
                    assert reg('rdi') == controller and not reg('eflags') & 0x200
                    record, kind = reg('rsi'), reg('rdx')
                    assert kind in (0, 1, 2)
                    if kind == 1:
                        assert record == controller+binding['directory_metadata_offset']
                        view = record
                    else:
                        start = controller+(binding['file_object_offset'] if kind == 0 else binding['pipe_pool_offset'])
                        assert 0 <= record-start < 512 and (record-start) % 128 == 0
                        assert words(record, 1) == (1,) and words(record+8, 1)[0] > 0
                        view = record+(24 if kind == 0 else 64)
                    before = words(view, 4)
                    assert before[0] & ~0o7777 == (0o100000, 0o40000, 0o10000)[kind]
                    assert reg('rcx') <= 4095 and reg('r8') > 0
                    zero(arena, 65536); zero(control, 64)
                    assert words(history, 1) == (state['generation'],)
                    current = dict(kind=kind, view=view, before=before, permissions=reg('rcx'),
                        seconds=reg('r8'), generation=state['generation']+1,
                        owner_before=memory(controller, binding['controller_owner']['bytes']),
                        ordinary_entries=0, source_returned=False, loads=0, stores=0, watches=[])
                elif self.kind == 'source':
                    assert current is not None
                    assert (reg('rdi'), reg('rsi'), reg('rdx'), reg('r8')) == (control, controller, current['generation'], 2)
                    current['frame'], current['source_stack'] = reg('rcx'), reg('rsp')
                    check_frame()
                    assert words(control, 8) == (export, controller, state['generation'], 0, current['view'], 32, 1, 0)
                    assert words(current['view'], 4) == current['before']
                    for operation in ('store', 'load'):
                        symbol = next(f for f in binding['source_code'] if f['symbol'].endswith('.primitive.unsigned.'+operation))
                        decoded = gdb.selected_frame().architecture().disassemble(symbol['address'], symbol['address']+symbol['bytes'])
                        instructions.update({i['addr']+i['length']: i['asm'] for i in decoded})
                    current['watches'] = [Payload(i) for i in range(4)]
                else:
                    assert current is None and not state['finalized']
                    zero(arena, 65536); zero(control, 64)
                Returned(self.kind)
                return False
            except BaseException:
                return failed()

    class Ordinary(gdb.Breakpoint):
        def stop(self):
            try:
                assert current is not None
                frame = gdb.newest_frame()
                assert tuple(natural(frame.read_var(v)) for v in ('permissions', 'seconds')) == (current['permissions'], current['seconds'])
                view = int(frame.read_var('region'))
                assert words(view, 8) == (control, 0, 32, 1, current['generation'], controller, export, 0)
                depth = current['source_stack']-reg('rsp')
                assert 0 <= depth <= library['maximum_stack_bytes']
                current['ordinary_entries'] += 1
                current['observed_entry_depth'] = depth
                return False
            except BaseException:
                return failed()

    class UserReturned(gdb.Breakpoint):
        def __init__(self, index):
            self.index = index
            super().__init__('*'+hex(cases[index]['returned']), internal=True)

        def stop(self):
            try:
                self.enabled = False
                case = cases[self.index]
                assert current is None and reg('rax') == case['expected'] & ((1 << 64)-1), (self.index, case, reg('rax'))
                assert reg('r12') == 0x51ab
                previous = state['syscalls'][-1]['actions'] if state['syscalls'] else 0
                expected = int(case['number'] in (90, 91, 268, 452) and case['expected'] == 0)
                assert len(state['calls'])-previous == expected
                zero(arena, 65536); zero(control, 64)
                state['syscalls'].append(dict(index=self.index, number=case['number'], result=case['expected'], actions=len(state['calls'])))
                if config.get('terminate_inactive') and case['number'] == 57:
                    if case['expected'] == 0:
                        parent = next(i for i in range(self.index+1, len(cases)) if cases[i]['number'] == 57)
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
                if self.index+1 < len(cases): UserReturned(self.index+1)
                return False
            except BaseException:
                return failed()

    class Finished(gdb.Breakpoint):
        def stop(self):
            try:
                assert current is None and state['finalized'] and state['calls']
                if config.get('terminate_inactive'):
                    assert state['inactive_child_retained_at_parent_exit']
                else:
                    assert len(state['syscalls']) == len(cases)
                assert {c['kind'] for c in state['calls']} == {0, 1, 2}
                zero(arena, 65536); zero(control, 64); zero(history, 8)
                zero(controller, binding['controller_owner']['bytes'])
                state['result'] = 'pass-controlled-inactive-terminal' if config.get('terminate_inactive') else 'pass'
                (output/'observation.json').write_text(json.dumps(state, indent=2, sort_keys=True)+'\n')
                return True
            except BaseException:
                return failed()

    class Fatal(gdb.Breakpoint):
        def stop(self):
            try:
                if config.get('bad_action'):
                    assert current is not None and current['source_returned']
                    check_owner()
                    zero(arena, 65536); zero(control, 64)
                    zero(current['frame']-4096, 4096); zero(current['frame'], 64)
                    assert words(history, 1) == (current['generation'],)
                    state.update(result='expected-admitted-action-fatal', bad_action=config['bad_action'],
                        no_owner_resumption=True, complete_physical_cleanup=True,
                        source_stores_survive=True, source_result=current['source_result'],
                        before=current['before'], after=words(current['view'], 4),
                        permissions=current['permissions'], seconds=current['seconds'])
                    assert state['before'] != state['after']
                else:
                    assert config.get('fault') and current is None and refusal is not None
                    assert [memory(controller, binding['controller_owner']['bytes']),
                        memory(arena, 65536), memory(control, 64), memory(history, 8)] == refusal
                    assert not state['calls']
                    state.update(result='expected-pre-entry-refusal', fault=config['fault'],
                        source_not_entered=True, no_owner_resumption=True, complete_owner_and_runtime_unchanged=True)
                (output/'observation.json').write_text(json.dumps(state, indent=2, sort_keys=True)+'\n')
                return True
            except BaseException:
                return failed()

    gdb.execute('set disassembly-flavor intel')
    Enter('topal_x86_systems_inode_mode_apply_v1', 'owner')
    Enter(library['native_export'], 'source')
    Enter('topal_x86_systems_inode_mode_finalize_v1', 'finalize')
    entry = next(f['symbol'] for f in library['functions'] if '.fn.apply.' in f['symbol'])
    Ordinary("'"+entry+"'", internal=True)
    Finished('*topal_x86_systems_fatal', internal=True)
    Fatal('*'+library['imported_hooks'][1], internal=True)
    UserReturned(0)
