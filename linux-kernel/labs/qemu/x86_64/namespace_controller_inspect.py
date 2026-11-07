"""Ring0 namespace-controller components; no V6/Linux handler qualification.

The debugger supplies controlled metadata/facts at an actual live kernel owner
and makes a native call. Source alone mutates persistent bytes during that call.
Stop before returning to the earlier V5 personality, whose entry layout differs.
"""
import hashlib
import json
from pathlib import Path
import struct
import traceback

import gdb


def install(config):
    output = Path(config['output'])
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['inspector_sha256']
    artifact = config['artifact']
    library = artifact['namespace_library']
    geometry = artifact['namespace_runtime']
    binding = geometry['invocation']
    retirement = geometry.get('admission_retirement')
    mode = config.get('mode', 'normal')
    current = None
    completed = []
    histories = []
    ready_events = []
    callee = ('rbx', 'rbp', 'r12', 'r13', 'r14', 'r15')
    registers = callee + ('rdi', 'rsi', 'rdx', 'rcx', 'r8', 'r9', 'r10', 'r11')

    def reg(name): return int(gdb.parse_and_eval('$' + name)) & ((1 << 64) - 1)
    def addr(name): return int(gdb.parse_and_eval("&'" + name + "'"))
    def memory(start, count): return bytes(gdb.selected_inferior().read_memory(start, count))
    def words(start, count): return struct.unpack('<' + 'Q' * count, memory(start, count * 8))
    def put(start, value): gdb.selected_inferior().write_memory(start, struct.pack('<Q', value))
    def zero(start, count): assert memory(start, count) == bytes(count), (hex(start), count)
    def vectors(): return tuple(str(gdb.parse_and_eval(f'$xmm{i}.uint128')) for i in range(16))
    def failure():
        (output / 'failed.txt').write_text(traceback.format_exc())
        return True

    owner = addr(binding['controller_owner']['symbol'])
    metadata = owner + geometry['metadata_offset']
    control = addr('topal_x86_systems_namespace_control')
    arena = addr('topal_x86_systems_namespace_arena')
    history = addr('topal_x86_systems_namespace_history')
    recovery = addr('topal_x86_systems_namespace_recovery')
    source = addr(library['native_export'])
    apply = addr('topal_x86_systems_namespace_apply_v1')
    private = [s for s in binding['storage'] if s['symbol'] != 'topal_x86_systems_namespace_history']

    def natural(pointer):
        cursor = words(arena + 8, 1)[0]
        extents = [(arena + 32, arena + cursor)] + [(s['address'], s['address'] + s['bytes']) for s in binding['source_data']]
        assert any(a <= pointer and pointer + 16 <= b for a, b in extents)
        sign, count = words(pointer, 2)
        assert sign == 0 and count <= 3
        assert any(a <= pointer and pointer + 16 + count * 4 <= b for a, b in extents)
        return int.from_bytes(memory(pointer + 16, count * 4), 'little') if count else 0

    def seed(operation):
        data = bytearray(4096)

        def write(offset, value): data[offset:offset + 8] = value.to_bytes(8, 'little')
        write(0, 11)
        write(32, 0o41777)
        for offset, value in [(40, 21), (48, 32), (56, 43)]: write(offset, value)
        for i in range(4):
            for offset, value in [(0, 1), (8, 7 + i), (16, 100 + i), (24, 0o100600 + i), (32, 100 + i), (40, 200 + i), (48, 300 + i), (64, 1)]:
                write(256 + 128 * i + offset, value)
        length = [1, 255, 3, 7][operation]
        name = bytes(128 + i % 128 for i in range(length))
        write(64, 7); write(72, 3); data[1536:1539] = b'old'
        write(312, 1); write(320, 0)
        if operation >= 2:
            write(80, 8); write(88, length); data[1792:1792 + length] = name
            write(440, 1); write(448, 0)
        data[3200:3200 + length] = name
        values = (0, 0, 4 if operation < 2 else 1, 0 if operation == 1 else 1, length, 7, 0 if operation < 2 else 8, 1)
        for i, value in enumerate(values): write(3072 + 8 * i, value)
        return bytes(data)

    def expected_from_stores():
        expected = bytearray(current['before'])
        for offset, value in current['stores']:
            start = geometry['metadata_offset'] + offset
            expected[start:start + 8] = value.to_bytes(8, 'little')
        return expected

    def desired_page(before, operation, seconds, include_fault=True):
        page = bytearray(before)

        def write(offset, value): page[offset:offset + 8] = value.to_bytes(8, 'little')
        if operation < 2:
            slot = 0 if operation == 1 else 1
            page[1536 + slot * 256:1792 + slot * 256] = before[3200:3456]
            write(72 + slot * 16, int.from_bytes(before[3104:3112], 'little'))
            if operation == 0:
                write(80, 7); write(312, 2); write(320, 0)
        elif operation == 2:
            write(80, 7); page[64:80] = bytes(16); page[1536:1792] = bytes(256)
            write(440, 0); write(432, seconds)
        else:
            write(64, 8); write(80, 7); write(432, seconds)
        write(304, seconds); write(48, seconds); write(56, seconds)
        if include_fault and mode == 'source-wrong-association': write(64, 999)
        return bytes(page)

    def save_owner(name):
        actual = memory(owner, binding['controller_owner']['bytes'])
        (output / f"call-{current['ordinal']}-{name}.bin").write_bytes(actual)
        return actual

    def cleaned_expected():
        expected = expected_from_stores()
        start = geometry['metadata_offset'] + 3072
        expected[start:start + 512] = bytes(512)
        return expected

    def check_cleanup(fatal):
        for storage in private: zero(storage['address'], storage['bytes'])
        zero(metadata + 3072, 512)
        # Includes the caller's restored spill slots on normal return. Fatal
        # recovery additionally clears the abandoned caller return-address word.
        count = 4096 + (184 if fatal else 176)
        zero(current['frame'] - 4096, count)
        if fatal and retirement:
            zero(current['entry_stack'] - retirement['retired_stack_bytes'], retirement['retired_stack_bytes'])
            zero(current['entry_stack'] - retirement['owning_adapter_frame_bytes'], retirement['owning_adapter_frame_bytes'])
        expected_generation = current['old_generation'] + int(current['source_entries'] != 0)
        assert words(history, 1)[0] == expected_generation
        assert save_owner('completed') == cleaned_expected(), 'persistent bytes rolled back or changed during cleanup'

    def finish(result):
        record = dict(result=result, mode=mode, calls=completed, histories=histories,
            combined_stack_bytes=binding['combined_stack_bytes'], controller_owner=owner,
            setup='debugger-supplied facts at actual live ring0 owner; no V6 syscall entry',
            all_source_code_and_data_verified=True)
        (output / 'observation.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')

    def summary():
        result = dict(operation=current['operation'], seconds=current['seconds'],
            generation=words(history, 1)[0], source_entries=current['source_entries'],
            source_result=current.get('source_result'), source_stores=len(current['stores']),
            frame=current['frame'], recovery_before_abort=current.get('recovery_before_abort'),
            erased_private=[(s['symbol'], s['bytes']) for s in private],
            stack_erased=4096 + (184 if mode != 'normal' else 176), staging_erased=512)
        if retirement and mode != 'normal':
            result['admission_stack_erased'] = dict(start=current['entry_stack'] - retirement['retired_stack_bytes'],
                bytes=retirement['retired_stack_bytes'], owning_frame_bytes=retirement['owning_adapter_frame_bytes'],
                owning_frame_witness='0xbadc0ffee0ddf00d -> zero')
        return result

    def enter_case(ordinal, return_pc, caller_stack):
        nonlocal current
        operation = ordinal if mode == 'normal' else 0
        seconds = [2**64 - 1, 0, 97, 123][operation]
        for storage in private: zero(storage['address'], storage['bytes'])
        data = seed(operation)
        gdb.selected_inferior().write_memory(metadata, data)
        if mode == 'pre-input': put(metadata + 3072, 4)
        if mode == 'pre-generation': put(history, 2**64 - 1)
        if mode.startswith('pre-overlap-'):
            other = dict(position='topal_x86_systems_file_position_control', page='topal_x86_systems_ram_page_control', inode='topal_x86_systems_inode_mode_control')[mode.removeprefix('pre-overlap-')]
            put(addr(other) + 24, 1)
        stack = caller_stack - 8
        frame = stack - 176
        assert 0x108000 <= stack <= 0x180000 - 8
        gdb.selected_inferior().write_memory(frame - 4096, bytes([0xa5]) * (4096 + 176))
        put(stack, return_pc)
        arguments = dict(rdi=owner + (4096 if mode == 'pre-owner' else 0),
            rsi=metadata + (8 if mode == 'pre-view' else 0), rdx=operation, rcx=seconds,
            rsp=stack, rip=apply)
        for name, value in arguments.items(): gdb.execute(f'set ${name} = {value}')
        current = dict(ordinal=ordinal, operation=operation, seconds=seconds,
            frame=frame, stack=stack, caller_stack=caller_stack, return_pc=return_pc,
            old_generation=words(history, 1)[0], source_entries=0, stores=[],
            registers=[reg(r) for r in registers], vectors=vectors())
        if retirement:
            entry_stack = words(reg('gs_base') + retirement['kernel_stack_context_offset'], 1)[0]
            assert retirement['minimum_stack_pointer'] <= entry_stack <= retirement['maximum_stack_pointer']
            assert entry_stack % retirement['stack_alignment'] == 0
            current['entry_stack'] = entry_stack
            if mode != 'normal':
                # Controlled owning-frame witness above the injected APPLY
                # frame; fatal completion abandons the enclosing dispatch.
                put(entry_stack - retirement['owning_adapter_frame_bytes'], 0xbadc0ffee0ddf00d)
        current['before'] = save_owner('before')
        current['desired'] = desired_page(memory(metadata, 4096), operation, seconds)
        current['planned'] = desired_page(memory(metadata, 4096), operation, seconds, False)
        histories.append(current['old_generation'])

    class SourceReturned(gdb.Breakpoint):
        def __init__(self):
            self.stack = reg('rsp')
            self.saved = [reg(r) for r in callee]
            self.simd = vectors()
            super().__init__('*' + hex(words(self.stack, 1)[0]), internal=True)

        def stop(self):
            try:
                self.enabled = False
                assert [reg(r) for r in callee] == self.saved and vectors() == self.simd
                assert reg('rsp') == self.stack + 8
                assert words(current['frame'], 2) == (current['operation'], current['seconds'])
                assert memory(metadata, 4096) == current['desired']
                assert save_owner('after-source') == expected_from_stores()
                expected = 0 if mode == 'source-false' else 1
                assert reg('rax') == expected
                current['source_result'] = reg('rax')
                assert words(control, 8) == (0, 0, current['old_generation'] + 1, 0, 0, 0, 0, 0)
                if mode == 'post-frame': put(current['frame'], current['operation'] + 1)
                if mode == 'post-control': put(control + 48, 1)
                return False
            except BaseException: return failure()

    class Source(gdb.Breakpoint):
        def stop(self):
            try:
                current['source_entries'] += 1
                generation = current['old_generation'] + 1
                assert reg('rdi') == control and reg('rsi') == owner and reg('rdx') == generation
                assert reg('rcx') == current['frame'] and reg('r8') == 2
                assert words(recovery, 4) == (current['frame'], generation, current['operation'], current['seconds'])
                assert words(history, 1)[0] == generation
                assert memory(addr('topal_x86_systems_namespace_expected'), 4096) == current['planned']
                assert memory(owner, binding['controller_owner']['bytes']) == current['before']
                SourceReturned()
                return False
            except BaseException: return failure()

    class StoreReturned(gdb.FinishBreakpoint):
        def __init__(self, offset, value):
            self.offset, self.value = offset, value
            super().__init__(internal=True)

        def stop(self):
            try:
                assert reg('rax') == 1 and words(metadata + self.offset, 1)[0] == self.value
                current['stores'].append((self.offset, self.value))
                if mode == 'runtime-abort' and len(current['stores']) == 1:
                    # Actual next native allocator failure, after persistent
                    # stores, invokes the linked source's own fatal hook.
                    put(arena + 8, 65536)
                return False
            except BaseException: return failure()

    class Store(gdb.Breakpoint):
        def stop(self):
            try:
                assert words(control + 24, 1)[0] == 1 and words(control + 32, 1)[0] == metadata
                offset, width, value = natural(reg('rsi')), reg('rcx'), natural(reg('rdx'))
                assert width == 8 and offset + width <= 4096 and value <= 2**64 - 1
                assert offset + width <= 3072 or offset >= 3584, 'source wrote immutable input'
                StoreReturned(offset, value)
                return False
            except BaseException: return failure()

    class Returned(gdb.Breakpoint):
        def stop(self):
            try:
                assert mode == 'normal', 'failed namespace action resumed its owner'
                assert reg('rsp') == current['stack'] + 8 and reg('rax') == 1
                assert [reg(r) for r in registers] == current['registers'] and vectors() == current['vectors']
                assert current['source_entries'] == 1 and current['source_result'] == 1
                check_cleanup(False)
                completed.append(summary())
                if len(completed) == 4:
                    finish('pass-native-namespace-controller-component')
                    return True
                enter_case(len(completed), current['return_pc'], current['caller_stack'])
                return False
            except BaseException: return failure()

    class Ready(gdb.Breakpoint):
        def stop(self):
            try:
                gs = reg('gs_base')
                ready_events.append(dict(gs=gs, owner_live=words(owner, 1)[0], stack=reg('rsp'), cs=reg('cs'), flags=reg('eflags')))
                (output / 'ready-events.json').write_text(json.dumps(ready_events, indent=2) + '\n')
                if words(owner, 1)[0] != 1:
                    return False
                assert not reg('eflags') & 0x200
                assert reg('cs') & 3 == 0, 'controller component must execute in ring0'
                # APPLY itself checks the actual GS-relative context. The
                # observer does not manufacture a GS base or a live authority.
                for bound in binding['code'] + binding['source_code'] + binding['source_data'] + [binding['controller_code']]:
                    assert hashlib.sha256(memory(bound['address'], bound['bytes'])).hexdigest() == bound['sha256'], bound['symbol']
                self.enabled = False
                pc, stack = reg('rip'), reg('rsp')
                Returned('*' + hex(pc), internal=True)
                enter_case(0, pc, stack)
                return False
            except BaseException: return failure()

    class Aborted(gdb.Breakpoint):
        def stop(self):
            try:
                assert mode != 'normal', 'normal namespace action aborted'
                expected = cleaned_expected() if current.get('check_entered') else expected_from_stores()
                assert memory(owner, binding['controller_owner']['bytes']) == expected
                current['recovery_before_abort'] = words(recovery, 4)
                assert words(recovery, 1)[0] == current['frame']
                if mode == 'runtime-abort':
                    assert current['stores'], 'runtime abort did not follow actual stores'
                    # The noreturn ABI permits an aborting callee to use these
                    # registers. Recovery must use its private saved frame.
                    for name in callee: gdb.execute(f'set ${name} = 123')
                return False
            except BaseException: return failure()

    class Checking(gdb.Breakpoint):
        def stop(self):
            try:
                assert reg('rdi') == metadata and reg('rsi') == current['source_result']
                assert memory(addr('topal_x86_systems_namespace_expected'), 4096) == current['planned']
                current['check_entered'] = True
                return False
            except BaseException: return failure()

    class Halted(gdb.Breakpoint):
        def stop(self):
            try:
                assert mode != 'normal'
                assert current['source_entries'] == (0 if mode.startswith('pre-') else 1)
                check_cleanup(True)
                completed.append(summary())
                finish('pass-native-namespace-fatal-component')
                return True
            except BaseException: return failure()

    Ready('*' + hex(addr(artifact['dispatch_library']['native_export'])), internal=True)
    Source('*' + hex(source), internal=True)
    Store("*'topal.region-instance.namespace-metadata.primitive.unsigned.store'", internal=True)
    Aborted('*' + hex(addr('topal_kernel_region_frame_namespace_metadata_abort_v1')), internal=True)
    Checking('*topal_x86_systems_namespace_check_v1', internal=True)
    halt = addr('topal_x86_systems_namespace_halt_v1')
    if retirement: halt += retirement['halt_instruction_offset']
    Halted('*' + hex(halt), internal=True)
