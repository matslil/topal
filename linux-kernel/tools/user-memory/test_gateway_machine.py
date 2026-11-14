"""Independent instruction counterexamples, never executed outside Python.

TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001. These models do not grant native state
or retirement authority and do not replace post-admission fatal coverage.
"""
from dataclasses import replace
from unittest import TestCase
from types import SimpleNamespace
import struct

from gateway_contract import machine_inputs, predicate, seed
from gateway_machine import ModelGap, Program, WORD, decode
from inspect_gateway_admission import caller, storage


LAYOUT = dict(context=0x600000, epoch=0x610000, recovery=0x620000,
              brk_return=0x700000, cleanup_return=0x700100, exec_return=0x700200)


def program(hexadecimal):
    raw = bytes.fromhex(hexadecimal)
    return Program(raw + b'\x55' + bytes.fromhex('faf4ebfd'), len(raw), len(raw) + 1)


def inputs():
    return dict(registers=[0] * 16, memory={}, cs=16, flags=2, cr3=0x12345000, msrs={})


class GatewayMachineTests(TestCase):
    def test_initial_condition_flags_and_zero_count_shift_are_preserved(self):
        for raw in ['0f8401000000', '48c1e000 0f8401000000']:
            value = program(raw)
            for flags, admitted in [(0, True), (0x40, False)]:
                state = inputs()
                state['flags'] = flags
                self.assertEqual(value.run(**state)['admitted'], admitted)

    def test_bound_storage_cannot_be_truncated_executable_or_unallocated(self):
        def elf(flags=3, size=48, address=0x1000, owner=1):
            return SimpleNamespace(symbols={'data': dict(info=1, section=owner, value=address, size=size)},
                sections=[None, (0, 8, flags, 0x1000, 0, 64)])
        self.assertEqual(storage(elf(), 'data', 48), 0x1000)
        for value in [elf(flags=2), elf(flags=7), elf(size=47), elf(size=65),
                      elf(address=0xfff), elf(owner=0), elf(owner=2)]:
            with self.assertRaises(ModelGap):
                storage(value, 'data', 48)

    def test_compiled_return_marker_must_follow_the_bound_relative_call(self):
        code = bytearray(0x100)
        code[0x1b] = 0xe8
        struct.pack_into('<i', code, 0x1c, 0x20)
        marker = dict(info=2, section=1, value=0x220, size=0)
        gateway = dict(section=1, value=0x240)
        elf = SimpleNamespace(symbols={'site': marker}, sections=[None, (0, 1, 6, 0x200, 0, 0x100)],
                              section_bytes=lambda _: code)
        self.assertEqual(caller(elf, 'site', gateway), 0x220)
        code[0x1c] ^= 1
        with self.assertRaisesRegex(ModelGap, 'actual gateway call'):
            caller(elf, 'site', gateway)
        code[0x1c] ^= 1
        code[0x1b] = 0xe9
        with self.assertRaisesRegex(ModelGap, 'actual gateway call'):
            caller(elf, 'site', gateway)

    def test_full_word_unsigned_above_and_below_are_distinct(self):
        # CMP RDI,1; JA selects the boundary; equality/below go to refusal.
        # The two branch displacements select different typed stop points.
        value = program('4881ff01000000 0f8705000000 e901000000')
        for candidate, admitted in [(0, False), (1, False), (2, True),
                                     (1 << 32, True), (1 << 63, True), (WORD, True)]:
            state = inputs()
            state['registers'][7] = candidate
            self.assertEqual(value.run(**state)['admitted'], admitted)

    def test_equal_low_dwords_cannot_hide_a_different_high_word(self):
        value = program('4839f7 0f8501000000')
        for left, right, admitted in [(1, 1, True), (1 << 32, 0, False),
                                      (WORD, 0xffffffff, False), (WORD, WORD, True)]:
            state = inputs()
            state['registers'][7], state['registers'][6] = left, right
            self.assertEqual(value.run(**state)['admitted'], admitted)

    def test_signed_immediates_are_extended_to_the_full_target_word(self):
        row = decode(bytes.fromhex('4981c2f0ffffff'), 0)
        self.assertEqual((row.operation, row.arguments), ('add-constant', (10, WORD - 15)))
        value = program('4981c2f0ffffff')
        state = inputs()
        state['registers'][10] = 8
        self.assertEqual(value.run(**state)['registers'][10], WORD - 7)

    def test_rex_loads_keep_extended_source_and_destination_registers(self):
        value = program('4d8b93f8ffffff')
        state = inputs()
        state['registers'][11] = 0x1008
        state['memory'][0x1000] = 1 << 57
        result = value.run(**state)
        self.assertEqual(result['registers'][10], 1 << 57)
        self.assertEqual(result['reads'], [0x1000])

    def test_unavailable_metadata_is_a_gap_and_cannot_become_a_refusal(self):
        value = program('498b8300000000')
        with self.assertRaisesRegex(ModelGap, 'unmodeled memory read'):
            value.run(**inputs())

    def test_stores_and_unknown_instruction_families_are_refused(self):
        for raw in ['49898300000000', '90', '0f05', '4883f800', '0f88ffffffff', '4a89c0']:
            with self.subTest(raw=raw), self.assertRaises(ModelGap):
                program(raw)

    def test_branches_cannot_enter_immediates_or_leave_the_selected_region(self):
        for raw in ['e9fdffffff', 'e980000000']:
            with self.assertRaisesRegex(ModelGap, 'branch enters'):
                program(raw)
        with self.assertRaisesRegex(ModelGap, 'cyclic'):
            program('e9fbffffff')

    def test_flag_projection_must_be_discarded_except_for_if_and_df(self):
        for raw in ['9c58', '9c584881e040000000', '9c4881e000060000']:
            with self.assertRaises(ModelGap):
                program(raw)
        value = program('9c58 4881e000060000 4881f800000000 0f8501000000')
        for flags, admitted in [(2, True), (1 << 9, False), (1 << 10, False),
                                ((1 << 9) | (1 << 10), False), (1 << 63, True)]:
            state = inputs()
            state['registers'][4] = 0x170008
            state['flags'] = flags
            result = value.run(**state)
            self.assertEqual(result['admitted'], admitted)
            self.assertEqual(result['registers'][4], 0x170008)
            self.assertEqual(set(result['scratch_writes']), {0x170000})

    def test_privileged_reads_after_wrong_cpl_are_coverage_gaps(self):
        for raw in ['0f20d8', '0f32']:
            state = inputs()
            state['cs'] = 19
            with self.assertRaisesRegex(ModelGap, 'privileged read'):
                program(raw).run(**state)

    def test_msr_halves_are_zero_extended_then_combined(self):
        value = program('0f32 48c1e220 4809d0')
        state = inputs()
        state['registers'][0] = state['registers'][2] = WORD
        state['registers'][1] = 0xabcd0000c0000101
        state['msrs'][0xc0000101] = 0x1234567890abcdef
        self.assertEqual(value.run(**state)['registers'][0], 0x1234567890abcdef)
        with self.assertRaisesRegex(ModelGap, 'MSR'):
            value.run(**inputs())

    def test_host_input_registers_and_metadata_remain_unchanged(self):
        state = inputs()
        state['registers'][11] = 0x1000
        state['memory'][0x1000] = WORD
        before = (list(state['registers']), dict(state['memory']))
        program('498b8300000000').run(**state)
        self.assertEqual((state['registers'], state['memory']), before)

    def test_prefix_requires_a_bound_refusal_marker(self):
        for body, boundary, refusal in [(bytes(8), 1, 4), (bytes.fromhex('faf4ebfd'), 1, 0)]:
            with self.assertRaisesRegex(ModelGap, 'refusal marker'):
                Program(body, boundary, refusal)

    def test_cleanup_uses_its_own_root_and_kernel_gs_binding(self):
        value = seed(LAYOUT, 6)
        self.assertTrue(predicate(replace(value, gs=WORD, self_pointer=WORD, root=WORD), LAYOUT))
        self.assertTrue(predicate(replace(value, number=231), LAYOUT))
        for change in [dict(cr3=value.root), dict(kernel_gs=0), dict(number=12), dict(state=1)]:
            self.assertFalse(predicate(replace(value, **change), LAYOUT))

    def test_abi_request_legality_is_separate_from_gateway_admission(self):
        value = seed(LAYOUT)
        for request in [0, 1 << 32, WORD]:
            self.assertTrue(predicate(replace(value, request=request, captured_request=request), LAYOUT))
        for change in [dict(entry=0), dict(number=59), dict(captured_request=1),
                       dict(recovery=(0, 0, 0, 1, 0, 0))]:
            self.assertFalse(predicate(replace(value, **change), LAYOUT))

    def test_malformed_model_words_are_never_kernel_outcomes(self):
        value = seed(LAYOUT)
        for change in [dict(entry=True), dict(sp=-1), dict(root=1 << 64), dict(recovery=(0,))]:
            with self.assertRaises(ValueError):
                machine_inputs(replace(value, **change), LAYOUT)
        with self.assertRaisesRegex(ValueError, 'outside selected'):
            predicate(replace(value, operation=8), LAYOUT)
