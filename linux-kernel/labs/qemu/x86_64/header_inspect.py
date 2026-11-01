"""GDB-only ownership, actual source bytes, ABI and exec-refusal observations."""
import hashlib
import json
from pathlib import Path
import gdb


def install(config):
    class State:
        active = None
        context = None
        arena = None
        table = None
        calls = []
        frames = []
        poison_pending = config["poison"]

    state = State()
    saved = ("rbx", "rbp", "r12", "r13", "r14", "r15")
    zeros = set(config["zero_addresses"])

    def reg(name):
        return int(gdb.parse_and_eval("$" + name))

    def memory(address, count):
        return bytes(gdb.selected_inferior().read_memory(address, count))

    def word(address):
        return int.from_bytes(memory(address, 8), "little")

    def symbol(name):
        return int(gdb.parse_and_eval("&'" + name + "'"))

    def simd():
        return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]" % (i, j)))
                 for j in range(2)] for i in range(16)]

    def breakpoint(kind, point):
        return kind(point, type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)

    def owned(pointer, count):
        assert state.arena + 32 <= pointer
        assert pointer + count <= state.arena + word(state.arena + 8)

    def number(pointer, byte=False):
        pointer = int(pointer)
        if pointer in zeros:
            assert memory(pointer, 16) == bytes(16)
        else:
            owned(pointer, 16)
        sign, length = word(pointer), word(pointer + 8)
        assert sign == 0 and length <= 1
        if pointer not in zeros:
            owned(pointer, 16 + 4 * length)
        value = int.from_bytes(memory(pointer + 16, 4), "little") if length else 0
        assert value <= (255 if byte else 65535)
        return value

    def bindings():
        expected_time = symbol("topal_kernel_sleep_deadline_policy_v1") if config["has_time"] else 0
        assert word(state.context + 960) == expected_time
        assert word(state.context + 968) == state.arena
        assert word(state.context + 976) == symbol("topal_kernel_static_elf64_header_policy_v1")
        expected_dispatch=symbol("topal_kernel_static_linux_dispatch_policy_v1") if config["has_dispatch"] else 0
        assert word(state.context+984)==expected_dispatch
        assert word(state.context+992)==(symbol(config["identity_export"]) if config["has_identity"] else 0)
        assert word(state.context+1000)==(symbol("topal_x86_systems_identity_value_store") if config["has_retained"] else 0)

    def write(result, **extra):
        record = dict(result=result, calls=state.calls, source_frames=state.frames, **extra)
        Path(config["result"]).write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")

    class Enter(gdb.Breakpoint):
        def stop(self):
            assert state.active is None
            state.context = reg("gs_base")
            state.arena = symbol("topal_x86_systems_numeric_arena")
            state.table = word(state.context + 928)
            bindings()
            assert word(state.arena) == state.context and word(state.arena + 8) == 32
            assert word(state.arena + 16) == 2 and word(state.arena + 24) == 0
            assert memory(state.arena + 32, 65504) == bytes(65504)
            assert reg("eflags") & 512 == 0 and reg("rsp") % 16 == 8
            address, length, machine, maximum = (reg(r) for r in ("rdi", "rsi", "rdx", "rcx"))
            assert (length, machine, maximum) == (64, 62, 16)
            assert address == word(state.table + 16)
            assert address == word(state.context + 936) + config["payload_offset"]
            assert word(state.table + 24) >= 64
            original = memory(address, 64)
            assert original == bytes(config["header"])
            stack = reg("rsp")
            state.active = dict(input_address=address, input_bytes=original,
                                machine=machine, maximum=maximum, expected=1,
                                entry_stack=stack, caller_stack=word(stack + 8) + 64,
                                callee_saved={r: reg(r) for r in saved}, simd=simd(),
                                table=memory(state.table, 0x43000),
                                exec_snapshot=memory(word(state.context + 904), 0xa000),
                                context=memory(state.context, 1008),
                                clock=memory(word(state.context + 944), 24),
                                retained=memory(symbol("topal_x86_systems_identity_value_store"),4096) if config["has_retained"] else None,
                                returned=False)
            breakpoint(Returned, "*" + hex(word(stack)))
            return False

    class Snapshot(gdb.Breakpoint):
        def stop(self):
            active = state.active
            assert active is not None
            frame = gdb.selected_frame()
            value = frame.read_var("bytes")
            address = int(value)
            owned(address, 16)
            assert word(address) == 64 and int(value.dereference()["entry_count"]) == 64
            assert number(frame.read_var("machine")) == active["machine"]
            assert number(frame.read_var("maximum")) == active["maximum"]
            nodes, pointers, values = [], [], []
            node = word(address + 8)
            for _ in range(64):
                owned(node, 16)
                assert node not in nodes
                nodes.append(node)
                pointers.append(word(node))
                values.append(number(pointers[-1], byte=True))
                node = word(node + 8)
            assert node == 0 and bytes(values) == active["input_bytes"]
            # The three ordinary source/adapter frames have qualified LLVM
            # frame metadata. The enclosing native syscall loop has no DWARF
            # unwind contract and its RBP is a live kernel fact.
            trace = gdb.execute("bt 3", to_string=True)
            assert "elf.t" in trace and "static-elf64-header.t" in trace
            assert "topal_kernel_static_elf64_header_policy_v1" in trace
            observed = dict(entry_count=64, bytes=values, machine=active["machine"],
                            maximum_program_headers=active["maximum"],
                            header_arena_offset=address-state.arena,
                            node_arena_offsets=[p-state.arena for p in nodes],
                            source_stack_offset=active["caller_stack"]-reg("rsp"),
                            all_allocated_storage_invocation_owned=True,
                            actual_bytes_match_owned_input=True, trace=trace)
            assert 0 < observed["source_stack_offset"] <= config["stack_bound"]
            if state.poison_pending:
                assert values[0] == 127 and word(pointers[0]+8) == 1
                owned(pointers[0], 20)
                gdb.selected_inferior().write_memory(pointers[0]+16, (126).to_bytes(4, "little"))
                assert number(pointers[0], byte=True) == 126
                active["expected"] = 0
                state.poison_pending = False
                observed.update(controlled_copied_byte=0, copied_byte_before=127, copied_byte_after=126)
            state.frames.append(observed)
            self.enabled = False
            return False

    class Returned(gdb.Breakpoint):
        def stop(self):
            active = state.active
            assert active is not None
            assert reg("rsp") == active["entry_stack"] + 8
            assert {r: reg(r) for r in saved} == active["callee_saved"]
            assert simd() == active["simd"]
            assert reg("rax") in (0, 1)
            assert word(state.arena) == state.context and word(state.arena + 16) == 2
            cursor, count = word(state.arena + 8), word(state.arena + 24)
            assert 32 <= cursor <= config["allocation_bound"] and count <= 8192
            assert memory(state.arena + cursor, 65536-cursor) == bytes(65536-cursor)
            active.update(returned=True, actual=reg("rax"), highwater=cursor, allocations=count)
            self.enabled = False
            breakpoint(Clean, "*" + hex(config["clean"]))
            return False

    class Clean(gdb.Breakpoint):
        def stop(self):
            active = state.active
            assert active is not None and active["returned"]
            assert reg("rsp") == active["caller_stack"]
            assert reg("rax") == active["actual"]
            assert {r: reg(r) for r in saved} == active["callee_saved"]
            assert simd() == active["simd"]
            assert all(reg(r) == 0 for r in ("rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"))
            assert memory(state.arena, 65536) == bytes(65536)
            assert memory(reg("rsp")-4096, 4096) == bytes(4096)
            assert memory(state.table, 0x43000) == active["table"]
            assert memory(state.context, 1008) == active["context"]
            assert memory(word(state.context+944), 24) == active["clock"]
            if config["has_retained"]:
                assert memory(symbol("topal_x86_systems_identity_value_store"),4096)==active["retained"]
            assert memory(active["input_address"], 64) == active["input_bytes"]
            bindings()
            record = dict(expected=active["expected"], observed=active["actual"] ,
                          arena_highwater=active["highwater"], allocation_count=active["allocations"],
                          arena_erased=True, retired_stack_erased=True,
                          native_state_preserved=True, volatile_pointers_cleared=True,
                          original_file_unchanged=True, retained_state_unchanged=True,
                          bindings_retained=True)
            state.calls.append(record)
            self.enabled = False
            if active["actual"] != active["expected"]:
                assert config["counterexample"]
                write("expected-header-rejection", independent_header_rejection=True,
                      observation_boundary="policy cleaned; native commit not yet executed")
                return True
            if active["expected"] == 0:
                breakpoint(Refused, "*" + hex(config["refused"]))
            else:
                state.active = None
            return False

    class Refused(gdb.Breakpoint):
        def stop(self):
            active = state.active
            assert active is not None and active["actual"] == 0
            assert reg("rax") & ((1 << 64)-1) == (1 << 64)-8, ("errno", hex(reg("rax")))
            expected = bytearray(active["exec_snapshot"])
            expected[0x9800:0xa000] = bytes(0x800)
            actual=memory(word(state.context+904),0xa000)
            assert actual == bytes(expected), ("exec_snapshot",[i for i,(a,b) in enumerate(zip(actual,expected)) if a!=b][:16])
            assert memory(state.table,0x43000) == active["table"]
            actual_context=memory(state.context,1008)
            assert actual_context == active["context"], ("context",[i for i,(a,b) in enumerate(zip(actual_context,active["context"])) if a!=b][:16])
            assert memory(word(state.context+944), 24) == active["clock"], "clock"
            if config["has_retained"]:
                assert memory(symbol("topal_x86_systems_identity_value_store"),4096)==active["retained"]
            assert memory(active["input_address"], 64) == active["input_bytes"]
            assert memory(state.arena, 65536) == bytes(65536)
            write("pass-controlled-refusal", native_error=8, staging_erased=True,
                  no_exec_commit=True, retained_state_preserved=True, original_file_unchanged=True,
                  observation_boundary="native ENOEXEC prepared; user return not yet executed")
            return True

    class Released(gdb.Breakpoint):
        def stop(self):
            assert state.active is None and state.calls and state.frames
            assert all(word(state.context+offset) == 0 for offset in (952, 960, 968, 976, 984, 992, 1000))
            assert memory(state.arena, 65536) == bytes(65536)
            assert memory(state.table, 0x43000) == bytes(0x43000)
            if config["has_retained"]:
                assert memory(symbol("topal_x86_systems_identity_value_store"),4096)==bytes(4096)
            write("pass", binding_erased=True, terminal_arena_erased=True, backing_erased=True)
            return True

    breakpoint(Enter, "*"+hex(symbol("topal_kernel_static_elf64_header_policy_v1")))
    breakpoint(Snapshot, "support/elf.t:"+str(config["source_line"]))
    breakpoint(Released, "*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")))
