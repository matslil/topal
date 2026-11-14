"""Source-case completion differs from a pending native memory operation.

TOPAL-SYSTEMS-PROCESS-MM-001. Host-only completion controls; native terminal
retirement assertions remain in each original cohort's observer and reader.
"""
import unittest

from readback_publication import cohort_complete


class CompletionControls(unittest.TestCase):
    def value(self, active):
        return dict(passed=True, pending=None, refusal_pending=None, active_case=active)

    def test_returning_cases_require_their_return_marker(self):
        for mode in ['process-mm', 'process-mm-exec']:
            with self.subTest(mode=mode):
                cohort_complete(self.value(None), mode)
                with self.assertRaisesRegex(ValueError, 'source case completion'):
                    cohort_complete(self.value(dict(before={})), mode)

    def test_terminal_cases_retain_the_original_case_marker(self):
        for mode in ['process-mm-allocation', 'process-mm-cancel', 'process-mm-relationship',
                     'process-mm-geometry-56', 'process-mm-geometry-57']:
            with self.subTest(mode=mode):
                cohort_complete(self.value(dict(before={})), mode)
                with self.assertRaisesRegex(ValueError, 'source case completion'):
                    cohort_complete(self.value(None), mode)

    def test_pending_request_and_refusal_are_never_completion(self):
        for mode, active in [('process-mm', None), ('process-mm-allocation', dict(before={}))]:
            for name in ['pending', 'refusal_pending']:
                value = self.value(active)
                value[name] = dict(request=1)
                with self.subTest(mode=mode, name=name), self.assertRaisesRegex(ValueError, 'cohort incomplete'):
                    cohort_complete(value, mode)

    def test_failed_observation_is_never_completion(self):
        for change in [dict(passed=False), dict(failure='failed native assertion')]:
            value = self.value(None)
            value.update(change)
            with self.assertRaisesRegex(ValueError, 'cohort incomplete'):
                cohort_complete(value, 'process-mm')

    def test_unrecognized_source_cohort_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'source cohort'):
            cohort_complete(self.value(None), 'unrecognized')
