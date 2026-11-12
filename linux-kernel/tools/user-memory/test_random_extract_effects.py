"""Counterexamples for extraction scratch/pointer/register effect boundaries.

TOPAL-SYSTEMS-USER-EXTENT-001.
"""
import unittest

from random_extract_effects import effects


class RandomExtractEffects(unittest.TestCase):
    def test_dword_scratch_stores_and_loads_have_exact_width_and_rex_destination(self):
        self.assertEqual(effects(bytes.fromhex('448b44247c'))['register_writes'],{8})
        self.assertEqual(effects(bytes.fromhex('448944247c'))['memory_writes'],[(4,124,4)])
        self.assertEqual(effects(bytes.fromhex('4889442478'))['memory_writes'],[(4,120,8)])

    def test_key_and_output_bases_cannot_be_confused_with_stack_storage(self):
        self.assertEqual(effects(bytes.fromhex('4189421c'))['memory_writes'],[(10,28,4)])
        self.assertEqual(effects(bytes.fromhex('4189431c'))['memory_writes'],[(11,28,4)])
        with self.assertRaises(AssertionError): effects(bytes.fromhex('418944241c'))

    def test_only_the_reviewed_scratch_allocation_has_an_explicit_stack_delta(self):
        self.assertEqual(effects(bytes.fromhex('4881ec80000000'))['scratch_delta'],-128)
        self.assertEqual(effects(bytes.fromhex('4881c480000000'))['scratch_delta'],128)
        for raw in ['4881ec88000000','4881c488000000','4889e0','4889c4','5c','e800000000']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))

    def test_partial_stack_pointer_and_pointer_alias_writes_fail_closed(self):
        for raw in ['31e4','448b54247c','448b5c247c','89442480','41896200']:
            # A negative displacement is exposed; the frame checker must reject
            # it against actual scratch bounds rather than silently zero-extend.
            if raw=='89442480':
                self.assertEqual(effects(bytes.fromhex(raw))['memory_writes'],[(4,-128,4)])
            else:
                with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))

    def test_indexed_and_unreviewed_implicit_memory_forms_fail_closed(self):
        for raw in ['8b042c','8b842480000000','f3a4','0f05','65488b042500010000']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))


if __name__=='__main__': unittest.main()
