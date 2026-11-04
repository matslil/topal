"""Publication checks for actual ordinary allocation/release VM observations."""
import hashlib
import json
from pathlib import Path
import unittest

LAB = Path(__file__).resolve().parent


class RamPageEvidence(unittest.TestCase):
    def test_current_source_owner_dispositions_and_actual_placement(self):
        # TOPAL-SYSTEMS-REGION-PAGE-{OWNER,ALLOCATE,RELEASE}-001.
        record = json.loads((LAB/'results/topal-ram-page-region-gate.json').read_text())
        digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
        self.assertEqual(record['harness_sha256'], digest(LAB/'ram_page_gate.py'))
        self.assertEqual(record['inspector_sha256'], digest(LAB/'ram_page_inspect.py'))
        self.assertEqual(record['caller_sha256'], digest(LAB/'static_positional_file_checks.py'))
        kernel = record['primary']['linked_kernel_sha256']
        for name in ('primary', 'lifetime', 'last_fit', 'inactive_terminal'):
            case = record[name]
            observation = case['observation']
            self.assertTrue(observation['finalized'])
            self.assertTrue(observation['calls'])
            self.assertEqual(case['source_library']['instance'], 'ram-page-pool')
            self.assertEqual(case['source_library']['scalar_count'], 2)
            self.assertLessEqual(case['binding']['combined_stack_bytes'], 4096)
            self.assertEqual(case['binding']['map_bytes'], 8)
            self.assertTrue(all(call['physical_cleanup'] and call['all_gprs_preserved']
                                and call['exact_association_publication']
                                and len(call['source_caller_chain']) == 4 for call in observation['calls']))
            self.assertTrue(any(call['action'] == 0 for call in observation['calls']))
            self.assertTrue(any(call['action'] == 1 for call in observation['calls']))
            if name != 'last_fit': self.assertEqual(case['linked_kernel_sha256'], kernel)
        self.assertEqual(record['primary']['observation']['calls'][0]['selected'], 0)
        self.assertEqual(record['last_fit']['observation']['calls'][0]['selected'], 7)
        self.assertNotEqual(record['primary']['source_library']['object_sha256'],
                            record['last_fit']['source_library']['object_sha256'])
        for mode in ('false', 'wrong-byte', 'multiple-delta'):
            for suffix, action in (('', 0), ('-release', 1)):
                case = record[mode+suffix]
                failed = case['observation']
                self.assertEqual(failed['result'], 'expected-admitted-action-fatal')
                self.assertEqual(failed['action'], action)
                self.assertEqual(failed['actual_source_result'], int(mode != 'false'))
                self.assertTrue(failed['physical_cleanup'] and failed['source_stores_survive']
                                and failed['no_association_publication'] and failed['no_owner_resumption'])
                self.assertNotEqual(failed['map_before'], failed['map_after'])
                self.assertTrue(case['debugger_completed_before_teardown'])
        for fault in ('foreign-owner', 'object-stride', 'interrupts-enabled', 'dirty-arena', 'generation-exhausted'):
            for suffix in ('-allocation', '-release'):
                case = record[fault+suffix]
                self.assertEqual(case['linked_kernel_sha256'], kernel)
                refused = case['observation']
                self.assertEqual(refused['result'], 'expected-pre-entry-refusal')
                self.assertTrue(refused['no_source_entry'] and refused['no_owner_resumption']
                                and refused['storage_unchanged'])
                self.assertEqual(refused['release_before_erasure'], suffix == '-release')
                self.assertTrue(case['debugger_completed_before_teardown'])
        self.assertTrue(record['inactive_terminal']['observation']['inactive_child_retained_at_parent_exit'])
        self.assertEqual(record['native_only']['result'], 'pass')
        self.assertTrue(record['native_only']['source_and_binding_absent'])
        self.assertEqual(record['native_only']['profile']['schema'], 'topal.volatile-file.x86_64/2')
        self.assertNotIn('page_allocation_map_offset', record['native_only']['profile'])


if __name__ == '__main__':
    unittest.main()
