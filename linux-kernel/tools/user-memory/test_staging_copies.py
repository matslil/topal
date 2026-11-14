"""Controls for bounded staging paths and observation lifetime.

TOPAL-SYSTEMS-USER-EXTENT-001. Scheduling models do not establish native memory
permissions, counterpart ownership or continuous mapping stability.
"""
from pathlib import Path
import runpy
import unittest

from audit_staging_copies import bounded_count_initializers


class StagingCopyControls(unittest.TestCase):
    def count_graph(self):
        raw = {0:'49 bf 02 00 00 00 00 00 00 00',10:'48 81 f8 03 00 00 00',
               17:'0f 84 10 00 00 00',23:'49 bf 05 00 00 00 00 00 00 00',
               33:'4c 39 fe',36:'0f 82 01 00 00 00'}
        return {at:dict(bytes=bytes.fromhex(value).hex()) for at,value in raw.items()}, {
            0:set(),10:{0},17:{10},23:{17},33:{17,23},36:{33},42:{36}}

    def test_every_reaching_count_definition_must_be_a_bounded_constant(self):
        nodes,preds = self.count_graph()
        self.assertEqual({r['value'] for r in bounded_count_initializers(nodes,preds,42,set())},{2,5})
        nodes[0]['bytes'] = '4d89ef'
        with self.assertRaises(AssertionError):
            bounded_count_initializers(nodes,preds,42,set())

    def test_a_new_entry_without_a_count_definition_is_not_a_bound(self):
        nodes,preds = self.count_graph()
        nodes[44] = dict(bytes='4c39fe')
        preds[44] = set()
        preds[33].add(44)
        with self.assertRaises(AssertionError):
            bounded_count_initializers(nodes,preds,42,set())

    def test_a_root_with_predecessors_does_not_inherit_their_count(self):
        nodes,preds = self.count_graph()
        for entry in [10,17,33,36]:
            with self.subTest(entry=entry), self.assertRaises(AssertionError):
                bounded_count_initializers(nodes,preds,42,{entry})

    def test_a_root_at_the_consumer_cannot_inherit_an_earlier_count(self):
        nodes,preds = self.count_graph()
        with self.assertRaises(AssertionError):
            bounded_count_initializers(nodes,preds,42,{42})

    def test_a_root_which_sets_a_bounded_count_remains_safe(self):
        nodes,preds = self.count_graph()
        definitions = bounded_count_initializers(nodes,preds,42,{0,23})
        self.assertEqual({row['value'] for row in definitions},{2,5})

    def observer(self):
        program = runpy.run_path(str(Path(__file__).resolve().parents[2] /
            'labs/qemu/x86_64/user_extent_copy_observer.py'))['PROGRAM']
        class Breakpoint:
            def __init__(self,*args,**kwargs): self.enabled = True
        row = dict(name='model',call_offset=0,carry_gate=14,copy_offset=30,copy_size=2,copy_width=1)
        audit = dict(passed=True,kernel_sha256='model',cases=[row],copy_groups=[dict(copy_offset=30,cases=['model'])])
        namespace = dict(Checked=Breakpoint,extent_inventory=dict(address=0,kernel_sha256='model',
            source_scoped_copies=True,copy_transfer_audit=audit,staging_copy_audit=audit))
        exec(program,namespace)
        namespace['install_copy_observation']()
        return namespace

    def test_heap_calls_do_not_arm_copy_breakpoints_and_open_transfers_cannot_close(self):
        ns = self.observer()
        for number in [12,1]:
            call = dict(number=number,args=[1,0x2000,3],phase_before=8)
            ns['copy_source_syscall_entered'](call)
            self.assertEqual(ns['copy_start_breakpoints'][30].enabled,number==1)
            if number == 1:
                ns['copy_pending'] = dict(accepted=True)
                with self.assertRaises(AssertionError): ns['copy_source_syscall_returned'](dict(**call,result=3,phase_after=8))
                ns['copy_pending'] = None
            ns['copy_source_syscall_returned'](dict(**call,result=0,phase_after=8))
            self.assertFalse(ns['copy_start_breakpoints'][30].enabled)

    def test_refusal_event_requires_actual_efault_and_matching_arguments(self):
        for result,pointer,phase,accepted in [(-14,0x2000,8,True),(-22,0x2000,8,False),
                (-14,0x2001,8,False),(-14,0x2000,9,False)]:
            ns = self.observer()
            call = dict(number=1,args=[1,0x2000,3],phase_before=8)
            ns['copy_source_syscall_entered'](call)
            ns['copy_source_scope']['refusals'].append(dict(name='model',candidate=pointer))
            returned = dict(**call,result=result,phase_after=phase)
            if accepted:
                ns['copy_source_syscall_returned'](returned)
                self.assertEqual(len(ns['copy_source_refusals']),1)
            else:
                with self.subTest(result=result,pointer=pointer,phase=phase), self.assertRaises(AssertionError):
                    ns['copy_source_syscall_returned'](returned)

    def test_source_completion_requires_success_and_the_actual_argument(self):
        for result,pointer,accepted in [(0,0x2000,True),(-14,0x2000,False),(0,0x2001,False)]:
            ns = self.observer()
            call = dict(number=5,args=[3,0x2000,0],phase_before=8)
            ns['copy_source_syscall_entered'](call)
            ns['copy_source_scope']['completed'].append(dict(name='model',candidate=pointer))
            returned = dict(**call,result=result,phase_after=8)
            if accepted:
                ns['copy_source_syscall_returned'](returned)
                self.assertEqual(len(ns['copy_source_completions']),1)
            else:
                with self.subTest(result=result,pointer=pointer),self.assertRaises(AssertionError):
                    ns['copy_source_syscall_returned'](returned)


if __name__ == '__main__':
    unittest.main()
