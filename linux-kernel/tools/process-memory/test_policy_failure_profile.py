"""Saved/model controls for reuse of the prior full compiler qualification.

TOPAL-SYSTEMS-PROCESS-MM-001. No guest or native-compiler execution here.
"""
import json
from unittest import TestCase
from unittest.mock import patch

import run_policy_failure as driver


class QualificationProfileTests(TestCase):
    def test_published_suite_can_only_cover_the_explicit_private_test_increment(self):
        private=b'src/topal-compiler/src/systems_process_mm_fault_tests.rs\n'
        path=driver.ROOT/'linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json'
        archived=json.loads(path.read_bytes())['qualification']['source_hashes_before']
        # Model the successful reference state. The current workspace may
        # legitimately have a later fixture/compiler change requiring full QA.
        def reference_hashes(paths):return {name:archived[name] for name in paths}
        with patch.object(driver.subprocess,'check_output',return_value=private):
            with patch.object(driver,'hashes',side_effect=reference_hashes):
                with patch.object(driver,'watched',return_value=sorted(archived)):
                    reference=driver.observer_reference()
        self.assertEqual(reference['commit'],driver.REFERENCE_COMMIT)
        self.assertGreater(reference['unchanged_compiler_inputs'],100)
        self.assertIn('compiler-units',driver.validation_commands('full'))
        self.assertNotIn('compiler-units',driver.validation_commands('observer'))

    def test_production_change_cannot_use_observer_qualification(self):
        changed=b'src/topal-compiler/src/runtime/kernel_frame_memory.ll\n'
        with patch.object(driver.subprocess,'check_output',return_value=changed):
            with self.assertRaises(AssertionError):driver.observer_reference()

    def test_another_private_fixture_change_also_requires_full_qualification(self):
        changed=b'src/topal-compiler/src/systems_user_source_fixture.rs\n'
        with patch.object(driver.subprocess,'check_output',return_value=changed):
            with self.assertRaises(AssertionError):driver.observer_reference()

    def test_changed_compiler_bytes_refuse_even_with_a_clean_diff_listing(self):
        path=driver.ROOT/'linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json'
        archived=json.loads(path.read_bytes())['qualification']['source_hashes_before']
        def altered(paths):
            result={name:archived[name] for name in paths}
            result['src/topal-compiler/src/runtime/kernel_frame_memory.ll']='0'*64
            return result
        with patch.object(driver.subprocess,'check_output',return_value=b''):
            with patch.object(driver,'hashes',side_effect=altered):
                with patch.object(driver,'watched',return_value=sorted(archived)):
                    with self.assertRaises(AssertionError):driver.observer_reference()

    def test_added_or_removed_compiler_input_refuses_before_hash_reuse(self):
        path=driver.ROOT/'linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json'
        archived=json.loads(path.read_bytes())['qualification']['source_hashes_before']
        changed=[set(archived)|{'src/topal-compiler/src/new_systems_component.rs'},
                 set(archived)-{'src/topal-compiler/src/runtime/kernel_frame_memory.ll'}]
        for paths in changed:
            with self.subTest(paths=len(paths)):
                with patch.object(driver.subprocess,'check_output',return_value=b''):
                    with patch.object(driver,'watched',return_value=sorted(paths)):
                        with patch.object(driver,'hashes') as hash_read:
                            with self.assertRaises(AssertionError):driver.observer_reference()
                            hash_read.assert_not_called()

    def test_failed_reference_compiler_suite_cannot_be_reused(self):
        path=driver.ROOT/'linux-kernel/tools/process-memory/results/stage-3-process-policy-failure-review.json'
        reference=json.loads(path.read_bytes())
        reference['qualification']['validation']['compiler-units']['exit_code']=101
        with patch.object(driver.subprocess,'check_output',return_value=b''):
            with patch.object(driver.Path,'read_bytes',return_value=json.dumps(reference).encode()):
                with self.assertRaises(AssertionError):driver.observer_reference()
