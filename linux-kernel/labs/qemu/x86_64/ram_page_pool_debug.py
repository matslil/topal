"""Actual source-component parameters/helpers/owner bytes; not kernel teardown proof."""
import gdb
import json
import struct
from pathlib import Path

config = json.loads(Path(gdb.parse_and_eval('$ram_config').string()).read_text())
record = config['record']
current = None
completed = 0
helpers = set()
stores = set()
maximum_depth = 0
owner_stores = 0
preserved = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')


def reg(name):
    return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)


def address(name):
    return int(gdb.parse_and_eval("&'" + name + "'"))


def memory(start, size):
    return bytes(gdb.selected_inferior().read_memory(start, size))


def words(start, count):
    return struct.unpack('<' + 'Q' * count, memory(start, 8 * count))


def vectors():
    return [[int(gdb.parse_and_eval('$xmm%d.v2_int64[%d]' % (i, j)))
             for j in range(2)] for i in range(16)]


def natural(pointer):
    pointer = int(pointer)
    arena, cursor = address('arena'), int(gdb.parse_and_eval('cursor'))
    extents = [(arena + 32, arena + cursor)]
    extents += [(item['address'], item['address'] + item['bytes']) for item in config['data']]
    assert any(low <= pointer and pointer + 16 <= high for low, high in extents)
    sign, limbs = words(pointer, 2)
    assert sign == 0 and limbs <= 2
    assert any(low <= pointer and pointer + 16 + limbs * 4 <= high for low, high in extents)
    return int.from_bytes(memory(pointer + 16, limbs * 4), 'little') if limbs else 0


def source_view(frame):
    global maximum_depth
    depth = current['stack'] - reg('rsp')
    assert 0 <= depth <= record['maximum_stack_bytes']
    maximum_depth = max(maximum_depth, depth)
    view = int(frame.read_var('pool'))
    arena, cursor = address('arena'), int(gdb.parse_and_eval('cursor'))
    assert arena + 32 <= view and view + 64 <= arena + cursor
    control = words(current['control'], 8)
    assert control[0] == address(record['native_export']) and control[3] == 1
    assert control[1:3] == (current['owner'], current['epoch'])
    assert words(view, 8) == (current['control'], 0, current['extent'], 1,
                            current['epoch'], current['owner'], control[0], 0)


class Returned(gdb.Breakpoint):
    def stop(self):
        global completed, current
        self.enabled = False
        assert current is not None and current['entered']
        assert reg('rax') == current['expected_result']
        assert reg('rsp') == current['stack'] + 8
        assert [reg(name) for name in preserved] == current['saved']
        assert vectors() == current['vectors']
        assert words(current['control'], 8) == (0, 0, current['epoch'], 0, 0, 0, 0, 0)
        assert words(current['frame'], 2) == current['facts']
        assert memory(current['base'], 8) == current['expected_map']
        assert current['store_calls'] == current['expected_result']
        current = None
        completed += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        global current
        if not int(gdb.parse_and_eval('ram_debug_sample')):
            return False
        assert current is None and reg('r8') == 2
        control = words(reg('rdi'), 8)
        current = dict(control=reg('rdi'), owner=reg('rsi'), epoch=reg('rdx'),
                       frame=reg('rcx'), facts=words(reg('rcx'), 2), stack=reg('rsp'),
                       base=control[4], extent=control[5],
                       expected_result=int(gdb.parse_and_eval('ram_expected_result')),
                       expected_map=memory(address('ram_expected_map'), 8),
                       saved=[reg(name) for name in preserved], vectors=vectors(), entered=False,
                       store_calls=0)
        Returned('*' + hex(words(reg('rsp'), 1)[0]), internal=True)
        return False


class Ordinary(gdb.Breakpoint):
    def stop(self):
        if current is None:
            return False
        assert not current['entered']
        frame = gdb.newest_frame()
        assert tuple(natural(frame.read_var(name)) for name in ('operation', 'slot')) == current['facts']
        source_view(frame)
        current['entered'] = True
        return False


class Helper(gdb.Breakpoint):
    def __init__(self, symbol, mode):
        super().__init__("'" + symbol + "'", internal=True)
        self.mode = mode

    def stop(self):
        if current is None:
            return False
        frame = gdb.newest_frame()
        source_view(frame)
        slot, occupied = (natural(frame.read_var(name)) for name in ('slot', 'occupied'))
        assert slot < 8 and memory(current['base'] + slot, 1)[0] == occupied
        if self.mode == 'release':
            assert current['facts'] == (1, slot)
        else:
            assert current['facts'] == (0, 0)
        helpers.add(self.mode)
        if occupied == (1 if self.mode == 'release' else 0):
            stores.add(self.mode)
        return False


class OwnerStore(gdb.Breakpoint):
    def stop(self):
        global owner_stores
        if current is not None:
            assert current['entered'] and current['expected_result'] == 1
            current['store_calls'] += 1
            assert current['store_calls'] == 1
            owner_stores += 1
        return False


gdb.execute('set pagination off')
gdb.execute('set confirm off')
gdb.execute('set detach-on-fork on')
gdb.execute('set follow-fork-mode parent')
entering = Enter('*' + record['native_export'], internal=True)
entry = next(item['symbol'] for item in record['functions'] if '.fn.apply.' in item['symbol'])
ordinary = Ordinary("'" + entry + "'", internal=True)
store = next(item['symbol'] for item in record['functions']
             if item['symbol'].endswith('.primitive.store'))
OwnerStore("'" + store + "'", internal=True)
for part, mode in [('reserve-if-free', 'allocate'), ('release-if-occupied', 'release')]:
    # Each constant slot may have its own real imported-function specialization.
    suffix = 'support_2epage_2dslots_2e' + part.replace('-', '_2d') + '.'
    symbols = [item['symbol'] for item in record['functions'] if suffix in item['symbol']]
    assert symbols, part
    for symbol in symbols:
        Helper(symbol, mode)
gdb.execute('run')
assert gdb.selected_inferior().pid == 0 and int(gdb.parse_and_eval('$_exitcode')) == 0
assert completed == 55 and current is None and helpers == stores == {'allocate', 'release'}
assert owner_stores == 19
print('RAM_PAGE_POOL_DEBUG_PASS samples=%d helper-kinds=%d observed-depth=%d source-bound=%d owner-stores=%d' %
      (completed, len(helpers), maximum_depth, record['maximum_stack_bytes'], owner_stores))
