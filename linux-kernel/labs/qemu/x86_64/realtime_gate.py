#!/usr/bin/env python3
"""Qualify actual RTC dates/representations with an unchanged kernel and BusyBox."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import socket
import tempfile

LAB=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('topal_gate',LAB/'topal_gate.py')
GATE=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)
CASES=(
    ('epoch','1970-01-01T00:00:00','bcd'),
    ('leap-day','2000-02-29T12:34:56','bcd'),
    ('non-leap-century','2100-03-01T00:00:00','bcd'),
    ('last-whole-year','2261-12-31T23:59:00','bcd'),
    ('binary-leap-day','2000-02-29T12:34:56','binary'),
)
COMMAND=b'exec /bin/busybox date +%s\n'


def configure_binary(socket_path: Path) -> None:
    """Test setup only: freeze an initialized RTC, convert its data, resume it.

    This runs while the process awaits its final command, before the kernel's
    clock service can use the RTC. The kernel never performs these writes.
    """
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as client:
        client.settimeout(5)
        client.connect(str(socket_path))
        reader=client.makefile('rb')
        def request(command):
            client.sendall(command.encode()+b'\n')
            reply=reader.readline().decode().strip()
            if not reply.startswith('OK'):
                raise GATE.GateError(f'qtest setup failed: {command}: {reply}')
            return reply.split()[1:]
        def read(index):
            request(f'outb 0x70 {index}')
            return int(request('inb 0x71')[0],0)
        def write(index,value):
            request(f'outb 0x70 {index}')
            request(f'outb 0x71 {value}')
        control=read(0x0b)
        if control&0x82 != 2:
            raise GATE.GateError('RTC setup requires a running 24-hour clock')
        write(0x0b,control|0x80)
        if not control&4:
            for index in (0,2,4,7,8,9,0x32):
                raw=read(index)
                if raw&15>9 or raw>>4>9:
                    raise GATE.GateError('RTC setup found malformed BCD')
                write(index,(raw>>4)*10+(raw&15))
        # QEMU interprets the fields with the old mode while releasing SET.
        # Commit binary mode while still frozen, then resume in that mode.
        write(0x0b,control|0x84)
        write(0x0b,(control|4)&~0x80)
        if read(0x0b)&0x86 != 6:
            raise GATE.GateError('RTC binary setup did not commit')


def verify(external: Path,timeout: float,write_evidence: bool) -> dict:
    GATE.checked_static_user_image(external)
    qemu_version=GATE.verify_host()
    original=GATE.qemu_command
    records=[]
    with tempfile.TemporaryDirectory(prefix='topal-rtc-',dir='/tmp') as temporary:
        root=Path(temporary)
        values=GATE.build_image(root/'artifact',external)
        image,resource,boot,artifact,initial,static,source=values
        for name,date,encoding in CASES:
            qtest_path=root/(name+'.sock')
            def command(*args):
                result=original(*args)
                result[result.index('-rtc')+1]=f'base={date},clock=vm,driftfix=slew'
                if encoding=='binary':
                    result+=['-qtest',f'unix:{qtest_path},server=on,wait=off']
                return result
            GATE.qemu_command=command
            try:
                setup=(lambda:configure_binary(qtest_path)) if encoding=='binary' else None
                serial=GATE.observe_gate(image,resource,root/(name+'.serial'),root/(name+'.qemu'),
                                         timeout,COMMAND,setup)
            finally:
                GATE.qemu_command=original
            match=re.search(rb'/\n([0-9]+)\n\nTOPAL_KERNEL_PROCESS_EXIT_STATUS=00\n',serial)
            if not match:
                raise GATE.GateError(f'{name}: date output/status did not match')
            minimum=int(datetime.datetime.fromisoformat(date).replace(tzinfo=datetime.timezone.utc).timestamp())
            observed=int(match[1])
            if not minimum<=observed<=minimum+30:
                raise GATE.GateError(f'{name}: expected [{minimum},{minimum+30}], observed {observed}')
            record=GATE.evidence_record(qemu_version,image,resource,boot,artifact,initial,static,source,serial)
            record.update(case=name,qualification_rtc_base=date,qualification_rtc_encoding=encoding,
                          epoch_minimum=minimum,epoch_maximum=minimum+30,observed_epoch=observed,
                          process_command_utf8=COMMAND.decode(),
                          process_command_sha256=hashlib.sha256(COMMAND).hexdigest(),
                          process_exit_status=0)
            records.append(record)
            print(f'PASS {name}: {observed}',flush=True)
    if len({record['linked_kernel_sha256'] for record in records})!=1:
        raise GATE.GateError('RTC cases did not use one kernel')
    result={'schema':'topal-kernel-realtime-seconds-qemu/1','cases':records}
    if write_evidence:
        path=LAB/'results'/'topal-realtime-seconds-gate.json'
        path.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
        print(f'wrote {path}')
    return result


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external-static-user-image',type=Path,required=True)
    parser.add_argument('--timeout',type=float,default=30)
    parser.add_argument('--write-evidence',action='store_true')
    args=parser.parse_args()
    verify(args.external_static_user_image,args.timeout,args.write_evidence)


if __name__=='__main__':
    main()
