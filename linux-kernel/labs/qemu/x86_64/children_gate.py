#!/usr/bin/env python3
"""Observe private child snapshots, shared histories and complete release."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import tempfile
import threading
import time

LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_gate", LAB / "topal_gate.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def entry_point(elf):
    lines = subprocess.check_output(["gdb", "-q", "-batch", "-ex", f"file {elf}",
        "-ex", "disassemble /r topal_x86_systems_static_image_process_enter"], text=True).splitlines()
    found = [int(re.search(r"0x([0-9a-f]+)\s+<", line)[1], 16)
             for index, line in enumerate(lines[:-1])
             if "swapgs" in line and "%rcx,%gs:0x128" in lines[index + 1].replace(" ", "")]
    if len(found) != 1:
        raise GATE.GateError("cannot identify the unique native process syscall entry")
    return found[0]


def script(elf, debug, result, failed, entry, external, corrupt):
    setup = f'''
import gdb,json,traceback
from pathlib import Path
records=[]
active=None
context=None
table=None
registers=("rbx","rcx","rdx","rsi","rdi","rbp","rsp","r8","r9","r10","r11","r12","r13","r14","r15")
def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
def memory(address,size):return bytes(gdb.selected_inferior().read_memory(address,size))
def word(address):return int.from_bytes(memory(address,8),"little")
def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
def gp():return {{name:reg(name) for name in registers}}
def simd():return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
def extended():return (simd(),reg("mxcsr"),reg("fctrl"),reg("fstat"),reg("ftag"))
def private_table():return memory(table,320)+memory(table+1344,8192-1344)
def descriptions():return [memory(table+320+64*i,64) for i in range(16)]
class Checked(gdb.Breakpoint):
    def stop(self):
        try:return self.inspect()
        except BaseException:
            Path({str(failed)!r}).write_text(traceback.format_exc())
            return True
class Rejected(Checked):
    def inspect(self):
        assert reg("rax")==0x12345679 and reg("r11")==0x12345678 and reg("eflags")&64==0
        assert memory(reg("rip"),2)==bytes.fromhex("75fe")
        Path({str(result)!r}).write_text(json.dumps(dict(result="expected-parent-restoration-rejection",
            snapshot_byte_injected=True,observed=reg("rax"),expected=reg("r11"),
            exact_user_comparison=True,execution="stopped before completion after independent rejection")))
        return True
class FullPressure(Checked):
    def inspect(self):
        assert active is not None and reg("rax")==15
        assert [word(table+64+16*i) for i in range(16)]==list(range(17,33))
        assert [word(table+320+64*i+8) for i in range(16)]==[1]*16
        assert [word(table+0x3d000+64*i) for i in range(16)]==[3]*16
        assert [word(table+0x3d000+64*i+8) for i in range(16)]==[1]*16
        active["description_peak"]=32
        self.enabled=False
        return False
class Returned(Checked):
    def inspect(self):
        global active
        assert active is not None
        assert gp()==active["registers"] and extended()==active["extended"] and reg("fs_base")==active["fs"]
        if reg("rax")==0:
            assert not active["child_returned"]
            assert word(table+0x2000)==1
            pid=word(table+0x2010)
            assert pid>=2 and word(table+56)==pid
            assert word(context+152)==active["clear_tid"] and word(context+160)==0, (word(context+152),active["clear_tid"],word(context+160))
            assert memory(table+0x2100+192,32)==bytes(32)
            assert active["key"]==memory(context+192,32)
            active.update(pid=pid,child_returned=True)
            if {corrupt!r}:
                address=table+0x7000+(0x5f2000-0x5e5000)
                assert memory(address,1)==bytes([0x78])
                gdb.selected_inferior().write_memory(address,bytes([0x79]))
            return False
        assert active["child_returned"] and active["child_exited"]
        assert reg("rax")==active["pid"] and word(table+56)==1 and word(table+0x2000)==2
        assert word(table+0x2018)==active["status"]
        assert private_table()==active["table"]
        for before,after in zip(active["descriptions"],descriptions()):
            assert before[:24]+before[32:]==after[:24]+after[32:]
        assert memory(context+192,32)==active["exit_key"]
        assert word(context+264)==active["exit_deliveries"]
        assert word(context+152)==active["parent_clear_tid"] and word(context+160)==active["parent_robust"]
        assert memory(table+0x2020,0x3e000-0x2020)==bytes(0x3e000-0x2020)
        if {external!r}:
            assert active["exec_requested"] and active["key"]!=active["exit_key"]
            assert active["exit_deliveries"]>active["deliveries"]
        if {corrupt!r}:
            assert word(0x5f2000)==0x12345679
            # Locate the independent fixture's first exact PRIVATE comparison.
            pattern=bytes.fromhex("48a100205f000000000049bb78563412000000004c39d875fe")
            code=memory(0x4f0000,0x8000)
            assert code.count(pattern)==1
            Rejected("*"+hex(0x4f0000+code.index(pattern)+len(pattern)-2),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
            self.enabled=False
            return False
        assert [memory(address,size) for address,size in active["backings"]]==active["private"]
        records.append(dict(pid=active["pid"],status=active["status"],child_returned=True,parent_returned=True,
            registers_preserved=True,extended_state_preserved=True,tls_preserved=True,private_backings_preserved=True,
            private_process_state_preserved=True,shared_description_references_preserved=True,
            snapshot_erased=True,thread_registrations_restored=True,entropy_history_preserved=True,
            entropy_history_advanced=active["key"]!=active["exit_key"],child_exec=active["exec_requested"],
            maximum_live_descriptions=active.get("description_peak",0)))
        active=None
        self.enabled=False
        incoming.enabled=True
        return False
class Incoming(Checked):
    def inspect(self):
        global active,context,table
        number=reg("rax")&0xffffffff
        if number in (56,57) and active is None:
            if not {external!r} and not 0x4f0000<=reg("rcx")<0x4f8000:return False
            context=reg("k_gs_base")
            table=word(context+928)
            if word(table+0x2000)!=0:return False
            assert word(table)==1 and word(table+56)==1
            flags=(reg("rdi")&0xffffffff) if number==56 else 17
            if flags&~0x1200000!=17 or (number==56 and reg("rsi")!=0):return False
            root=word(context+24)
            physical_mask=0xffffffffff000
            data=word(word(context+112)+0xf28)&physical_mask
            stack=[word(root+offset)&physical_mask for offset in (0x4ff0,0x4ff8)]
            backings=[(data,0x12000),(word(context+120),0x22000)]+[(address,4096) for address in stack]
            active=dict(registers=gp(),extended=extended(),fs=reg("fs_base"),table=private_table(),
                descriptions=descriptions(),key=memory(context+192,32),deliveries=word(context+264),
                parent_clear_tid=word(context+152),parent_robust=word(context+160),
                clear_tid=reg("r10") if flags&0x200000 else 0,
                backings=backings,private=[memory(address,size) for address,size in backings],
                child_returned=False,child_exited=False,exec_requested=False)
            Returned("*"+hex(reg("rcx")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        elif active is not None:
            assert word(table+56)==active["pid"]
            if number==257 and not {external!r} and len(records)==3:
                slots=[word(table+64+16*i) for i in range(16)]
                if len([value for value in slots if value])==15:
                    FullPressure("*"+hex(reg("rcx")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
            if number==59:active["exec_requested"]=True
            if number in (60,231):
                active.update(child_exited=True,status=reg("rdi")&255,exit_key=memory(context+192,32),
                              exit_deliveries=word(context+264))
                self.enabled=False
        return False
class Released(Checked):
    def inspect(self):
        assert active is None and records
        assert len(records)=={1 if external else 4}
        assert memory(table,0x43000)==bytes(0x43000)
        assert word(context+152)==0 and word(context+160)==0
        assert word(context+952)==0 and word(context+960)==0 and word(context+968)==0
        Path({str(result)!r}).write_text(json.dumps(dict(calls=records,process_backing_bytes=0x43000,
            terminal_storage_erased=True,terminal_thread_registrations_erased=True,
            terminal_library_bindings_erased=True,result="pass"),indent=2,sort_keys=True))
        return True
incoming=Incoming("*{entry}",type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Released("*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
'''
    return f"set pagination off\nset confirm off\nfile {elf}\ntarget remote {debug}\npython\n{setup}\nend\ncontinue\ndetach\n"


def observe(root, values, external_image, corrupt, timeout, workload=None,
            rejection="independent parent restoration oracle rejected the injected snapshot"):
    root.mkdir()
    debug, result, failed = root / "debug.sock", root / "result.json", root / "failed.txt"
    elf = values[3].parent / "kernel.elf"
    commands = root / "debug.gdb"
    commands.write_text(script(elf, debug, result, failed, entry_point(elf), external_image is not None, corrupt))
    outcome = []

    def inspect():
        try:
            until = time.monotonic() + 5
            while not debug.exists() and time.monotonic() < until:
                time.sleep(.02)
            run = subprocess.run(["gdb", "-q", "-batch", "-x", str(commands)],
                                 stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout + 5)
            (root / "debug.log").write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            import traceback
            (root / "debug.log").write_text(traceback.format_exc())
            outcome.append(str(error))

    original_build, original_command, original_observe = GATE.build_image, GATE.qemu_command, GATE.observe_gate
    thread = threading.Thread(target=inspect, daemon=True)

    def command(*args):
        thread.start()
        return original_command(*args) + ["-S", "-gdb", f"unix:{debug},server=on,wait=off"]

    def failure():
        if failed.exists():
            return "child debugger assertion failed: " + failed.read_text()
        if corrupt and result.exists():
            return rejection
        return None

    GATE.build_image = lambda *_args, **_kwargs: values
    GATE.qemu_command = command
    GATE.observe_gate = lambda *args, **kwargs: original_observe(*args, **kwargs, failure_check=failure)
    try:
        try:
            runtime = GATE.verify(timeout, False, external_image,
                                  workload or ("child-echo" if external_image else "true"))
        except GATE.GateError as error:
            thread.join(timeout=5)
            if not corrupt or rejection not in str(error):
                log = (root / "debug.log").read_text() if (root / "debug.log").exists() else repr(outcome)
                raise GATE.GateError(str(error) + "\nChild debugger:\n" + log) from error
            runtime = None
        thread.join(timeout=5)
        if outcome != [0] or failed.exists() or not result.exists():
            raise GATE.GateError("child debugger did not qualify: " + repr(outcome) + "\n" +
                                 ((root / "debug.log").read_text() if (root / "debug.log").exists() else ""))
        record = dict(observation=json.loads(result.read_text()), linked_kernel_sha256=GATE.digest(elf))
        if runtime is not None:
            record["runtime"] = runtime
        return record
    finally:
        GATE.build_image, GATE.qemu_command, GATE.observe_gate = original_build, original_command, original_observe


def verify(external_image, timeout=60, write_evidence=False):
    GATE.checked_static_user_image(external_image)
    with tempfile.TemporaryDirectory(prefix="topal-children-", dir="/tmp") as temporary:
        root = Path(temporary)
        clean = GATE.build_image(root / "clean-artifact")
        external = GATE.build_image(root / "external-artifact", external_image)
        record = dict(schema="topal-static-process-children-qemu/1", harness_sha256=GATE.digest(Path(__file__)),
                      fixture_sha256=GATE.digest(LAB / "static_child_checks.py"),
                      clean=observe(root / "clean", clean, None, False, timeout),
                      external=observe(root / "external", external, external_image, False, timeout),
                      counterexample=observe(root / "corrupt", clean, None, True, timeout))
        assert len({record[name]["linked_kernel_sha256"] for name in ("clean", "external", "counterexample")}) == 1
    if write_evidence:
        (LAB / "results/topal-static-process-children-gate.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    print("PASS child snapshots, native/TLS/extended state, shared history, release and independent rejection", flush=True)
    return record


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--external-static-user-image", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--write-evidence", action="store_true")
    args = parser.parse_args()
    verify(args.external_static_user_image, args.timeout, args.write_evidence)
