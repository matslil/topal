"""Count, wrap, and source-scope models for guarded pipe byte loops.

TOPAL-SYSTEMS-USER-EXTENT-001. No native memory or ownership qualification.
"""
from pathlib import Path
import runpy
import unittest

from audit_pipe_loops import loop_state


class PipeLoopControls(unittest.TestCase):
    def test_every_count_has_exact_finite_user_offsets_and_a_zero_terminal_count(self):
        for length in range(1,4097):
            for completed in {0,length//2,length-1}:
                state = loop_state(length,completed,4095)
                self.assertEqual(state['user_offset'],completed)
                self.assertEqual(state['remaining'],length-completed)
                self.assertEqual(state['next_completed'],completed+1)
                self.assertTrue(0 <= state['ring_offset'] < 4096)
            self.assertEqual(loop_state(length,length-1,0)['next_completed'],length)

    def test_full_ring_visits_every_slot_once_for_each_wrapping_start(self):
        for cursor in [0,1,24,4090,4094,4095]:
            positions = [loop_state(4096,i,cursor)['ring_offset'] for i in range(4096)]
            self.assertEqual(len(set(positions)),4096)
            self.assertEqual(positions,[(cursor+i)%4096 for i in range(4096)])

    def test_zero_oversize_and_exhausted_counts_are_not_transfer_states(self):
        for length,completed,cursor in [(0,0,0),(4097,0,0),(1,1,0),(1,-1,0),
                                       (1,0,4096),(1,0,-1),(1<<64,0,0)]:
            with self.subTest(length=length,completed=completed,cursor=cursor),self.assertRaises(AssertionError):
                loop_state(length,completed,cursor)

    def namespace(self):
        path = Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_pipe_loop_observer.py'
        class Point:
            def __init__(self,*args,**kwargs): self.enabled = False
        namespace = dict(Checked=Point)
        exec(runpy.run_path(str(path))['PROGRAM'],namespace)
        namespace.update(extent_inventory=dict(address=0,kernel_sha256='model',pipe_loop_audit=dict(
            passed=True,complete=False,kernel_sha256='model',cases=[dict(name=name,
            call_offset=at,carry_gate=at+14,start_offset=at+20,end_offset=at+40)
            for name,at in [('pipe-read-loop',0),('pipe-write-loop',100)]])),
            selected_extent_observers=lambda:['pipe-loops'])
        namespace['install_pipe_loop_observation']()
        return namespace

    def test_only_selected_pipe_read_write_calls_arm_points(self):
        ns = self.namespace()
        for number,fd,enabled in [(12,0,False),(16,3,False),(1,1,False),(0,3,True),(1,4,True)]:
            call = dict(number=number,args=[fd,0x2000,3],phase_before=8)
            ns['pipe_loop_syscall_entered'](call)
            self.assertTrue(all(p.enabled == (enabled and p.kind!='finish') for p in ns['pipe_loop_points']))
            ns['pipe_loop_syscall_returned'](dict(**call,result=0,phase_after=8))
            self.assertFalse(any(p.enabled for p in ns['pipe_loop_points']))

    def test_refusal_requires_efault_the_actual_output_argument_and_unchanged_phase(self):
        for result,pointer,phase,accepted in [(-14,0x2000,8,True),(-22,0x2000,8,False),
                                           (-14,0x2001,8,False),(-14,0x2000,9,False)]:
            ns = self.namespace()
            call = dict(number=0,args=[3,0x2000,4],phase_before=8)
            ns['pipe_loop_syscall_entered'](call)
            ns['pipe_loop_scope']['refusals'].append(dict(name='model',candidate=pointer,size=4))
            returned = dict(**call,result=result,phase_after=phase)
            if accepted: ns['pipe_loop_syscall_returned'](returned)
            else:
                with self.subTest(result=result,pointer=pointer,phase=phase),self.assertRaises(AssertionError):
                    ns['pipe_loop_syscall_returned'](returned)

    def test_completion_requires_the_actual_positive_return_size(self):
        for result,pointer,accepted in [(4,0x2000,True),(3,0x2000,False),(-14,0x2000,False),(4,0x2001,False)]:
            ns = self.namespace()
            call = dict(number=1,args=[4,0x2000,4],phase_before=8)
            ns['pipe_loop_syscall_entered'](call)
            ns['pipe_loop_scope']['records'].append(dict(name='model',candidate=pointer,size=4))
            returned = dict(**call,result=result,phase_after=8)
            if accepted: ns['pipe_loop_syscall_returned'](returned)
            else:
                with self.subTest(result=result,pointer=pointer),self.assertRaises(AssertionError):
                    ns['pipe_loop_syscall_returned'](returned)


if __name__ == '__main__':
    unittest.main()
