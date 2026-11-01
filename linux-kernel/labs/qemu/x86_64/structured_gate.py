#!/usr/bin/env python3
"""Qualify source-produced nested values, full history and owned publication."""
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

LAB=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("retained_gate",LAB/"retained_gate.py")
BASE=importlib.util.module_from_spec(spec);spec.loader.exec_module(BASE)
GATE,DISPATCH,LIBRARY=BASE.GATE,BASE.DISPATCH,BASE.LIBRARY
EXPORT="topal_kernel_structured_process_identity_policy_v1"


def points(elf,library):
    def address(line):
        match=re.search(r"0x([0-9a-f]+)\s+<",line)
        if not match:raise GATE.GateError("structured witness lacks an instruction address")
        return int(match[1],16)
    lines=DISPATCH.disassemble(elf,"topal_x86_systems_static_image_process_enter")
    before=[address(line) for line in lines if re.search(r"\bmov\s+0x2008\(%r15\),%r8",line)]
    clean=[address(line) for i,line in enumerate(lines[:-1]) if re.search(r"\bmov\s+%rax,%r14",line)
        and re.search(r"\bmov\s+%r14,0x2010\(%r15\)",lines[i+1])]
    cursor=[i for i,line in enumerate(lines) if re.search(r"\bmov\s+%rdx,0x2008\(%r15\)",line)]
    if len(cursor)!=1:raise GATE.GateError("structured witness needs one native cursor publication")
    committed=[address(line) for line in lines[cursor[0]+1:] if re.search(r"%gs:0x100,\s*%r11",line)][:1]
    bridge=DISPATCH.disassemble(elf,EXPORT)
    call=[address(line) for line in bridge if re.search(r"\bcall\s+.*<topal\.identity\.",line)]
    result=[address(bridge[i+1]) for i,line in enumerate(bridge[:-1]) if re.search(r"\bcall\s+.*<topal\.identity\.",line)]
    helpers=[f["symbol"] for f in library["functions"] if "support_2eprocess_2didentity_2efollowing." in f["symbol"]]
    if len(helpers)!=1:raise GATE.GateError("structured witness needs its imported PID helper")
    source=[address(line) for line in DISPATCH.disassemble(elf,helpers[0])
        if re.search(r"\bcall\s+.*<topal\.pid\.runtime\.int\.compare",line)]
    if any(len(values)!=1 for values in (before,clean,committed,call,result,source)):
        raise GATE.GateError("structured witness lacks unique source/publication boundaries")
    return dict(before=before[0],clean=clean[0],committed=committed[0],source_call=call[0],source_result=result[0],source=source[0])


def observe(root,values,artifact,timeout,mode=None,counterexample=False,reordered=False):
    root.mkdir(parents=True,exist_ok=True)
    elf=values[3].parent/"kernel.elf";library=artifact["process_identity_library"]
    if library["interface"]!="structured-process-identity-v1" or library["retained_state"]["maximum_retained_graph_bytes"]!=88:
        raise GATE.GateError("structured witness needs its exact qualified whole-value schema")
    debug,result,failed,script=(root/n for n in ("debug.sock","structured.json","failed.txt","structured.gdb"))
    config=dict(result=str(result),failed=str(failed),elf=str(elf),**points(elf,library),
        readonly=DISPATCH.readonly_sections(elf),export=EXPORT,mode=mode,counterexample=counterexample,
        expected_outer=[1,0] if reordered else [0,1],expected_state=[0,1] if reordered else [1,0],
        expected_history=[1,0] if reordered else [0,1],stack_bound=library["maximum_stack_bytes"],
        allocation_bound=library["allocation"]["maximum_bytes"],
        has_time=artifact.get("time_library") is not None,has_image=artifact.get("image_library") is not None,
        has_dispatch=artifact.get("dispatch_library") is not None)
    script.write_text(f'''set pagination off
file {elf}
target remote {debug}
python
import sys
sys.path.insert(0,{str(LAB)!r})
import structured_inspect
structured_inspect.install({config!r})
end
continue
detach
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
    thread=threading.Thread(target=inspect,daemon=True)
    def command(*args):
        thread.start();return original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off"]
    def failure():
        if failed.exists():return "structured debugger assertion failed: "+failed.read_text()
        if (mode or counterexample) and result.exists():return "structured observation stopped before publication"
        return None
    GATE.build_image=lambda *_a,**_k:values;GATE.qemu_command=command
    GATE.observe_gate=lambda *a,**k:original_observe(*a,**k,failure_check=failure)
    runtime=None
    try:
        try:runtime=GATE.verify(timeout,False)
        except GATE.GateError as error:
            thread.join(timeout=5)
            if not (mode or counterexample) or not result.exists() or failed.exists():
                log=(root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
                raise GATE.GateError(str(error)+"\nStructured debugger:\n"+log) from error
        else:
            if mode or counterexample:raise GATE.GateError("controlled structured run unexpectedly completed")
    finally:GATE.build_image,GATE.qemu_command,GATE.observe_gate=original_build,original_command,original_observe
    thread.join(timeout=5)
    if outcome!=[0] or failed.exists() or not result.exists():raise GATE.GateError("structured debugger did not finish: "+repr(outcome))
    observation=json.loads(result.read_text())
    expected="expected-structured-history-rejection" if counterexample else "pass-controlled-"+mode if mode else "pass"
    if observation["result"]!=expected:raise GATE.GateError("structured witness missed its exact result")
    return dict(runtime=runtime,observation=observation,linked_kernel_sha256=GATE.digest(elf))


def verify(timeout=60,write_evidence=False):
    with tempfile.TemporaryDirectory(prefix="topal-structured-",dir="/tmp") as temporary:
        root=Path(temporary);values=GATE.build_image(root/"artifact");artifact=json.loads(values[3].read_text())
        record=dict(schema="topal-structured-source-identity-qemu/1",harness_sha256=GATE.digest(Path(__file__)),
            inspector_sha256=GATE.digest(LAB/"structured_inspect.py"),library=artifact["process_identity_library"],
            primary=observe(root/"primary",values,artifact,timeout),
            module_publication=LIBRARY.module_publication(values[3].parent,artifact,"process_identity_library","kernel-process-identity-source-module:"))
        for mode in ("wrap","count"):
            record[mode]=observe(root/mode,values,artifact,timeout,mode=mode)
        for kind in ("reordered","wrong-history"):
            case=root/kind;DISPATCH.copy_libraries(case)
            unit=case/"policy/structured-process-identity.t";helper=case/"support/identity-history.t"
            if kind=="reordered":
                text=unit.read_text();state="Record (next : Nat, history : Record (creations : Nat, wrapped : Boolean))"
                text=text.replace(f"-> Record (allocated : Nat, state : {state})",f"-> Record (state : {state}, allocated : Nat)")
                text=text.replace("(allocated is allocated, state is (next is following, history is history))","(state is (next is following, history is history), allocated is allocated)")
                text=text.replace(state,"Record (history : Record (creations : Nat, wrapped : Boolean), next : Nat)")
                text=text.replace("(next is following, history is history)","(history is history, next is following)")
                text=text.replace("Record (creations : Nat, wrapped : Boolean)","Record (wrapped : Boolean, creations : Nat)")
                if text==unit.read_text():raise GATE.GateError("structured order variant did not change source")
                unit.write_text(text)
                text=helper.read_text().replace("Record (creations : Nat, wrapped : Boolean)","Record (wrapped : Boolean, creations : Nat)")
                helper.write_text(text.replace("(creations is creations, wrapped is wrapped)","(wrapped is wrapped, creations is creations)"))
            else:
                text=helper.read_text().replace("one : Nat is 1","one : Nat is 2")
                if text==helper.read_text():raise GATE.GateError("structured history counterexample did not change its helper")
                helper.write_text(text)
            changed_values=GATE.build_image(case/"artifact",library_root=case);changed_artifact=json.loads(changed_values[3].read_text())
            DISPATCH.same_native_inputs(artifact,changed_artifact)
            proof=observe(case/"observation",changed_values,changed_artifact,timeout,
                counterexample=kind=="wrong-history",reordered=kind=="reordered")
            proof.update(library=changed_artifact["process_identity_library"],native_root_and_provider_unchanged=True)
            record["order_variant" if kind=="reordered" else "counterexample"]=proof
        original_source=GATE.SOURCE_PATH;only=original_source.read_text()
        for slot in ("time-library","image-library","dispatch-library"):only=DISPATCH.without_import(only,slot)
        source=root/"structured-only-root.t";source.write_text(only);GATE.SOURCE_PATH=source
        try:
            only_values=GATE.build_image(root/"structured-only-artifact");only_artifact=json.loads(only_values[3].read_text())
            if any(only_artifact.get(key) is not None for key in ("time_library","image_library","dispatch_library")):
                raise GATE.GateError("structured-only control retained another numeric policy")
            single=observe(root/"structured-only",only_values,only_artifact,timeout)
            single.update(library=only_artifact["process_identity_library"],other_numeric_libraries_absent=True,root_source=only)
            record["structured_only"]=single
        finally:GATE.SOURCE_PATH=original_source
    if write_evidence:
        (LAB/"results/topal-structured-source-identity-gate.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
    print("PASS whole source state, nested order, full numeric history, owned publication and complete retirement",flush=True)
    return record


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout",type=int,default=60);parser.add_argument("--write-evidence",action="store_true")
    args=parser.parse_args();verify(args.timeout,args.write_evidence)
