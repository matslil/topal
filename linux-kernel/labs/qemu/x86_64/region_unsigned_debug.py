"""Independent source/ABI, payload-access and physical erasure observations."""
import json
from pathlib import Path
import re
import struct
import gdb

config = json.loads(Path(gdb.parse_and_eval('$unsigned_config').string()).read_text())
record = config['record']
current = None
retired = None
completed = erased = reads = writes = helpers = 0
preserved = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')


def reg(name):
    return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)


def address(name):
    return int(gdb.parse_and_eval("&'" + name + "'"))


def memory(start, size):
    return bytes(gdb.selected_inferior().read_memory(start, size))


def words(start, count):
    return struct.unpack('<' + 'Q' * count, memory(start, count * 8))


def vectors():
    return [[int(gdb.parse_and_eval('$xmm%d.v2_int64[%d]' % (i, j)))
             for j in range(2)] for i in range(16)]


def natural(pointer):
    start = int(pointer)
    arena, cursor = address('arena'), int(gdb.parse_and_eval('cursor'))
    extents = [(arena + 32, arena + cursor)]
    extents += [(d['address'], d['address'] + d['bytes']) for d in config['data']]
    assert any(low <= start and start + 16 <= high for low, high in extents)
    sign, count = words(start, 2)
    assert sign == 0 and count <= 8, (sign, count)
    assert any(low <= start and start + 16 + count * 4 <= high for low, high in extents)
    return int.from_bytes(memory(start + 16, count * 4), 'little') if count else 0


class Payload(gdb.Breakpoint):
    def __init__(self, base, operation):
        # One aligned watch covers the full eight-byte field; it is enabled
        # only across the actual source invocation, before C checks/cleanup.
        super().__init__('*(unsigned long long*)' + hex(base), type=gdb.BP_WATCHPOINT,
                         wp_class=gdb.WP_ACCESS, internal=True)
        self.operation = operation

    def stop(self):
        global reads, writes
        instruction = instructions[self.operation].get(reg('rip'))
        assert instruction is not None, ('unexpected payload access', hex(reg('rip')))
        if self.operation == 'store':
            assert re.match(r'mov\s+BYTE PTR \[.+\],', instruction), instruction
            writes += 1
        else:
            assert re.match(r'movzx\s+\w+,\s*BYTE PTR \[.+\]', instruction), instruction
            reads += 1
        return False


class Returned(gdb.Breakpoint):
    def stop(self):
        global current, retired, completed
        self.enabled = False
        assert current is not None and current['entered']
        assert reg('rax') == (1 if completed < 2 else 0)
        assert reg('rsp') == current['stack'] + 8
        assert [reg(name) for name in preserved] == current['saved']
        assert vectors() == current['vectors']
        assert words(current['control'], 8) == (0, 0, current['epoch'], 0, 0, 0, 0, 0)
        assert words(current['frame'], 3) == current['values']
        if current.get('watch'):
            current['watch'].delete()
        retired = current
        current = None
        completed += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        global current
        assert current is None and reg('r8') == 3
        frame, stack = reg('rcx'), reg('rsp')
        current = dict(control=reg('rdi'), owner=reg('rsi'), epoch=reg('rdx'),
                       frame=frame, values=words(frame, 3), stack=stack,
                       saved=[reg(name) for name in preserved], vectors=vectors(), entered=False)
        if completed < 2:
            base = words(current['control'], 8)[4]
            assert base % 8 == 0
            current['watch'] = Payload(base, 'store' if completed == 0 else 'load')
        Returned('*' + hex(words(stack, 1)[0]), internal=True)
        return False


class Ordinary(gdb.Breakpoint):
    def __init__(self, symbol, role):
        super().__init__(symbol, internal=True)
        self.role = role

    def stop(self):
        global helpers
        assert current is not None
        frame = gdb.newest_frame()
        offset, value, command = current['values']
        if self.role == 'entry':
            assert not current['entered']
            actual = tuple(natural(frame.read_var(name)) for name in ('offset', 'value', 'command'))
            assert actual == current['values'], (actual, current['values'])
            current['entered'] = True
        else:
            assert current['entered']
            if command == 3:
                value += 1 << 64
            elif command == 4:
                offset += 1 << 64
            elif command == 5:
                value += 1 << 255
            elif command == 6:
                offset += 1 << 255
            assert natural(frame.read_var('offset')) == offset
            assert natural(frame.read_var('value' if self.role == 'write' else 'expected')) == value
            helpers += 1
        assert 0 <= current['stack'] - reg('rsp') <= record['maximum_stack_bytes']
        view = int(frame.read_var('region'))
        control = words(current['control'], 8)
        assert control[0] == address(record['native_export']) and control[3] == 1
        assert control[1:3] == (current['owner'], current['epoch'])
        fields = words(view, 8)
        assert fields[0] == current['control'] and fields[4:7] == (
            current['epoch'], current['owner'], control[0]) and fields[7] == 0
        assert fields[1] <= control[5] and fields[2] <= control[5] - fields[1]
        assert fields[3] <= control[6]
        if self.role == 'write' and command == 2:
            assert fields[2:4] == (8, 0)
        return False


class Erased(gdb.Breakpoint):
    def stop(self):
        global retired, erased
        assert retired is not None and current is None
        assert not any(memory(address('arena'), 65536))
        assert not any(memory(retired['frame'], 24))
        assert int(gdb.parse_and_eval('active')) == 0
        retired = None
        erased += 1
        return False


gdb.execute('set pagination off')
gdb.execute('set confirm off')
gdb.execute('set disassembly-flavor intel')
gdb.execute('starti debug')
instructions = {}
for operation in ('store', 'load'):
    symbol = next(f['symbol'] for f in record['functions']
                  if f['symbol'].endswith('.primitive.unsigned.' + operation))
    decoded = gdb.selected_frame().architecture().disassemble(address(symbol), count=256)
    instructions[operation] = {ins['addr'] + ins['length']: ins['asm'] for ins in decoded}
Enter('*' + record['native_export'], internal=True)
for spelling, role in [('apply', 'entry'), ('write_2dfield', 'write'), ('read_2dfield', 'read')]:
    symbols = [f['symbol'] for f in record['functions'] if '.fn.' + spelling + '.' in f['symbol']]
    assert symbols, spelling
    for symbol in symbols:
        Ordinary(symbol, role)
Erased('erased_checkpoint', internal=True)
gdb.execute('continue')
assert gdb.selected_inferior().pid == 0 and int(gdb.parse_and_eval('$_exitcode')) == 0
assert current is None and retired is None and completed == erased == 30, (completed, erased)
assert helpers == 30 and reads == writes == 8, (helpers, reads, writes)
print('UNSIGNED_DEBUG_PASS calls=30 erased=30 payload-loads=8 payload-stores=8 source-values/gpr/simd/stack/return')
