"""Independent GDB observations of actual framed source parameters and return."""
import gdb
import json
import struct
from pathlib import Path

config = json.loads(Path(gdb.parse_and_eval('$frame_config').string()).read_text())
record = config['record']
completed = 0
current = None
preserved = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')


def reg(name):
    return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)


def address(name):
    return int(gdb.parse_and_eval("&'" + name + "'"))


def memory(start, size):
    return bytes(gdb.selected_inferior().read_memory(start, size))


def words(start, count):
    if not count:
        return ()
    return struct.unpack('<' + 'Q' * count, memory(start, count * 8))


def vectors():
    return [[int(gdb.parse_and_eval('$xmm%d.v2_int64[%d]' % (i, j)))
             for j in range(2)] for i in range(16)]


def natural(pointer):
    start = int(pointer)
    arena, cursor = address('arena'), int(gdb.parse_and_eval('cursor'))
    extents = [(arena + 32, arena + cursor)]
    extents += [(item['address'], item['address'] + item['bytes']) for item in config['data']]
    assert any(low <= start and start + 16 <= high for low, high in extents)
    sign, count = words(start, 2)
    assert sign == 0 and count <= 2, (sign, count)
    assert any(low <= start and start + 16 + count * 4 <= high for low, high in extents)
    return int.from_bytes(memory(start + 16, count * 4), 'little') if count else 0


class Returned(gdb.Breakpoint):
    def stop(self):
        global current, completed
        self.enabled = False
        assert current is not None and current['entered']
        assert reg('rax') == 1
        assert reg('rsp') == current['stack'] + 8
        assert [reg(name) for name in preserved] == current['saved']
        assert vectors() == current['vectors']
        assert words(current['control'], 8) == (0, 0, current['epoch'], 0, 0, 0, 0, 0)
        assert words(current['frame'], len(current['values'])) == current['values']
        current = None
        completed += 1
        if completed == 48:
            entering.enabled = ordinary.enabled = False
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        global current
        assert current is None and reg('r8') == record['scalar_count']
        count, frame, stack = reg('r8'), reg('rcx'), reg('rsp')
        assert (frame == 0) == (count == 0)
        current = dict(control=reg('rdi'), owner=reg('rsi'), epoch=reg('rdx'),
                       frame=frame, values=words(frame, count), stack=stack,
                       saved=[reg(name) for name in preserved], vectors=vectors(), entered=False)
        Returned('*' + hex(words(stack, 1)[0]), internal=True)
        return False


class Ordinary(gdb.Breakpoint):
    def stop(self):
        assert current is not None and not current['entered']
        frame = gdb.newest_frame()
        values = tuple(natural(frame.read_var('fact' + str(i)))
                       for i in range(record['scalar_count']))
        assert values == current['values'], (values, current['values'])
        assert 0 <= current['stack'] - reg('rsp') <= record['maximum_stack_bytes']
        view = int(frame.read_var('region'))
        arena, cursor = address('arena'), int(gdb.parse_and_eval('cursor'))
        assert arena + 32 <= view and view + 64 <= arena + cursor
        control = words(current['control'], 8)
        assert control[0] == address(record['native_export']) and control[3] == 1
        assert control[1:3] == (current['owner'], current['epoch'])
        assert words(view, 8) == (current['control'], 0, 16, 1, current['epoch'],
                                 current['owner'], control[0], 0)
        current['entered'] = True
        return False


gdb.execute('set pagination off')
gdb.execute('set confirm off')
gdb.execute('set detach-on-fork on')
gdb.execute('set follow-fork-mode parent')
entering = Enter('*' + record['native_export'], internal=True)
entry = next(f['symbol'] for f in record['functions'] if '.fn.apply.' in f['symbol'])
ordinary = Ordinary(entry, internal=True)
gdb.execute('run')
assert gdb.selected_inferior().pid == 0 and int(gdb.parse_and_eval('$_exitcode')) == 0
assert completed == 48 and current is None
print('FRAME_DEBUG_PASS words=%d calls=%d source-values/gpr/simd/stack/return' %
      (record['scalar_count'], completed))
