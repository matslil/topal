#!/usr/bin/env python3
"""Observe copied ELF header policy, refusal atomicity and independent ownership."""
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

LAB = Path(__file__).resolve().parent


def module(name):
    spec = importlib.util.spec_from_file_location(name, LAB/(name+".py"))
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


GATE = module("topal_gate")
NUMERIC = module("numeric_gate")
LIBRARY = module("library_gate")


def points(elf):
    lines = subprocess.check_output(["gdb", "-q", "-batch", "-ex", f"file {elf}",
        "-ex", "disassemble /r topal_x86_systems_static_image_process_enter"], text=True).splitlines()
    def address(line):
        match = re.search(r"0x([0-9a-f]+)\s+<", line)
        if not match:
            raise GATE.GateError("header witness has no instruction address")
        return int(match[1], 16)
    starts = [i for i, line in enumerate(lines[:-1])
              if re.search(r"%gs:0x3d0,\s*%rax", line)
              and re.search(r"\bcmp\s+\$0x0,\s*%rax", lines[i+1])]
    if len(starts) != 1:
        raise GATE.GateError("header witness needs one private image binding")
    tails = [i for i in range(starts[0]+1, len(lines)-2)
             if re.search(r"\bmov\s+%r8,\s*%rax", lines[i])
             and re.search(r"\$0x0,\s*%r8", lines[i+1])
             and re.search(r"\btest\s+%rax,\s*%rax", lines[i+2])]
    if not tails:
        raise GATE.GateError("header witness cannot identify the cleaned result")
    clean = tails[0]+2
    errors = [i for i in range(clean+1, min(clean+80, len(lines)-1))
              if re.search(r"\$0xfffffffffffffff8,\s*%rax", lines[i])]
    if len(errors) != 1:
        raise GATE.GateError("header witness needs one precommit ENOEXEC boundary")
    return address(lines[clean]), address(lines[errors[0]+1])


def payload_offset(path):
    """Read the actual newc file, independently of the provider's fixed offset."""
    data = Path(path).read_bytes()
    cursor = 0
    while cursor+110 <= len(data):
        header = data[cursor:cursor+110]
        if header[:6] != b"070701":
            raise GATE.GateError("header witness needs canonical newc")
        size, namesize = int(header[54:62], 16), int(header[94:102], 16)
        name_end = cursor+110+namesize
        if not namesize or name_end > len(data):
            raise GATE.GateError("header witness has malformed archive name")
        name = data[cursor+110:name_end]
        payload = (name_end+3)&~3
        if payload+size > len(data) or name[-1:] != b"\0":
            raise GATE.GateError("header witness has malformed archive payload")
        if name[:-1] == b"static-user.elf":
            return payload
        if name[:-1] == b"TRAILER!!!":
            break
        cursor = (payload+size+3)&~3
    raise GATE.GateError("header witness cannot find its owned file")


def observe(root, values, artifact, timeout, poison=False, counterexample=False):
    root.mkdir(parents=True, exist_ok=True)
    original_build, original_command, original_observe = GATE.build_image, GATE.qemu_command, GATE.observe_gate
    elf = values[3].parent/"kernel.elf"
    library = artifact["image_library"]
    clean, refused = points(elf)
    debug, result, script = root/"debug.sock", root/"header.json", root/"header.gdb"
    source = (GATE.LINUX_KERNEL/"kernel/support/elf.t").read_text().splitlines()
    config = dict(result=str(result), clean=clean, refused=refused, poison=poison,
                  counterexample=counterexample, source_line=source.index("  magic is read-magic bytes")+1,
                  header=list(values[5].read_bytes()[:64]), payload_offset=payload_offset(values[1]),
                  has_time=artifact.get("time_library") is not None,
                  has_identity=artifact.get("process_identity_library") is not None, has_dispatch=artifact.get("dispatch_library") is not None,
                  zero_addresses=NUMERIC.immutable_zero_addresses(elf),
                  stack_bound=library["maximum_stack_bytes"],
                  allocation_bound=library["allocation"]["maximum_bytes"])
    script.write_text(f'''set pagination off
file {elf}
target remote {debug}
python
import sys
sys.path.insert(0,{str(LAB)!r})
import header_inspect
header_inspect.install({config!r})
end
continue
detach
''')
    outcome = []
    def inspect():
        try:
            until = time.monotonic()+5
            while not debug.exists() and time.monotonic() < until:
                time.sleep(.02)
            run = subprocess.run(["gdb", "-q", "-batch", "-x", str(script)],
                                 stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+5)
            (root/"debug.log").write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            outcome.append(str(error))
    thread = threading.Thread(target=inspect, daemon=True)
    def command(*args):
        thread.start()
        return original_command(*args)+["-S", "-gdb", f"unix:{debug},server=on,wait=off"]
    stop = poison or counterexample
    reason = "independent source header observation complete"
    def failure():
        return reason if result.exists() else None
    GATE.build_image = lambda *_a, **_k: values
    GATE.qemu_command = command
    if stop:
        GATE.observe_gate = lambda *a, **k: original_observe(*a, **k, failure_check=failure)
    runtime = None
    try:
        try:
            runtime = GATE.verify(timeout, False)
        except GATE.GateError as error:
            thread.join(timeout=5)
            # A controlled stop writes its exact boundary proof before GDB
            # detaches. QEMU may finish before the monitor next polls the file;
            # accept only that proof, which is validated below with GDB status.
            if not stop or not result.exists():
                log = (root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
                raise GATE.GateError(str(error)+"\nHeader debugger:\n"+log) from error
        else:
            if stop:
                raise GATE.GateError("controlled header observation unexpectedly completed the user fixture")
    finally:
        GATE.build_image, GATE.qemu_command, GATE.observe_gate = original_build, original_command, original_observe
    thread.join(timeout=5)
    if outcome != [0] or not result.exists():
        log = (root/"debug.log").read_text() if (root/"debug.log").exists() else repr(outcome)
        raise GATE.GateError("header debugger did not finish: "+repr(outcome)+"\n"+log)
    observation = json.loads(result.read_text())
    expected = "expected-header-rejection" if counterexample else "pass-controlled-refusal" if poison else "pass"
    if observation["result"] != expected:
        raise GATE.GateError("header witness missed its specific result")
    return dict(runtime=runtime, observation=observation, linked_kernel_sha256=GATE.digest(elf))


def verify(timeout=30, write_evidence=False):
    with tempfile.TemporaryDirectory(prefix="topal-header-", dir="/tmp") as directory:
        root = Path(directory)
        values = GATE.build_image(root/"artifact")
        artifact = json.loads(values[3].read_text())
        primary = observe(root/"primary", values, artifact, timeout)
        controlled = observe(root/"controlled-refusal", values, artifact, timeout, poison=True)
        record = dict(schema="topal-source-elf64-header-qemu/1",
                      harness_sha256=GATE.digest(Path(__file__)), inspector_sha256=GATE.digest(LAB/"header_inspect.py"),
                      library=artifact["image_library"], primary=primary, controlled_refusal=controlled,
                      module_publication=LIBRARY.module_publication(values[3].parent, artifact,
                                                                   "image_library", "kernel-image-source-module:"))
        case = root/"wrong-magic"
        shutil.copytree(GATE.LINUX_KERNEL/"kernel/policy", case/"policy")
        shutil.copytree(GATE.LINUX_KERNEL/"kernel/support", case/"support")
        LIBRARY.copy_architecture_libraries(case)
        source = case/"support/elf.t"
        before = source.read_text()
        after = before.replace("  magic = 1179403647\n    true then check-ident (bytes, machine, maximum)\n    false then false",
                               "  check-ident (bytes, machine, maximum)")
        if after == before:
            raise GATE.GateError("changed magic helper did not change source")
        source.write_text(after)
        changed_values = GATE.build_image(case/"artifact", library_root=case)
        changed = json.loads(changed_values[3].read_text())
        for prefix in ("topal.systems-root-object", "topal.provider-object"):
            old = [i for i in artifact["inputs"] if i["identity"].startswith(prefix)]
            new = [i for i in changed["inputs"] if i["identity"].startswith(prefix)]
            if len(old) != 1 or old != new:
                raise GATE.GateError("changed header helper changed native root/provider")
        counter = observe(case/"counterexample", changed_values, changed, timeout, poison=True, counterexample=True)
        counter.update(library=changed["image_library"], native_root_and_provider_unchanged=True)
        record["counterexample"] = counter

        original_source = GATE.SOURCE_PATH
        text = original_source.read_text()
        start = text.index("  time-library is")
        end = text.index("  image-library is", start)
        only = root/"header-only-root.t"
        only_text=text[:start]+text[end:]
        dispatch_start=only_text.index("  dispatch-library is")
        dispatch_end=only_text.index("  bootstrap-storage is",dispatch_start)
        only_text=only_text[:dispatch_start]+only_text[dispatch_end:]
        only.write_text(only_text)
        GATE.SOURCE_PATH = only
        try:
            only_values = GATE.build_image(root/"header-only-artifact")
            only_artifact = json.loads(only_values[3].read_text())
            if any(only_artifact.get(key) is not None for key in ("time_library","dispatch_library","process_identity_library")):
                raise GATE.GateError("header-only control retained another numeric policy")
            single = observe(root/"header-only", only_values, only_artifact, timeout)
            single.update(library=only_artifact["image_library"], time_library_absent=True,
                          dispatch_library_absent=True, root_source=only_text)
            record["header_only"] = single
        finally:
            GATE.SOURCE_PATH = original_source
    if write_evidence:
        path = LAB/"results/topal-source-elf64-header-gate.json"
        path.write_text(json.dumps(record, indent=2, sort_keys=True)+"\n")
    print("PASS source header bytes, independent refusal, shared/header-only ownership and full erasure", flush=True)
    return record


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=30)
    parser.add_argument("--write-evidence", action="store_true")
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence)
