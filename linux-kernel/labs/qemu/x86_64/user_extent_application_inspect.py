"""Observe transferred signal ownership during an unchanged BusyBox exit.

TOPAL-SYSTEMS-USER-EXTENT-001. Breakpoints observe transferred signal ownership and
terminal retirement; the observer never supplies guest registers or data.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

import frame_root_inspect as harness
import topal_gate


def invariant_halt(elf):
    lines = subprocess.check_output([
        'gdb', '-nx', '-batch', '-ex', f'file {elf}',
        '-ex', 'disassemble /r topal_x86_systems_static_image_process_enter',
    ], text=True).splitlines()
    points = [
        int(re.search(r'0x([0-9a-f]+)\s+<', line)[1], 16)
        for index, line in enumerate(lines[:-1])
        if re.search(r'\bcli\s*$', line) and re.search(r'\bhlt\s*$', lines[index + 1])
    ]
    assert len(points) == 1, 'expected one private invariant-failure entry'
    exits = [
        int(re.search(r'0x([0-9a-f]+)\s+<', line)[1], 16)
        for line in lines if re.search(r'\bmov\s+0x9f10\(%r8\),%rax', line)
    ]
    assert len(exits) == 2, 'expected both private exit-command checks'
    return dict(halt=points[0], exits=exits)


def members(raw):
    at = 0
    result = {}
    while True:
        assert raw[at:at + 6] == b'070701'
        length = int(raw[at + 54:at + 62], 16)
        size = int(raw[at + 94:at + 102], 16)
        name = raw[at + 110:at + 110 + size - 1].decode()
        at = (at + 110 + size + 3) & -4
        if name == 'TRAILER!!!':
            assert length == 0 and not any(raw[at:])
            return result
        assert name not in result
        result[name] = raw[at:at + length]
        at = (at + length + 3) & -4


PROGRAM = r'''

import gdb
import json
import struct
import traceback

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(pointer, size): return bytes(gdb.selected_inferior().read_memory(pointer, size))
def word(pointer): return struct.unpack('<Q', memory(pointer, 8))[0]
def words(pointer, count): return list(struct.unpack('<'+'Q'*count, memory(pointer, count*8)))

context = address('topal_bootstrap_storage') + 512
epoch = address('topal_x86_frame_mm_epoch_v1')
observed = dict(passed=False)

class CheckedBreakpoint(gdb.Breakpoint):
    def stop(self):
        try:
            return self.observe()
        except Exception:
            traceback.print_exc()
            observed['observer_failure'] = traceback.format_exc()
            with open(observation_path, 'w') as handle:
                json.dump(observed, handle, indent=2)
            return True

def consumed_prelude():
    assert all(word(context + offset) == 0 for offset in [168, 176, 224, 232])
    assert memory(context + 417, 1) == bytes(1)

class SerialReturned(CheckedBreakpoint):
    def observe(self):
        self.enabled = False
        assert reg('cs') & 3 == 3 and reg('rax') == 5
        assert memory(context + 104, 1) == bytes([32])
        assert memory(context + 418, 1) == bytes([6])
        snapshot = observed['snapshot']
        assert word(snapshot + 0x9f10) == 5 and word(snapshot + 0x9f08) == 0
        consumed_prelude()
        table = observed['table']
        assert word(table) == 1
        child = words(table + 1792 + 16*32, 4)
        quit_action = words(table + 1792 + 2*32, 4)
        expected = observed['prelude_child'][:]
        expected[3] &= ~0x40100
        assert child == expected, (child, expected)
        assert quit_action == [1, 0x4000000, 0x4a9740, expected[3]], quit_action
        observed.update(imported_child=child, imported_quit=quit_action)
        observed['serial_delivered'] = True
        return False

class SerialInput(CheckedBreakpoint):
    def observe(self):
        if (word(epoch + 8) != 0 or memory(context + 104, 1) != bytes([31])
                or word(context + 312) != 0):
            return False
        table = word(context + 928)
        observed['serial_admission'] = dict(owner=memory(context + 418, 1)[0],
            count=reg('r9'), child=words(table + 1792 + 16*32, 4),
            quit_action=words(table + 1792 + 2*32, 4))
        assert 'table' not in observed and memory(context + 418, 1) == bytes([5])
        assert reg('cs') & 3 == 0 and reg('cr0') & (1 << 16) and reg('r9') == 5
        table = word(context + 928)
        assert table and memory(table, 8192) == bytes(8192)
        assert memory(context + 417, 1) == bytes([1])
        child = [word(context + offset) for offset in [168, 176, 224, 232]]
        assert 0x401000 <= child[0] < 0x58c285 and child[1] == 0x4000000
        assert child[2] == 0x4a9740
        assert child[3] == 0xfffffffe7fffffff
        observed.update(table=table, snapshot=word(context + 904),
                        prelude_child=child)
        SerialReturned('*'+hex(word(context + 296)), internal=True)
        return False

class ExitSelected(CheckedBreakpoint):
    def observe(self):
        assert observed['serial_delivered'] and word(epoch + 8) == 231
        assert word(context + 312) == 0 and word(context + 928) == observed['table']
        assert word(observed['table']) == 1
        consumed_prelude()
        observed['terminal_child'] = words(observed['table'] + 1792 + 16*32, 4)
        observed['terminal_quit'] = words(observed['table'] + 1792 + 2*32, 4)
        observed['exit_selected'] = True
        return False

class Complete(CheckedBreakpoint):
    def observe(self):
        assert observed['exit_selected']
        assert memory(observed['snapshot'] + 0x9e00, 512) == bytes(512)
        assert memory(observed['table'], 8192) == bytes(8192)
        assert word(context + 904) == 0
        assert memory(context + 152, 16) == bytes(16)
        assert memory(context + 416, 8) == bytes(8)
        assert memory(address('topal_x86_frame_mm_state_v1'), 32) == bytes(32)
        assert memory(address('topal_x86_frame_mm_tokens_v1'), 48) == bytes(48)
        observed['passed'] = True
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

class InvariantHalt(CheckedBreakpoint):
    def observe(self):
        observed['invariant_failure'] = dict(
            syscall=word(epoch + 8), phase=memory(context + 104, 1)[0],
            owner=memory(context + 418, 1)[0],
            signal_state=[word(context + offset) for offset in [168, 176, 224, 232]],
        )
        print('OWNED_EXIT_INVARIANT_FAILURE', observed['invariant_failure'], flush=True)
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

try:
    SerialInput('*topal_x86_user_extent_writable_v1', internal=True)
    Complete('*topal_x86_systems_fatal', internal=True)
    InvariantHalt('*'+hex(native_sites['halt']), internal=True)
    for site in native_sites['exits']:
        ExitSelected('*'+hex(site), internal=True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    gdb.execute('quit 1')
'''


if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    image = (build / 'static-elf-abi-fixture.elf').read_bytes()
    assert hashlib.sha256(image).hexdigest() == topal_gate.PINNED_STATIC_BUSYBOX_SHA256
    archive = members((build / 'initramfs.cpio').read_bytes())
    assert set(archive) == {'init', 'static-user.elf'} and archive['static-user.elf'] == image
    harness.GDB_PROGRAM = PROGRAM
    harness.main(work_name='first-image-exit', process_command=b'exit\n',
                 program_prefix=f'native_sites={invariant_halt(build / "kernel.elf")!r}\n',
                 input_scope='unchanged pinned BusyBox first-image exit', guest_timeout_seconds=180)
    serial = (build / 'first-image-exit/serial.log').read_bytes()
    assert topal_gate.EXEC_COMMIT_MARKER not in serial
    marker = b'TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK'
    following = b'TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK'
    assert serial.count(marker) == 1 and serial.count(following) == 1
    application = serial.split(marker, 1)[1].split(following, 1)[0]
    assert application == (
        b'topal\n/bin\n/bin/busybox\nbusybox-file\nbusybox-executable\n'
        b'busybox-nonempty\nbusybox-open\nbusybox-read\nbusybox-close\n/\n'
    )
