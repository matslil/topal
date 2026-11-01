"""GDB-only source selector/tag, ABI preservation and before-handler reclamation."""
import json
from pathlib import Path
import gdb


def install(config):
    class State:
        pending=None
        calls=[]
        frames=[]
        context=None
        arena=None
        table=None
        entry=None
        image=None
    state=State()
    saved=("rbx","rbp","r12","r13","r14","r15")
    captured={"rcx":296,"r11":304,"rdi":312,"rsi":320,"rdx":328,"r10":336,"r8":344,"r9":352}
    semantic={int(number):name for number,name in config["mapping"]}
    encodings={name:int(number) for number,name in config["mapping"]}
    encodings["Unsupported"]=(1<<64)-1
    by_tag={tag:name for tag,name in zip(config["source_tags"],config["choices"])}

    def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
    def memory(address,count):return bytes(gdb.selected_inferior().read_memory(address,count))
    def word(address):return int.from_bytes(memory(address,8),"little")
    def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
    def simd():
        return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
    def breakpoint(kind,point):return kind(point,type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def bindings():
        for offset,selected,name in [(960,config["has_time"],"topal_kernel_sleep_deadline_policy_v1"),
                                    (976,config["has_image"],"topal_kernel_static_elf64_header_policy_v1"),
                                    (984,True,"topal_kernel_static_linux_dispatch_policy_v1"),
                                    (992,config["has_identity"],config["identity_export"]),
                                    (1000,config["has_retained"],"topal_x86_systems_identity_value_store")]:
            assert word(state.context+offset)==(symbol(name) if selected else 0)
        assert word(state.context+968)==state.arena
    def owned(pointer,count):
        assert state.arena+32<=pointer and pointer+count<=state.arena+word(state.arena+8)
    def readonly(pointer,count):
        for address,offset,size in config["readonly"]:
            if address<=pointer and pointer+count<=address+size:
                start=offset+pointer-address
                assert memory(pointer,count)==state.image[start:start+count]
                return True
        return False
    def number(pointer):
        pointer=int(pointer)
        allocated=state.arena+32<=pointer<state.arena+65536
        if allocated:owned(pointer,16)
        else:assert readonly(pointer,16)
        sign,length=word(pointer),word(pointer+8)
        assert sign==0 and length<=2
        if allocated:owned(pointer,16+4*length)
        else:assert readonly(pointer,16+4*length)
        return int.from_bytes(memory(pointer+16,4*length),"little")
    def write(result,**extra):
        Path(config["result"]).write_text(json.dumps(dict(result=result,calls=state.calls,source_frames=state.frames,**extra),indent=2,sort_keys=True)+"\n")

    class Raw(gdb.Breakpoint):
        def stop(self):
            assert state.pending is None
            state.context=reg("gs_base")
            state.arena=symbol("topal_x86_systems_numeric_arena")
            state.table=word(state.context+928)
            bindings()
            raw=reg("rax")
            state.pending=dict(raw_selector=raw,selector=raw&0xffff_ffff,
                expected=semantic.get(raw&0xffff_ffff,"Unsupported"),caller_stack=reg("rsp"),
                facts={name:word(state.context+offset) for name,offset in captured.items()},
                callee_saved={name:reg(name) for name in saved},simd=simd(),
                context=memory(state.context,1008),table=memory(state.table,0x43000),
                clock=memory(word(state.context+944),24),
                retained=memory(symbol("topal_x86_systems_identity_value_store"),4096) if config["has_retained"] else None,
                returned=False)
            assert reg("eflags")&512==0
            assert memory(state.arena,65536)==bytes(65536)
            assert {name:reg(name) for name in captured}==state.pending["facts"]
            return False

    class Enter(gdb.Breakpoint):
        def stop(self):
            active=state.pending
            assert active is not None
            bindings()
            assert reg("rdi")==active["selector"]
            assert reg("rsp")%16==8 and reg("eflags")&512==0
            assert word(state.arena)==state.context and word(state.arena+8)==32
            assert word(state.arena+16)==3 and word(state.arena+24)==0
            assert memory(state.arena+32,65504)==bytes(65504)
            active.update(entry_stack=reg("rsp"),return_address=word(reg("rsp")))
            self.enabled=False
            breakpoint(Tag,"*"+hex(config["tag_point"]))
            return False

    class Source(gdb.Breakpoint):
        def stop(self):
            active=state.pending
            frame=gdb.selected_frame()
            actual=number(frame.read_var("number"))
            expected=number(frame.read_var("expected"))
            assert actual==active["selector"] and expected in semantic
            trace=gdb.execute("bt 3",to_string=True)
            assert "support/syscall.t" in trace and "static-linux-dispatch.t" in trace
            depth=active["caller_stack"]-reg("rsp")
            assert 0<depth<=config["stack_bound"]
            state.frames.append(dict(selector=actual,comparison_candidate=expected,
                actual_fact_matches_captured_input=True,source_value_invocation_owned_or_immutable=True,
                source_stack_offset=depth,trace=trace))
            self.enabled=False
            return False

    class Tag(gdb.Breakpoint):
        def stop(self):
            active=state.pending
            tag=reg("rax")&0xffff_ffff
            assert tag in by_tag
            active.update(source_tag=tag,selected=by_tag[tag])
            self.enabled=False
            breakpoint(Returned,"*"+hex(active["return_address"]))
            return False

    class Returned(gdb.Breakpoint):
        def stop(self):
            active=state.pending
            assert reg("rsp")==active["entry_stack"]+8
            assert {name:reg(name) for name in saved}==active["callee_saved"]
            assert simd()==active["simd"]
            assert reg("rax")==encodings[active["selected"]]
            cursor,count=word(state.arena+8),word(state.arena+24)
            assert 32<=cursor<=config["allocation_bound"] and count<=1
            assert word(state.arena)==state.context and word(state.arena+16)==3
            assert memory(state.arena+cursor,65536-cursor)==bytes(65536-cursor)
            active.update(returned=True,operation_code=reg("rax"),highwater=cursor,allocations=count)
            self.enabled=False
            breakpoint(Clean,"*"+hex(config["clean_point"]))
            return False

    class Clean(gdb.Breakpoint):
        def stop(self):
            active=state.pending
            assert active is not None and active["returned"]
            assert reg("rax")==active["operation_code"] and reg("rsp")==active["caller_stack"]
            assert {name:reg(name) for name in saved}==active["callee_saved"]
            assert {name:reg(name) for name in captured}==active["facts"]
            assert simd()==active["simd"] and reg("eflags")&512==0
            assert memory(state.arena,65536)==bytes(65536)
            assert memory(reg("rsp")-4096,4096)==bytes(4096)
            assert memory(state.context,1008)==active["context"]
            assert memory(state.table,0x43000)==active["table"]
            assert memory(word(state.context+944),24)==active["clock"]
            if config["has_retained"]:
                assert memory(symbol("topal_x86_systems_identity_value_store"),4096)==active["retained"]
            bindings()
            state.calls.append(dict(raw_selector=active["raw_selector"],selector=active["selector"],
                expected_operation=active["expected"],selected_operation=active["selected"],
                actual_source_tag=active["source_tag"],adapted_operation_code=active["operation_code"],
                arena_highwater=active["highwater"],allocation_count=active["allocations"],
                actual_nominal_result_adapted=True,arena_erased=True,retired_stack_erased=True,
                native_callee_state_preserved=True,captured_argument_and_return_facts_restored=True,
                simd_preserved=True,retained_state_unchanged=True,bindings_retained=True,
                observation_boundary="selection cleaned; handler not yet entered"))
            if len(state.calls)%100==0:
                Path(config["progress"]).write_text(json.dumps(dict(calls=len(state.calls),selector=active["selector"])))
            self.enabled=False
            if active["selected"]!=active["expected"]:
                assert config["counterexample"]
                write("expected-source-selection-rejection",independent_operation_rejection=True,
                      native_routing_uses_actual_source_result=True,handler_not_entered=True)
                return True
            state.pending=None
            state.entry.enabled=True
            return False

    class Released(gdb.Breakpoint):
        def stop(self):
            assert state.pending is None and state.calls and state.frames
            assert all(word(state.context+offset)==0 for offset in (952,960,968,976,984,992,1000))
            assert memory(state.arena,65536)==bytes(65536)
            assert memory(state.table,0x43000)==bytes(0x43000)
            if config["has_retained"]:
                assert memory(symbol("topal_x86_systems_identity_value_store"),4096)==bytes(4096)
            write("pass",bindings_erased=True,terminal_arena_erased=True,backing_erased=True)
            return True

    state.image=Path(config["elf"]).read_bytes()
    breakpoint(Raw,"*"+hex(config["raw_point"]))
    state.entry=breakpoint(Enter,"*"+hex(symbol("topal_kernel_static_linux_dispatch_policy_v1")))
    # One exact location avoids multiplying one source-line breakpoint across
    # the compiler's separate helper instantiations and exceeding four slots.
    breakpoint(Source,"*"+hex(config["source_point"]))
    breakpoint(Released,"*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")))
