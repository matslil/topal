#!/usr/bin/env python3
"""Observe ordinary Topal policy/helper calls and their private kernel ABI."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
import struct
import tempfile
import threading
import time

LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_gate", LAB / "topal_gate.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def commands(elf, debug, result, helper):
    setup = f'''
import gdb, json
from pathlib import Path
calls=[]
frames=[]
context=None
active=None
saved=("rbx","rbp","r12","r13","r14","r15")
expected=[0,3,0,2,1,1,0,2]
def reg(name): return int(gdb.parse_and_eval("$"+name))
def simd():
    return [[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)]
def word(address): return int.from_bytes(gdb.selected_inferior().read_memory(address,8),"little")
def symbol(name): return int(gdb.parse_and_eval("&'"+name+"'"))
class Returned(gdb.Breakpoint):
    def stop(self):
        global active
        assert active is not None
        assert reg("rsp")==active["entry_stack"]+8
        assert {{n:reg(n) for n in saved}}==active["callee_saved"]
        assert simd()==active["simd"]
        assert reg("rax")==expected[active["facts"]]
        assert word(context+952)==symbol("topal_kernel_boot_open_policy_v1")
        calls.append(dict(facts=active["facts"],choice=reg("rax"),
                          stack_aligned=True,callee_saved_preserved=True,simd_preserved=True))
        active=None
        self.enabled=False
        return False
class Enter(gdb.Breakpoint):
    def stop(self):
        global context,active
        assert active is None
        context=reg("gs_base")
        assert word(context+952)==reg("rip")
        assert reg("rsp")%16==8
        facts=reg("rdi")
        assert 0<=facts<8
        active=dict(facts=facts,entry_stack=reg("rsp"),callee_saved={{n:reg(n) for n in saved}},simd=simd())
        Returned("*"+hex(word(reg("rsp"))),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        return False
class Helper(gdb.Breakpoint):
    def stop(self):
        assert active is not None
        if not frames:
            trace=gdb.execute("bt",to_string=True)
            assert "boot-open-access.t" in trace,trace
            assert "topal_kernel_boot_open_policy_v1" in trace,trace
            frames.append(trace)
        return False
class Released(gdb.Breakpoint):
    def stop(self):
        assert context is not None and active is None
        assert word(context+952)==0
        assert set(c["facts"] for c in calls)==set(range(8)),calls
        assert frames
        Path({str(result)!r}).write_text(json.dumps(dict(calls=calls,source_helper_frames=frames,
            binding_erased=True,complete_finite_domain=True,result="pass"),indent=2,sort_keys=True)+"\\n")
        return True
Enter("*"+hex(symbol("topal_kernel_boot_open_policy_v1")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Helper("*"+hex(symbol({helper!r})),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Released("*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
'''
    return (f"set pagination off\nfile {elf}\ntarget remote {debug}\npython\n{setup}\nend\n"
            "continue\ndetach\n")


def verify(timeout=30, write_evidence=False):
    original_build = GATE.build_image
    original_command = GATE.qemu_command
    with tempfile.TemporaryDirectory(prefix="topal-library-", dir="/tmp") as directory:
        root = Path(directory)
        values = original_build(root / "artifact")
        artifact = json.loads(values[3].read_text())
        library = artifact["kernel_library"]
        functions = {f["symbol"]: f for f in library["functions"]}
        entry = functions[library["native_export"]]["callees"][0]
        helper = functions[entry]["callees"][0]
        debug, result = root / "debug.sock", root / "library.json"
        script = root / "library.gdb"
        script.write_text(commands(root / "artifact/linked/kernel.elf", debug, result, helper))
        outcome = []

        def inspect():
            try:
                until = time.monotonic() + 5
                while not debug.exists() and time.monotonic() < until:
                    time.sleep(.02)
                run = subprocess.run(["gdb", "-q", "-batch", "-x", str(script)],
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+5)
                (root / "debug.log").write_bytes(run.stdout)
                outcome.append(run.returncode)
            except BaseException as error:
                outcome.append(str(error))

        def command(*args):
            threading.Thread(target=inspect, daemon=True).start()
            return original_command(*args) + ["-S", "-gdb", f"unix:{debug},server=on,wait=off"]

        GATE.build_image = lambda *_args, **_kwargs: values
        GATE.qemu_command = command
        try:
            runtime = GATE.verify(timeout, False)
        except GATE.GateError as error:
            log = (root / "debug.log").read_text() if (root / "debug.log").exists() else repr(outcome)
            raise GATE.GateError(str(error) + "\nLibrary debugger:\n" + log) from error
        finally:
            GATE.build_image = original_build
            GATE.qemu_command = original_command
        if outcome != [0] or not result.exists():
            raise GATE.GateError("library debugger did not finish: " + repr(outcome) + "\n" + (root/"debug.log").read_text())
        observation = json.loads(result.read_text())
        record = dict(schema="topal-kernel-source-library-qemu/1",
                      harness_sha256=GATE.digest(Path(__file__)), library=library,
                      observation=observation, runtime=runtime)
        record["variants"] = variants(root, values, artifact, timeout)
    if write_evidence:
        (LAB / "results/topal-kernel-source-library-gate.json").write_text(
            json.dumps(record, indent=2, sort_keys=True) + "\n")
    print("PASS source/helper frames, complete finite domain, private ABI and binding release", flush=True)
    return record


def variants(root, primary, artifact, timeout):
    """Keep alternative kernels and an expected failure outside primary results."""
    original_build = GATE.build_image
    original_observe = GATE.observe_gate
    original_command = GATE.qemu_command
    source = (GATE.LINUX_KERNEL / "kernel/policy/boot-open-access.t").read_text()
    outcomes = []
    for name, text in [
        ("enum-reordered", source.replace("ReadDescription, NotDirectory, DirectoryWrite, ReadOnlyView",
                                          "ReadOnlyView, DirectoryWrite, NotDirectory, ReadDescription")),
        ("wrong-denial", source.replace("false then ReadOnlyView", "false then NotDirectory")),
    ]:
        case = root / name
        unit = case / "policy/boot-open-access.t"
        unit.parent.mkdir(parents=True)
        unit.write_text(text)
        values = original_build(case / "artifact", library_root=case)
        changed = json.loads(values[3].read_text())
        for prefix in ("topal.systems-root-object", "topal.provider-object"):
            before = [v for v in artifact["inputs"] if v["identity"].startswith(prefix)]
            after = [v for v in changed["inputs"] if v["identity"].startswith(prefix)]
            if len(before) != 1 or before != after:
                raise GATE.GateError("policy source variant changed native root/provider")
        if changed["kernel_library"]["object_sha256"] == artifact["kernel_library"]["object_sha256"]:
            raise GATE.GateError("policy variant did not change library object")
        observed = []

        def observe(*args):
            try:
                return original_observe(*args)
            finally:
                if args[2].exists():
                    observed.append(args[2].read_bytes())

        GATE.build_image = lambda *_args, **_kwargs: values
        GATE.observe_gate = observe
        debugger = []
        failure_result = case / "failure.json"
        if name == "wrong-denial":
            # The independent user witness loops at its failing comparison.
            spec = importlib.util.spec_from_file_location("checks", LAB / "static_process_checks.py")
            checks = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(checks)
            spec = importlib.util.spec_from_file_location("namespace", LAB / "static_namespace_checks.py")
            namespace = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(namespace)
            witness = namespace.code("after_exec", checks.Witness)
            compare = witness.index(b"\x49\xbb" + struct.pack("<Q", (1 << 64)-30))
            failure_pc = namespace.ENTRY + compare + 13
            debug = case / "debug.sock"
            script = case / "failure.gdb"
            script.write_text(f'''set pagination off
file {values[3].parent / 'kernel.elf'}
target remote {debug}
hbreak *{failure_pc}
continue
python
import gdb,json
from pathlib import Path
pc=int(gdb.parse_and_eval("$rip"))
actual=int(gdb.parse_and_eval("$rax"))&((1<<64)-1)
expected=int(gdb.parse_and_eval("$r11"))&((1<<64)-1)
assert pc=={failure_pc}
assert bytes(gdb.selected_inferior().read_memory(pc,2))==bytes.fromhex("75fe")
assert actual==(1<<64)-20 and expected==(1<<64)-30
assert int(gdb.parse_and_eval("$eflags"))&64==0
Path({str(failure_result)!r}).write_text(json.dumps(dict(pc=pc,observed_errno=20,expected_errno=30,
    user_comparison_failed=True,witness_failure_loop=True)))
end
detach
''')

            def inspect_failure():
                until = time.monotonic() + 5
                while not debug.exists() and time.monotonic() < until:
                    time.sleep(.02)
                run = subprocess.run(["gdb", "-q", "-batch", "-x", str(script)],
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+5)
                (case / "debug.log").write_bytes(run.stdout)
                debugger.append(run.returncode)

            def command(*args):
                threading.Thread(target=inspect_failure, daemon=True).start()
                return original_command(*args) + ["-S", "-gdb", f"unix:{debug},server=on,wait=off"]
            GATE.qemu_command = command
        try:
            if name == "enum-reordered":
                runtime = GATE.verify(timeout, False)
                outcome = dict(case=name, result="pass", runtime=runtime)
            else:
                try:
                    GATE.verify(timeout, False)
                except GATE.GateError as error:
                    serial = observed[-1] if observed else b""
                    if debugger != [0] or not failure_result.exists():
                        log = (case / "debug.log").read_text() if (case / "debug.log").exists() else repr(debugger)
                        raise GATE.GateError("wrong policy did not reach its specific user-witness rejection:\n" + log) from error
                    outcome = dict(case=name, result="expected-behavior-rejection",
                                   failure=json.loads(failure_result.read_text()),
                                   rejection=str(error), serial_utf8=serial.decode(errors="replace"))
                else:
                    raise GATE.GateError("wrong denial escaped the Linux behavior witness")
        finally:
            GATE.build_image = original_build
            GATE.observe_gate = original_observe
            GATE.qemu_command = original_command
        outcome.update(root_and_provider_unchanged=True, library=changed["kernel_library"],
                       linked_kernel_sha256=GATE.digest(values[3].parent / "kernel.elf"))
        outcomes.append(outcome)
        print("PASS", name, outcome["result"], flush=True)
    return outcomes


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-evidence", action="store_true")
    parser.add_argument("--timeout", type=float, default=30)
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence)
