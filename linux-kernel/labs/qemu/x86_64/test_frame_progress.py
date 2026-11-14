"""Host-runner guards; these do not qualify guest progress or kernel behavior."""
import unittest
from frame_root_inspect import OutputActivity


class OutputActivityTests(unittest.TestCase):
    def test_quiet_output_fails_at_the_inactivity_limit(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((0, 0), 4.99)
        with self.assertRaisesRegex(AssertionError, 'output inactive'):
            guard.check((0, 0), 5)

    def test_new_debugger_output_resets_only_the_inactivity_clock(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((0, 10), 4)
        guard.check((0, 10), 8.99)
        with self.assertRaises(AssertionError):
            guard.check((0, 10), 9)

    def test_repeated_polls_cannot_keep_quiet_output_alive(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((10, 0), 4)
        for now in [5, 6, 7, 8]:
            guard.check((10, 0), now)
        with self.assertRaises(AssertionError):
            guard.check((10, 0), 9)

    def test_invalid_inactivity_limits_fail_closed(self):
        for seconds in [0, -1, 1.5]:
            with self.subTest(seconds=seconds), self.assertRaises(AssertionError):
                OutputActivity(seconds, 0)


if __name__ == '__main__':
    unittest.main()
