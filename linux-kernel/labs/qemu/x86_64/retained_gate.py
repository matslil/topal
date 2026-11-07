#!/usr/bin/env python3
"""Qualify actual retained source values, owner publication and full retirement."""
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
from qemu_control import stopped_status

LAB=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("dispatch_gate",LAB/"dispatch_gate.py")
DISPATCH=importlib.util.module_from_spec(spec);spec.loader.exec_module(DISPATCH)
GATE,LIBRARY=DISPATCH.GATE,DISPATCH.LIBRARY


def points(elf,library):
    def address(line):
        match=re.search(r"0x([0-9a-f]+)\s+<",line)
        if not match:raise GATE.GateError("retained witness lacks an instruction address")
        return int(match[1],16)
    lines=DISPATCH.disassemble(elf,"topal_x86_systems_static_image_process_enter")
    before=[address(line) for line in lines if re.search(r"\bmov\s+0x2008\(%r15\),%r8",line)]
    clean=[address(line) for i,line in enumerate(lines[:-1]) if re.search(r"\bmov\s+%rax,%r14",line)
           and re.search(r"\bmov\s+%r14,0x2010\(%r15\)",lines[i+1])]
    cursor=[i for i,line in enumerate(lines) if re.search(r"\bmov\s+%rdx,0x2008\(%r15\)",line)]
    if len(cursor)!=1:raise GATE.GateError("retained witness needs one native cursor publication")
    committed=[address(line) for line in lines[cursor[0]+1:] if re.search(r"%gs:0x100,\s*%r11",line)][:1]
    bridge=DISPATCH.disassemble(elf,"topal_kernel_retained_process_identity_policy_v1")
    call=[address(line) for line in bridge if re.search(r"\bcall\s+.*<topal\.identity\.",line)]
    result=[address(bridge[i+1]) for i,line in enumerate(bridge[:-1]) if re.search(r"\bcall\s+.*<topal\.identity\.",line)]
    constructor=[address(line) for line in bridge if re.search(r"\bcall\s+.*<topal\.pid\.runtime\.int\.from\.u64",line)]
    helpers=[f["symbol"] for f in library["functions"] if "_2efollowing." in f["symbol"]]
    if len(helpers)!=1:raise GATE.GateError("retained witness needs one imported successor helper")
    source=[address(line) for line in DISPATCH.disassemble(elf,helpers[0])
            if re.search(r"\bcall\s+.*<topal\.pid\.runtime\.int\.compare",line)]
    if any(len(values)!=1 for values in (before,clean,committed,call,result,constructor,source)):
        raise GATE.GateError("retained witness lacks unique publication/source boundaries")
    return dict(before=before[0],clean=clean[0],committed=committed[0],source_call=call[0],
        source_result=result[0],initial_constructor=constructor[0],source=source[0])


def observe(root,values,artifact,timeout,wrap=False,counterexample=False):
    root.mkdir(parents=True,exist_ok=True)
    elf=values[3].parent/"kernel.elf";library=artifact["process_identity_library"]
    if library["interface"]!="retained-process-identity-v1" or library["retained_state"] is None:
        raise GATE.GateError("retained witness needs its exact owned-state interface")
    debug,result,failed,script=(root/name for name in ("debug.sock","retained.json","failed.txt","retained.gdb"))
    controlled = bool(wrap or counterexample)
    qmp = root / "qmp.sock"
    stopped = []
    # Delay every control to exercise the result-file/teardown race.
    exit_delay = 0.2 if controlled else 0.0
    pause = f"python import time; time.sleep({exit_delay})\n" if controlled else ""
    disconnect = "disconnect" if controlled else "detach"
    config=dict(result=str(result),failed=str(failed),elf=str(elf),**points(elf,library),
        readonly=DISPATCH.readonly_sections(elf),field_order=library["result_field_order"],
        expected_order=[1,0] if "(next is following, allocated is allocated)" in (values[3].parent/"process-identity-library.t").read_text() else [0,1],
        stack_bound=library["maximum_stack_bytes"],allocation_bound=library["allocation"]["maximum_bytes"],
        has_time=artifact.get("time_library") is not None,has_image=artifact.get("image_library") is not None,
        dispatch_export=(artifact.get("dispatch_library") or {}).get("native_export","topal_kernel_static_linux_dispatch_policy_v1"), has_dispatch=artifact.get("dispatch_library") is not None,wrap=wrap,counterexample=counterexample)
    script.write_text(f'''set pagination off
file {elf}
target remote {debug}
python
import sys
sys.path.insert(0,{str(LAB)!r})
import retained_inspect
retained_inspect.install({config!r})
end
continue
{pause}{disconnect}
''')
    outcome=[]
    def inspect():
        try:
            until=time.monotonic()+5
            while not debug.exists() and time.monotonic()<until:time.sleep(.02)
            run=subprocess.run(["gdb","-q","-batch","-x",str(script)],stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,timeout=timeout+5)
            (root/"debug.log").write_bytes(run.stdout);outcome.append(run.returncode)
        except BaseException as error:outcome.append(str(error))
    original_build,original_command,original_observe=GATE.build_image,GATE.qemu_command,GATE.observe_gate
    original_markers = GATE.MARKERS
    thread=threading.Thread(target=inspect,daemon=True)
    def command(*args):
        thread.start()
        command = original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off"]
        if controlled:
            command += ["-qmp", f"unix:{qmp},server=on,wait=off"]
        return command
    def failure():
        if failed.exists():return "retained debugger assertion failed: "+failed.read_text()
        if outcome and outcome != [0]:
            failed.write_text("debugger exited unsuccessfully: "+repr(outcome))
            return failed.read_text()
        if controlled and result.exists() and outcome == [0]:
            try:
                if not stopped: stopped.append(stopped_status(qmp))
            except BaseException as error:
                failed.write_text(repr(error)); return failed.read_text()
            return "retained observation stopped before publication"
        return None
    GATE.build_image=lambda *_a,**_k:values;GATE.qemu_command=command
    GATE.observe_gate=lambda *a,**k:original_observe(*a,**k,failure_check=failure)
    if controlled:
        GATE.MARKERS = (*original_markers, b"RETAINED_CONTROL_REQUIRES_DEBUGGER_DISCONNECT")
    runtime=None
    try:
        try:runtime=GATE.verify(timeout,False)
        except GATE.GateError as error:
            thread.join(timeout=5)
            if not (wrap or counterexample) or not result.exists() or failed.exists():
                log=(root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
                raise GATE.GateError(str(error)+"\nRetained debugger:\n"+log) from error
        else:
            if wrap or counterexample:raise GATE.GateError("controlled retained run unexpectedly completed")
    finally:
        GATE.build_image,GATE.qemu_command,GATE.observe_gate=original_build,original_command,original_observe
        GATE.MARKERS = original_markers
    thread.join(timeout=5)
    if outcome!=[0] or failed.exists() or not result.exists():
        log = (root/"debug.log").read_text() if (root/"debug.log").exists() else "no debugger log"
        raise GATE.GateError("retained debugger did not finish: "+repr(outcome)+"\n"+log)
    observation=json.loads(result.read_text())
    expected="expected-retained-transition-rejection" if counterexample else "pass-controlled-wrap" if wrap else "pass"
    if observation["result"]!=expected:raise GATE.GateError("retained witness missed its exact result")
    record = dict(runtime=runtime,observation=observation,linked_kernel_sha256=GATE.digest(elf))
    if controlled:
        if not stopped: raise GATE.GateError("controlled retained guest lacks stopped-state evidence")
        record.update(debugger_completed_before_teardown=True, debugger_exit_delay_seconds=exit_delay, stopped_target=stopped[0])
    return record


def verify_retained(timeout=60,write_evidence=False):
    with tempfile.TemporaryDirectory(prefix="topal-retained-",dir="/tmp") as temporary:
        root=Path(temporary);values=GATE.build_image(root/"artifact");artifact=json.loads(values[3].read_text())
        primary=observe(root/"primary",values,artifact,timeout)
        record=dict(schema="topal-retained-source-identity-qemu/1",harness_sha256=GATE.digest(Path(__file__)), qmp_control_sha256=GATE.digest(LAB/"qemu_control.py"),
            inspector_sha256=GATE.digest(LAB/"retained_inspect.py"),library=artifact["process_identity_library"],primary=primary,
            module_publication=LIBRARY.module_publication(values[3].parent,artifact,"process_identity_library","kernel-process-identity-source-module:"))
        record["wrap"]=observe(root/"wrap",values,artifact,timeout,wrap=True)
        for kind in ("reordered","wrong-successor"):
            case=root/kind;DISPATCH.copy_libraries(case)
            source=case/("policy/process-identity.t" if kind=="reordered" else "support/process-identity.t")
            text=source.read_text()
            if kind=="reordered":
                changed=text.replace("Record (allocated : Nat, next : Nat)","Record (next : Nat, allocated : Nat)")
                changed=changed.replace("(allocated is allocated, next is following)","(next is following, allocated is allocated)")
            else:changed=text.replace("false then allocated + one","false then allocated")
            if changed==text:raise GATE.GateError("retained variant did not change source")
            source.write_text(changed)
            changed_values=GATE.build_image(case/"artifact",library_root=case);changed_artifact=json.loads(changed_values[3].read_text())
            DISPATCH.same_native_inputs(artifact,changed_artifact)
            case_proof=observe(case/"observation",changed_values,changed_artifact,timeout,counterexample=kind=="wrong-successor")
            case_proof.update(library=changed_artifact["process_identity_library"],native_root_and_provider_unchanged=True)
            record["order_variant" if kind=="reordered" else "counterexample"]=case_proof
        original_source=GATE.SOURCE_PATH;only=original_source.read_text()
        for slot in ("time-library","image-library","dispatch-library"):only=DISPATCH.without_import(only,slot)
        source=root/"retained-only-root.t";source.write_text(only);GATE.SOURCE_PATH=source
        try:
            only_values=GATE.build_image(root/"retained-only-artifact");only_artifact=json.loads(only_values[3].read_text())
            if any(only_artifact.get(key) is not None for key in ("time_library","image_library","dispatch_library")):
                raise GATE.GateError("retained-only control retained another numeric policy")
            single=observe(root/"retained-only",only_values,only_artifact,timeout)
            single.update(library=only_artifact["process_identity_library"],other_numeric_libraries_absent=True,root_source=only)
            record["retained_only"]=single
        finally:GATE.SOURCE_PATH=original_source
    if write_evidence:
        (LAB/"results/topal-retained-source-identity-gate.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
    print("PASS actual retained source graphs, cleanup before publication, version retirement and full reclamation",flush=True)
    return record


def verify(timeout=60,write_evidence=False):
    original=GATE.SOURCE_PATH
    with tempfile.TemporaryDirectory(prefix="topal-retained-v1-",dir="/tmp") as temporary:
        source=Path(temporary)/"retained-root.t"
        text=original.read_text().replace("structured-process-identity-v1","retained-process-identity-v1").replace("policy/structured-process-identity.t","policy/process-identity.t")
        source.write_text(text);GATE.SOURCE_PATH=source
        try:record=verify_retained(timeout,False)
        finally:GATE.SOURCE_PATH=original
    record["retained_compatibility_control"]=True
    if write_evidence:
        (LAB/"results/topal-retained-source-identity-gate.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
    return record


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout",type=int,default=60);parser.add_argument("--write-evidence",action="store_true")
    args=parser.parse_args();verify(args.timeout,args.write_evidence)
