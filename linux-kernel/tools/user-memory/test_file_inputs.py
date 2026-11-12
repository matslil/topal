"""RTC effect, count-alias and live spill-slot counterexamples.

TOPAL-SYSTEMS-USER-EXTENT-001. These models grant no native owners/lifetime.
"""
from pathlib import Path
import runpy
import unittest

from file_input_effects import effects


class FileInputControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_copy_observer.py'
        ns=dict(Checked=type('Breakpoint',(),{}))
        exec(runpy.run_path(str(path))['PROGRAM'],ns)
        cls.live_words=staticmethod(ns['copy_live_stack_words'])

    def test_stack_shape_is_exact_and_ordinary_copies_keep_the_call_rsp(self):
        self.assertEqual(self.live_words({},0x1000,0x1000),0)
        self.assertEqual(self.live_words(dict(copy_live_stack_words=1),0x1000,0xff8),1)
        for row,current in [({},0xff8),(dict(copy_live_stack_words=1),0x1000),
                            (dict(copy_live_stack_words=1),0xff0),
                            (dict(copy_live_stack_words=2),0xff0),
                            (dict(copy_live_stack_words=-1),0x1008)]:
            with self.subTest(row=row,current=current),self.assertRaises(AssertionError):
                self.live_words(row,0x1000,current)

    def test_stack_pointer_sources_cannot_hide_behind_other_destination_registers(self):
        for raw in ['4889e7','4989e5','4989a42480050000','4869fc01000000']:
            with self.subTest(raw=raw): self.assertTrue(effects(bytes.fromhex(raw))['stack_pointer_operand'])
        self.assertFalse(effects(bytes.fromhex('4c89e7'))['stack_pointer_operand'])

    def test_multiply_effect_uses_rex_destination_including_the_stack_register(self):
        self.assertEqual(effects(bytes.fromhex('4d69ff80510100'))['register_writes'],{15})
        self.assertEqual(effects(bytes.fromhex('4869e001000000'))['register_writes'],{4})

    def test_rtc_index_io_and_byte_extension_expose_their_effects(self):
        self.assertEqual(effects(bytes.fromhex('e471'))['register_writes'],{0})
        self.assertEqual(effects(bytes.fromhex('e670'))['register_writes'],set())
        self.assertEqual(effects(bytes.fromhex('e670'))['io'],('out',0x70))
        self.assertEqual(effects(bytes.fromhex('0fb6c0'))['register_writes'],{0})

    def test_captured_table_reload_is_the_exact_selected_gs_field(self):
        row=effects(bytes.fromhex('654c8b2425a0030000'))
        self.assertEqual(row['register_writes'],{12})
        self.assertEqual(row['memory_reads'],[('captured-gs',928,8)])
        with self.assertRaises(AssertionError): effects(bytes.fromhex('654c8b2425a8030000'))

    def test_unreviewed_memory_multiply_and_partial_stack_loads_fail_closed(self):
        for raw in ['4869242401000000','8b2424','89e7','0f05','9d']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))


if __name__=='__main__': unittest.main()
