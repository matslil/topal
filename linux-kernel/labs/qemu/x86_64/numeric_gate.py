#!/usr/bin/env python3
"""Observe real numeric source/runtime calls and complete arena/stack erasure."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import time

LAB=Path(__file__).resolve().parent
def module(name):
    spec=importlib.util.spec_from_file_location(name,LAB/(name+".py"))
    loaded=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded
GATE=module("topal_gate")
TIMING=module("time_gate")

def commands(elf,debug,result,target,bound):
    setup=f'''
import gdb,json
from pathlib import Path
calls=[]
frames=[]
active=None
context=None
arena=None
saved=("rbx","rbp","r12","r13","r14","r15")
def reg(name):return int(gdb.parse_and_eval("$"+name))
def word(address):return int.from_bytes(gdb.selected_inferior().read_memory(address,8),"little")
def memory(address,size):return bytes(gdb.selected_inferior().read_memory(address,size))
def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
def simd():return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
class Clean(gdb.Breakpoint):
    def stop(self):
        global active
        assert active is not None and active["returned"]
        assert reg("rsp")==active["caller_stack"]
        assert reg("r14")==active["expected_deadline"]
        assert memory(arena,65536)==bytes(65536)
        assert memory(reg("rsp")-4096,4096)==bytes(4096)
        assert all(reg(n)==0 for n in ("rax","rcx","rdx","rsi","rdi","r8","r9","r10","r11"))
        assert word(context+960)==symbol("topal_kernel_sleep_deadline_policy_v1")
        assert word(context+968)==arena
        record={{k:v for k,v in active.items() if k not in ("callee_saved","simd","entry_stack","caller_stack")}}
        record.update(arena_erased=True,retired_stack_erased=True,volatile_pointers_cleared=True,
                      binding_retained=True,result="pass")
        calls.append(record)
        active=None
        self.enabled=False
        return False
class Returned(gdb.Breakpoint):
    def stop(self):
        assert active is not None
        assert reg("rsp")==active["entry_stack"]+8
        assert {{n:reg(n) for n in saved}}==active["callee_saved"]
        assert simd()==active["simd"]
        assert reg("rax")==active["expected_deadline"]
        assert word(arena)==context and word(arena+16)==1
        cursor,count=word(arena+8),word(arena+24)
        assert 32<=cursor<={bound} and 0<=count<=8192
        assert memory(arena+cursor,65536-cursor)==bytes(65536-cursor)
        active.update(returned=True,observed_deadline=reg("rax"),arena_highwater=cursor,
                      allocation_count=count,callee_saved_preserved=True,simd_preserved=True,
                      unused_storage_untouched=True)
        self.enabled=False
        Clean("*{target}",type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        return False
class Enter(gdb.Breakpoint):
    def stop(self):
        global active,context,arena
        assert active is None
        context=reg("gs_base")
        arena=symbol("topal_x86_systems_numeric_arena")
        assert word(context+960)==reg("rip") and word(context+968)==arena
        assert word(arena)==context and word(arena+8)==32 and word(arena+16)==1 and word(arena+24)==0
        assert memory(arena+32,65504)==bytes(65504)
        assert reg("eflags")&512==0 and reg("rsp")%16==8
        seconds,nanos,ticks,wrapped=(reg(n) for n in ("rdi","rsi","rdx","rcx"))
        maximum=(1<<63)-1
        duration=maximum if seconds>=9223372036 else seconds*1000000000+nanos
        absolute=maximum if wrapped else min(ticks*10+duration,maximum)
        active=dict(seconds=seconds,nanoseconds=nanos,raw_ticks=ticks,wrapped=wrapped,
                    expected_deadline=(absolute+9)//10,entry_stack=reg("rsp"),
                    caller_stack=word(reg("rsp")+8),callee_saved={{n:reg(n) for n in saved}},
                    simd=simd(),returned=False,stack_aligned=True,interrupts_masked=True,
                    exec_lifetime=word(context+896))
        Returned("*"+hex(word(reg("rsp"))),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        return False
class Runtime(gdb.Breakpoint):
    def stop(self):
        assert active is not None and word(arena+16)==1
        if not frames:
            trace=gdb.execute("bt",to_string=True)
            assert "topal.runtime.int.divmod" in trace,trace
            assert "relative-sleep-deadline.t" in trace,trace
            assert "topal_kernel_sleep_deadline_policy_v1" in trace,trace
            frames.append(trace)
        return False
class Released(gdb.Breakpoint):
    def stop(self):
        assert active is None and calls and frames
        assert word(context+960)==0 and word(context+968)==0
        assert memory(arena,65536)==bytes(65536)
        Path({str(result)!r}).write_text(json.dumps(dict(calls=calls,source_runtime_frames=frames,
            binding_erased=True,terminal_arena_erased=True,result="pass"),indent=2,sort_keys=True)+"\\n")
        return True
Enter("*"+hex(symbol("topal_kernel_sleep_deadline_policy_v1")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Runtime("*"+hex(symbol("topal.runtime.int.divmod.absolute")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Released("*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
'''
    return (f"set pagination off\nfile {elf}\ntarget remote {debug}\npython\n{setup}\nend\ncontinue\ndetach\n")

def verify(timeout=30,write_evidence=False):
    original_build,original_command=GATE.build_image,GATE.qemu_command
    with tempfile.TemporaryDirectory(prefix="topal-numeric-",dir="/tmp") as directory:
        root=Path(directory)
        values=original_build(root/"artifact")
        artifact=json.loads(values[3].read_text())
        library=artifact["time_library"]
        _,target=TIMING.points(root/"artifact/linked/kernel.elf")
        debug,result=root/"debug.sock",root/"numeric.json"
        script=root/"numeric.gdb"
        script.write_text(commands(root/"artifact/linked/kernel.elf",debug,result,target,
                                   library["allocation"]["maximum_bytes"]))
        outcome=[]
        def inspect():
            try:
                until=time.monotonic()+5
                while not debug.exists() and time.monotonic()<until:time.sleep(.02)
                run=subprocess.run(["gdb","-q","-batch","-x",str(script)],stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT,timeout=timeout+5)
                (root/"debug.log").write_bytes(run.stdout)
                outcome.append(run.returncode)
            except BaseException as error:outcome.append(str(error))
        def command(*args):
            threading.Thread(target=inspect,daemon=True).start()
            return original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off"]
        GATE.build_image=lambda *_args,**_kwargs:values
        GATE.qemu_command=command
        try:runtime=GATE.verify(timeout,False)
        except GATE.GateError as error:
            log=(root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
            raise GATE.GateError(str(error)+"\nNumeric debugger:\n"+log) from error
        finally:GATE.build_image,GATE.qemu_command=original_build,original_command
        if outcome!=[0] or not result.exists():
            raise GATE.GateError("numeric debugger did not finish: "+repr(outcome)+"\n"+(root/"debug.log").read_text())
        record=dict(schema="topal-owned-numeric-runtime-qemu/1",harness_sha256=GATE.digest(Path(__file__)),
                    library=library,observation=json.loads(result.read_text()),runtime=runtime)
        record["counterexample"]=wrong_policy(root,artifact,timeout)
    if write_evidence:
        (LAB/"results/topal-owned-numeric-runtime-gate.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
    print("PASS ordinary numeric source/runtime frames, ABI, arena/stack erasure and binding release",flush=True)
    return record

def wrong_policy(root,primary,timeout):
    """Reject a changed scalar at the independent deadline oracle, before completion."""
    case=root/"wrong-deadline"
    policy=case/"policy"
    policy.mkdir(parents=True)
    original=(GATE.LINUX_KERNEL/"kernel/policy/relative-sleep-deadline.t").read_text()
    changed=original.replace("false then ordinary-deadline (seconds, nanoseconds, ticks)","false then 0")
    if changed==original:raise GATE.GateError("counterexample did not alter the source policy")
    (policy/"relative-sleep-deadline.t").write_text(changed)
    (policy/"boot-open-access.t").write_bytes((GATE.LINUX_KERNEL/"kernel/policy/boot-open-access.t").read_bytes())
    original_build,original_command,original_observe=GATE.build_image,GATE.qemu_command,GATE.observe_gate
    values=original_build(case/"artifact",library_root=case)
    artifact=json.loads(values[3].read_text())
    for prefix in ("topal.systems-root-object","topal.provider-object"):
        before=[v for v in primary["inputs"] if v["identity"].startswith(prefix)]
        after=[v for v in artifact["inputs"] if v["identity"].startswith(prefix)]
        if len(before)!=1 or before!=after:raise GATE.GateError("numeric body changed native root/provider")
    if artifact["time_library"]["object_sha256"]==primary["time_library"]["object_sha256"]:
        raise GATE.GateError("changed numeric source did not change the object")
    _,target=TIMING.points(values[3].parent/"kernel.elf")
    debug,result,script=case/"debug.sock",case/"failure.json",case/"failure.gdb"
    script.write_text(f'''set pagination off
file {values[3].parent/'kernel.elf'}
target remote {debug}
hbreak *{target}
continue
python
import gdb,json
from pathlib import Path
def reg(n):return int(gdb.parse_and_eval("$"+n))
assert reg("rip")=={target}
s,n,t,w=(reg(k) for k in ("rbx","rbp","r13","r15"))
maximum=(1<<63)-1
duration=maximum if s>=9223372036 else s*1000000000+n
expected=((maximum if w else min(t*10+duration,maximum))+9)//10
assert w==0 and t>0 and expected>0
assert reg("r14")==0 and reg("r14")!=expected
Path({str(result)!r}).write_text(json.dumps(dict(seconds=s,nanoseconds=n,raw_ticks=t,wrapped=w,
    expected_deadline=expected,observed_deadline=reg("r14"),independent_deadline_rejection=True)))
end
detach
''')
    outcome=[]
    def inspect():
        try:
            until=time.monotonic()+5
            while not debug.exists() and time.monotonic()<until:time.sleep(.02)
            run=subprocess.run(["gdb","-q","-batch","-x",str(script)],stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT,timeout=timeout+5)
            (case/"debug.log").write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:outcome.append(str(error))
    thread=threading.Thread(target=inspect,daemon=True)
    def command(*args):
        thread.start()
        return original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off"]
    def failure():return "independent numeric deadline oracle rejected the changed source" if result.exists() else None
    GATE.build_image=lambda *_args,**_kwargs:values
    GATE.qemu_command=command
    GATE.observe_gate=lambda *args,**kwargs:original_observe(*args,**kwargs,failure_check=failure)
    try:
        try:GATE.verify(timeout,False)
        except GATE.GateError as error:
            thread.join(timeout=5)
            if outcome!=[0] or not result.exists() or "independent numeric deadline oracle rejected" not in str(error):
                log=(case/"debug.log").read_text() if (case/"debug.log").exists() else repr(outcome)
                raise GATE.GateError("counterexample missed its specific deadline rejection:\n"+log) from error
        else:raise GATE.GateError("wrong deadline escaped the independent oracle")
    finally:GATE.build_image,GATE.qemu_command,GATE.observe_gate=original_build,original_command,original_observe
    print("PASS changed numeric body rejected at the independent deadline oracle",flush=True)
    return dict(result="expected-deadline-rejection",root_and_provider_unchanged=True,
                library=artifact["time_library"],linked_kernel_sha256=GATE.digest(values[3].parent/"kernel.elf"),
                observation=json.loads(result.read_text()),execution="stopped before completion after independent rejection")

if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout",type=float,default=30)
    parser.add_argument("--write-evidence",action="store_true")
    args=parser.parse_args()
    verify(args.timeout,args.write_evidence)
