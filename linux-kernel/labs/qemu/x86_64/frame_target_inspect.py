"""Unpublished bootstrap allocator component; no full kernel acceptance."""
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
import gdb, json, traceback

observed = dict(mode=mode, passed=False, created=False, zeros=[])
observed['gateway_calls']=[]
pending_gateway=None
successful = mode in ['real-map','sparse-map','unaligned-map','duplicate-create','zero-reservation','source-abort','bad-result']
allocating = mode in ['sparse-map','unaligned-map','zero-reservation','source-abort','bad-result']
observed['source_traces']={}

def value(s): return int(gdb.parse_and_eval(s))
def address(s): return value('(unsigned long long)&'+s)
def memory(p,n): return bytes(gdb.selected_inferior().read_memory(p,n))
def word(p): return int.from_bytes(memory(p,8),'little')
def put(p,n,size=8): gdb.selected_inferior().write_memory(p,int(n).to_bytes(size,'little'))
def privileged():
    assert value('$cs')&3 == 0
    assert value('$eflags')&512 == 0

class Entry(gdb.Breakpoint):
    def stop(self):
        try:
            privileged()
            boot=value('$rsi')
            end=address('topal_x86_kernel_reservation_end_v1')
            base=(end+4095)&-4096
            resource=int.from_bytes(memory(boot+536,4),'little')
            size=int.from_bytes(memory(boot+540,4),'little')
            if mode in ['sparse-map','zero-reservation','source-abort','bad-result']:
                claims=[(0,base+64*4096,1),(base+8*4096,4096,2)]
            elif mode=='unaligned-map':
                claims=[(0,base+64*4096,1),(base+5*4096+13,3*4096+17,1),
                        (base+20*4096+5,33,2)]
            elif mode=='unsupported-map': claims=[(0,(1<<30)+4096,1)]
            elif mode=='overflow-map': claims=[((1<<64)-4096,8192,1)]
            elif mode=='empty-map': claims=[]
            elif mode=='reserved-map': claims=[(0,1<<30,2)]
            else: claims=None
            if claims is not None:
                put(boot+488,len(claims),1)
                for i,(start,length,kind) in enumerate(claims):
                    put(boot+720+20*i,start)
                    put(boot+728+20*i,length)
                    put(boot+736+20*i,kind,4)
            n=memory(boot+488,1)[0]
            observed['e820']=[[word(boot+720+20*i),word(boot+728+20*i),
                int.from_bytes(memory(boot+736+20*i,4),'little')] for i in range(n)]
            observed['reservations']=[[0,end],[resource,resource+size]]
            observed['boot_params']=boot
            put(address('topal_frame_target_probe_mode_v1'),int(allocating))
            if mode=='duplicate-create': put(address('topal_frame_target_probe_mode_v1'),2)
            if mode=='interrupts': gdb.execute('set $eflags = $eflags | 512')
            if mode=='bad-stack': gdb.execute('set $rsp = 0x300000')
            if gateway and mode=='direction-flag':gdb.execute('set $eflags = $eflags | 1024')
            if gateway and mode=='busy-gateway':put(address('topal_x86_frame_gateway_recovery_v1')+8,85)
            count,alignment=(5,8) if mode=='unaligned-map' else (3,4)
            request=address('topal_frame_target_probe_request_v1')
            put(request,count);put(request+8,alignment)
            observed['runtime_request']=[count,alignment]
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class Created(gdb.Breakpoint):
    def stop(self):
        try:
            if mode=='interrupts':
                assert value('$eflags')&512
                gdb.execute('set $eflags = $eflags & ~512')
            privileged()
            ctx=value('$rdi')
            assert bool(ctx)==successful,(mode,hex(ctx))
            observed['created']=bool(ctx)
            context=address('topal_x86_frame_context_v1')
            if not ctx:
                assert word(context)==(0 if mode in ['interrupts','bad-stack'] else 3)
                observed['frames']=[]
                return False
            assert ctx==context and word(ctx)==1 and word(ctx+8)==1 and word(ctx+16)==1
            n=word(ctx+24)
            assert 0<n<=262144
            table=address('topal_x86_frame_table_v1')
            assert word(table)==n
            frames=[]
            data=memory(table+128,n*24)
            for i in range(n):
                at=i*24
                identity=int.from_bytes(data[at:at+8],'little')
                provenance=int.from_bytes(data[at+8:at+16],'little')
                frame=int.from_bytes(data[at+16:at+24],'little')
                assert identity==0 and provenance==1
                assert not frames or frames[-1]<frame
                for start,end in observed['reservations']:
                    assert (frame+1)*4096<=start or frame*4096>=end
                frames.append(frame)
            observed['frames']=frames
            observed['n']=n
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class ZeroReturn(gdb.Breakpoint):
    def __init__(self,at,physical,count,victim=None):
        super().__init__('*'+str(at),internal=True,temporary=True)
        self.physical,self.count=physical,count
        self.victim=victim
    def stop(self):
        try:
            privileged()
            if self.victim is not None:
                assert value('$rax')&255 == 0
                assert memory(self.physical*4096,self.count*4096)==self.victim
                observed['zeros'][-1]['reserved_bytes_unchanged']=True
            else:
                assert value('$rax')&255 == 1
                assert memory(self.physical*4096,self.count*4096)==bytes(self.count*4096)
                observed['zeros'][-1]['actual_physical_bytes_zero']=True
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class Zero(gdb.Breakpoint):
    def stop(self):
        try:
            privileged()
            ctx,physical,count=value('$rdi'),value('$rsi'),value('$rdx')
            assert ctx==address('topal_x86_frame_context_v1') and word(ctx)==2
            assert word(ctx+14*8)==count and word(ctx+15*8)==word(ctx+2*8)
            issuer=word(ctx+2*8)
            registry=address('topal_x86_frame_registry_v1')
            records=memory(registry,observed['n']*64)
            for at in range(0,len(records),64):
                assert not int.from_bytes(records[at+56:at+64],'little') or int.from_bytes(records[at+8:at+16],'little')!=issuer
            if len(observed['zeros'])==2:
                assert memory(physical*4096,count*4096)==bytes([165])*(count*4096)
                assert physical==observed['zeros'][0]['physical_frame']
            observed['zeros'].append(dict(physical_frame=physical,count=count,id=issuer))
            victim=None
            if mode=='zero-reservation':
                physical=0x200
                gdb.execute('set $rsi = 0x200')
                victim=memory(physical*4096,count*4096)
            ZeroReturn(word(value('$rsp')),physical,count,victim)
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class Complete(gdb.Breakpoint):
    def stop(self):
        try:
            privileged()
            assert mode not in ['zero-reservation','source-abort','bad-result'], 'fatal allocator resumed its caller'
            result=word(address('topal_frame_target_probe_result_v1'))
            assert result==(1 if successful else (1<<64)-1),(mode,result)
            if successful:
                ctx=address('topal_x86_frame_context_v1')
                assert word(ctx)==1
                assert word(ctx+16)==(4 if allocating else 1)
                n=observed['n']
                for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),
                                  ('registry',n*64),('tokens',n*24)]:
                    assert memory(address('topal_x86_frame_'+name+'_v1'),size)==bytes(size),name
                history=address('topal_x86_frame_history_v1')
                assert word(history+24)==0 and word(history+32)==0
                assert word(address('topal_frame_allocator_active_v1'))==0
                assert word(address('topal_x86_tail_boundary_active_v1'))==0
                assert len(observed['zeros'])==(3 if allocating else 0)
                observed['history']=[word(history+i*8) for i in range(5)]
                data=memory(address('topal_x86_frame_table_v1')+128,n*24)
                assert all(int.from_bytes(data[i*24:i*24+8],'little')==0 for i in range(n))
            observed['passed']=True
            return True
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')

class Fatal(gdb.Breakpoint):
    def stop(self):
        try:
            privileged()
            assert mode in ['zero-reservation','source-abort']
            ctx=address('topal_x86_frame_context_v1')
            assert word(ctx)==3 and word(ctx+16)==1
            start,count,identity=[word(ctx+i*8) for i in [13,14,15]]
            if mode=='zero-reservation':
                assert count==observed['runtime_request'][0] and identity==1
                for i in range(count):
                    assert word(address('topal_x86_frame_table_v1')+128+(start+i)*24)==identity
            else:
                assert [start,count,identity]==[0,0,0]
                data=memory(address('topal_x86_frame_table_v1')+128,observed['n']*24)
                assert data.hex()==observed['source_fault_table']
            history=address('topal_x86_frame_history_v1')
            assert word(history+24)==0 and word(history+32)==1
            for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),
                              ('registry',observed['n']*64),('tokens',observed['n']*24)]:
                assert memory(address('topal_x86_frame_'+name+'_v1'),size)==bytes(size),name
            assert word(address('topal_frame_allocator_active_v1'))==0
            assert word(address('topal_x86_tail_boundary_active_v1'))==0
            if mode=='zero-reservation':
                assert len(observed['zeros'])==1 and observed['zeros'][0]['reserved_bytes_unchanged']
            else:assert not observed['zeros']
            observed.update(passed=True, poisoned=True, source_writes_retained=True,
                            pending_issuance_retained=True, caller_retirement_qualified=False)
            return True
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')

class Gateway(gdb.Breakpoint):
    def __init__(self,symbol,role):
        super().__init__('*'+symbol)
        self.role=role
    def stop(self):
        global pending_gateway
        try:
            if self.role=='entry':
                assert pending_gateway is None
                pending_gateway=dict(sp=value('$rsp'),op=value('$rdi'),
                    registers={r:value('$'+r) for r in ['rbx','rbp','r12','r13','r14','r15']})
                return False
            if self.role=='refused':
                assert mode in ['interrupts','bad-stack','direction-flag','busy-gateway']
                assert word(address('topal_x86_frame_context_v1'))==0
                if mode=='direction-flag':assert value('$eflags')&1024
                recovery=address('topal_x86_frame_gateway_recovery_v1')
                assert word(recovery+8)==(85 if mode=='busy-gateway' else 0)
                observed.update(passed=True,created=False,frames=[],gateway_refused=True)
                return True
            if self.role=='inject-late':
                if mode=='late-cleanup':
                    privileged()
                    sp=pending_gateway['sp']
                    assert memory(sp-48-gateway_retired,48+gateway_retired)==bytes(48+gateway_retired)
                    recovery=address('topal_x86_frame_gateway_recovery_v1')
                    assert memory(recovery,48)==bytes(48)
                    put(recovery,0xdead)
                    for i in range(6):put(sp-48+i*8,0x1111222233334400+i)
                    observed['late_fault_after_retirement']=True
                self.enabled=False
                return False
            if self.role=='dispatched':
                if mode=='bad-result' and pending_gateway['op']==1:
                    ctx=address('topal_x86_frame_context_v1')
                    assert value('$rax')==1 and word(ctx+16)==2
                    observed['issued_table']=memory(address('topal_x86_frame_table_v1')+128,observed['n']*24).hex()
                    observed['issued_registry']=memory(address('topal_x86_frame_registry_v1'),observed['n']*64).hex()
                    observed['issued_tokens']=memory(address('topal_x86_frame_tokens_v1'),observed['n']*24).hex()
                    gdb.execute('set $rax = 2')
                    self.enabled=False
                return False
            privileged()
            assert pending_gateway is not None
            sp=pending_gateway['sp']
            assert value('$rsp')==sp
            assert not value('$eflags')&1024
            for reg,wanted in pending_gateway['registers'].items():assert value('$'+reg)==wanted,reg
            for reg in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:assert value('$'+reg)==0,reg
            assert memory(sp-48-gateway_retired,48+gateway_retired)==bytes(48+gateway_retired)
            assert memory(address('topal_x86_frame_gateway_recovery_v1'),48)==bytes(48)
            trace=gdb.execute('bt',to_string=True)
            assert 'probe.gateway_' in trace,trace
            observed['gateway_calls'].append(dict(op=pending_gateway['op'],result=value('$rax'),
                physical_caller_bytes_retired=48+gateway_retired,callee_registers_preserved=True,trace=trace))
            pending_gateway=None
            if self.role=='late':
                assert mode=='late-cleanup' and observed['late_fault_after_retirement']
                assert word(address('topal_x86_frame_context_v1'))==3
                history=address('topal_x86_frame_history_v1')
                assert word(history+24)==0 and word(history+32)==1
                assert memory(address('topal_x86_frame_details_v1'),32)==bytes(32)
                observed.update(passed=True,created=False,frames=[],gateway_late_retired=True,
                                caller_retirement_qualified=True)
                return True
            if self.role=='fatal':
                assert mode in ['zero-reservation','source-abort','bad-result']
                assert word(address('topal_x86_frame_context_v1'))==3
                if mode=='bad-result':
                    assert word(address('topal_x86_frame_context_v1')+16)==2
                    for name,size,key in [('table',observed['n']*24,'issued_table'),('registry',observed['n']*64,'issued_registry'),('tokens',observed['n']*24,'issued_tokens')]:
                        start=address('topal_x86_frame_'+name+'_v1')+(128 if name=='table' else 0)
                        assert memory(start,size).hex()==observed[key]
                    observed.update(source_writes_retained=True,issued_identity_retained=True)
                else:assert observed.get('source_writes_retained')
                for name,size in [('private',65528),('stack',4096),('facts',24),('output',56),('details',32)]:
                    assert memory(address('topal_x86_frame_'+name+'_v1'),size)==bytes(size)
                observed.update(passed=True,gateway_fatal_retired=True,
                                caller_retirement_qualified=True)
                return True
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class SourceTrace(gdb.Breakpoint):
    def __init__(self,symbol,role):
        super().__init__('*'+symbol)
        self.role=role
    def stop(self):
        try:
            privileged()
            stack=address('topal_x86_frame_stack_v1')
            assert stack<=value('$rsp')<stack+4096
            trace=gdb.execute('bt',to_string=True)
            assert 'topal_x86_frame_gateway_v1' in trace and 'probe.gateway_' in trace,trace
            frame=gdb.newest_frame()
            while frame and frame.name()!='topal_x86_frame_gateway_v1':frame=frame.older()
            assert frame and frame.older(),trace
            for reg,wanted in pending_gateway['registers'].items():
                assert int(frame.older().read_register(reg))==wanted,(reg,trace)
            observed['source_traces'][self.role]=trace
            self.enabled=False
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

class SourceFault(gdb.Breakpoint):
    def stop(self):
        try:
            n=observed['n']
            table=address('topal_x86_frame_table_v1')+128
            data=memory(table,n*24)
            cells=[i for i in range(n) if int.from_bytes(data[i*24:i*24+8],'little')==1]
            if cells:
                assert len(cells)<observed['runtime_request'][0]
                observed['source_fault_cells']=cells
                observed['source_fault_table']=data.hex()
                gdb.execute('set $rdx = 1')
                self.enabled=False
        except Exception:
            traceback.print_exc();gdb.execute('quit 1')
        return False

try:
    Entry('*_topal_kernel_entry')
    Created('*topal_frame_target_created_v1')
    Zero('*topal_frame_kernel_zero_v1')
    Complete('*topal_frame_target_complete_v1')
    if gateway:
        # Observe source cleanup/preservation before the owning fatal gateway,
        # then continue to its separate physical caller-retirement marker.
        class SourceFatal(Fatal):
            def stop(self):
                super().stop()
                observed['passed']=False
                return False
        SourceFatal('*topal_x86_frame_target_halt_v1')
        Gateway('topal_x86_frame_gateway_v1','entry')
        Gateway('topal_x86_frame_gateway_return_reclaimed_v1','return')
        Gateway('topal_x86_frame_gateway_fatal_reclaimed_v1','fatal')
        Gateway('topal_x86_frame_gateway_refused_v1','refused')
        Gateway('topal_x86_frame_gateway_recovery_check_v1','inject-late')
        Gateway('topal_x86_frame_gateway_late_reclaimed_v1','late')
        Gateway('topal_x86_frame_gateway_dispatched_v1','dispatched')
        for symbol,role in [('topal_tail_step_component_v1','source'),('topal_tail_checkpoint_body_v1','checkpoint'),('topal_tail_complete_body_v1','complete')]:SourceTrace(symbol,role)
        if mode=='source-abort':SourceFault('*topal_tail_stage_number_v1')
    else:Fatal('*topal_x86_frame_target_halt_v1')
    gdb.execute('continue')
    assert observed['passed'],observed
    with open(observation_path,'w') as f: json.dump(observed,f,indent=2)
    print('FRAME_TARGET_GDB_PASS',mode)
    gdb.execute('detach')
except Exception:
    traceback.print_exc();gdb.execute('quit 1')
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('work', type=Path)
    parser.add_argument('--mode', required=True)
    parser.add_argument('--gateway',action='store_true')
    parser.add_argument('--gateway-retired-bytes',type=int,default=4032)
    args = parser.parse_args()
    build = args.work.resolve()
    work = build / args.mode
    work.mkdir()
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    master, slave = pty.openpty()
    command = topal_gate.qemu_command(build / 'bzImage', build / 'initramfs.cpio',
                                      work / 'serial.log', os.ttyname(slave))
    command += ['-S', '-gdb', f'tcp:127.0.0.1:{port}']
    (work / 'qemu.command.json').write_text(json.dumps(command, indent=2))
    script = work / 'inspect.gdb'
    script.write_text('set pagination off\nset confirm off\nset remotetimeout 10\n'
                      f'file {json.dumps(str(build / "kernel.elf"))}\n'
                      f'target remote 127.0.0.1:{port}\npython\n'
                      f'mode = {args.mode!r}\n'
                      f'gateway = {args.gateway!r}\n'
                      f'gateway_retired = {args.gateway_retired_bytes}\n'
                      f'observation_path = {str(work / "observations.json")!r}\n'
                      + GDB_PROGRAM + '\nend\n')
    with (work / 'qemu.stdout.log').open('wb') as out, (work / 'qemu.stderr.log').open('wb') as err:
        process = subprocess.Popen(command, stdout=out, stderr=err)
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
            result = subprocess.run(['gdb', '-nx', '-batch', '-x', str(script)],
                                    capture_output=True, timeout=55, check=False)
            (work / 'gdb.stdout.log').write_bytes(result.stdout)
            (work / 'gdb.stderr.log').write_bytes(result.stderr)
            assert result.returncode == 0, result.stdout.decode(errors='replace') + result.stderr.decode(errors='replace')
            assert b'FRAME_TARGET_GDB_PASS' in result.stdout
        finally:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
            os.close(master)
            os.close(slave)
    print('FRAME_TARGET_PASS', args.mode)


if __name__ == '__main__':
    main()
