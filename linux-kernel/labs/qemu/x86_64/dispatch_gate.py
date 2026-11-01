#!/usr/bin/env python3
"""Observe source syscall facts/tags and cleanup before native handler routing."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import threading
import time

LAB=Path(__file__).resolve().parent


def module(name):
    spec=importlib.util.spec_from_file_location(name,LAB/(name+".py"))
    loaded=importlib.util.module_from_spec(spec);spec.loader.exec_module(loaded)
    return loaded


GATE=module("topal_gate")
LIBRARY=module("library_gate")


def mapping(interface='static-linux-dispatch-v1'):
    specification=(GATE.REPOSITORY/"spec/systems-profile.md").read_text()
    rule=specification.split("### TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001",1)[1]
    table=rule.split("```text\n",1)[1].split("```",1)[0]
    pairs=[(int(number),name) for number,name in re.findall(r"\b(\d+)\s+([A-Za-z]+)",table)]
    if len(pairs)!=55 or len(set(number for number,_ in pairs))!=55:
        raise GATE.GateError("dispatch witness needs the complete independent normative map")
    if interface in ('static-linux-dispatch-v2','static-linux-dispatch-v3'):
        rule=specification.split('### TOPAL-SYSTEMS-LINUX-DISPATCH-V2-001',1)[1].split('### ',1)[0]
        added=[(int(number),name) for number,name in re.findall(r'\b(77|95)\s+(TruncateFile|FileCreationMask)',rule)]
        if len(added)!=2:raise GATE.GateError('extended dispatch witness needs the complete normative additions')
        pairs=sorted(pairs+added)
    elif interface!='static-linux-dispatch-v1':
        raise GATE.GateError('unqualified dispatch version')
    if interface=='static-linux-dispatch-v3':
        rule=specification.split('### TOPAL-SYSTEMS-LINUX-DISPATCH-V3-001',1)[1].split('### ',1)[0]
        if '`Unlink` (x86-64 87)' not in rule or '`UnlinkAt` (263)' not in rule:
            raise GATE.GateError('named dispatch witness needs both normative additions')
        pairs=sorted(pairs+[(87,'Unlink'),(263,'UnlinkAt')])
    return pairs


def disassemble(elf,symbol):
    return subprocess.check_output(["gdb","-q","-batch","-ex",f"file {elf}","-ex",f"disassemble /r '{symbol}'"],text=True).splitlines()


def points(elf,library):
    def address(line):
        match=re.search(r"0x([0-9a-f]+)\s+<",line)
        if not match:raise GATE.GateError("dispatch witness lacks an instruction address")
        return int(match[1],16)
    lines=disassemble(elf,"topal_x86_systems_static_image_process_enter")
    raw=[address(line) for i,line in enumerate(lines[:-1])
         if re.search(r"\bmov\s+%eax,%eax",line) and re.search(r"\bmov\s+%rax,%r8",lines[i+1])]
    clean=[address(lines[i+1]) for i,line in enumerate(lines[:-1])
           if re.search(r"%gs:0x160,\s*%r9",line) and re.search(r"%gs:0x3a0,\s*%r11",lines[i+1])]
    bridge=disassemble(elf,library['native_export'])
    tag=[address(bridge[i+1]) for i,line in enumerate(bridge[:-1])
         if re.search(r"\bcall\s+.*<topal\.dispatch\.",line)]
    if len(raw)!=1 or len(clean)!=1 or len(tag)!=1:
        raise GATE.GateError("dispatch witness needs unique capture/tag/before-handler boundaries")
    leaves=[f["symbol"] for f in library["functions"] if ".select_2drange_2d12_2d12." in f["symbol"]]
    if len(leaves)!=1:raise GATE.GateError("dispatch witness needs one first-startup leaf")
    calls=[re.search(r"<(topal\.dispatch\.[^>]+)>",line).group(1) for line in disassemble(elf,leaves[0])
           if re.search(r"\bcall\s+.*<topal\.dispatch\.",line)]
    if len(calls)!=1:raise GATE.GateError("dispatch witness needs one first imported comparison")
    snapshots=[address(line) for line in disassemble(elf,calls[0])
               if re.search(r"\bcall\s+.*<topal\.syscall\.runtime\.int\.compare",line)]
    if len(snapshots)!=1:raise GATE.GateError("dispatch witness needs one source snapshot point")
    return raw[0],tag[0],clean[0],snapshots[0]


def readonly_sections(elf):
    data=Path(elf).read_bytes()
    if data[:6]!=b"\x7fELF\x02\x01":raise GATE.GateError("dispatch witness needs little-endian ELF64")
    start=struct.unpack_from("<Q",data,40)[0]
    stride,count=struct.unpack_from("<HH",data,58)
    sections=[]
    for index in range(count):
        _,kind,flags,address,offset,size,*_=struct.unpack_from("<IIQQQQIIQQ",data,start+index*stride)
        if kind==1 and flags&7==2:
            if offset+size>len(data):raise GATE.GateError("readonly protocol constant escapes ELF")
            sections.append((address,offset,size))
    return sections


def observe(root,values,artifact,timeout,counterexample=False):
    root.mkdir(parents=True,exist_ok=True)
    original_build,original_command,original_observe=GATE.build_image,GATE.qemu_command,GATE.observe_gate
    elf=values[3].parent/"kernel.elf"
    library=artifact["dispatch_library"]
    raw,tag,clean,snapshot=points(elf,library)
    pairs=mapping(library['interface']);choices=[name for _,name in pairs]+["Unsupported"]
    text=(values[3].parent/"dispatch-library.t").read_text()
    declaration=next(line for line in text.splitlines() if line.startswith("pub StaticLinuxOperation is Enum ("))
    alternatives=declaration.split("Enum (",1)[1][:-1].split(", ")
    if [alternatives.index(name) for name in choices]!=library["source_tags"]:
        raise GATE.GateError("published nominal source order and tag adapter disagree")
    debug,result,script=root/"debug.sock",root/"dispatch.json",root/"dispatch.gdb"
    source=(GATE.LINUX_KERNEL/"kernel/support/syscall.t").read_text().splitlines()
    config=dict(result=str(result),progress=str(root/"progress.json"),elf=str(elf),raw_point=raw,tag_point=tag,clean_point=clean,
        source_point=snapshot,
        readonly=readonly_sections(elf),source_line=source.index("  number = expected")+1,
        mapping=pairs,choices=choices,source_tags=library["source_tags"],
        dispatch_export=library['native_export'],source_basename=Path(library['source']).name,
        has_retained=bool((artifact.get("process_identity_library") or {}).get("retained_state")),identity_export=(artifact.get("process_identity_library") or {}).get("native_export","topal_kernel_process_identity_transition_v1"),has_identity=artifact.get("process_identity_library") is not None,has_time=artifact.get("time_library") is not None,has_image=artifact.get("image_library") is not None,
        stack_bound=library["maximum_stack_bytes"],allocation_bound=library["allocation"]["maximum_bytes"],
        counterexample=counterexample)
    script.write_text(f'''set pagination off
file {elf}
target remote {debug}
python
import sys
sys.path.insert(0,{str(LAB)!r})
import dispatch_inspect
dispatch_inspect.install({config!r})
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
    thread=threading.Thread(target=inspect,daemon=True)
    def command(*args):
        thread.start();return original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off"]
    def failure():return "independent source selection observation complete" if result.exists() else None
    GATE.build_image=lambda *_a,**_k:values;GATE.qemu_command=command
    if counterexample:GATE.observe_gate=lambda *a,**k:original_observe(*a,**k,failure_check=failure)
    runtime=None
    try:
        try:runtime=GATE.verify(timeout,False)
        except GATE.GateError as error:
            thread.join(timeout=5)
            if not counterexample or not result.exists():
                log=(root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
                raise GATE.GateError(str(error)+"\nDispatch debugger:\n"+log) from error
        else:
            if counterexample:raise GATE.GateError("changed selection unexpectedly completed the user fixture")
    finally:GATE.build_image,GATE.qemu_command,GATE.observe_gate=original_build,original_command,original_observe
    thread.join(timeout=5)
    if outcome!=[0] or not result.exists():
        log=(root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
        raise GATE.GateError("dispatch debugger did not finish: "+repr(outcome)+"\n"+log)
    observation=json.loads(result.read_text())
    expected="expected-source-selection-rejection" if counterexample else "pass"
    if observation["result"]!=expected:raise GATE.GateError("dispatch witness missed its exact result")
    return dict(runtime=runtime,observation=observation,linked_kernel_sha256=GATE.digest(elf))


def copy_libraries(directory):
    shutil.copytree(GATE.LINUX_KERNEL/"kernel/policy",directory/"policy")
    shutil.copytree(GATE.LINUX_KERNEL/"kernel/support",directory/"support")
    LIBRARY.copy_architecture_libraries(directory)


def same_native_inputs(original,changed):
    for prefix in ("topal.systems-root-object","topal.provider-object"):
        before=[i for i in original["inputs"] if i["identity"].startswith(prefix)]
        after=[i for i in changed["inputs"] if i["identity"].startswith(prefix)]
        if len(before)!=1 or before!=after:raise GATE.GateError("source selection changed native root/provider objects")


def without_import(text,slot):
    start=text.index("  "+slot+" is")
    end=text.index("\n  ),\n",start)+6
    return text[:start]+text[end:]


def verify(timeout=180,write_evidence=False):
    with tempfile.TemporaryDirectory(prefix="topal-dispatch-",dir="/tmp") as directory:
        root=Path(directory)
        values=GATE.build_image(root/"artifact");artifact=json.loads(values[3].read_text())
        primary=observe(root/"primary",values,artifact,timeout)
        record=dict(schema="topal-source-linux-dispatch-qemu/1",harness_sha256=GATE.digest(Path(__file__)),
            inspector_sha256=GATE.digest(LAB/"dispatch_inspect.py"),library=artifact["dispatch_library"],primary=primary,
            module_publication=LIBRARY.module_publication(values[3].parent,artifact,"dispatch_library","kernel-dispatch-source-module:"))
        for kind in ("reordered","wrong-selection"):
            case=root/kind;copy_libraries(case)
            source=case/artifact['dispatch_library']['source'];text=source.read_text()
            if kind=="reordered":
                declaration=next(line for line in text.splitlines() if line.startswith("pub StaticLinuxOperation is Enum ("))
                alternatives=declaration.split("Enum (",1)[1][:-1].split(", ")
                changed=text.replace(declaration,"pub StaticLinuxOperation is Enum ("+", ".join(reversed(alternatives))+")",1)
            else:changed=text.replace("    true then ProgramBreak","    true then Unsupported",1)
            if changed==text:raise GATE.GateError("dispatch variant did not change source")
            source.write_text(changed)
            changed_values=GATE.build_image(case/"artifact",library_root=case)
            changed_artifact=json.loads(changed_values[3].read_text());same_native_inputs(artifact,changed_artifact)
            observation=observe(case/"observation",changed_values,changed_artifact,timeout,counterexample=kind=="wrong-selection")
            observation.update(library=changed_artifact["dispatch_library"],native_root_and_provider_unchanged=True)
            record["order_variant" if kind=="reordered" else "counterexample"]=observation
        original_source=GATE.SOURCE_PATH
        only=without_import(without_import(without_import(original_source.read_text(),"time-library"),"image-library"),"process-identity-library")
        only_source=root/"dispatch-only-root.t";only_source.write_text(only);GATE.SOURCE_PATH=only_source
        try:
            only_values=GATE.build_image(root/"dispatch-only-artifact")
            only_artifact=json.loads(only_values[3].read_text())
            if any(only_artifact.get(key) is not None for key in ("time_library","image_library","process_identity_library")):
                raise GATE.GateError("dispatch-only numeric control retained another policy")
            single=observe(root/"dispatch-only",only_values,only_artifact,timeout)
            single.update(library=only_artifact["dispatch_library"],time_and_image_libraries_absent=True,root_source=only)
            record["dispatch_only"]=single
        finally:GATE.SOURCE_PATH=original_source
        for version,source_name,key in [(1,'static-linux-dispatch.t','legacy_compatibility'),(2,'static-linux-dispatch-v2.t','legacy_v2_compatibility')]:
            legacy=original_source.read_text().replace('policy/static-linux-dispatch-v3.t','policy/'+source_name).replace('interface is static-linux-dispatch-v3',f'interface is static-linux-dispatch-v{version}')
            if legacy==original_source.read_text():raise GATE.GateError('legacy dispatch control did not select the frozen interface')
            legacy_source=root/f'legacy-dispatch-v{version}-root.t';legacy_source.write_text(legacy);GATE.SOURCE_PATH=legacy_source
            try:
                legacy_values=GATE.build_image(root/f'legacy-dispatch-v{version}-artifact')
                legacy_artifact=json.loads(legacy_values[3].read_text())
                if legacy_artifact['dispatch_library']['interface']!=f'static-linux-dispatch-v{version}':
                    raise GATE.GateError('legacy control lost its explicit source interface')
                compatibility=observe(root/f'legacy-dispatch-v{version}',legacy_values,legacy_artifact,timeout)
                compatibility.update(library=legacy_artifact['dispatch_library'],explicit_legacy_dispatch_control=True,root_source=legacy)
                record[key]=compatibility
            finally:GATE.SOURCE_PATH=original_source
    if write_evidence:
        (LAB/"results/topal-source-linux-dispatch-gate.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
    print("PASS actual source selectors/tags, independent native adaptation, before-handler cleanup and policy lifetime",flush=True)
    return record


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout",type=int,default=180);parser.add_argument("--write-evidence",action="store_true")
    args=parser.parse_args();verify(args.timeout,args.write_evidence)
