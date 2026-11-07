#!/usr/bin/env python3
"""Observe ordinary Topal policy/helper calls and their private kernel ABI."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
import struct
import shutil
import tempfile
import threading
import time

LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_gate", LAB / "topal_gate.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def module_publication(directory, artifact, key, prefix):
    """Bind the emitted dependency bytes to both input and output provenance."""
    name = {"kernel_library": "kernel-library.modules.json",
            "time_library": "time-library.modules.json",
            "image_library": "image-library.modules.json",
            "dispatch_library": "dispatch-library.modules.json",
            "process_identity_library": "process-identity-library.modules.json"}[key]
    path = directory / name
    sources = json.loads(path.read_text())
    library = artifact[key]
    digest = GATE.digest(path)
    if digest != library["module_snapshot_sha256"]:
        raise GATE.GateError("module snapshot digest disagrees with its library binding")
    expected = [dict(source=m["source"], identity=m["identity"],
                     sha256=GATE.hashlib.sha256(m["text"].encode()).hexdigest()) for m in sources]
    if expected != library["modules"]:
        raise GATE.GateError("exact dependency bytes disagree with their module records")
    inputs = [i for i in artifact["inputs"] if i["identity"].startswith(prefix)]
    required = sorted([dict(identity=prefix+m["source"], sha256=m["sha256"]) for m in expected],key=lambda i:i["identity"])
    if inputs != required:
        raise GATE.GateError("module source inputs disagree with published dependencies")
    outputs = [o for o in artifact["outputs"] if o["identity"] == name]
    if outputs != [dict(identity=name, sha256=digest)]:
        raise GATE.GateError("module snapshot output is missing or ambiguous")
    return dict(sources=sources, snapshot_sha256=digest, inputs=inputs, outputs=outputs)


def copy_architecture_libraries(directory):
    relative = Path("arch/x86_64/policy")
    shutil.copytree(GATE.LINUX_KERNEL/"kernel"/relative, directory/relative)
    shutil.copytree(GATE.LINUX_KERNEL/"kernel/memory", directory/"memory")
    # Some counterexamples construct a deliberately small policy tree. Retain
    # independent primary imports while changing only their selected policy.
    for name in ('file-open-access.t','bootstrap-byte-copy.t',
                 'bootstrap-byte-copy-frame.t','file-position.t'):
        policy=directory/'policy'/name
        policy.parent.mkdir(parents=True,exist_ok=True)
        policy.write_bytes((GATE.LINUX_KERNEL/'kernel/policy'/name).read_bytes())


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
            assert "logic.t" in trace and "boot.t" in trace,trace
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
    original_source=GATE.SOURCE_PATH
    original_build = GATE.build_image
    original_command = GATE.qemu_command
    with tempfile.TemporaryDirectory(prefix="topal-library-", dir="/tmp") as directory:
        root = Path(directory)
        legacy=root/'legacy-root.t'
        # This is the frozen boot-open component. V6's mutable namespace
        # selection independently requires the named-file open plan.
        frozen_source=original_source.read_text().replace('static-linux-dispatch-v6','static-linux-dispatch-v5')
        if '  namespace-library is' in frozen_source:
            start=frozen_source.index('  namespace-library is')
            end=frozen_source.index('  ),\n',start)+len('  ),\n')
            frozen_source=frozen_source[:start]+frozen_source[end:]
        legacy.write_text(frozen_source.replace('policy/file-open-access.t','policy/boot-open-access.t')
            .replace('export is file-open-decision','export is boot-open-decision')
            .replace('interface is file-open-decision-v1','interface is boot-open-decision-v1'))
        GATE.SOURCE_PATH=legacy
        try:
            values = original_build(root / "artifact")
            artifact = json.loads(values[3].read_text())
            library = artifact["kernel_library"]
            functions = {f["symbol"]: f for f in library["functions"]}
            entry = functions[library["native_export"]]["callees"][0]
            helper = next(name for name in functions if "support_2elogic_2eabsent_3f" in name)
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
            record = dict(explicit_frozen_open_compatibility_control=True, schema="topal-kernel-source-library-qemu/1",
                          harness_sha256=GATE.digest(Path(__file__)), library=library,
                          observation=observation, runtime=runtime)
            record["module_publication"] = module_publication(values[3].parent,artifact,"kernel_library","kernel-source-module:")
            record["variants"] = variants(root, values, artifact, timeout)
        finally:GATE.SOURCE_PATH=original_source
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
        # Keep every primary dependency while changing only this policy.
        shutil.copytree(GATE.LINUX_KERNEL / "kernel/policy", unit.parent)
        unit.write_text(text)
        shutil.copytree(GATE.LINUX_KERNEL / "kernel/support",case / "support")
        copy_architecture_libraries(case)
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
