"""Observe the unpublished full boot Root using the actual source allocator.

The static ELF is an ABI fixture. This does not qualify Linux applications or
publish an artifact. The guest uses its unmodified QEMU firmware memory map.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import pty
import socket
import subprocess
import time

import topal_gate


GDB_PROGRAM = r'''
import gdb, json, time, traceback

observed = dict(passed=False, calls=[], live={}, firmware_map_modified=False)
pending = None
preserved = ['rbx','rbp','r12','r13','r14','r15']

def value(s): return int(gdb.parse_and_eval(s))
def address(s): return value('(unsigned long long)&'+s)
def memory(p,n): return bytes(gdb.selected_inferior().read_memory(p,n))
def words(p,n):
    data=memory(p,n*8)
    return [int.from_bytes(data[i:i+8],'little') for i in range(0,len(data),8)]
def word(p): return words(p,1)[0]
def zeros(p,n): assert memory(p,n)==bytes(n),(hex(p),n)
def privileged():
    assert value('$cs')&3==0 and value('$eflags')&0x600==0
def record(token):
    issuer,identity,index=words(token,3)
    assert issuer==1 and identity>0 and index<observed['n']
    p=address('topal_x86_frame_registry_v1')+index*64
    data=words(p,8)
    assert data[:2]==[issuer,identity] and data[7]==1
    return p,data
def cells(start,count):
    return words(address('topal_x86_frame_table_v1')+128+start*24,count*3)
def fail():
    traceback.print_exc()
    with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
    gdb.execute('quit 1')

class Gateway(gdb.Breakpoint):
    def __init__(self,symbol,role):
        super().__init__('*'+symbol)
        self.role=role
    def stop(self):
        global pending
        try:
            privileged()
            if self.role=='entry':
                assert pending is None
                zeros(address('topal_x86_frame_details_v1'),32)
                pending=dict(op=value('$rdi'),a=value('$rsi'),b=value('$rdx'),
                    token=value('$rcx'),facts=value('$r8'),sp=value('$rsp'),
                    registers={r:value('$'+r) for r in preserved},started=time.monotonic())
                op=pending['op']
                if op==0:
                    boot=pending['a']
                    n=memory(boot+488,1)[0]
                    observed['e820']=[words(boot+720+20*i,2)+[
                        int.from_bytes(memory(boot+736+20*i,4),'little')] for i in range(n)]
                    resource=int.from_bytes(memory(boot+536,4),'little')
                    size=int.from_bytes(memory(boot+540,4),'little')
                    observed['reservations']=[[0,address('topal_x86_kernel_reservation_end_v1')],
                                              [resource,resource+size]]
                else:
                    assert 1<=op<=4
                    delta=pending['token']-address('topal_x86_frame_root_tokens_v1')
                    assert 0<=delta<token_bytes and delta%24==0
                    pending['next_id']=word(address('topal_x86_frame_context_v1')+16)
                    if op!=1:
                        p,r=record(pending['token'])
                        pending.update(record_address=p,record=r)
                        assert str(r[1]) in observed['live']
                        if op==2:assert r[6]==0
                print('FRAME_ROOT_ENTER',len(observed['calls']),op,pending['a'],flush=True)
                return False
            assert pending is not None
            assert value('$rsp')==pending['sp']
            for r,n in pending['registers'].items():assert value('$'+r)==n,r
            for r in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:assert value('$'+r)==0,r
            zeros(pending['sp']-4080,4080)
            for name,size in [('private',65528),('stack',4096),('facts',24),
                              ('output',56),('gateway_recovery',48)]:
                zeros(address('topal_x86_frame_'+name+'_v1'),size)
            assert word(address('topal_frame_allocator_active_v1'))==0
            assert word(address('topal_x86_tail_boundary_active_v1'))==0
            ctx=address('topal_x86_frame_context_v1')
            assert word(ctx)==1
            op=pending['op']
            assert value('$rax')==(ctx if op==0 else 1),(op,value('$rax'))
            if op==0:
                n=word(ctx+24)
                assert 0<n<=262144
                observed['n']=n
                data=cells(0,n)
                frames=data[2::3]
                assert data[0::3]==[0]*n and data[1::3]==[1]*n
                assert frames==sorted(set(frames))
                for frame in frames:
                    assert any(kind==1 and start<=frame*4096 and (frame+1)*4096<=start+size
                               for start,size,kind in observed['e820'])
                    assert all(kind==1 or (frame+1)*4096<=start or frame*4096>=start+size
                               for start,size,kind in observed['e820'])
                    assert all((frame+1)*4096<=start or frame*4096>=end
                               for start,end in observed['reservations'])
                observed['physical_frames']=frames
                # Independent complete boundary sweep: checking only the frames
                # returned by the constructor would miss silently dropped RAM.
                claims=[(start,start+size,kind) for start,size,kind in observed['e820']]
                cuts=sorted({v for start,end,_ in claims for v in (start,end)} |
                            {v for pair in observed['reservations'] for v in pair})
                expected=[]
                for start,end in zip(cuts,cuts[1:]):
                    ram=any(kind==1 and a<=start and end<=b for a,b,kind in claims)
                    blocked=any(kind!=1 and a<end and start<b for a,b,kind in claims)
                    reserved=any(a<end and start<b for a,b in observed['reservations'])
                    if ram and not blocked and not reserved:
                        expected.extend(range((start+4095)//4096,end//4096))
                assert frames==expected,'constructor omitted or added allocatable RAM'
                observed['complete_topology_checked']=True
                assert word(ctx+16)==1
            elif op==1:
                p,r=record(pending['token'])
                issuer,identity,start,count,alignment,provenance,pins,live=r
                assert identity==pending['next_id'] and word(ctx+16)==identity+1
                assert [count,alignment]==[pending['a'],pending['b']]
                assert pins==0 and provenance==1
                frames=observed['physical_frames'][start:start+count]
                assert frames==list(range(frames[0],frames[0]+count))
                assert frames[0]%alignment==0
                for old in observed['live'].values():
                    assert start+count<=old['start'] or start>=old['start']+old['count']
                assert cells(start,count)==sum(([identity,1,f] for f in frames),[])
                zeros(frames[0]*4096,count*4096)
                observed['live'][str(identity)]=dict(start=start,count=count,physical=frames[0],
                    pins=0,record=p,token=pending['token'])
                pending['issued_record']=r
            else:
                r=pending['record'];identity=r[1]
                assert word(ctx+16)==pending['next_id']
                if op==2:
                    zeros(pending['record_address'],64);zeros(pending['token'],24)
                    assert cells(r[2],r[3])[0::3]==[0]*r[3]
                    del observed['live'][str(identity)]
                elif op==3:
                    pins=r[6]+(1 if pending['a']==1 else -1)
                    assert word(pending['record_address']+48)==pins
                    observed['live'][str(identity)]['pins']=pins
                else:
                    assert pending['facts']==address('topal_x86_frame_details_v1')
                    assert words(pending['facts'],4)==[
                        observed['physical_frames'][r[2]],r[3],r[4],r[5]]
            pending['elapsed_seconds']=time.monotonic()-pending.pop('started')
            observed['calls'].append(pending)
            pending=None
        except Exception:fail()
        return False

class Unexpected(gdb.Breakpoint):
    def stop(self):
        try:raise AssertionError('unexpected allocator/legacy selector path: '+self.location)
        except Exception:fail()
        return True

class TranslationEdit(gdb.Breakpoint):
    def stop(self):
        try:assert value('$rdx')==1,'legacy translation edit allocated map storage'
        except Exception:fail()
        return False

class Complete(gdb.Breakpoint):
    def stop(self):
        try:
            privileged();assert pending is None
            assert len(observed['live'])==1,observed['live']
            only=next(iter(observed['live'].values()))
            assert only['count']==5 and only['pins']==1
            assert value('$cr3')==only['physical']*4096
            n=observed['n'];table=cells(0,n)
            assert table[1::3]==[1]*n
            assert table[2::3]==observed['physical_frames']
            registry=words(address('topal_x86_frame_registry_v1'),n*8)
            identity=int(next(iter(observed['live'])))
            for i in range(n):
                assert table[i*3]==(identity if only['start']<=i<only['start']+5 else 0)
                if i*64+address('topal_x86_frame_registry_v1')!=only['record']:
                    assert registry[i*8:i*8+8]==[0]*8
            tokens=address('topal_x86_frame_root_tokens_v1')
            for at in range(tokens,tokens+token_bytes,24):
                if at!=only['token']:zeros(at,24)
            assert words(address('topal_x86_frame_history_v1')+24,2)==[0,0]
            assert word(address('topal_x86_frame_context_v1'))==1
            zeros(address('topal_x86_frame_details_v1'),32)
            observed['passed']=True
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

try:
    Gateway('topal_x86_frame_gateway_v1','entry')
    Gateway('topal_x86_frame_gateway_return_reclaimed_v1','return')
    for symbol in ['topal_x86_frame_gateway_refused_v1','topal_x86_frame_gateway_fatal_reclaimed_v1',
                   'topal_x86_frame_gateway_late_reclaimed_v1',
                   'topal_x86_systems_allocate_physical_frames','topal_x86_systems_begin_bootstrap_translation']:
        Unexpected('*'+symbol)
    TranslationEdit('*topal_x86_systems_begin_active_translation_edit')
    Complete('*topal_x86_systems_fatal')
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:fail()
'''


class OutputActivity:
    """Bound quiet diagnostic output separately from total workload time."""
    def __init__(self, seconds, now):
        assert isinstance(seconds, int) and seconds > 0
        self.seconds, self.changed, self.stamp = seconds, now, None

    def check(self, stamp, now):
        if stamp != self.stamp:
            self.stamp, self.changed = stamp, now
        assert now-self.changed < self.seconds, (
            f'full-root guest output inactive ({self.seconds}s)')


def main(*, work_name='guest', program_prefix='', expect_backing=True, ram_mib=None,
         process_command=None, input_scope='ABI fixture', guest_timeout_seconds=300,
         serial_dialogue=(), output_activity_seconds=None):
    assert isinstance(guest_timeout_seconds, int) and 0 < guest_timeout_seconds <= 3600
    if output_activity_seconds is not None:
        assert isinstance(output_activity_seconds, int) and 0 < output_activity_seconds <= guest_timeout_seconds
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('work', type=Path)
    args = parser.parse_args()
    build = args.work.resolve()
    work = build / work_name
    work.mkdir()
    assert all(isinstance(marker,bytes) and marker and isinstance(payload,bytes) and
               0<len(payload)<=16 for marker,payload in serial_dialogue)
    tokens = json.loads((build / 'root-tokens.json').read_text())
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    master, slave = pty.openpty()
    os.set_blocking(master, False)
    command = topal_gate.qemu_command(build / 'bzImage', build / 'initramfs.cpio',
                                    work / 'serial.log', os.ttyname(slave))
    if ram_mib is not None:
        assert isinstance(ram_mib, int) and ram_mib > 0
        command[command.index('-m') + 1] = f'{ram_mib}M'
    command += ['-S', '-gdb', f'tcp:127.0.0.1:{port}']
    (work / 'qemu.command.json').write_text(json.dumps(command, indent=2))
    script = work / 'inspect.gdb'
    script.write_text('set pagination off\nset confirm off\nset remotetimeout 10\n'
                     f'file {json.dumps(str(build / "kernel.elf"))}\n'
                     f'target remote 127.0.0.1:{port}\npython\n'
                     f'token_bytes={tokens["bytes"]}\n'
                     f'observation_path={str(work / "observations.json")!r}\n'
                     + program_prefix + GDB_PROGRAM + '\nend\n')
    with (work / 'qemu.stdout.log').open('wb') as out, (work / 'qemu.stderr.log').open('wb') as err:
        process = subprocess.Popen(command, stdout=out, stderr=err)
        debugger = None
        received = bytearray()
        dialogue=[]
        dialogue_cursor=0
        try:
            deadline = time.monotonic() + 10
            while True:
                assert process.poll() is None, f'QEMU exited {process.returncode}'
                try:
                    with socket.create_connection(('127.0.0.1', port), timeout=.1):
                        break
                except OSError:
                    if time.monotonic() >= deadline:
                        raise
                    time.sleep(.05)
            with (work / 'gdb.stdout.log').open('wb') as gout, (work / 'gdb.stderr.log').open('wb') as gerr:
                debugger = subprocess.Popen(['gdb', '-nx', '-batch', '-x', str(script)],
                                            stdout=gout, stderr=gerr)
                deadline = time.monotonic() + guest_timeout_seconds
                activity = (OutputActivity(output_activity_seconds, time.monotonic())
                            if output_activity_seconds is not None else None)
                sent_read = sent_command = False
                while True:
                    try:
                        received.extend(os.read(master, 65536))
                    except (BlockingIOError, OSError):
                        pass
                    if not sent_read and b'TOPAL_KERNEL_BOOT_RESOURCE_OK' in received:
                        os.write(master, topal_gate.SERIAL_INPUT)
                        sent_read = True
                    if not sent_command and b'TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK' in received:
                        os.write(master, topal_gate.PROCESS_COMMAND if process_command is None else process_command)
                        sent_command = True
                    if len(dialogue)<len(serial_dialogue):
                        marker,payload=serial_dialogue[len(dialogue)]
                        at=received.find(marker,dialogue_cursor)
                        if at>=0:
                            assert os.write(master,payload)==len(payload)
                            dialogue.append(dict(marker_hex=marker.hex(),payload_hex=payload.hex(),
                                                 serial_offset=at))
                            dialogue_cursor=at+len(marker)
                            (work/'serial-dialogue.json').write_text(json.dumps(dialogue,indent=2)+'\n')
                    (work / 'serial.log').write_bytes(received)
                    if debugger.poll() is not None:
                        break
                    if activity is not None:
                        activity.check((len(received), (work/'gdb.stdout.log').stat().st_size), time.monotonic())
                    assert time.monotonic() < deadline, f'full-root guest timed out ({guest_timeout_seconds}s)'
                    time.sleep(.03)
            stdout = (work / 'gdb.stdout.log').read_bytes()
            stderr = (work / 'gdb.stderr.log').read_bytes()
            assert debugger.returncode == 0, stdout.decode(errors='replace') + stderr.decode(errors='replace')
            assert b'FRAME_ROOT_GDB_PASS' in stdout
            assert len(dialogue)==len(serial_dialogue), 'serial dialogue incomplete'
            if expect_backing:
                assert b'TOPAL_KERNEL_USER_BACKING_OK' in received, received.decode(errors='replace')
                assert sent_read and sent_command
        finally:
            if debugger is not None and debugger.poll() is None:
                debugger.kill()
                debugger.wait(timeout=5)
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
            os.close(master)
            os.close(slave)
    print(f'FRAME_ROOT_PASS: real firmware map, source allocations, mapping pins, release and {input_scope}')


if __name__ == '__main__':
    main()
