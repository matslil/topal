"""Counterexamples for finite-loop bounds, captured reloads and observation."""
from pathlib import Path
import runpy
import struct
import unittest

from audit_staged_transfers import finite_indices, gs_effect, preparation


class StagedTransferControls(unittest.TestCase):
    def test_body_first_loops_require_positive_nonwrapping_signed_immediate(self):
        for count in [1, 14, 109, 390]:
            indices = list(finite_indices(count))
            self.assertEqual((indices[0], indices[-1], len(indices)), (0, count-1, count))
        for count in [0, -1, 1 << 31, 1 << 64]:
            with self.subTest(count=count), self.assertRaises(AssertionError):
                finite_indices(count)

    def test_gs_load_and_store_effects_are_not_owner_facts(self):
        load = gs_effect(bytes.fromhex('65 48 8b 34 25 40 01 00 00'))
        self.assertEqual(load, dict(field=320, width=8, action='read', register_write=6))
        store = gs_effect(bytes.fromhex('65 48 c7 04 25 e8 02 00 00 00 00 00 00'))
        self.assertEqual((store['field'], store['action'], store['register_write']), (744, 'write', None))
        self.assertNotIn('owner', load)
        self.assertNotIn('owner', store)

    def test_unsupported_segments_partial_reload_and_index_modes_refused(self):
        for text in ['64 48 8b 34 25 40 01 00 00', '65 8b 34 25 40 01 00 00',
                     '65 48 8b 34 16', '65 48 01 34 25 40 01 00 00']:
            with self.subTest(text=text), self.assertRaises((AssertionError, ValueError)):
                gs_effect(bytes.fromhex(text))

    def prep(self, raw):
        start = 6+len(raw)
        node = dict(offset=6, size=len(raw), bytes=raw.hex(), instruction='model effect')
        return preparation({6: node}, {6: {start}}, {6: {0}, start: {6}},
            {0}, bytes(start+1), 0, start, 6, 320)

    def test_captured_pointer_cannot_be_overwritten_or_reloaded_from_another_field(self):
        self.assertEqual(len(self.prep(bytes.fromhex('65 48 8b 34 25 40 01 00 00'))), 1)
        for field in [319, 320, 324, 327]:
            raw = bytes.fromhex('65 48 c7 04 25')+struct.pack('<I', field)+bytes(4)
            with self.subTest(field=field), self.assertRaises(AssertionError):
                self.prep(raw)
        with self.assertRaises(AssertionError):
            self.prep(bytes.fromhex('65 48 8b 34 25 48 01 00 00'))

    def test_preparation_cannot_hide_candidate_arithmetic_or_calls(self):
        for raw in ['48 81 c6 01 00 00 00', 'e8 00 00 00 00', '0f 01 f8']:
            with self.subTest(raw=raw), self.assertRaises(AssertionError):
                self.prep(bytes.fromhex(raw))

    def test_actual_observer_pairs_loop_boundaries_and_checks_final_index(self):
        # Scheduling/candidate model only: translation and memory checks are
        # explicitly stubbed and provide no native permission evidence.
        program = runpy.run_path(str(Path(__file__).resolve().parents[2] /
            'labs/qemu/x86_64/user_extent_staged_observer.py'))['PROGRAM']
        class Breakpoint:
            def __init__(self, *args, **kwargs):
                self.enabled = True
        namespace = {'Checked': Breakpoint}
        exec(program, namespace)
        registers = dict(rsi=0x2000, r10=0, rsp=0x4000, gs_base=0x1000)
        names = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15']
        row = dict(name='model', start_offset=10, end_offset=30, call_offset=0,
            user_register=6, index_register=10, length=3, direction='read',
            counterpart={'kind': 'gs', 'offset': 744})
        import hashlib
        namespace.update(extent_inventory={'address': 0}, context=0x1000,
            copy_register_names=names, reg=lambda r: registers[r],
            memory=lambda p, n: b'abc' if n == 3 else bytes(n),
            copy_mode=lambda previous=None: None, copy_range=lambda *args: [],
            staged_counts={'model': {'completed': 0, 'bytes': 0}}, hashlib=hashlib)
        begin, end = namespace['StagedStarted'](row), namespace['StagedFinished'](row)
        namespace['staged_start_breakpoints']['model'] = begin
        namespace['staged_finish_breakpoints']['model'] = end
        self.assertFalse(end.enabled)
        namespace['staged_pending'] = dict(name='model', accepted=True, candidate=0x2000,
            size=3, rsp=0x4000, mode=None, captured=bytes(64))
        begin.observe()
        self.assertFalse(begin.enabled)
        self.assertTrue(end.enabled)
        registers['r10'] = 4
        with self.assertRaises(AssertionError):
            end.observe()
        registers['r10'] = 3
        end.observe()
        self.assertTrue(begin.enabled)
        self.assertFalse(end.enabled)
        self.assertEqual(namespace['staged_counts']['model'], {'completed': 1, 'bytes': 3})


if __name__ == '__main__':
    unittest.main()
