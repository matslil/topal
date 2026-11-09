"""Observe actual frame-memory ownership for an unchanged pinned BusyBox workload.

Reuse the syscall owner's admission, mapping, byte preservation, affine extent,
source CFI and physical-retirement checks. The terminal checks below concern
application execution; the separate ABI fixture retains its original minimum
operation/count requirements in frame_memory_inspect.py.
"""
import os

import frame_memory_inspect
import topal_gate

harness = frame_memory_inspect.harness
common, separator, _ = harness.GDB_PROGRAM.partition('class Complete(gdb.Breakpoint):')
assert separator, 'missing shared syscall-memory observer contract'
harness.GDB_PROGRAM = common + r'''
class ApplicationGate(Gate):
    def stop(self):
        try:
            creating=self.role=='return' and pending is not None and pending['op']==0
            if self.role=='entry' and value('$rdi')==0:
                boot=value('$rsi');count=memory(boot+488,1)[0]
                observed['e820']=[words(boot+720+20*i,2)+[
                    int.from_bytes(memory(boot+736+20*i,4),'little')] for i in range(count)]
                resource=int.from_bytes(memory(boot+536,4),'little')
                size=int.from_bytes(memory(boot+540,4),'little')
                observed['reservations']=[[0,address('topal_x86_kernel_reservation_end_v1')],
                    [resource,resource+size]]
            result=super().stop()
            if self.role=='return':
                assert word(address('topal_frame_allocator_active_v1'))==0
                assert word(address('topal_x86_tail_boundary_active_v1'))==0
                assert word(address('topal_x86_frame_context_v1'))==1
            if creating:
                n=word(address('topal_x86_frame_context_v1')+24)
                cells=words(address('topal_x86_frame_table_v1')+128,n*3)
                assert cells[0::3]==[0]*n and cells[1::3]==[1]*n
                # Admit complete RAM frames, then exclude every boundary
                # crossing, overlapping non-RAM claim and actual reservation.
                # This differs from the target's interval-sweep construction.
                candidates=set();boundaries=set()
                for start,length,kind in observed['e820']:
                    boundaries.update([start,start+length])
                    if kind==1:candidates.update(range((start+4095)//4096,(start+length)//4096))
                expected=[]
                for frame in sorted(candidates):
                    a,b=frame*4096,(frame+1)*4096
                    if any(a<x<b for x in boundaries):continue
                    if any(kind!=1 and a<start+length and start<b for start,length,kind in observed['e820']):continue
                    if any(a<end and start<b for start,end in observed['reservations']):continue
                    expected.append(frame)
                assert cells[2::3]==expected and n==len(expected)
                observed['physical_frames']=expected
                observed['complete_topology_checked']=True
            return result
        except Exception:fail()

class Complete(gdb.Breakpoint):
    def stop(self):
        try:
            privileged();assert pending is None
            assert observed['source_trace'] is not None
            assert observed['syscall_caller_cfi_registers_checked']
            assert observed['complete_topology_checked']
            assert any(c['op']==5 and c['request']>0 for c in observed['memory'])
            assert any(c['op']==7 for c in observed['memory'])
            assert sum(c['op']==6 for c in observed['memory'])==1
            assert state()==[0,0,0,0]
            zeros(address('topal_x86_frame_mm_tokens_v1'),48)
            n=word(address('topal_x86_frame_context_v1')+24)
            registry=words(address('topal_x86_frame_registry_v1'),n*8)
            live=[registry[i:i+8] for i in range(0,len(registry),8) if registry[i+7]]
            assert len(live)==1 and live[0][3]==5 and live[0][6]==1,live
            start=live[0][2];identity=live[0][1]
            table=words(address('topal_x86_frame_table_v1')+128,n*3)
            assert table[0::3]==[identity if start<=i<start+5 else 0 for i in range(n)]
            assert value('$cr3')==table[start*3+2]*4096
            observed.update(passed=True,n=n,live=live,workload=application_workload,
                scope='Unchanged pinned BusyBox on the selected production frame profile; finite current MM geometry')
            with open(observation_path,'w') as f:json.dump(observed,f,indent=2)
            return True
        except Exception:fail()

try:
    ApplicationGate('topal_x86_frame_gateway_v1','entry')
    ApplicationGate('topal_x86_frame_gateway_return_reclaimed_v1','return')
    for symbol in ['topal_x86_frame_gateway_refused_v1','topal_x86_frame_gateway_fatal_reclaimed_v1','topal_x86_frame_gateway_late_reclaimed_v1']:
        Unexpected('*'+symbol)
    Complete('*topal_x86_systems_fatal')
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    print('FRAME_APPLICATION_GDB_PASS',application_workload,len(observed['memory']))
    gdb.execute('detach')
except Exception:fail()
'''

if __name__ == '__main__':
    workload = os.environ['TOPAL_FRAME_WORKLOAD']
    command = topal_gate.PROCESS_WORKLOADS[workload][0]
    harness.main(work_name='application-'+workload, process_command=command,
                 program_prefix=f'application_workload={workload!r}\n',
                 input_scope=f'unchanged pinned BusyBox ({workload})')
