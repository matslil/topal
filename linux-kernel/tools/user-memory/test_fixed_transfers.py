"""Negative controls for the conditional scalar/dominance checker (not native).

TOPAL-SYSTEMS-USER-EXTENT-001: aliases, raw x86 operand widths, and nonvacuous
success-edge dominance must not be inferred from an addressed register name.
"""
import unittest

from audit_fixed_transfers import reachable, scalar


class FixedTransferControls(unittest.TestCase):
    def test_refusal_path_can_bypass_a_success_edge(self):
        graph = {0: {1}, 1: {2, 3}, 2: {4}, 3: {5}, 4: {5}}
        self.assertIn(4, reachable({0}, graph))
        self.assertNotIn(4, reachable({0}, graph, (1, 2)))
        graph[3].add(4)
        self.assertIn(4, reachable({0}, graph, (1, 2)))

    def test_additional_root_invalidates_dominance(self):
        graph = {0: {1}, 1: {2}, 2: {3}}
        self.assertNotIn(3, reachable({0}, graph, (1, 2)))
        self.assertIn(3, reachable({0, 2}, graph, (1, 2)))
        self.assertNotIn(7, reachable({0}, graph))  # Absence alone is vacuous.

    def test_cycles_and_call_return_assumptions_remain_conservative(self):
        graph = {0: {1}, 1: {2, 8}, 2: {3}, 3: {1}, 8: {9}}
        self.assertEqual(reachable({0}, graph), {0, 1, 2, 3, 8, 9})
        self.assertIn(9, reachable({0}, graph, (1, 2)))

    def test_raw_width_and_displacement_include_last_byte(self):
        self.assertEqual(scalar(bytes.fromhex('44 0f b7 4e 04')),
                         ('read', 4, 2, 6, 9))
        self.assertEqual(scalar(bytes.fromhex('49 c7 42 08 ff ff ff ff')),
                         ('write', 8, 8, 10, None))
        self.assertEqual(scalar(bytes.fromhex('66 83 7e 10 66')),
                         ('read', 16, 2, 6, None))
        self.assertEqual(scalar(bytes.fromhex('48 8b 76 ff')),
                         ('read', -1, 8, 6, 6))  # Also reports base clobber.

    def test_unknown_address_modes_and_read_modify_write_refused(self):
        for raw in ['48 8b 04 16', '48 8b 05 00 00 00 00',
                    '48 83 06 01', '65 48 8b 06', '67 48 8b 06',
                    'f3 a4', '48 87 06', '48 c7 06 00 00 00 00 00']:
            with self.subTest(raw=raw), self.assertRaises(AssertionError):
                scalar(bytes.fromhex(raw))


if __name__ == '__main__':
    unittest.main()
