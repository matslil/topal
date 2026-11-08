"""Unpublished CPL0 component experiment; does not mint artifact provenance."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import pty
import socket
import subprocess
import time
import topal_gate


GDB_PROGRAM = r'''
import gdb, json, traceback

preserved = ['rbx', 'rbp', 'r12', 'r13', 'r14', 'r15']
volatile = ['rax', 'rcx', 'rdx', 'rsi', 'rdi', 'r8', 'r9', 'r10', 'r11']
counts = {'entry': 0, 'source': 0, 'checkpoint': 0, 'complete': 0, 'return': 0, 'owner_resume': 0, 'finalize':0, 'final':0, 'fatal':0, 'refused':0}
errors, pending, traces = [], {}, []
walk = None
walked = {'apply':0, 'return':0, 'fatal':0}

def value(expression):
    return int(gdb.parse_and_eval(expression))

def address(name):
    return value('(unsigned long long)&' + name)

def memory(start, size):
    return bytes(gdb.selected_inferior().read_memory(start, size))

def zeros(start, size):
    assert memory(start, size) == bytes(size), (hex(start), size)

def word(start, offset=0):
    return int.from_bytes(memory(start + offset, 8), 'little')

def put(start, n):
    gdb.selected_inferior().write_memory(start, int(n).to_bytes(8, 'little'))

def history():
    start = address('topal_x86_tail_kernel_history_v1')
    return [word(start, i) for i in range(0,40,8)]

def caller_registers(frame):
    assert frame is not None
    for register in preserved:
        assert int(frame.read_register(register)) == pending['registers'][register], register

def privileged():
    assert value('$cs') & 3 == 0
    assert value('$eflags') & 0x200 == 0

def clean_domains(expected_active=0):
    for name, size in [('private', 65528), ('stack', 4096), ('output', 56), ('recovery', 32)]:
        zeros(address('topal_x86_tail_kernel_' + name + '_v1'), size)
    assert word(address('topal_x86_tail_boundary_active_v1')) == expected_active

class Probe(gdb.Breakpoint):
    def __init__(self, name, role):
        super().__init__('*' + name)
        self.role = role

    def stop(self):
        global walk
        try:
            privileged()
            role = self.role
            counts[role] += 1
            if role == 'entry':
                assert counts[role] == 1
                pending.update(sp=value('$rsp'), owner=value('$rdi'), key=value('$rdx'),
                    registers={r:value('$' + r) for r in preserved})
                assert value('$rsi') == 64
                assert pending['key'] == pending['owner'] - address('topal_bootstrap_storage') + 1
                assert memory(pending['owner'], 64) == bytes([90]) + bytes(63)
                domains = [(pending['owner'],64), (address('topal_bootstrap_storage'),4096)]
                for name, size in [('private',65528),('stack',4096),('history',40),('output',56),('recovery',32)]:
                    domains.append((address('topal_x86_tail_kernel_'+name+'_v1'),size))
                domains += [(address('topal_x86_tail_boundary_active_v1'),8),
                    (0x108000,0x78000), (0x400000,0x400000)]
                for i, (start, size) in enumerate(domains):
                    for other, length in domains[i+1:]:
                        assert start+size <= other or other+length <= start, (start,size,other,length)
                pending['domains'] = domains
                clean_domains()
                zeros(address('topal_x86_tail_kernel_history_v1'), 40)
                hist = address('topal_x86_tail_kernel_history_v1')
                if mode == 'bad_key':
                    gdb.execute('set $rdx = $rdx + 1')
                elif mode == 'busy':
                    put(address('topal_x86_tail_boundary_active_v1'), 85)
                elif mode == 'interrupts':
                    gdb.execute('set $eflags = $eflags | 512')
                elif mode == 'bad_stack':
                    gdb.execute('set $rsp = 0x200000')
                elif mode == 'generation_exhaustion':
                    for offset, n in [(0,2),(8,2),(16,2**64-1)]: put(hist+offset, n)
                elif mode == 'frame_exhaustion':
                    for offset, n in [(0,2**64-1),(8,2),(16,1)]: put(hist+offset, n)
                elif mode == 'dirty_private':
                    put(address('topal_x86_tail_kernel_private_v1'), 1)
                pending['history_before'] = history()
                if mode == 'normal':
                    walk = 'apply'
                    return True
            elif role in ['source', 'checkpoint', 'complete']:
                stack = address('topal_x86_tail_kernel_stack_v1')
                assert stack <= value('$rsp') < stack + 4096
                private = address('topal_x86_tail_kernel_private_v1')
                assert value('$rdi') == private + 64
                assert word(address('topal_x86_tail_boundary_active_v1')) == private
                if counts[role] == 1:
                    trace = gdb.execute('bt', to_string=True)
                    assert 'topal_tail_owner_invoke_v1' in trace, trace
                    assert 'topal_x86_systems_region_library_apply_v1' in trace, trace
                    frame = gdb.newest_frame()
                    while frame and frame.name() != 'topal_x86_systems_region_library_apply_v1':
                        frame = frame.older()
                    assert frame is not None and frame.older() is not None, trace
                    caller_registers(frame.older())
                    traces.append({'role':role, 'trace':trace})
            elif role == 'return':
                assert value('$rsp') == pending['sp']
                for register in preserved:
                    assert value('$' + register) == pending['registers'][register], register
                for register in volatile:
                    assert value('$' + register) == 0, register
                clean_domains()
                # Six original saved-register copies plus the actual argument /
                # alignment area and complete compiled controller call paths.
                zeros(pending['sp'] - 48 - retired_bytes, 48 + retired_bytes)
                assert memory(pending['owner'], 64) == bytes(64)
                assert history() == [194,194,1,0,0], history()
            elif role == 'owner_resume':
                assert mode == 'normal', mode
                clean_domains()
                assert memory(pending['owner'], 64) == bytes(64)
                assert counts == dict(entry=1, source=65, checkpoint=65, complete=1, **{'return':1}, owner_resume=1, finalize=0, final=0, fatal=0, refused=0), counts
            elif role == 'finalize':
                assert mode == 'normal' and counts['owner_resume'] == 1
                assert history() == [194,194,1,0,0], history()
                clean_domains()
            elif role == 'final':
                assert mode == 'normal' and counts['finalize'] == 1, counts
                assert history() == [0,0,0,0,0], history()
                clean_domains()
                return True
            elif role == 'refused':
                assert mode in ['bad_key', 'busy', 'interrupts', 'bad_stack'], mode
                assert counts['source'] == counts['owner_resume'] == 0, counts
                for register in preserved:
                    assert value('$'+register) == pending['registers'][register], register
                assert history() == pending['history_before']
                assert memory(pending['owner'],64) == bytes([90])+bytes(63)
                if mode == 'busy':
                    assert word(address('topal_x86_tail_boundary_active_v1')) == 85
                clean_domains(expected_active=85 if mode == 'busy' else 0)
                return True
            elif role == 'fatal':
                assert mode in ['source_abort','false_result','generation_exhaustion','frame_exhaustion','dirty_private'], mode
                assert counts['owner_resume'] == counts['return'] == 0
                assert value('$rsp') == pending['sp']
                for register in preserved:
                    assert value('$'+register) == pending['registers'][register], register
                for register in volatile:
                    assert value('$'+register) == 0, register
                clean_domains()
                zeros(pending['sp'] - 48 - retired_bytes, 48 + retired_bytes)
                if mode == 'source_abort':
                    assert history() == [4,4,1,0,1], history()
                    assert counts['source'] == 1 and counts['checkpoint'] == 0
                    assert memory(pending['owner'],64) == bytes(64)
                elif mode == 'false_result':
                    assert history() == [194,194,1,0,1], history()
                    assert counts['source'] == counts['checkpoint'] == 65 and counts['complete'] == 1
                    assert memory(pending['owner'],64) == bytes(64)
                else:
                    assert counts['source'] == counts['checkpoint'] == counts['complete'] == 0
                    assert history() == pending['history_before']
                    assert memory(pending['owner'],64) == bytes([90])+bytes(63)
                trace = gdb.execute('bt', to_string=True)
                traces.append({'role':'fatal-reclaimed', 'trace':trace})
                return True
        except Exception:
            errors.append(traceback.format_exc())
            return True
        return False

class Fault(gdb.Breakpoint):
    def stop(self):
        try:
            if mode == 'source_abort':
                # First staging hook follows the real first source store.
                assert memory(pending['owner'],64) == bytes(64)
                gdb.execute('set $rdx = 1')
            else:
                gdb.execute('set $rsi = 0')
            self.enabled = False
        except Exception:
            errors.append(traceback.format_exc())
            return True
        return False

class Walk(gdb.Breakpoint):
    def __init__(self, name, role):
        super().__init__('*'+name)
        self.role = role
    def stop(self):
        global walk
        walk = self.role
        self.enabled = False
        return True

try:
    Probe('topal_x86_systems_region_library_apply_v1', 'entry')
    Probe('topal_tail_step_component_v1', 'source')
    Probe('topal_tail_checkpoint_body_v1', 'checkpoint')
    Probe('topal_tail_complete_body_v1', 'complete')
    Probe('topal_x86_tail_kernel_return_reclaimed_v1', 'return')
    Probe('topal_x86_systems_atomic_word_create', 'owner_resume')
    Probe('topal_x86_systems_region_library_finalize_v1', 'finalize')
    Probe('topal_x86_systems_fatal', 'final')
    Probe('topal_x86_tail_kernel_refused_v1', 'refused')
    Probe('topal_x86_tail_kernel_fatal_reclaimed_v1', 'fatal')
    if mode in ['source_abort', 'false_result']:
        Fault('*'+('topal_tail_stage_number_v1' if mode == 'source_abort' else 'topal_tail_result_boolean_v1'))
    if mode == 'normal':
        Walk('topal_x86_tail_kernel_owner_returned_v1', 'return')
    if mode == 'source_abort':
        Walk('topal_tail_boundary_fatal_v1', 'fatal')
    gdb.execute('continue')
    while walk is not None and not errors:
        role, walk = walk, None
        steps = 0
        target = address('topal_tail_owner_invoke_v1' if role=='apply' else
            'topal_x86_tail_kernel_return_reclaimed_v1' if role=='return' else
            'topal_x86_tail_kernel_fatal_reclaimed_v1')
        while value('$pc') != target:
            assert steps < 250, (role, steps)
            frame = gdb.newest_frame().older()
            # Fatal recovery deliberately has no return chain until it pivots
            # to the independently pinned outer caller's saved-register frame.
            if role != 'fatal' or frame is not None:
                caller_registers(frame)
            instructions = gdb.selected_frame().architecture().disassemble(value('$pc'), count=2)
            if instructions[0]['asm'].startswith('rep'):
                # Repeated string operations leave the CFA and preserved
                # registers unchanged. Observe each instruction's two ends;
                # whole-domain readback is checked separately above.
                gdb.Breakpoint('*'+str(instructions[1]['addr']), temporary=True, internal=True)
                gdb.execute('continue', to_string=True)
            else:
                gdb.execute('stepi', to_string=True)
            steps += 1
        walked[role] = steps
        if role == 'apply':
            assert value('$rdi') == pending['owner'] and value('$rsi') == 64
            pointer = value('$rdx')
            assert memory(pointer,24) == b''.join(n.to_bytes(8,'little') for n in [64,0,1])
            assert pending['sp']-48-retired_bytes <= pointer < pending['sp']-48
            for register, symbol in [('rcx','history'),('r8','private'),('r9','stack')]:
                assert value('$'+register) == address('topal_x86_tail_kernel_'+symbol+'_v1')
            assert word(value('$rsp')+8) == address('topal_x86_tail_kernel_output_v1')
        if counts['fatal'] or counts['refused'] or counts['final']:
            break
        gdb.execute('continue')
    assert not errors, errors
    assert counts['final'] + counts['fatal'] + counts['refused'] == 1, counts
    with open(observation_path, 'w') as output:
        json.dump({'scope':'unpublished privileged native component experiment',
            'mode':mode, 'counts':counts, 'caller':pending, 'traces':traces, 'walked_instructions':walked,
            'retired_controller_bytes':retired_bytes}, output, indent=2)
    print('TAIL_KERNEL_GDB_PASS', counts)
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    gdb.execute('quit 1')
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('work', type=Path)
    parser.add_argument('--retired-bytes', type=int, required=True)
    parser.add_argument('--mode', default='normal', choices=['normal','source_abort','false_result',
        'bad_key','busy','interrupts','bad_stack','generation_exhaustion','frame_exhaustion','dirty_private'])
    args = parser.parse_args()
    build = args.work.resolve()
    work = build / args.mode
    work.mkdir(exist_ok=True)
    with socket.socket() as port_socket:
        port_socket.bind(('127.0.0.1', 0))
        port = port_socket.getsockname()[1]
    master_fd, slave_fd = pty.openpty()
    os.set_blocking(master_fd, False)
    commands = topal_gate.qemu_command(build / 'bzImage', build / 'initramfs.cpio',
        work / 'serial.log', os.ttyname(slave_fd))
    commands += ['-S', '-gdb', f'tcp:127.0.0.1:{port}']
    (work / 'qemu.command.json').write_text(json.dumps(commands, indent=2))
    script = work / 'guest-inspect.gdb'
    script.write_text('set pagination off\nset confirm off\nset remotetimeout 10\n'
        f'file {json.dumps(str(build / "kernel.elf"))}\n'
        f'target remote 127.0.0.1:{port}\npython\n'
        f'retired_bytes = {args.retired_bytes}\n'
        f'mode = {args.mode!r}\n'
        f'observation_path = {str(work / "guest-observations.json")!r}\n'
        + GDB_PROGRAM + '\nend\n')
    with (work / 'qemu.stdout.log').open('wb') as out, (work / 'qemu.stderr.log').open('wb') as err:
        process = subprocess.Popen(commands, stdout=out, stderr=err)
        debugger = None
        try:
            deadline = time.monotonic() + 10
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f'QEMU exited {process.returncode}')
                try:
                    with socket.create_connection(('127.0.0.1', port), timeout=.1):
                        break
                except OSError:
                    if time.monotonic() >= deadline:
                        raise
                    time.sleep(.05)
            debugger = subprocess.Popen(['gdb', '-nx', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            received, sent_read, sent_command = bytearray(), False, False
            deadline = time.monotonic() + 45
            while debugger.poll() is None:
                assert time.monotonic() < deadline, 'guest debugger timed out'
                try:
                    block = os.read(master_fd, 65536)
                    received.extend(block)
                except (BlockingIOError, OSError):
                    pass
                if not sent_read and b'TOPAL_KERNEL_BOOT_RESOURCE_OK' in received:
                    os.write(master_fd, topal_gate.SERIAL_INPUT)
                    sent_read = True
                if not sent_command and b'TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK' in received:
                    os.write(master_fd, topal_gate.PROCESS_COMMAND)
                    sent_command = True
                (work / 'serial.log').write_bytes(received)
                time.sleep(.03)
            stdout, stderr = debugger.communicate(timeout=5)
            (work / 'gdb.stdout.log').write_bytes(stdout)
            (work / 'gdb.stderr.log').write_bytes(stderr)
            assert debugger.returncode == 0, stdout.decode(errors='replace') + stderr.decode(errors='replace')
            assert b'TAIL_KERNEL_GDB_PASS' in stdout
            serial = received.decode(errors='replace')
            assert 'TOPAL_KERNEL_BOOT' in serial, serial
        finally:
            if debugger is not None and debugger.poll() is None:
                debugger.kill()
                stdout, stderr = debugger.communicate(timeout=5)
                (work / 'gdb.stdout.log').write_bytes(stdout)
                (work / 'gdb.stderr.log').write_bytes(stderr)
            process.kill()
            process.wait(timeout=5)
            os.close(master_fd)
            os.close(slave_fd)
    print('TAIL_KERNEL_PASS: actual CPL0/IF0 owner, source/checkpoint/completion, caller/private retirement')


if __name__ == '__main__':
    main()
