"""GDB-only immutable identity proposals and owner commit observations."""
import json
from pathlib import Path
import traceback
import gdb


def install(config):
    class State:
        pending = None
        awaiting_child = None
        context = None
        arena = None
        table = None
        calls = []
        frames = []
        image = Path(config["elf"]).read_bytes()

    state = State()
    saved = ("rbx", "rbp", "r12", "r13", "r14", "r15")
    def reg(name): return int(gdb.parse_and_eval("$"+name)) & ((1 << 64)-1)
    def memory(address, count): return bytes(gdb.selected_inferior().read_memory(address, count))
    def word(address): return int.from_bytes(memory(address, 8), "little")
    def symbol(name): return int(gdb.parse_and_eval("&'"+name+"'"))
    def simd():
        return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]" % (i,j))) for j in range(2)] for i in range(16)]
    def breakpoint(kind, point): return kind(point, type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
    def bindings():
        for offset, selected, name in [
            (960, config["has_time"], "topal_kernel_sleep_deadline_policy_v1"),
            (976, config["has_image"], "topal_kernel_static_elf64_header_policy_v1"),
            (984, config["has_dispatch"], "topal_kernel_static_linux_dispatch_policy_v1"),
            (992, True, "topal_kernel_process_identity_transition_v1"),
        ]:
            assert word(state.context+offset) == (symbol(name) if selected else 0)
        assert word(state.context+968) == state.arena
        assert word(state.context+1000) == 0
    def number(pointer):
        pointer = int(pointer)
        owned = state.arena+32 <= pointer < state.arena+word(state.arena+8)
        def check(count):
            if owned:
                assert pointer+count <= state.arena+word(state.arena+8)
            else:
                matches = [(address, offset, size) for address, offset, size in config["readonly"]
                           if address <= pointer and pointer+count <= address+size]
                assert len(matches) == 1
                address, offset, _ = matches[0]
                start = offset+pointer-address
                assert memory(pointer, count) == state.image[start:start+count]
        check(16)
        assert word(pointer) == 0
        length = word(pointer+8)
        assert length <= 1
        check(16+4*length)
        return int.from_bytes(memory(pointer+16, 4*length), "little") if length else 0
    def write(result, **extra):
        Path(config["result"]).write_text(json.dumps(dict(result=result, calls=state.calls,
            source_frames=state.frames, **extra), indent=2, sort_keys=True)+"\n")
    class Checked(gdb.Breakpoint):
        def stop(self):
            try: return self.inspect()
            except BaseException:
                Path(config["failed"]).write_text(traceback.format_exc())
                return True

    class Before(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None
            state.context = reg("gs_base")
            state.arena = symbol("topal_x86_systems_numeric_arena")
            state.table = word(state.context+928)
            assert reg("r15") == state.table
            bindings()
            assert word(state.table+0x2000) == 0 and word(state.table+56) == 1
            assert memory(state.arena, 65536) == bytes(65536)
            cursor = word(state.table+0x2008)
            if state.calls: assert cursor == state.calls[-1]["successor_cursor"]
            else: assert cursor == 0
            state.pending = dict(cursor=cursor, fact=cursor, caller_stack=reg("rsp"),
                callee_saved={name:reg(name) for name in saved}, simd=simd(),
                context=memory(state.context,1008), table=memory(state.table,0x43000),
                clock=memory(word(state.context+944),24), return_ip=word(state.context+296))
            return False

    class Enter(Checked):
        def inspect(self):
            active = state.pending
            assert active is not None and reg("rdi") == active["cursor"]
            assert reg("rsp") % 16 == 8 and reg("eflags") & 512 == 0
            assert word(state.arena) == state.context and word(state.arena+8) == 32
            assert word(state.arena+16) == 4 and word(state.arena+24) == 0
            assert memory(state.arena+32,65504) == bytes(65504)
            active.update(entry_stack=reg("rsp"), return_address=word(reg("rsp")))
            if config["wrap"]:
                assert not state.calls and active["cursor"] == 0
                gdb.execute("set $rdi = 2147483647")
                active["fact"] = 2147483647
            allocated = active["fact"] or 2
            active["expected"] = dict(allocated=allocated, next=2 if allocated == 2147483647 else allocated+1)
            self.enabled = False
            breakpoint(SourceCall,"*"+hex(config["source_call"]))
            return False

    class SourceCall(Checked):
        def inspect(self):
            # The ordinary 24-byte result retains its order suffix, so LLVM
            # lowers fastcc through a hidden caller-owned stack result slot.
            active = state.pending
            address = reg("rdi")
            assert active["caller_stack"]-4096 <= address
            assert address+24 <= active["entry_stack"]
            active["source_result_address"] = address
            self.enabled = False
            breakpoint(SourceResult,"*"+hex(config["source_result"]))
            return False

    class Source(Checked):
        def inspect(self):
            active = state.pending
            frame = gdb.selected_frame()
            allocated = number(frame.read_var("allocated"))
            root = frame.older()
            snapshot = root.read_var("state")
            cursor = number(snapshot["next"])
            assert cursor == active["fact"]
            assert allocated == active["expected"]["allocated"]
            trace = gdb.execute("bt 3", to_string=True)
            assert "process-identity.t" in trace and "topal_kernel_process_identity_transition_v1" in trace
            depth = active["caller_stack"]-reg("rsp")
            assert 0 < depth <= config["stack_bound"]
            state.frames.append(dict(copied_next=cursor, allocated_input=allocated, source_stack_offset=depth,
                actual_record_matches_copied_fact=True, values_invocation_owned_or_immutable=True, trace=trace))
            self.enabled = False
            return False

    class SourceResult(Checked):
        def inspect(self):
            active = state.pending
            address = active["source_result_address"]
            pointers = [word(address),word(address+8)]
            order = [int.from_bytes(memory(address+offset,4),"little") for offset in (16,20)]
            assert sorted(order) == [0,1] and order == config["expected_order"]
            values = {name:number(pointer) for name,pointer in zip(config["field_order"],pointers)}
            active["source_result"] = values
            active["source_order"] = order
            self.enabled = False
            breakpoint(Returned,"*"+hex(active["return_address"]))
            return False

    class Returned(Checked):
        def inspect(self):
            active = state.pending
            assert reg("rsp") == active["entry_stack"]+8
            assert {name:reg(name) for name in saved} == active["callee_saved"]
            assert simd() == active["simd"]
            actual = dict(allocated=reg("rax"),next=reg("rdx"))
            assert actual == active["source_result"]
            assert all(2 <= value <= 2147483647 for value in actual.values())
            assert word(state.arena) == state.context and word(state.arena+16) == 4
            cursor,count = word(state.arena+8),word(state.arena+24)
            assert 32 <= cursor <= config["allocation_bound"] and count <= 8192
            assert memory(state.arena+cursor,65536-cursor) == bytes(65536-cursor)
            active.update(actual=actual,highwater=cursor,allocations=count)
            self.enabled = False
            breakpoint(Clean,"*"+hex(config["clean"]))
            return False

    class Clean(Checked):
        def inspect(self):
            active = state.pending
            assert reg("rsp") == active["caller_stack"]
            assert dict(allocated=reg("rax"),next=reg("rdx")) == active["actual"]
            assert {name:reg(name) for name in saved} == active["callee_saved"]
            assert simd() == active["simd"] and reg("eflags") & 512 == 0
            assert all(reg(name) == 0 for name in ("rcx","rsi","rdi","r8","r9","r10","r11"))
            assert memory(state.arena,65536) == bytes(65536)
            assert memory(reg("rsp")-4096,4096) == bytes(4096)
            assert memory(state.context,1008) == active["context"]
            assert memory(state.table,0x43000) == active["table"]
            assert memory(word(state.context+944),24) == active["clock"]
            bindings()
            state.calls.append(dict(retained_cursor_before=active["cursor"], copied_cursor=active["fact"],
                allocated=active["actual"]["allocated"], successor_cursor=active["actual"]["next"],
                expected=active["expected"], actual_named_source_result=active["source_result"],
                actual_source_field_order=active["source_order"],ordinary_stack_record_decoded=True,
                arena_highwater=active["highwater"], allocation_count=active["allocations"],
                arena_erased=True,retired_stack_erased=True,native_callee_state_preserved=True,
                simd_preserved=True,volatile_pointers_cleared=True,retained_state_unchanged=True,
                bindings_retained=True,observation_boundary="proposal cleaned; identity state not yet committed"))
            self.enabled = False
            if active["actual"] != active["expected"]:
                assert config["counterexample"]
                write("expected-identity-transition-rejection",independent_transition_rejection=True,
                    owner_state_not_committed=True)
                return True
            if config["wrap"]:
                write("pass-controlled-wrap",controlled_copied_cursor=True,persistent_cursor_unchanged=True,
                    owner_state_not_committed=True)
                return True
            breakpoint(Committed,"*"+hex(config["committed"]))
            return False

    class Committed(Checked):
        def inspect(self):
            active = state.pending
            actual = active["actual"]
            assert word(state.table+0x2008) == actual["next"]
            assert word(state.table+0x2010) == word(state.table+0x2058) == actual["allocated"]
            assert word(state.table+0x2000) == 0 and word(state.table+56) == 1
            state.calls[-1].update(owner_cursor_and_identity_committed=True)
            state.awaiting_child = active
            state.pending = None
            self.enabled = False
            state.entry.enabled = True
            breakpoint(ChildReturned,"*"+hex(active["return_ip"]))
            return False

    class ChildReturned(Checked):
        def inspect(self):
            active = state.awaiting_child
            assert active is not None and reg("rax") == 0
            assert word(state.table+56) == word(state.table+0x2010) == active["actual"]["allocated"]
            assert word(state.table+0x2008) == active["actual"]["next"] and word(state.table+0x2000) == 1
            state.calls[-1].update(child_pid_published=True)
            state.awaiting_child = None
            self.enabled = False
            return False

    class Released(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None
            assert state.calls and state.frames and all(call["child_pid_published"] for call in state.calls)
            assert all(word(state.context+offset) == 0 for offset in (952,960,968,976,984,992,1000))
            assert memory(state.arena,65536) == bytes(65536)
            assert memory(state.table,0x43000) == bytes(0x43000)
            write("pass",bindings_erased=True,terminal_arena_erased=True,backing_erased=True)
            return True

    breakpoint(Before,"*"+hex(config["before"]))
    state.entry = breakpoint(Enter,"*"+hex(symbol("topal_kernel_process_identity_transition_v1")))
    breakpoint(Source,"*"+hex(config["source"]))
    breakpoint(Released,"*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")))
