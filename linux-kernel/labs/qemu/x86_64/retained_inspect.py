"""GDB-only actual retained graph borrowing, publication and retirement."""
import json
from pathlib import Path
import traceback
import gdb


def install(config):
    class State:
        pending=None
        awaiting_child=None
        calls=[]
        frames=[]
        context=None
        arena=None
        table=None
        cell=None
        image=Path(config["elf"]).read_bytes()
    state=State()
    saved=("rbx","rbp","r12","r13","r14","r15")
    def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
    def memory(address,count):return bytes(gdb.selected_inferior().read_memory(address,count))
    def word(address):return int.from_bytes(memory(address,8),"little")
    def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
    def simd():return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
    def bp(kind,address):return kind(address,type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def bindings():
        for offset,selected,name in [(960,config["has_time"],"topal_kernel_sleep_deadline_policy_v1"),
            (976,config["has_image"],"topal_kernel_static_elf64_header_policy_v1"),
            (984,config["has_dispatch"],"topal_kernel_static_linux_dispatch_policy_v1"),
            (992,True,"topal_kernel_retained_process_identity_policy_v1")]:
            assert word(state.context+offset)==(symbol(name) if selected else 0)
        assert word(state.context+968)==state.arena and word(state.context+1000)==state.cell
    def bank(index):return state.cell+64+index*2016
    def readonly(pointer,count):
        for address,offset,size in config["readonly"]:
            if address<=pointer and pointer+count<=address+size:
                start=offset+pointer-address
                assert memory(pointer,count)==state.image[start:start+count]
                return True
        return False
    def number(pointer):
        pointer=int(pointer)
        if state.arena+32<=pointer<state.arena+word(state.arena+8):
            limit=state.arena+word(state.arena+8)
        elif state.pending and state.pending["initialized"] and pointer==bank(state.pending["old_bank"])+16:
            limit=bank(state.pending["old_bank"])+40
        else:
            assert readonly(pointer,16)
            limit=None
        if limit is not None:assert pointer+16<=limit
        assert word(pointer)==0
        length=word(pointer+8);assert length<=1
        if limit is not None:assert pointer+16+4*length<=limit
        else:assert readonly(pointer,16+4*length)
        return int.from_bytes(memory(pointer+16,4*length),"little") if length else 0
    def graph(index,value):
        address=bank(index)
        assert word(address)==address+16 and word(address+8)==0
        assert word(address+16)==0 and word(address+24)==1 and word(address+32)==value
        assert memory(address+40,1976)==bytes(1976)
    def write(result,**extra):
        Path(config["result"]).write_text(json.dumps(dict(result=result,calls=state.calls,source_frames=state.frames,**extra),indent=2,sort_keys=True)+"\n")
    class Checked(gdb.Breakpoint):
        def stop(self):
            try:return self.inspect()
            except BaseException:
                Path(config["failed"]).write_text(traceback.format_exc());return True

    class Before(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None
            state.context=reg("gs_base");state.arena=symbol("topal_x86_systems_numeric_arena")
            state.cell=symbol("topal_x86_systems_identity_value_store");state.table=word(state.context+928)
            bindings()
            assert reg("r15")==state.table and word(state.table+0x2000)==0 and word(state.table+56)==1
            assert memory(state.arena,65536)==bytes(65536)
            cursor=word(state.table+0x2008)
            initialized,index,epoch=(word(state.cell+offset) for offset in (16,24,32))
            assert word(state.cell)==state.context and word(state.cell+8)==symbol("topal_kernel_retained_process_identity_policy_v1")
            assert memory(state.cell+40,24)==bytes(24)
            assert initialized in (0,1) and index in (0,1)
            assert memory(bank(1-index),2016)==bytes(2016)
            if initialized:
                assert epoch==len(state.calls) and cursor==state.calls[-1]["successor_cursor"]
                graph(index,cursor)
            else:
                assert not state.calls and (cursor,index,epoch)==(0,0,0)
                assert memory(state.cell+64,4032)==bytes(4032)
            state.pending=dict(cursor=cursor,fact=cursor,initialized=initialized,old_bank=index,epoch=epoch,
                caller_stack=reg("rsp"),callee_saved={name:reg(name) for name in saved},simd=simd(),
                context=memory(state.context,1008),table=memory(state.table,0x43000),
                cell=memory(state.cell,4096),clock=memory(word(state.context+944),24),return_ip=word(state.context+296))
            return False

    class Enter(Checked):
        def inspect(self):
            active=state.pending
            assert active and reg("rdi")==active["cursor"]
            assert reg("rsi")==state.cell and reg("rdx")==state.context
            assert reg("rcx")==symbol("topal_kernel_retained_process_identity_policy_v1")
            assert reg("rsp")%16==8 and reg("eflags")&512==0
            assert word(state.arena)==state.context and word(state.arena+8)==32
            assert word(state.arena+16)==4 and word(state.arena+24)==0
            assert memory(state.arena+32,65504)==bytes(65504)
            active.update(entry_stack=reg("rsp"),return_address=word(reg("rsp")))
            self.enabled=False
            bp(Constructor if config["wrap"] else SourceCall,"*"+hex(config["initial_constructor"] if config["wrap"] else config["source_call"]))
            return False

    class Constructor(Checked):
        def inspect(self):
            active=state.pending
            assert not state.calls and not active["initialized"] and reg("rdi")==0
            assert memory(state.cell,4096)==active["cell"]
            gdb.execute("set $rdi = 2147483647")
            active["fact"]=2147483647
            self.enabled=False;bp(SourceCall,"*"+hex(config["source_call"]))
            return False

    class SourceCall(Checked):
        def inspect(self):
            active=state.pending
            address=reg("rdi")
            assert active["caller_stack"]-4096<=address and address+24<=active["entry_stack"]
            active["source_result_address"]=address
            self.enabled=False;bp(SourceResult,"*"+hex(config["source_result"]))
            return False

    class Source(Checked):
        def inspect(self):
            active=state.pending
            frame=gdb.selected_frame();root=frame.older()
            pointer=int(root.read_var("state")["next"])
            value=number(pointer)
            assert value==active["fact"]
            if active["initialized"]:
                assert pointer==bank(active["old_bank"])+16
                assert memory(bank(active["old_bank"]),2016)==active["cell"][64+active["old_bank"]*2016:64+(active["old_bank"]+1)*2016]
            elif not config["wrap"]:assert value==0 and readonly(pointer,16)
            else:assert state.arena+32<=pointer<state.arena+word(state.arena+8)
            allocated=number(frame.read_var("allocated"))
            assert allocated==(active["fact"] or 2)
            trace=gdb.execute("bt 3",to_string=True)
            assert "process-identity.t" in trace and "topal_kernel_retained_process_identity_policy_v1" in trace
            depth=active["caller_stack"]-reg("rsp");assert 0<depth<=config["stack_bound"]
            state.frames.append(dict(cursor=value,allocated_input=allocated,published_epoch=active["epoch"],
                initialized=bool(active["initialized"]),actual_published_node_borrowed=bool(active["initialized"]),
                ordinary_zero_initialization=not active["initialized"] and not config["wrap"],
                controlled_constructor_fact=bool(config["wrap"]),source_stack_offset=depth,trace=trace))
            return False

    class SourceResult(Checked):
        def inspect(self):
            active=state.pending;address=active["source_result_address"]
            order=[int.from_bytes(memory(address+offset,4),"little") for offset in (16,20)]
            assert sorted(order)==[0,1] and order==config["expected_order"]
            active["source_result"]={name:number(word(address+8*i)) for i,name in enumerate(config["field_order"])}
            active["source_order"]=order
            assert memory(state.cell,4096)==active["cell"]
            self.enabled=False;bp(Returned,"*"+hex(active["return_address"]))
            return False

    class Returned(Checked):
        def inspect(self):
            active=state.pending
            assert reg("rsp")==active["entry_stack"]+8
            assert {name:reg(name) for name in saved}==active["callee_saved"] and simd()==active["simd"]
            actual=dict(allocated=reg("rax"),next=reg("rdx"));assert actual==active["source_result"]
            assert all(2<=value<=2147483647 for value in actual.values())
            assert memory(state.cell,40)==active["cell"][:40]
            index=1-active["old_bank"]
            assert word(state.cell+40)==1 and word(state.cell+48)==index and word(state.cell+56)==actual["next"]
            graph(index,actual["next"])
            assert memory(bank(active["old_bank"]),2016)==active["cell"][64+active["old_bank"]*2016:64+(active["old_bank"]+1)*2016]
            assert word(state.arena)==state.context and word(state.arena+16)==4
            cursor,count=word(state.arena+8),word(state.arena+24)
            assert 32<=cursor<=config["allocation_bound"] and count<=8192
            assert memory(state.arena+cursor,65536-cursor)==bytes(65536-cursor)
            active.update(actual=actual,highwater=cursor,allocations=count,prepared=memory(state.cell,4096))
            self.enabled=False;bp(Clean,"*"+hex(config["clean"]))
            return False

    class Clean(Checked):
        def inspect(self):
            active=state.pending
            assert reg("rsp")==active["caller_stack"] and dict(allocated=reg("rax"),next=reg("rdx"))==active["actual"]
            assert {name:reg(name) for name in saved}==active["callee_saved"] and simd()==active["simd"]
            assert all(reg(name)==0 for name in ("rcx","rsi","rdi","r8","r9","r10","r11"))
            assert memory(state.arena,65536)==bytes(65536) and memory(reg("rsp")-4096,4096)==bytes(4096)
            assert memory(state.context,1008)==active["context"] and memory(state.table,0x43000)==active["table"]
            assert memory(word(state.context+944),24)==active["clock"] and memory(state.cell,4096)==active["prepared"]
            bindings()
            allocated=active["fact"] or 2
            expected=dict(allocated=allocated,next=2 if allocated==2147483647 else allocated+1)
            state.calls.append(dict(native_cursor=active["cursor"],source_cursor=active["fact"],
                allocated=active["actual"]["allocated"],successor_cursor=active["actual"]["next"],expected=expected,
                source_result=active["source_result"],source_order=active["source_order"],
                published_epoch_before=active["epoch"],old_bank=active["old_bank"],prepared_bank=1-active["old_bank"],
                arena_highwater=active["highwater"],allocation_count=active["allocations"],
                old_graph_preserved=True,prepared_graph_independently_owned=True,prepared_graph_survives_cleanup=True,
                arena_erased=True,retired_stack_erased=True,native_callee_and_simd_preserved=True,
                volatile_pointers_cleared=True,context_table_clock_unchanged=True,bindings_retained=True,
                observation_boundary="temporary cleanup complete; published version and PID unchanged"))
            self.enabled=False
            if active["actual"]!=expected:
                assert config["counterexample"]
                write("expected-retained-transition-rejection",independent_transition_rejection=True,published_state_unchanged=True)
                return True
            if config["wrap"]:
                write("pass-controlled-wrap",controlled_initializer_before=0,controlled_initializer_after=2147483647,
                    published_state_still_uninitialized=True,native_cursor_unchanged=True)
                return True
            bp(Committed,"*"+hex(config["committed"]))
            return False

    class Committed(Checked):
        def inspect(self):
            active=state.pending;actual=active["actual"];index=1-active["old_bank"]
            assert word(state.table+0x2008)==actual["next"]
            assert word(state.table+0x2010)==word(state.table+0x2058)==actual["allocated"]
            assert word(state.table+0x2000)==0 and word(state.table+56)==1
            assert word(state.cell+16)==1 and word(state.cell+24)==index and word(state.cell+32)==active["epoch"]+1
            assert memory(state.cell+40,24)==bytes(24) and memory(bank(active["old_bank"]),2016)==bytes(2016)
            graph(index,actual["next"]);bindings()
            assert memory(state.arena,65536)==bytes(65536) and memory(reg("rsp")-4096,4096)==bytes(4096)
            state.calls[-1].update(owner_committed=True,published_epoch_after=active["epoch"]+1,
                prepared_obligation_consumed=True,old_bank_erased_and_read_back=True,successor_graph_live=True)
            state.awaiting_child=active;state.pending=None
            self.enabled=False;bp(ChildReturned,"*"+hex(active["return_ip"]))
            return False

    class ChildReturned(Checked):
        def inspect(self):
            active=state.awaiting_child
            assert active and reg("rax")==0 and word(state.table+0x2000)==1
            assert word(state.table+56)==active["actual"]["allocated"] and word(state.cell+32)==active["epoch"]+1
            graph(1-active["old_bank"],active["actual"]["next"])
            state.calls[-1]["child_pid_published"]=True
            state.awaiting_child=None;self.enabled=False;state.entry.enabled=True
            return False

    class Released(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None and len(state.calls)==9
            assert len(state.frames)==9 and all(c["child_pid_published"] for c in state.calls)
            assert all(word(state.context+offset)==0 for offset in (952,960,968,976,984,992,1000))
            assert memory(state.arena,65536)==bytes(65536) and memory(state.cell,4096)==bytes(4096)
            assert memory(state.table,0x43000)==bytes(0x43000)
            write("pass",complete_store_erased=True,all_bindings_erased=True,terminal_arena_erased=True,
                process_backing_erased=True,published_history_preserved_across_child_and_exec=True)
            return True

    bp(Before,"*"+hex(config["before"]))
    state.entry=bp(Enter,"*"+hex(symbol("topal_kernel_retained_process_identity_policy_v1")))
    bp(Source,"*"+hex(config["source"]))
    bp(Released,"*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")))
