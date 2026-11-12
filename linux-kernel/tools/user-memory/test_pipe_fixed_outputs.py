"""Counterexamples for pipe output endpoints and alternate graph entries.

TOPAL-SYSTEMS-USER-EXTENT-001. Small graph models do not establish native
mapping permissions, callee behavior, actual owners or stable lifetime.
"""
import copy
import unittest

from audit_pipe_fixed_outputs import Instructions, check_local


class PipeFixedOutputControls(unittest.TestCase):
    def interval(self, encoding='418900'):
        raw = bytes.fromhex(encoding)
        row = dict(offset=20, bytes=raw.hex(), size=len(raw))
        return {20: row}, {0}, {0: {14}, 14: {20, 30}}, {20: {14}}, [row]

    def check(self, data, direction='write', width=4):
        nodes, roots, successors, predecessors, rows = data
        check_local(nodes, roots, successors, predecessors, 14, 20, rows,
                    [(rows[0], direction, width)])

    def test_writable_admission_allows_exact_output_and_bounded_readback(self):
        self.check(self.interval())
        self.check(self.interval('49898000000000'), width=8)
        self.check(self.interval('4d8b9800000000'), direction='read', width=8)

    def test_different_base_width_or_offset_cannot_use_the_same_admission(self):
        for encoding in ['418901', '498900', '41894004']:
            with self.subTest(encoding=encoding), self.assertRaisesRegex(
                    AssertionError, 'pipe output endpoint'):
                self.check(self.interval(encoding))

    def test_a_root_cannot_enter_an_admitted_interval_directly(self):
        data = self.interval()
        data[1].add(20)
        with self.assertRaisesRegex(AssertionError, 'alternate local root'):
            self.check(data)

    def test_external_predecessor_cannot_enter_an_admitted_interval(self):
        data = self.interval()
        data[3][20].add(40)
        with self.assertRaisesRegex(AssertionError, 'alternate local entry'):
            self.check(data)

    def test_closed_local_predecessors_are_not_a_global_dominance_proof(self):
        # A second edge from the gate enters a later local instruction. Both
        # local predecessors are allowed; removing the success edge still
        # leaves a root-to-access path. Edges and predecessors agree.
        first = dict(offset=20, bytes='90', size=1)
        access = dict(offset=21, bytes='418900', size=3)
        nodes = {20: first, 21: access}
        successors = {0: {14}, 14: {20, 21}, 20: {21}}
        predecessors = {20: {14}, 21: {14, 20}}
        with self.assertRaisesRegex(AssertionError, 'success-edge dominance'):
            check_local(nodes, {0}, successors, predecessors, 14, 20,
                        [first, access], [(access, 'write', 4)])

    def test_the_success_edge_must_actually_reach_the_access(self):
        data = self.interval()
        data[2][14].remove(20)
        with self.assertRaises(AssertionError):
            self.check(data)

    def test_resealed_instruction_metadata_cannot_redirect_the_candidate(self):
        original = bytes.fromhex('4d89e6')  # MOV R14,R12 in reviewed preparation.
        for changed in ['4d89e0', '4531c0', '0fa290']:
            raw = bytes.fromhex(changed)
            nodes = {0: dict(bytes=raw.hex(), size=len(raw))}
            with self.subTest(changed=changed), self.assertRaisesRegex(
                    AssertionError, 'pipe instruction contract'):
                Instructions(raw, nodes, 0).take(original.hex())

    def test_decoded_boundaries_cannot_split_a_reviewed_instruction(self):
        raw = bytes.fromhex('418900')
        for node in [dict(bytes='4189', size=2), dict(bytes='418900', size=2)]:
            with self.subTest(node=node), self.assertRaises(AssertionError):
                Instructions(raw, {0: copy.deepcopy(node)}, 0).take(raw.hex())


if __name__ == '__main__':
    unittest.main()
