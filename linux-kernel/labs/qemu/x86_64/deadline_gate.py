#!/usr/bin/env python3
"""Inject one completed deadline notification at suspended-frame resumption."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import tempfile
import threading
import time

LAB=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('topal_gate',LAB/'topal_gate.py')
GATE=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def points(elf: Path) -> tuple[int,int|None]:
    text=subprocess.check_output(['gdb','-q','-batch','-ex',f'file {elf}',
                                  '-ex','disassemble /r topal_x86_systems_deadline_entry'],text=True)
    call=re.search(r'0x([0-9a-f]+)\s+<[^>]+>:\s+e8\b[^\n]*<topal_x86_systems_context_preempt_current>',text)
    if not call:
        raise GATE.GateError('cannot locate the suspended deadline continuation')
    # The generated call is followed by its checked boolean result (2+6 bytes).
    resume=int(call[1],16)+5+8
    eoi=re.search(r'0x([0-9a-f]+)\s+<[^>]+>:[^\n]*movabs\s+\$0xfee000b0',text)
    return resume,int(eoi[1],16) if eoi else None


def commands(elf: Path,sock: Path,result: Path,resume: int,filtered: int|None) -> str:
    # The debugger performs declared hardware test setup. No source operation
    # exposes this vector, target register, temporary immediate or helper call.
    setup=f'''
import gdb, hashlib, json
from pathlib import Path
inferior=gdb.selected_inferior()
base=int(gdb.parse_and_eval("&topal_bootstrap_storage"))
stage=int.from_bytes(inferior.read_memory(base+64,1),"little")
assert stage==3, "injection requires the completed event"
sp=int(gdb.parse_and_eval("$rsp"))
frame=bytes(inferior.read_memory(sp,160))
names=("rax","rbx","rcx","rdx","rbp","rsi","rdi","rsp","r8","r9","r10","r11","r12","r13","r14","r15","rip","eflags")
before={{name:int(gdb.parse_and_eval("$"+name)) for name in names}}
helper=int(gdb.parse_and_eval("&topal_x86_systems_local_notification_send"))
original=bytes(inferior.read_memory(helper,77))
assert original[62:68]==bytes.fromhex("c702f1400400")
scratch=base+600
scratch_byte=bytes(inferior.read_memory(scratch,1))
gdb.execute("set {{unsigned char}}"+str(helper+64)+"=0xf2")
gdb.execute("call ((unsigned long long (*)(void*)) topal_x86_systems_local_notification_send)((void*)"+str(scratch)+")")
gdb.execute("set {{unsigned char}}"+str(helper+64)+"=0xf1")
inferior.write_memory(scratch,scratch_byte)
assert bytes(inferior.read_memory(helper,77))==original
after={{name:int(gdb.parse_and_eval("$"+name)) for name in names}}
assert after==before, "debugger setup changed interrupted registers"
assert bytes(inferior.read_memory(sp,160))==frame
record={{"completed_event_state":stage,"resume_offset":{resume},
         "raw_vector":242,"temporary_helper_vector":{{"original":241,"injected":242,"restored":241}},
         "helper_sha256_before_and_after":hashlib.sha256(original).hexdigest(),
         "setup_registers_unchanged":True,"setup_frame_unchanged":True,
         "interrupted_frame_sha256":hashlib.sha256(frame).hexdigest(),
         "filtered_entry_observed":False}}
gdb.execute("delete breakpoints")
if {filtered!r} is not None:
    gdb.execute("hbreak *"+str({filtered!r}))
    gdb.execute("continue")
    assert int(gdb.parse_and_eval("$rip"))=={filtered!r}
    second_sp=int(gdb.parse_and_eval("$rsp"))
    second=bytes(inferior.read_memory(second_sp,160))
    assert second==frame, "filtered notification changed the saved interrupted image"
    assert int.from_bytes(inferior.read_memory(base+64,1),"little")==3
    record.update(filtered_entry_observed=True,filtered_frame_sha256=hashlib.sha256(second).hexdigest())
    gdb.execute("delete breakpoints")
gdb.execute("detach")
Path({str(result)!r}).write_text(json.dumps(record,indent=2,sort_keys=True)+"\\n")
'''
    return (f'set pagination off\nfile {elf}\ntarget remote {sock}\n'
            f'hbreak *{resume}\ncontinue\npython\n{setup}\nend\n')


def observe(artifact: Path,root: Path,timeout: float,expect_filter: bool) -> tuple[bytes,dict]:
    elf=artifact/'linked/kernel.elf'
    resume,filtered=points(elf)
    if (filtered is not None)!=expect_filter:
        raise GATE.GateError('artifact deadline admission does not match selected expectation')
    sock=root/'debug.sock'
    result=root/'injection.json'
    script=root/'inject.gdb'
    script.write_text(commands(elf,sock,result,resume,filtered))
    outcome=[]
    def inspect():
        try:
            deadline=time.monotonic()+5
            while not sock.exists() and time.monotonic()<deadline:
                time.sleep(.02)
            r=subprocess.run(['gdb','-q','-batch','-x',str(script)],stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT,timeout=timeout+5)
            (root/'debug.log').write_bytes(r.stdout)
            outcome.append(r.returncode)
        except BaseException as error:
            outcome.append(str(error))
    original=GATE.qemu_command
    def command(*args):
        threading.Thread(target=inspect,daemon=True).start()
        return original(*args)+['-S','-gdb',f'unix:{sock},server=on,wait=off']
    GATE.qemu_command=command
    failure=None
    try:
        serial=GATE.observe_gate(artifact/'boot/bzImage',artifact/'boot/initramfs.cpio',
                                 root/'serial.log',root/'qemu.log',timeout,
                                 GATE.PROCESS_WORKLOADS['head-alias'][0])
    except GATE.GateError as error:
        failure=str(error)
        serial=(root/'serial.log').read_bytes()
    finally:
        GATE.qemu_command=original
    if outcome!=[0] or not result.exists():
        log=(root/'debug.log').read_text() if (root/'debug.log').exists() else str(outcome)
        raise GATE.GateError('injection did not complete: '+log)
    record=json.loads(result.read_text())
    if expect_filter and failure:
        raise GATE.GateError('corrected deadline gate failed: '+failure)
    if not expect_filter:
        if not failure or 'timed out' not in failure:
            raise GATE.GateError('predecessor did not reject the injected notification')
        if b'TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED' not in serial or b'TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED' in serial:
            raise GATE.GateError('predecessor failed outside the matched resumption point')
    record.update(result='pass' if expect_filter else 'expected-predecessor-rejection',
                  linked_kernel_sha256=GATE.digest(elf),boot_image_sha256=GATE.digest(artifact/'boot/bzImage'),
                  boot_resource_sha256=GATE.digest(artifact/'boot/initramfs.cpio'),
                  serial_utf8=serial.decode('utf-8',errors='replace'),
                  serial_sha256=hashlib.sha256(serial).hexdigest())
    return serial,record


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external-static-user-image',type=Path,required=True)
    parser.add_argument('--predecessor-artifact',type=Path)
    parser.add_argument('--write-evidence',action='store_true')
    args=parser.parse_args()
    qemu=GATE.verify_host()
    GATE.checked_static_user_image(args.external_static_user_image)
    with tempfile.TemporaryDirectory(prefix='topal-deadline-',dir='/tmp') as directory:
        root=Path(directory)
        values=GATE.build_image(root/'artifact',args.external_static_user_image)
        current=root/'current'
        current.mkdir()
        serial,record=observe(root/'artifact',current,30,True)
        record['runtime']=GATE.evidence_record(qemu,*values[:6],values[6],serial)
        predecessor=None
        if args.predecessor_artifact:
            previous=root/'previous'
            previous.mkdir()
            _,predecessor=observe(args.predecessor_artifact,previous,10,False)
        result={'schema':'topal-deadline-completed-notification-qemu/1',
                'harness_sha256':GATE.digest(Path(__file__)),
                'current':record,'predecessor':predecessor}
        if args.write_evidence:
            (LAB/'results/topal-deadline-entry-filter-gate.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
        print('PASS completed notification filtered; predecessor rejection:',predecessor is not None)


if __name__=='__main__':
    main()
