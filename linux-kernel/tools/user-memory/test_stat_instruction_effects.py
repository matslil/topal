"""Counterexamples for stat register effects and implicit/stack operations."""
import unittest

from stat_instruction_effects import effects


class StatInstructionEffectControls(unittest.TestCase):
    def test_memory_load_destroys_its_destination_not_its_address_base(self):
        row = effects(bytes.fromhex('4d8b8000000000'))
        self.assertEqual(row['register_writes'],{8})
        self.assertEqual(row['memory_reads'],[(8,0,8)])
        self.assertEqual(effects(bytes.fromhex('4d899800000000'))['memory_writes'],[(8,0,8)])

    def test_compare_reads_without_modifying_the_candidate(self):
        for raw in ['4981f890000000','49398000000000','493b8000000000']:
            with self.subTest(raw=raw): self.assertEqual(effects(bytes.fromhex(raw))['register_writes'],set())
        self.assertEqual(effects(bytes.fromhex('4d01c0'))['register_writes'],{8})
        self.assertEqual(effects(bytes.fromhex('4c29e0'))['register_writes'],{0})

    def test_partial_register_writes_and_unknown_implicit_effects_are_rejected(self):
        for raw in ['4531c0','4189c0','664189c0','4188c0','0fa2','0f05','49f7e0','48f7e0','48ffc0','fd','9d','5c']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))

    def test_saved_stack_accesses_retain_exact_signed_offsets(self):
        self.assertEqual(effects(bytes.fromhex('48c78424d8ffffff00000000'))['memory_writes'],[(4,-40,8)])
        self.assertEqual(effects(bytes.fromhex('4881bc24d0ffffff00000000'))['memory_reads'],[(4,-48,8)])
        self.assertEqual(effects(bytes.fromhex('4150'))['push'],8)
        self.assertEqual(effects(bytes.fromhex('4159'))['pop'],9)

    def test_unreviewed_address_forms_and_memory_arithmetic_fail_closed(self):
        for raw in ['498b0440','488b0500000000','4b8b8000000000','4981800000000001000000']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError): effects(bytes.fromhex(raw))

    def test_string_operations_expose_all_implicit_register_and_memory_effects(self):
        compare = effects(bytes.fromhex('f348a7'))
        self.assertEqual(compare['register_writes'],{1,6,7})
        self.assertEqual(compare['memory_reads'],[(6,0,8),(7,0,8)])
        self.assertEqual(compare['memory_writes'],[])
        self.assertEqual(effects(bytes.fromhex('f348ab'))['memory_writes'],[(7,0,8)])


if __name__ == '__main__':
    unittest.main()
