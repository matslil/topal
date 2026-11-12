"""Observer selection cannot publish evidence from an unrun observer group."""
from pathlib import Path
import runpy
import sys
import unittest


class ObserverSelection(unittest.TestCase):
    def namespace(self):
        qemu = str(Path(__file__).resolve().parents[2] / 'labs/qemu/x86_64')
        sys.path.insert(0, qemu)
        try:
            program = runpy.run_path(qemu + '/user_extent_call_coverage.py')['PROGRAM']
        finally:
            sys.path.remove(qemu)
        class Breakpoint:
            def __init__(self, *args, **kwargs):
                pass
        namespace = dict(Checked=Breakpoint)
        exec(program, namespace)
        namespace.update(extent_inventory=dict(address=100, function='model', calls=[dict(offset=0)],
                         kernel_sha256='model', inventory_sha256='model'),
                         extent_first_reach={0: dict(offset=0)}, address=lambda name: 100,
                         observed={})
        for name in ['callback', 'copy', 'staged', 'pipe_fixed']:
            namespace[name + '_events'] = []
        for name, install, complete in [
            ('callback', 'install_callback_observation', 'callback_coverage_complete'),
            ('copy', 'install_copy_observation', 'copy_observation_complete'),
            ('staged', 'install_staged_observation', 'staged_observation_complete'),
            ('pipe_fixed', 'install_pipe_fixed_observation', 'pipe_fixed_observation_complete')]:
            namespace[install] = lambda name=name: namespace[name + '_events'].append('install')
            namespace[complete] = lambda name=name: namespace[name + '_events'].append('complete')
        return namespace

    def test_existing_cohorts_keep_all_observer_groups(self):
        ns = self.namespace()
        ns['install_extent_coverage']()
        ns['extent_coverage_complete']()
        for name in ['callback', 'copy', 'staged']:
            self.assertEqual(ns[name + '_events'], ['install', 'complete'])

    def test_staged_cohort_never_completes_unrun_observers(self):
        ns = self.namespace()
        ns['extent_inventory']['native_observers'] = ['staged']
        ns['install_extent_coverage']()
        ns['extent_coverage_complete']()
        self.assertEqual(ns['staged_events'], ['install', 'complete'])
        self.assertEqual(ns['callback_events'] + ns['copy_events'], [])

    def test_pipe_observation_runs_only_when_explicitly_selected(self):
        for selected in [False,True]:
            ns = self.namespace()
            if selected:
                ns['extent_inventory']['native_observers'] = ['callbacks','copies','staged','pipe-fixed']
            ns['install_extent_coverage']()
            ns['extent_coverage_complete']()
            self.assertEqual(ns['pipe_fixed_events'],['install','complete'] if selected else [])

    def test_unknown_empty_and_duplicate_selections_fail_closed(self):
        for groups in [[], ['unknown'], ['staged', 'staged']]:
            ns = self.namespace()
            ns['extent_inventory']['native_observers'] = groups
            with self.subTest(groups=groups), self.assertRaises(AssertionError):
                ns['install_extent_coverage']()
            self.assertEqual(ns['callback_events'] + ns['copy_events'] + ns['staged_events'], [])

    def source_namespace(self):
        ns = self.namespace()
        # Actual observer scheduling, with no guest translation/memory evidence.
        row = dict(name='message',direction='read',length=3,call_offset=0,
                   carry_gate=2,start_offset=3,end_offset=4)
        ns['extent_inventory'].update(source_scoped_staged=True,
            staged_transfer_audit=dict(passed=True,kernel_sha256='model',cases=[row]))
        # namespace() stubs only the installer; load the actual staged program
        # again to exercise its real breakpoint registry and scoped lifecycle.
        path = Path(__file__).resolve().parents[2] / 'labs/qemu/x86_64/user_extent_staged_observer.py'
        exec(runpy.run_path(str(path))['PROGRAM'], ns)
        ns['install_staged_observation']()
        return ns

    def test_source_scope_keeps_staged_breakpoints_off_during_heap_calls(self):
        ns = self.source_namespace()
        for number in [12, 1, 12]:
            call = dict(number=number,args=[1,0x2000,3],phase_before=8)
            ns['staged_source_syscall_entered'](call)
            for key in ['staged_call_breakpoints','staged_result_breakpoints','staged_start_breakpoints']:
                self.assertEqual(ns[key]['message'].enabled, number == 1)
            self.assertFalse(ns['staged_finish_breakpoints']['message'].enabled)
            ns['staged_source_syscall_returned'](dict(**call,result=0,phase_after=8))
            self.assertIsNone(ns['staged_source_scope'])
            self.assertFalse(ns['staged_start_breakpoints']['message'].enabled)

    def test_refusal_requires_matching_pointer_efault_and_unchanged_phase(self):
        for result, phase, pointer, accepted in [(-14,8,0x2000,True),
                (-22,8,0x2000,False),(-14,9,0x2000,False),(-14,8,0x2001,False)]:
            ns = self.source_namespace()
            call = dict(number=1,args=[1,0x2000,3],phase_before=8)
            ns['staged_source_syscall_entered'](call)
            ns['staged_source_scope']['refusals'].append(dict(name='message',candidate=pointer))
            returned = dict(**call,result=result,phase_after=phase)
            if accepted:
                ns['staged_source_syscall_returned'](returned)
                self.assertEqual(len(ns['staged_refusal_events']),1)
            else:
                with self.subTest(result=result,phase=phase,pointer=pointer), self.assertRaises(AssertionError):
                    ns['staged_source_syscall_returned'](returned)


if __name__ == '__main__':
    unittest.main()
