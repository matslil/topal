#!/usr/bin/env python3
"""Observe native sleep targets with explicit QTest HPET test inputs."""
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
SPEC=importlib.util.spec_from_file_location("topal_gate",LAB/"topal_gate.py")
GATE=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)
MAX=(1<<63)-1
CASES=(("conversion-saturation","9223372036",9223372036,0),
       ("addition-saturation","9223372035",9223372035,0),
       ("resolution-rounding","0.000000001",0,1))


def points(elf):
    lines=subprocess.check_output(["gdb","-q","-batch","-ex",f"file {elf}",
          "-ex","disassemble /r topal_x86_systems_static_image_process_enter"],text=True).splitlines()
    def address(line):
        match=re.search(r"0x([0-9a-f]+)\s+<",line)
        if not match:
            raise GATE.GateError("missing timing instruction address")
        return int(match[1],16)
    samples=[address(line) for line in lines if re.search(r"%gs:0x3b0,\s*%rdi",line)]
    starts=[i for i,line in enumerate(lines) if re.search(r"%gs:0x3c0,\s*%rax",line)]
    if len(samples)!=1 or len(starts)!=1:
        raise GATE.GateError("cannot identify unique private clock binding/target")
    targets=[address(line) for line in lines[starts[0]+1:] if "pause" in line]
    if not targets:
        raise GATE.GateError("cannot identify relative wait boundary")
    return samples[0],targets[0]


def commands(elf,debug,qtest,result,sample,target,seconds,nanos):
    setup=f'''
import gdb, json, socket
from pathlib import Path
inferior=gdb.selected_inferior()
base=int(gdb.parse_and_eval("&topal_bootstrap_storage"))
clock=base+24
context=int(gdb.parse_and_eval("$gs_base"))
assert int.from_bytes(inferior.read_memory(context+944,8),"little")==clock
assert bytes(inferior.read_memory(clock,24))==bytes(24)
client=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
client.settimeout(5)
client.connect({str(qtest)!r})
reader=client.makefile("rb")
def request(command):
    client.sendall(command.encode()+b"\\n")
    reply=reader.readline().decode().strip()
    assert reply.startswith("OK"),(command,reply)
    return reply.split()[1:]
def counter():
    return int(request("readq 0xfed000f0")[0],0)
def set_counter(value):
    control=int(request("readq 0xfed00010")[0],0)
    request("writeq 0xfed00010 "+hex(control&~1))
    request("writeq 0xfed000f0 "+hex(value))
    request("writeq 0xfed00010 "+hex(control))
    assert int(request("readq 0xfed00010")[0],0)==control
    return control
before=counter()
initial=before+300000000
initial_control=set_counter(initial)
installed=counter()
assert initial<=installed<=initial+100000
initial_setup={{"hardware_counter_before":before,"hardware_counter_requested":initial,
               "hardware_counter_observed":installed,"borrowed_root_clock_binding":True,
               "initial_root_clock_state_zero":True,"initial_hpet_configuration_restored":initial_control}}
gdb.execute("delete breakpoints")
gdb.execute("hbreak *{target}")
gdb.execute("continue")
assert int(gdb.parse_and_eval("$rip"))=={target}
actual_seconds=int(gdb.parse_and_eval("$rbx"))
actual_nanos=int(gdb.parse_and_eval("$rbp"))
assert (actual_seconds,actual_nanos)==({seconds},{nanos}),(actual_seconds,actual_nanos)
duration={MAX} if actual_seconds>=9223372036 else actual_seconds*1000000000+actual_nanos
raw=int.from_bytes(inferior.read_memory(clock,8),"little")
epoch=int.from_bytes(inferior.read_memory(clock+8,8),"little")
assert epoch==0
assert int(gdb.parse_and_eval("$r13"))==raw
assert int(gdb.parse_and_eval("$r15"))==0
arena=int.from_bytes(inferior.read_memory(context+968,8),"little")
assert arena==int(gdb.parse_and_eval("&topal_x86_systems_numeric_arena"))
assert bytes(inferior.read_memory(arena,65536))==bytes(65536)
stack=int(gdb.parse_and_eval("$rsp"))
assert bytes(inferior.read_memory(stack-4096,4096))==bytes(4096)
expected=(min((epoch*(1<<64)+raw)*10+duration,{MAX})+9)//10
actual=int(gdb.parse_and_eval("$r14"))
assert actual==expected,(actual,expected)
if {seconds}>0:
    assert actual==({MAX}+9)//10
else:
    assert actual==raw+1
names=("rax","rbx","rcx","rdx","rbp","rsi","rdi","rsp","r8","r9","r10","r11","r12","r13","r14","r15","rip","eflags")
registers={{name:int(gdb.parse_and_eval("$"+name)) for name in names}}
advance=None
advance_control=None
if {seconds}>0:
    advance_control=set_counter(actual)
    advance=counter()
    assert actual<=advance<=actual+100000
assert {{name:int(gdb.parse_and_eval("$"+name)) for name in names}}==registers
record=dict(initial_setup,seconds=actual_seconds,nanoseconds=actual_nanos,
            converted_duration_nanoseconds=duration,deadline_sample_ticks=raw,
            deadline_sample_epoch=epoch,expected_deadline_ticks=expected,
            observed_deadline_ticks=actual,advanced_counter_observed=advance,
            hardware_advance_only_for_saturated_wait={seconds}>0,
            setup_registers_unchanged=True,advance_hpet_configuration_restored=advance_control,result="pass")
client.close()
gdb.execute("delete breakpoints")
gdb.execute("detach")
Path({str(result)!r}).write_text(json.dumps(record,indent=2,sort_keys=True)+"\\n")
'''
    return (f"set pagination off\nfile {elf}\ntarget remote {debug}\n"
            f"hbreak *{sample}\ncontinue\npython\n{setup}\nend\n")


def verify(external,timeout,write_evidence):
    original_build=GATE.build_image
    original_command=GATE.qemu_command
    records=[]
    with tempfile.TemporaryDirectory(prefix="topal-time-",dir="/tmp") as directory:
        root=Path(directory)
        values=original_build(root/"artifact",external)
        sample,target=points(root/"artifact/linked/kernel.elf")
        GATE.build_image=lambda *_args,**_kwargs:values
        try:
            for name,argument,seconds,nanos in CASES:
                case=root/name
                case.mkdir()
                debug,qtest=case/"debug.sock",case/"qtest.sock"
                result,script=case/"timing.json",case/"timing.gdb"
                script.write_text(commands(root/"artifact/linked/kernel.elf",debug,qtest,result,
                                           sample,target,seconds,nanos))
                outcome=[]
                def inspect():
                    try:
                        until=time.monotonic()+5
                        while not debug.exists() and time.monotonic()<until:
                            time.sleep(.02)
                        run=subprocess.run(["gdb","-q","-batch","-x",str(script)],
                            stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout+5)
                        (case/"debug.log").write_bytes(run.stdout)
                        outcome.append(run.returncode)
                    except BaseException as error:
                        outcome.append(str(error))
                def command(*args):
                    threading.Thread(target=inspect,daemon=True).start()
                    return original_command(*args)+["-S","-gdb",f"unix:{debug},server=on,wait=off",
                                                    "-qtest",f"unix:{qtest},server=on,wait=off"]
                GATE.qemu_command=command
                GATE.PROCESS_WORKLOADS[name]=(f"exec /bin/busybox sleep {argument}\n".encode(),b"",0)
                try:
                    runtime=GATE.verify(timeout,False,external,name)
                except GATE.GateError as error:
                    log=(case/"debug.log").read_text() if (case/"debug.log").exists() else repr(outcome)
                    raise GATE.GateError(str(error)+"\nTiming debugger:\n"+log) from error
                if outcome!=[0] or not result.exists():
                    raise GATE.GateError("timing setup did not finish: "+repr(outcome))
                record=json.loads(result.read_text())
                record.update(case=name,runtime=runtime)
                records.append(record)
                print("PASS",name,"deadline",record["observed_deadline_ticks"],flush=True)
        finally:
            GATE.build_image=original_build
            GATE.qemu_command=original_command
    if len({r["runtime"]["linked_kernel_sha256"] for r in records})!=1:
        raise GATE.GateError("timing cases did not use one kernel")
    result={"schema":"topal-static-process-time-qemu/1","harness_sha256":GATE.digest(Path(__file__)),
            "hardware_setup":"QTest HPET counter inputs; saturated waits explicitly advanced",
            "cases":records}
    if write_evidence:
        (LAB/"results/topal-static-process-time-gate.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--external-static-user-image",type=Path,required=True)
    parser.add_argument("--timeout",type=float,default=30)
    parser.add_argument("--write-evidence",action="store_true")
    args=parser.parse_args()
    verify(args.external_static_user_image,args.timeout,args.write_evidence)


if __name__=="__main__":
    main()
