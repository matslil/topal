"""Publication contract for actual source-driven Linux inode mode operations."""
import gzip
import hashlib
import json
from pathlib import Path
import unittest
import topal_gate as GATE

LAB = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class InodeModeEvidence(unittest.TestCase):
    def test_actual_source_mutation_admission_lifetime_and_physical_cleanup(self):
        # TOPAL-SYSTEMS-INODE-MODE-{STATE,ABI,ACTION}-001.
        # TOPAL-SYSTEMS-LINUX-DISPATCH-V5-001.
        archive = LAB/'results/tk280/stage-3'
        proof = json.loads((archive/'inode-qualification.json').read_text())
        self.assertEqual((proof['schema'], proof['result']), ('topal-inode-mode-qualification/1', 'pass'))
        self.assertEqual(proof['suite_sha256'], digest(LAB/'inode_mode_suite.py'))
        cases = proof['cases']
        faults = ('foreign-owner', 'record-stride', 'interrupts-enabled', 'permissions-range',
                  'dirty-arena', 'generation-exhausted', 'bad-type')
        self.assertEqual(set(cases), {'primary', 'lifetime', 'inactive_terminal',
            'false-after-mode', 'wrong-mode', *faults})
        kernel = proof['linked_kernel_sha256']
        selected = json.loads((LAB/'results/topal-toolchain-gate.json').read_text())
        self.assertEqual(kernel, selected['linked_kernel_sha256'])
        for name, case in cases.items():
            for key, filename in [('harness_sha256', 'inode_mode_gate.py'),
                ('inspector_sha256', 'inode_mode_inspect.py'), ('caller_sha256', 'static_inode_mode_checks.py'),
                ('gate_sha256', 'topal_gate.py')]:
                self.assertEqual(case[key], digest(LAB/filename))
            if name not in ('false-after-mode', 'wrong-mode'):
                self.assertEqual(case['linked_kernel_sha256'], kernel)
            if name not in ('primary', 'lifetime'):
                self.assertIs(case['stopped_target']['running'], False)
                self.assertEqual(case['stopped_target']['status'], 'debug')
        for name, calls, actions in [('primary', 82, 14), ('lifetime', 131, 20), ('inactive_terminal', 106, 16)]:
            case = cases[name]
            state, bound = case['observation'], case['binding']
            self.assertTrue(state['finalized'])
            self.assertEqual((len(state['syscalls']), len(state['calls']), state['generation']), (calls, actions, actions))
            self.assertEqual((bound['maximum_source_stack_bytes'], bound['live_frame_bytes'],
                bound['caller_alignment_bytes'], bound['combined_stack_bytes']), (1296, 184, 64, 1544))
            self.assertEqual((bound['directory_metadata_offset'], bound['file_object_offset'],
                bound['file_metadata_offset'], bound['pipe_pool_offset'], bound['pipe_metadata_offset'],
                bound['pipe_creation_time_offset'], bound['metadata_bytes']),
                (0x43020, 0x43100, 24, 0x3e000, 64, 96, 32))
            self.assertEqual((case['source_library']['instance'], bound['instance'], bound['scalar_count']),
                ('inode-metadata', 'inode-metadata', 2))
            self.assertEqual(case['source_library']['allocation']['numeric']['maximum_bytes'], 12208)
            self.assertEqual({v['kind'] for v in state['calls']}, {0, 1, 2})
            for generation, call in enumerate(state['calls'], 1):
                self.assertEqual(call['generation'], generation)
                self.assertEqual((call['ordinary_entries'], call['source_result'], call['loads'], call['stores']), (1, 1, 8, 16))
                self.assertTrue(call['source_returned'] and call['complete_physical_cleanup']
                    and call['actual_owner_exact_delta'] and call['all_caller_gprs_and_simd_preserved'])
                self.assertGreater(call['seconds'], 0)
                self.assertLessEqual(call['seconds'], (1 << 64)-1)
                self.assertLessEqual(call['permissions'], 0o7777)
                self.assertEqual(call['before'][0] & ~0o7777, (0o100000, 0o40000, 0o10000)[call['kind']])
                self.assertLessEqual(call['arena_peak'], 12208)
            self.assertEqual(state['result'], 'pass-controlled-inactive-terminal' if name == 'inactive_terminal' else 'pass')
        self.assertTrue(cases['inactive_terminal']['observation']['inactive_child_retained_at_parent_exit'])
        for name in faults:
            state = cases[name]['observation']
            self.assertEqual((state['result'], state['fault']), ('expected-pre-entry-refusal', name))
            self.assertTrue(state['source_not_entered'] and state['no_owner_resumption']
                and state['complete_owner_and_runtime_unchanged'])
        for name in ('false-after-mode', 'wrong-mode'):
            case, state = cases[name], cases[name]['observation']
            self.assertNotEqual(case['linked_kernel_sha256'], kernel)
            self.assertTrue(case['native_root_and_provider_unchanged'])
            self.assertEqual(state['result'], 'expected-admitted-action-fatal')
            self.assertTrue(state['no_owner_resumption'] and state['complete_physical_cleanup']
                and state['source_stores_survive'])
            before, after = state['before'], state['after']
            self.assertEqual(after[1:3], before[1:3])
            if name == 'false-after-mode':
                self.assertEqual(state['source_result'], 0)
                self.assertEqual(after[0], before[0] & ~0o7777 | state['permissions'])
                self.assertEqual(after[3], before[3])
            else:
                self.assertEqual((state['source_result'], after[0], after[3]), (1, 0, state['seconds']))
        control = json.loads((LAB/'results/topal-inode-mode-linux-control.json').read_text())
        self.assertIs(control['passed'], True)
        self.assertEqual((control['schema'], control['kernel_release'], control['effective_uid']),
            ('topal-linux-inode-mode-control/1', '7.2.9', 0))
        self.assertEqual(len(control['events']), 33)
        self.assertTrue(control['qemu_snapshot'])
        self.assertEqual(control['driver_sha256'], digest(LAB/'inode_mode_linux.py'))
        self.assertEqual(control['guest_source_sha256'], digest(LAB/'inode_mode_linux_guest.py'))
        self.assertEqual(control['vm_manifest_sha256'], digest(LAB/'manifest.json'))
        reference = json.loads((LAB/'results/topal-volatile-file-linux-control.json').read_text())
        self.assertEqual(control['kernel_sha256'], reference['reference_kernel_sha256'])
        events = {e['case']: e for e in control['events']}
        self.assertEqual(events['chmod-readonly-existing-target']['errno'], 30)
        self.assertEqual(events['chmod-missing-target-before-readonly-mount-denial']['errno'], 2)
        self.assertEqual(events['fchmodat2-flags-before-copy-and-fd']['errno'], 22)
        index = json.loads((archive/'archive.json').read_text())
        self.assertEqual((index['machine_dispositions'], index['busybox_workloads'], len(index['logs'])), (12, 5, 49))
        for entry in index['logs']:
            data = gzip.decompress((archive/entry['path']).read_bytes())
            self.assertEqual(len(data), entry['uncompressed_bytes'])
            self.assertEqual(hashlib.sha256(data).hexdigest(), entry['uncompressed_sha256'])
        utilities = json.loads((archive/'inode-busybox.json').read_text())
        self.assertEqual(utilities['result'], 'pass')
        self.assertEqual(set(utilities['workloads']), {'mode-numeric', 'mode-symbolic', 'mode-directory',
            'mode-retained-reader', 'mode-umask-independent'})
        for name, case in utilities['workloads'].items():
            self.assertEqual(case['linked_kernel_sha256'], kernel)
            self.assertEqual(case['static_user_image_sha256'], utilities['busybox_sha256'])
            self.assertEqual(case['process_output_utf8'].encode(), GATE.PROCESS_WORKLOADS[name][1])


if __name__ == '__main__':
    unittest.main()
