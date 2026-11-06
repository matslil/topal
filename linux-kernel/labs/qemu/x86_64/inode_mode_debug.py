"""Independent source/ABI/actual-field observations for an offline component.

The C caller erases its arena and frame. Kernel retired-stack/owner reclamation
remains a separate machine qualification and is not inferred from this script.
"""
import json
from pathlib import Path
import re
import struct
import gdb

config = json.loads(Path(gdb.parse_and_eval('$inode_config').string()).read_text())
record = config['record']
current = retired = None
completed = erased = reads = writes = 0
roles = {}
returns = {'prefix': 0, 'mode-true': 0, 'mode-false': 0, 'time-true': 0}
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
    extents += [(data['address'], data['address'] + data['bytes']) for data in config['data']]
    assert any(low <= start and start + 16 <= high for low, high in extents)
    sign, count = words(start, 2)
    assert sign == 0 and count <= 8, (sign, count)
    assert any(low <= start and start + 16 + count * 4 <= high for low, high in extents)
    return int.from_bytes(memory(start + 16, count * 4), 'little') if count else 0


class Payload(gdb.Breakpoint):
    def __init__(self, base, field):
        super().__init__('*(unsigned long long*)' + hex(base + field * 8),
                         type=gdb.BP_WATCHPOINT, wp_class=gdb.WP_ACCESS, internal=True)
        self.field = field

    def stop(self):
        global reads, writes
        assert self.field in (0, 3), ('unexpected access to preserved timestamp', self.field)
        instruction = instructions.get(reg('rip'))
        assert instruction is not None, ('unexpected owner access', hex(reg('rip')))
        if re.match(r'mov\s+BYTE PTR \[.+\],', instruction):
            assert current['expected'] == 1
            writes += 1
        else:
            assert self.field == 0
            assert re.match(r'movzx\s+\w+,\s*BYTE PTR \[.+\]', instruction), instruction
            reads += 1
        return False


class Returned(gdb.Breakpoint):
    def stop(self):
        global current, retired, completed
        self.enabled = False
        assert current is not None and current['entered']
        assert reg('rax') == current['expected']
        assert reg('rsp') == current['stack'] + 8
        assert [reg(name) for name in preserved] == current['saved']
        assert vectors() == current['vectors']
        assert words(current['control'], 8) == (0, 0, current['epoch'], 0, 0, 0, 0, 0)
        assert words(current['frame'], 2) == current['values']
        for watch in current['watches']:
            watch.delete()
        if current['before'] is not None:
            wanted = list(current['before'])
            if current['expected']:
                wanted[0] = (wanted[0] & ~0o7777) | current['values'][0]
                wanted[3] = current['values'][1]
            assert words(current['base'], 4) == tuple(wanted)
        retired = current
        current = None
        completed += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        global current
        assert current is None and reg('r8') == 2
        frame, stack = reg('rcx'), reg('rsp')
        control = words(reg('rdi'), 8)
        accessible = completed < 5 or completed == 8
        current = dict(control=reg('rdi'), owner=reg('rsi'), epoch=reg('rdx'),
                       frame=frame, values=words(frame, 2), stack=stack,
                       saved=[reg(name) for name in preserved], vectors=vectors(),
                       expected=int(gdb.parse_and_eval('debug_expected')),
                       base=control[4], before=words(control[4], 4) if accessible else None,
                       entered=False, watches=[])
        if accessible:
            assert current['base'] % 8 == 0
            current['watches'] = [Payload(current['base'], i) for i in range(4)]
        Returned('*' + hex(words(stack, 1)[0]), internal=True)
        return False


class HelperReturn(gdb.FinishBreakpoint):
    def __init__(self, role):
        super().__init__(gdb.newest_frame(), internal=True)
        self.role = role

    def stop(self):
        if self.role == 'prefix':
            expected = current['before'][0] // 4096 * 4096
            assert natural(reg('rax')) == expected
            returns['prefix'] += 1
        elif self.role == 'mode':
            # Internal fastcc returns LLVM i1; only its low bit is defined.
            result = reg('rax') & 1
            assert result == current['expected']
            returns['mode-true' if result else 'mode-false'] += 1
        else:
            assert reg('rax') & 1 == 1
            returns['time-true'] += 1
        return False


class Ordinary(gdb.Breakpoint):
    def __init__(self, symbol, role):
        super().__init__(symbol, internal=True)
        self.role = role

    def stop(self):
        assert current is not None
        roles[self.role] = roles.get(self.role, 0) + 1
        frame = gdb.newest_frame()
        permissions, seconds = current['values']
        if self.role == 'entry':
            assert not current['entered']
            current['entered'] = True
        else:
            assert current['entered']
        if self.role in ('entry', 'import', 'read', 'preserve'):
            assert natural(frame.read_var('permissions')) == permissions
            assert natural(frame.read_var('seconds')) == seconds
        if self.role in ('prefix', 'mode'):
            value = natural(frame.read_var('mode'))
            expected = current['before'][0]
            if self.role == 'mode':
                expected = expected // 4096 * 4096 + permissions
                assert natural(frame.read_var('seconds')) == seconds
            assert value == expected
            HelperReturn(self.role)
        if self.role == 'preserve':
            assert natural(frame.read_var('old-mode')) == current['before'][0]
        if self.role == 'time':
            assert natural(frame.read_var('seconds')) == seconds
            HelperReturn(self.role)
        assert 0 <= current['stack'] - reg('rsp') <= record['maximum_stack_bytes']
        if self.role != 'prefix':
            view = int(frame.read_var('region'))
            control = words(current['control'], 8)
            assert control[0] == address(record['native_export']) and control[3] == 1
            assert control[1:3] == (current['owner'], current['epoch'])
            fields = words(view, 8)
            assert fields[0] == current['control'] and fields[4:7] == (
                current['epoch'], current['owner'], control[0]) and fields[7] == 0
            assert fields[1:4] == (0, control[5], control[6])
        return False


class Erased(gdb.Breakpoint):
    def stop(self):
        global retired, erased
        assert retired is not None and current is None
        assert not any(memory(address('arena'), 65536))
        assert not any(memory(retired['frame'], 16))
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
    symbol = next(function['symbol'] for function in record['functions']
                  if function['symbol'].endswith('.primitive.unsigned.' + operation))
    decoded = gdb.selected_frame().architecture().disassemble(address(symbol), count=256)
    instructions.update({item['addr'] + item['length']: item['asm'] for item in decoded})
Enter('*' + record['native_export'], internal=True)
for spelling, role in [('apply', 'entry'), ('support_2einode_2dmetadata_2eapply_2dmode', 'import'),
                       ('support_2einode_2dmetadata_2eread_2dmode', 'read'),
                       ('support_2einode_2dmetadata_2epreserve_2dtype', 'preserve'),
                       ('support_2einode_2dmetadata_2etype_2dprefix', 'prefix'),
                       ('support_2einode_2dmetadata_2ewrite_2dmode_2dand_2dtime', 'mode'),
                       ('support_2einode_2dmetadata_2ewrite_2dtime', 'time')]:
    symbols = [function['symbol'] for function in record['functions']
               if '.fn.' + spelling + '.' in function['symbol']]
    assert symbols, spelling
    for symbol in symbols:
        Ordinary(symbol, role)
Erased('erased_checkpoint', internal=True)
gdb.execute('continue')
assert gdb.selected_inferior().pid == 0 and int(gdb.parse_and_eval('$_exitcode')) == 0
assert current is None and retired is None and completed == erased == 9
assert roles == {'entry': 9, 'import': 9, 'read': 6, 'preserve': 6, 'prefix': 6, 'mode': 5, 'time': 4}, roles
assert returns == {'prefix': 6, 'mode-true': 4, 'mode-false': 1, 'time-true': 4}, returns
assert reads == 48 and writes == 64, (reads, writes)
print('INODE_DEBUG_PASS calls=9 erased=9 payload-loads=48 payload-stores=64 source-facts/type/fields/gpr/simd/stack/return')
