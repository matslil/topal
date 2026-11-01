"""GDB-only independent nested-value, ABI, lifetime and publication oracle."""
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
        seed=None
        image=Path(config["elf"]).read_bytes()
    state=State();saved=("rbx","rbp","r12","r13","r14","r15")
    def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
    def memory(address,count):return bytes(gdb.selected_inferior().read_memory(address,count))
    def word(address):return int.from_bytes(memory(address,8),"little")
    def u32(address):return int.from_bytes(memory(address,4),"little")
    def put(address,value):gdb.selected_inferior().write_memory(address,value.to_bytes(8,"little"))
    def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
    def simd():return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
    def bp(kind,address):return kind(address,type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def bank(index):return state.cell+64+index*2016
    def bindings():
        for offset,selected,name in [(960,config["has_time"],"topal_kernel_sleep_deadline_policy_v1"),
            (976,config["has_image"],"topal_kernel_static_elf64_header_policy_v1"),
            (984,config["has_dispatch"],"topal_kernel_static_linux_dispatch_policy_v1"),(992,True,config["export"])]:
            assert word(state.context+offset)==(symbol(name) if selected else 0)
        assert word(state.context+968)==state.arena and word(state.context+1000)==state.cell
    def readonly(pointer,count):
        for address,offset,size in config["readonly"]:
            if address<=pointer and pointer+count<=address+size:
                start=offset+pointer-address
                assert memory(pointer,count)==state.image[start:start+count]
                return True
        return False
    def number(pointer):
        pointer=int(pointer)
        if state.arena+32<=pointer<state.arena+word(state.arena+8):limit=state.arena+word(state.arena+8)
        elif pointer in [bank(index)+offset for index in (0,1) for offset in (40,64)]:limit=pointer+24
        else:assert readonly(pointer,16);limit=None
        if limit is not None:assert pointer+16<=limit
        assert word(pointer)==0
        length=word(pointer+8);assert length<=2
        if limit is not None:assert pointer+16+4*length<=limit
        else:assert readonly(pointer,16+4*length)
        value=int.from_bytes(memory(pointer+16,4*length),"little") if length else 0
        assert length==0 or value>>(32*(length-1))!=0
        return value
    def orders(address):
        result=dict(state=[u32(address+32),u32(address+36)],history=[u32(address+12),u32(address+16)])
        assert all(sorted(value)==[0,1] for value in result.values());return result
    def decode(address,persistent=False):
        byte=memory(address+8,1)[0]
        if persistent:assert byte in (0,1)
        return dict(next=number(word(address+24)),history=dict(creations=number(word(address)),wrapped=bool(byte&1)))
    def graph_bytes(index,value,presentation):
        address=bank(index);result=bytearray(2016)
        def write(offset,value,count=8):result[offset:offset+count]=value.to_bytes(count,"little")
        write(0,address+40);write(24,address+64);write(8,int(value["history"]["wrapped"]),1)
        for offset,order in [(12,presentation["history"][0]),(16,presentation["history"][1]),
            (32,presentation["state"][0]),(36,presentation["state"][1])]:write(offset,order,4)
        for offset,payload in [(40,value["history"]["creations"]),(64,value["next"])]:
            assert 0<payload<(1<<64)
            write(offset,0);write(offset+8,1 if payload<=0xffff_ffff else 2);write(offset+16,payload)
        return bytes(result)
    def graph(index,value,presentation):assert memory(bank(index),2016)==graph_bytes(index,value,presentation)
    def write(result,**extra):
        Path(config["result"]).write_text(json.dumps(dict(result=result,calls=state.calls,source_frames=state.frames,**extra),indent=2,sort_keys=True)+"\n")
    class Checked(gdb.Breakpoint):
        def stop(self):
            try:return self.inspect()
            except BaseException:Path(config["failed"]).write_text(traceback.format_exc());return True

    class Before(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None
            state.context=reg("gs_base");state.arena=symbol("topal_x86_systems_numeric_arena")
            state.cell=symbol("topal_x86_systems_identity_value_store");state.table=word(state.context+928)
            bindings()
            assert reg("r15")==state.table and word(state.table+0x2000)==0 and word(state.table+56)==1
            assert memory(state.arena,65536)==bytes(65536)
            assert word(state.cell)==state.context and word(state.cell+8)==symbol(config["export"])
            assert memory(state.cell+40,24)==bytes(24)
            if config["mode"]:
                assert not state.calls and state.seed is None
                assert memory(state.cell+16,4080)==bytes(4080) and word(state.table+0x2008)==0
                facts=dict(next=2147483647 if config["mode"]=="wrap" else 2,
                    history=dict(creations=1 if config["mode"]=="wrap" else (1<<64)-2,wrapped=config["mode"]=="count"))
                presentation=dict(state=[0,1],history=[0,1])
                put(state.cell+16,1);put(state.cell+24,0);put(state.cell+32,1)
                gdb.selected_inferior().write_memory(bank(0),graph_bytes(0,facts,presentation))
                put(state.table+0x2008,facts["next"])
                state.seed=dict(controlled_owned_version_seed=True,initially_uninitialized=True,
                    published_epoch=1,state=facts,orders=presentation,
                    scope="valid-domain seed before invocation; stop before next publication")
            cursor=word(state.table+0x2008)
            initialized,index,epoch=(word(state.cell+offset) for offset in (16,24,32))
            assert initialized in (0,1) and index in (0,1) and memory(bank(1-index),2016)==bytes(2016)
            if initialized:
                facts=decode(bank(index),True);presentation=orders(bank(index));graph(index,facts,presentation)
                assert facts["next"]==cursor
                if state.seed:assert facts==state.seed["state"] and epoch==1
                else:assert epoch==len(state.calls) and facts==state.calls[-1]["source_result"]["state"]
            else:
                assert not state.calls and (cursor,index,epoch)==(0,0,0)
                assert memory(state.cell+64,4032)==bytes(4032)
                facts=dict(next=0,history=dict(creations=0,wrapped=False));presentation=dict(state=[0,1],history=[0,1])
            state.pending=dict(cursor=cursor,facts=facts,input_orders=presentation,initialized=initialized,old_bank=index,epoch=epoch,
                caller_stack=reg("rsp"),callee_saved={name:reg(name) for name in saved},simd=simd(),
                context=memory(state.context,1008),table=memory(state.table,0x43000),cell=memory(state.cell,4096),
                clock=memory(word(state.context+944),24),return_ip=word(state.context+296))
            return False
    class Enter(Checked):
        def inspect(self):
            active=state.pending
            assert active and reg("rdi")==active["cursor"] and reg("rsi")==state.cell
            assert reg("rdx")==state.context and reg("rcx")==symbol(config["export"])
            assert reg("rsp")%16==8 and reg("eflags")&512==0
            assert word(state.arena)==state.context and word(state.arena+8)==32
            assert word(state.arena+16)==4 and word(state.arena+24)==0
            assert memory(state.arena+32,65504)==bytes(65504)
            active.update(entry_stack=reg("rsp"),return_address=word(reg("rsp")))
            self.enabled=False;bp(SourceCall,"*"+hex(config["source_call"]));return False
    class SourceCall(Checked):
        def inspect(self):
            active=state.pending;address=reg("rdi")
            assert active["caller_stack"]-4096<=address and address+56<=active["entry_stack"]
            active["source_result_address"]=address
            self.enabled=False;bp(SourceResult,"*"+hex(config["source_result"]));return False
    class Source(Checked):
        def inspect(self):
            active=state.pending;frame=gdb.selected_frame();root=frame.older();value=root.read_var("state")
            pointer=int(value["next"]);count_pointer=int(value["history"]["creations"])
            facts=dict(next=number(pointer),history=dict(creations=number(count_pointer),wrapped=bool(int(value["history"]["wrapped"])&1)))
            assert facts==active["facts"]
            address=int(value.address);assert int(value.type.sizeof)==40
            assert active["caller_stack"]-4096<=address and address+40<=active["caller_stack"]
            presentation=orders(address);assert presentation==active["input_orders"]
            if active["initialized"]:
                assert pointer==bank(active["old_bank"])+64 and count_pointer==bank(active["old_bank"])+40
            else:
                assert readonly(pointer,16) and readonly(count_pointer,16)
            assert memory(state.cell,4096)==active["cell"]
            allocated=number(frame.read_var("allocated"));assert allocated==(facts["next"] or 2)
            trace=gdb.execute("bt 3",to_string=True)
            assert "structured-process-identity.t" in trace and config["export"] in trace
            depth=active["caller_stack"]-reg("rsp");assert 0<depth<=config["stack_bound"]
            state.frames.append(dict(state=facts,orders=presentation,allocated_input=allocated,published_epoch=active["epoch"],
                initialized=bool(active["initialized"]),actual_published_nodes_borrowed=2 if active["initialized"] else 0,
                ordinary_zero_initialization=not active["initialized"],source_stack_offset=depth,trace=trace))
            return False
    class SourceResult(Checked):
        def inspect(self):
            active=state.pending;address=active["source_result_address"]
            outer=[u32(address+48),u32(address+52)];presentation=orders(address+8)
            assert outer==config["expected_outer"] and sorted(outer)==[0,1]
            assert presentation==dict(state=config["expected_state"],history=config["expected_history"])
            active["source_result"]=dict(allocated=number(word(address)),state=decode(address+8))
            active["source_orders"]=dict(outer=outer,**presentation)
            assert memory(state.cell,4096)==active["cell"]
            self.enabled=False;bp(Returned,"*"+hex(active["return_address"]));return False
    class Returned(Checked):
        def inspect(self):
            active=state.pending;source=active["source_result"]
            assert reg("rsp")==active["entry_stack"]+8
            assert {name:reg(name) for name in saved}==active["callee_saved"] and simd()==active["simd"]
            actual=dict(allocated=reg("rax"),next=reg("rdx"))
            assert actual==dict(allocated=source["allocated"],next=source["state"]["next"])
            assert all(2<=v<=2147483647 for v in actual.values())
            assert memory(state.cell,40)==active["cell"][:40]
            index=1-active["old_bank"]
            assert word(state.cell+40)==1 and word(state.cell+48)==index and word(state.cell+56)==actual["next"]
            graph(index,source["state"],dict(state=active["source_orders"]["state"],history=active["source_orders"]["history"]))
            assert memory(bank(active["old_bank"]),2016)==active["cell"][64+active["old_bank"]*2016:64+(active["old_bank"]+1)*2016]
            assert word(state.arena)==state.context and word(state.arena+16)==4
            cursor,count=word(state.arena+8),word(state.arena+24)
            assert 32<=cursor<=config["allocation_bound"] and count<=8192
            assert memory(state.arena+cursor,65536-cursor)==bytes(65536-cursor)
            active.update(actual=actual,highwater=cursor,allocations=count,prepared=memory(state.cell,4096))
            self.enabled=False;bp(Clean,"*"+hex(config["clean"]));return False
    class Clean(Checked):
        def inspect(self):
            active=state.pending
            assert reg("rsp")==active["caller_stack"] and dict(allocated=reg("rax"),next=reg("rdx"))==active["actual"]
            assert {name:reg(name) for name in saved}==active["callee_saved"] and simd()==active["simd"]
            assert all(reg(name)==0 for name in ("rcx","rsi","rdi","r8","r9","r10","r11"))
            assert memory(state.arena,65536)==bytes(65536) and memory(reg("rsp")-4096,4096)==bytes(4096)
            assert memory(state.context,1008)==active["context"] and memory(state.table,0x43000)==active["table"]
            assert memory(word(state.context+944),24)==active["clock"] and memory(state.cell,4096)==active["prepared"]
            bindings();facts=active["facts"];allocated=facts["next"] or 2
            expected=dict(allocated=allocated,state=dict(next=2 if allocated==2147483647 else allocated+1,
                history=dict(creations=facts["history"]["creations"]+1,wrapped=facts["history"]["wrapped"] or allocated==2147483647)))
            state.calls.append(dict(input_state=facts,input_orders=active["input_orders"],native_cursor=active["cursor"],
                native_result=active["actual"],source_result=active["source_result"],source_orders=active["source_orders"],expected=expected,
                published_epoch_before=active["epoch"],old_bank=active["old_bank"],prepared_bank=1-active["old_bank"],
                arena_highwater=active["highwater"],allocation_count=active["allocations"],old_graph_preserved=True,
                whole_source_state_independently_owned=True,nested_orders_preserved=True,all_numeric_limbs_preserved=True,
                prepared_graph_survives_cleanup=True,arena_erased=True,retired_stack_erased=True,
                native_callee_and_simd_preserved=True,volatile_pointers_cleared=True,context_table_clock_unchanged=True,bindings_retained=True,
                observation_boundary="whole source value prepared; temporary cleanup complete; publication not yet executed"))
            self.enabled=False
            if active["source_result"]!=expected:
                assert config["counterexample"]
                assert active["actual"]==dict(allocated=expected["allocated"],next=expected["state"]["next"])
                write("expected-structured-history-rejection",independent_history_rejection=True,published_state_unchanged=True,
                    native_scalar_result_still_matches=True)
                return True
            if config["mode"]:
                write("pass-controlled-"+config["mode"],controlled_seed=state.seed,
                    seeded_published_state_unchanged=True,native_cursor_unchanged=True)
                return True
            bp(Committed,"*"+hex(config["committed"]));return False
    class Committed(Checked):
        def inspect(self):
            active=state.pending;actual=active["actual"];index=1-active["old_bank"]
            assert word(state.table+0x2008)==actual["next"]
            assert word(state.table+0x2010)==word(state.table+0x2058)==actual["allocated"]
            assert word(state.table+0x2000)==0 and word(state.table+56)==1
            assert word(state.cell+16)==1 and word(state.cell+24)==index and word(state.cell+32)==active["epoch"]+1
            assert memory(state.cell+40,24)==bytes(24) and memory(bank(active["old_bank"]),2016)==bytes(2016)
            graph(index,active["source_result"]["state"],dict(state=active["source_orders"]["state"],history=active["source_orders"]["history"]))
            bindings();assert memory(state.arena,65536)==bytes(65536) and memory(reg("rsp")-4096,4096)==bytes(4096)
            state.calls[-1].update(owner_committed=True,published_epoch_after=active["epoch"]+1,
                prepared_obligation_consumed=True,old_bank_erased_and_read_back=True,complete_successor_graph_live=True)
            state.awaiting_child=active;state.pending=None
            self.enabled=False;bp(ChildReturned,"*"+hex(active["return_ip"]));return False
    class ChildReturned(Checked):
        def inspect(self):
            active=state.awaiting_child
            assert active and reg("rax")==0 and word(state.table+0x2000)==1
            assert word(state.table+56)==active["actual"]["allocated"] and word(state.cell+32)==active["epoch"]+1
            graph(1-active["old_bank"],active["source_result"]["state"],dict(state=active["source_orders"]["state"],history=active["source_orders"]["history"]))
            state.calls[-1]["child_pid_published"]=True
            state.awaiting_child=None;self.enabled=False;state.entry.enabled=True;return False
    class Released(Checked):
        def inspect(self):
            assert state.pending is None and state.awaiting_child is None and len(state.calls)==len(state.frames)==9
            assert all(c["child_pid_published"] for c in state.calls)
            assert all(word(state.context+offset)==0 for offset in (952,960,968,976,984,992,1000))
            assert memory(state.arena,65536)==bytes(65536) and memory(state.cell,4096)==bytes(4096)
            assert memory(state.table,0x43000)==bytes(0x43000)
            write("pass",complete_store_erased=True,all_bindings_erased=True,terminal_arena_erased=True,
                process_backing_erased=True,whole_history_preserved_across_child_and_exec=True);return True

    bp(Before,"*"+hex(config["before"]))
    state.entry=bp(Enter,"*"+hex(symbol(config["export"])))
    bp(Source,"*"+hex(config["source"]))
    bp(Released,"*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")))
