"""Counterexamples for REP count, aliases, DF effects and sampler preservation."""
import unittest
from pathlib import Path
import runpy

from audit_copy_transfers import byte_count, constant, sampler_writes, step


class CopyTransferControls(unittest.TestCase):
    def test_observer_pairs_start_and_completion_around_rep_iterations(self):
        # Exercise the actual observer program with a debugger scheduling model.
        # Translation/permission checks are deliberately stubbed: this test
        # proves breakpoint lifecycle only, never native memory safety.
        program = runpy.run_path(str(Path(__file__).resolve().parents[2] /
            'labs/qemu/x86_64/user_extent_copy_observer.py'))['PROGRAM']
        class Breakpoint:
            def __init__(self, *args, **kwargs):
                self.enabled = True
        namespace = {'Checked': Breakpoint}
        exec(program, namespace)
        row = dict(name='sample', copy_offset=100, call_offset=50,
                   admitted_direction='write', copy_width=1)
        group = dict(copy_offset=100, cases=['sample'])
        registers = dict(rsi=0x2000, rdi=0x3000, rcx=3, rsp=0x4000, eflags=0)
        namespace.update(extent_inventory={'address': 0}, copy_rows={'sample': row},
            context=0x1000, reg=lambda r: registers[r],
            memory=lambda p, n: b'abc' if n == 3 else bytes(n),
            copy_mode=lambda previous=None: None, copy_range=lambda *args: [],
            copy_counts={'sample': {'completed': 0, 'bytes': 0}})
        import hashlib
        namespace['hashlib'] = hashlib
        start = namespace['CopyStarted'](group)
        finish = namespace['CopyFinished'](group, 2)
        namespace['copy_start_breakpoints'][100] = start
        namespace['copy_finish_breakpoints'][100] = finish
        self.assertFalse(finish.enabled)  # A null-input bypass is not a copy.
        namespace['copy_pending'] = dict(name='sample', accepted=True, size=3,
            candidate=0x3000, mode=None, rsp=0x4000, captured=bytes(64))
        start.observe()
        self.assertFalse(start.enabled)  # Revisited REP PC cannot restart it.
        self.assertTrue(finish.enabled)
        registers.update(rsi=0x2003, rdi=0x3003, rcx=0)
        finish.observe()
        self.assertTrue(start.enabled)
        self.assertFalse(finish.enabled)
        self.assertIsNone(namespace['copy_active'])
        self.assertIsNone(namespace['copy_pending'])
        self.assertEqual(namespace['copy_counts']['sample'], {'completed': 1, 'bytes': 3})

    def test_scaled_count_must_not_wrap(self):
        self.assertEqual(byte_count(constant(2), 8), constant(16))
        with self.assertRaises(AssertionError):
            byte_count(constant(1 << 61), 8)
        with self.assertRaises(AssertionError):
            byte_count(('entry-register', 15), 8)
        self.assertEqual(byte_count(('entry-register', 15), 1), ('entry-register', 15))

    def test_overwriting_an_alias_changes_its_value(self):
        registers = {i: ('entry-register', i) for i in range(16)}
        step(bytes.fromhex('4c 89 c7'), registers, 0, 0)
        self.assertEqual(registers[7], registers[8])
        step(bytes.fromhex('48 bf 01 00 00 00 00 00 00 00'), registers, 3, 0)
        self.assertNotEqual(registers[7], registers[8])
        step(bytes.fromhex('48 81 c7 01 00 00 00'), registers, 13, 0)
        self.assertEqual(registers[7], constant(2))

    def test_unsupported_direction_and_addressing_are_refused(self):
        for raw in ['fd', '67 f3 a4', 'f2 a4', 'f3 a6', '48 8d 34 16', '48 ff c1']:
            with self.subTest(raw=raw), self.assertRaises(AssertionError):
                step(bytes.fromhex(raw), {i: constant(0) for i in range(16)}, 0, 0)

    def test_memory_preparation_is_an_effect_not_an_owner(self):
        registers = {i: ('entry-register', i) for i in range(16)}
        effect = step(bytes.fromhex('49 89 84 24 80 05 00 00'), registers, 0, 0)
        self.assertEqual(effect['kind'], 'other-memory-write')
        self.assertEqual(effect['address'], ('add-word', ('entry-register', 12), 1408))
        self.assertNotIn('owned', effect)

    def test_sampler_effect_decoder_catches_protected_register_writes(self):
        self.assertEqual(sampler_writes(bytes.fromhex('48 89 c3')), {3})
        self.assertEqual(sampler_writes(bytes.fromhex('4d 8b 20')), {12})
        self.assertEqual(sampler_writes(bytes.fromhex('49 89 40 10')), set())
        self.assertEqual(sampler_writes(bytes.fromhex('49 b8 00 00 d0 fe 00 00 00 00')), {8})
        with self.assertRaises(AssertionError):
            sampler_writes(bytes.fromhex('0f 22 d8'))  # MOV CR3 has no admitted effect.


if __name__ == '__main__':
    unittest.main()
