"""Physical root loan, ordinary frames, owner bytes and complete reclamation."""
import gdb
import json
import struct
from pathlib import Path


def install(config):
    OUT = Path(config['output'])
    META = config['artifact']
    expected = bytearray(64)
    expected[0] = expected[config.get('destination', 1)] = 90
    if config.get('store24'):
        expected[24] = 37
    expected_owner = bytes(expected)
    state = {'root_returns': 0, 'source_returns': 0, 'helpers': 0, 'finalized': False, 'minimum_rsp': 2**64}
    regs = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')

    def reg(name): return int(gdb.parse_and_eval('$' + name))
    def addr(name): return int(gdb.parse_and_eval('&\'' + name + '\''))
    def memory(start, size): return bytes(gdb.selected_inferior().read_memory(start, size))
    def words(start, count): return struct.unpack('<' + 'Q' * count, memory(start, count * 8))
    def zero(start, size): assert memory(start, size) == bytes(size), (hex(start), size)
    def number(value):
        start = int(value); sign, count = words(start, 2)
        assert sign == 0 and count <= 2
        if count == 0: return 0
        return sum(word << (32 * index) for index, word in enumerate(struct.unpack('<' + 'I' * count, memory(start + 16, count * 4))))
    def failed(error):
        (OUT/'failed.txt').write_text(repr(error))
        return True

    CONTROL = addr('topal_x86_systems_region_control')
    ARENA = addr('topal_x86_systems_region_arena')
    HISTORY = addr('topal_x86_systems_region_history')
    POOL = addr('topal_bootstrap_storage')
    EXPORT = addr('topal_kernel_region_apply_v1')

    class Return(gdb.FinishBreakpoint):
        def __init__(self, kind, saved, stack):
            super().__init__(internal=True)
            self.kind, self.saved, self.stack = kind, saved, stack
        def stop(self):
            try:
                assert [reg(name) for name in regs] == self.saved, (self.kind, 'callee registers')
                assert reg('rsp') == self.stack + 8, (self.kind, 'stack')
                if self.kind == 'source':
                    assert words(CONTROL, 8) == (0, 0, 1, 0, 0, 0, 0, 0)
                    header = words(ARENA, 4)
                    assert header[0] == CONTROL and 32 <= header[1] <= META['region_library']['allocation']['numeric']['maximum_bytes'] and header[2] == 1 and header[3] > 0, header
                    state['observed_arena_bytes'] = header[1]
                    assert memory(state['owner'], 64) == expected_owner
                    state['source_returns'] += 1
                elif self.kind == 'root':
                    zero(CONTROL, 64); zero(ARENA, 65536)
                    zero(state['frame_sp'] - 4096, 4096)
                    assert words(HISTORY, 1) == (1,)
                    assert memory(state['owner'], 64) == expected_owner
                    assert memory(POOL, 4096) == state['private_prefix']
                    assert all(reg(name) == 0 for name in ('rax', 'rcx', 'rdx', 'rsi', 'rdi', 'r8', 'r9', 'r10', 'r11'))
                    state['root_returns'] += 1
                else:
                    zero(HISTORY, 8)
                    state['finalized'] = True
                return False
            except BaseException as error: return failed(error)

    class Enter(gdb.Breakpoint):
        def __init__(self, symbol, kind):
            super().__init__('*' + symbol, internal=True)
            self.kind = kind
        def stop(self):
            try:
                if self.kind == 'root':
                    assert not reg('eflags') & 0x200
                    assert reg('rdi') == POOL + 4096
                    assert reg('rsi') == 64 and reg('rdx') == 4097
                    zero(CONTROL, 64); zero(ARENA, 65536); zero(HISTORY, 8)
                    fault = config.get('fault')
                    if fault == 'owner-prefix':
                        gdb.execute('set $rdi = ' + str(POOL)); gdb.execute('set $rdx = 1')
                    elif fault == 'interrupts-enabled':
                        gdb.execute('set $eflags = $eflags | 0x200')
                    elif fault == 'dirty-arena':
                        gdb.selected_inferior().write_memory(ARENA + 40, bytes([7]))
                    state['entry_snapshot'] = [memory(address, size) for address, size in ((POOL, 4160), (CONTROL, 64), (ARENA, 65536), (HISTORY, 8))]
                    state['root_return_pc'] = words(reg('rsp'), 1)[0]
                    state['owner'] = reg('rdi')
                    root = next(place for place in META['placements'] if place['symbol'] == META['entry_symbol'])
                    assert root['address'] <= state['root_return_pc'] < root['address'] + root['size']
                    state['private_prefix'] = memory(POOL, 4096)
                    state['frame_sp'] = reg('rsp') - 48
                elif self.kind == 'source':
                    assert reg('rdi') == CONTROL and reg('rsi') == 4097 and reg('rdx') == 1
                    assert reg('r12') == state['frame_sp']
                Return(self.kind, [reg(name) for name in regs], reg('rsp'))
                return False
            except BaseException as error: return failed(error)

    class Helper(gdb.Breakpoint):
        def stop(self):
            try:
                frame = gdb.newest_frame()
                view = frame.read_var('region')
                assert 'KernelByteRegion' in str(view.type)
                assert int(view) == ARENA + 32
                assert words(int(view), 8) == (CONTROL, 0, 64, 1, 1, 4097, EXPORT, 0)
                assert words(CONTROL, 8) == (EXPORT, 4097, 1, 1, state['owner'], 64, 1, 1)
                assert number(frame.read_var('source')) == 0
                assert number(frame.read_var('destination')) == config.get('destination', 1)
                chain = []
                current = frame
                for _ in range(5):
                    if current is None: break
                    chain.append(current.name()); current = current.older()
                state['helper_chain'] = chain
                assert chain[4] == '_topal_kernel_entry', chain
                current = frame.older().older().older().older()
                assert current.pc() == state['root_return_pc'], (current.pc(), state['root_return_pc'])
                assert chain[0] in ['support.bytes.copy-byte', helper] and chain[1] in ['apply', 'topal.region.topal.fn.apply.1'] and chain[2] == 'topal_kernel_region_apply_v1', chain
                assert chain[3] == 'topal_x86_systems_region_library_apply_v1', chain
                source = frame.find_sal().symtab.fullname()
                state['helper_file'] = source
                state['helper_file_exists'] = Path(source).is_file()
                assert state['helper_file_exists'], source
                assert source.endswith('support/bytes.t'), source
                state['helpers'] += 1
                return False
            except BaseException as error: return failed(error)

    class Depth(gdb.Breakpoint):
        def stop(self):
            try:
                stack = reg('rsp')
                state['minimum_rsp'] = min(state['minimum_rsp'], stack)
                assert state['frame_sp'] - stack <= META['region_runtime']['combined_stack_bytes']
                return False
            except BaseException as error: return failed(error)

    class Fatal(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['root_returns'] == state['source_returns'] == state['helpers'] == 1, state
                if config.get('wrong_helper'):
                    assert not state['finalized']
                    assert words(HISTORY, 1) == (1,)
                    state['result'] = 'expected-owner-byte-guard-refusal'
                    publish()
                    return True
                assert state['finalized']
                zero(CONTROL, 64); zero(ARENA, 65536); zero(HISTORY, 8)
                state['observed_source_stack_bytes'] = state['frame_sp'] - state['minimum_rsp']
                state['proved_source_caller_stack_bytes'] = META['region_runtime']['combined_stack_bytes']
                state['result'] = 'pass'
                publish()
                return True
            except BaseException as error: return failed(error)

    def publish():
        state.pop('private_prefix', None)
        state.pop('entry_snapshot', None)
        (OUT/'observation.json').write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')

    class Abort(gdb.Breakpoint):
        def stop(self):
            try:
                assert config.get('fault'), 'unexpected region abort'
                assert state['root_returns'] == state['source_returns'] == state['helpers'] == 0
                actual = [memory(address, size) for address, size in ((POOL, 4160), (CONTROL, 64), (ARENA, 65536), (HISTORY, 8))]
                assert actual == state['entry_snapshot'], 'refusal changed owned storage'
                state['result'] = 'expected-pre-entry-refusal'
                publish()
                return True
            except BaseException as error: return failed(error)

    Enter('topal_x86_systems_region_library_apply_v1', 'root')
    Enter('topal_kernel_region_apply_v1', 'source')
    Enter('topal_x86_systems_region_library_finalize_v1', 'finalize')
    helper = next(function['symbol'] for function in META['region_library']['functions'] if 'copy_2dbyte' in function['symbol'])
    Helper(helper, internal=True)
    for function in META['region_library']['functions']:
        Depth('*' + hex(addr(function['symbol'])), internal=True)
    for symbol in META['region_library']['imported_hooks']:
        Depth('*' + hex(addr(symbol)), internal=True)
    Fatal('*topal_x86_systems_fatal', internal=True)

    Abort('*topal_kernel_region_abort_v1', internal=True)
