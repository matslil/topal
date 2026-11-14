"""Counterexamples for byte aliases, shifted pointers and preparation effects."""
import unittest

from audit_additional_scalars import register_effect, relative_offset, setup
from audit_fixed_transfers import scalar
from audit_staged_transfers import gs_effect


class AdditionalScalarControls(unittest.TestCase):
    def test_byte_load_reports_destruction_of_the_full_pointer_alias(self):
        self.assertEqual(scalar(bytes.fromhex('44 8a 06')),('read',0,1,6,8))
        self.assertEqual(scalar(bytes.fromhex('41 0f b6 80 00 00 00 00')),('read',0,1,8,0))
        self.assertEqual(scalar(bytes.fromhex('38 06')),('read',0,1,6,None))
        self.assertEqual(scalar(bytes.fromhex('c6 06 00')),('write',0,1,6,None))
        with self.assertRaises(AssertionError):
            scalar(bytes.fromhex('8a 26'))  # AH cannot be treated as an RSP write.

    def test_shifted_output_uses_the_admitted_candidate_not_the_original_base(self):
        self.assertEqual(relative_offset(-6,6,2,2),0)
        for bias,offset,width,length in [(-6,0,2,2),(-6,7,2,2),(0,0,8,2)]:
            with self.subTest(bias=bias,offset=offset), self.assertRaises(AssertionError):
                relative_offset(bias,offset,width,length)

    def test_setup_must_bind_the_exact_shift_and_register(self):
        raw = bytes.fromhex('48 89 f8 48 81 c0 06 00 00 00 49 89 c0 49 b9 02 00 00 00 00 00 00 00')
        self.assertEqual(setup(raw,len(raw),2,0,'poll-result'),(0,{8:0,7:-6}))
        for index in [2,6,12]:
            changed = bytearray(raw)
            changed[index] ^= 1
            with self.subTest(index=index), self.assertRaises(AssertionError):
                setup(bytes(changed),len(raw),2,0,'poll-result')

    def test_register_moves_preserve_aliases_but_partial_writes_do_not(self):
        self.assertEqual(register_effect(bytes.fromhex('49 89 f9')),({9},(9,7)))
        self.assertEqual(register_effect(bytes.fromhex('45 31 c0')),({8},None))
        for raw in ['49 8b 00','f3 a4','48 87 06','0f 01 f8']:
            with self.subTest(raw=raw), self.assertRaises((AssertionError,IndexError)):
                register_effect(bytes.fromhex(raw))

    def test_header_byte_effects_cannot_be_confused_with_wide_captured_reloads(self):
        self.assertEqual(gs_effect(bytes.fromhex('65 8a 04 25 e8 02 00 00'))['register_write'],0)
        store = gs_effect(bytes.fromhex('65 c6 04 25 e8 02 00 00 7f'))
        self.assertEqual((store['action'],store['width'],store['field']),('write',1,744))
        with self.assertRaises(AssertionError):
            gs_effect(bytes.fromhex('65 8a 34 25 40 01 00 00'))

    def test_operand_size_override_cannot_shorten_a_rex_w_immediate(self):
        for raw in ['66 48 c7 06 00 00','66 48 c7 06 00 00 00 00',
                    '66 48 81 3e 00 00','66 48 81 3e 00 00 00 00']:
            with self.subTest(raw=raw), self.assertRaises(AssertionError):
                scalar(bytes.fromhex(raw))


if __name__ == '__main__':
    unittest.main()
