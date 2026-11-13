"""Host-only counterexamples for TOPAL-SYSTEMS-USER-EXTENT-001 boot readback.

These byte arrays are never executed, loaded into a VM, or used as authority.
"""
import struct
import unittest

from boot_layout import BASE, GATES, IDT, IDTR, bind_loads, gdt, idt, load_segments, paging, take


def saved_idt():
    payload = bytearray(0x8000)
    struct.pack_into('<HQ', payload, IDTR - BASE, 4095, IDT)
    targets = {}
    for vector, (name, kind) in GATES.items():
        # Exercise all three target fields, including a nonzero high dword.
        target = 0xffff800012340000 + vector * 16
        targets[name] = target
        struct.pack_into('<HHBBHII', payload, IDT - BASE + vector * 16,
                         target & 0xffff, 16, 0, 0x80 | kind,
                         (target >> 16) & 0xffff, target >> 32, 0)
    return payload, targets


def saved_elf():
    raw = bytearray(0x3000)
    raw[:7] = b'\x7fELF\x02\x01\x01'
    struct.pack_into('<HH', raw, 16, 2, 62)
    struct.pack_into('<Q', raw, 32, 64)
    struct.pack_into('<HHH', raw, 52, 64, 56, 1)
    struct.pack_into('<II6Q', raw, 64, 1, 5, 0x1000, 0x200000, 0x200000, 16, 32, 4096)
    raw[0x1000:0x1010] = bytes(range(16))
    return raw


class BootLayoutTests(unittest.TestCase):
    def test_complete_64bit_idt_target_is_reconstructed(self):
        payload, targets = saved_idt()
        value = idt(payload, targets)
        self.assertEqual(len(value['empty_vectors']), 252)
        self.assertEqual([row['vector'] for row in value['gates']], [3, 14, 241, 242])
        for row in value['gates']:
            self.assertEqual(row['target'], targets[row['symbol']])
        self.assertEqual(value['gates'][0]['kind'], 'trap')
        self.assertEqual(value['gates'][1]['kind'], 'interrupt')

    def test_idt_cannot_substitute_target_or_truncate_high_word(self):
        payload, targets = saved_idt()
        for at in [0, 6, 8]:
            altered = bytearray(payload)
            altered[IDT - BASE + 3 * 16 + at] ^= 1
            with self.assertRaisesRegex(ValueError, 'IDT target'):
                idt(altered, targets)

    def test_idt_rejects_user_gate_wrong_selector_and_stack_fields(self):
        payload, targets = saved_idt()
        for offset, value in [(2, 24), (4, 1), (4, 8), (5, 0xef), (5, 0x0f), (5, 0x8e), (12, 1)]:
            altered = bytearray(payload)
            altered[IDT - BASE + 3 * 16 + offset] = value
            with self.assertRaisesRegex(ValueError, 'IDT selector'):
                idt(altered, targets)

    def test_idt_rejects_missing_gate_and_hidden_extra_entry(self):
        payload, targets = saved_idt()
        missing = bytearray(payload)
        missing[IDT - BASE + 14 * 16:IDT - BASE + 15 * 16] = bytes(16)
        with self.assertRaises(ValueError):
            idt(missing, targets)
        for offset in [IDT - BASE + 16, IDT - BASE + 255 * 16 + 15]:
            extra = bytearray(payload)
            extra[offset] = 1
            with self.assertRaisesRegex(ValueError, 'unexpected nonzero'):
                idt(extra, targets)

    def test_idtr_must_cover_the_bound_table(self):
        payload, targets = saved_idt()
        for offset in [0, 2, 9]:
            altered = bytearray(payload)
            altered[IDTR - BASE + offset] ^= 1
            with self.assertRaisesRegex(ValueError, 'IDTR'):
                idt(altered, targets)

    def test_gdt_transition_cannot_select_compatibility_code_or_user_data(self):
        payload = bytearray(0x8000)
        descriptors = [0, 0x00cf9a000000ffff, 0x00af9a000000ffff, 0x00cf92000000ffff,
                       0x00cff2000000ffff, 0x00affa000000ffff]
        struct.pack_into('<6Q', payload, 0x4000, *descriptors)
        self.assertTrue(gdt(payload)['descriptors'][2]['long'])
        for index, mask in [(0, 1), (1, 1 << 47), (2, 1 << 53), (2, 1 << 54),
                            (3, 1 << 45), (3, 1 << 16)]:
            altered = bytearray(payload)
            struct.pack_into('<Q', altered, 0x4000 + index * 8, descriptors[index] ^ mask)
            with self.assertRaisesRegex(ValueError, 'GDT'):
                gdt(altered)

    def test_initial_paging_refuses_hidden_entries_and_user_grants(self):
        payload = bytearray(0x8000)
        struct.pack_into('<Q', payload, 0x1000, 0x102003)
        struct.pack_into('<Q', payload, 0x2000, 0x103003)
        struct.pack_into('<Q', payload, 0x2018, 0x107003)
        for index in range(512):
            struct.pack_into('<Q', payload, 0x3000 + index * 8, index * 0x200000 | 0x83)
        struct.pack_into('<Q', payload, 0x7000 + 502 * 8, 0xfec0009b)
        struct.pack_into('<Q', payload, 0x7000 + 503 * 8, 0xfee0009b)
        self.assertFalse(paging(payload)['user_access'])
        for offset, mask in [(0x1000, 4), (0x1008, 1), (0x2018, 0x10),
                             (0x3000, 4), (0x3ff8, 0x80), (0x7000 + 502 * 8, 0x10)]:
            altered = bytearray(payload)
            altered[offset] ^= mask
            with self.assertRaisesRegex(ValueError, 'paging table'):
                paging(altered)

    def test_loads_bind_elf_file_bytes_and_zero_bss(self):
        kernel = saved_elf()
        segments = load_segments(kernel)
        payload = bytearray(0x100020)
        payload[0x100000:0x100010] = bytes(range(16))
        bind_loads(kernel, payload, segments)
        for offset in [0x100000, 0x10000f, 0x100010, 0x10001f]:
            altered = bytearray(payload)
            altered[offset] ^= 1
            with self.assertRaises(ValueError):
                bind_loads(kernel, altered, segments)
        with self.assertRaisesRegex(ValueError, 'payload size'):
            bind_loads(kernel, payload[:-1], segments)

    def test_load_headers_refuse_physical_aliases_overlap_and_file_escape(self):
        kernel = saved_elf()
        for offset, value in [(64 + 24, 0x201000), (64 + 32, 33), (64 + 8, 0x3000),
                              (64 + 16, (1 << 64) - 16), (64 + 48, 2048)]:
            altered = bytearray(kernel)
            struct.pack_into('<Q', altered, offset, value)
            with self.assertRaises(ValueError):
                load_segments(altered)
        overlap = bytearray(kernel)
        struct.pack_into('<H', overlap, 56, 2)
        overlap[120:176] = kernel[64:120]
        with self.assertRaisesRegex(ValueError, 'overlap'):
            load_segments(overlap)

    def test_load_table_and_structure_reads_are_bounded(self):
        kernel = saved_elf()
        struct.pack_into('<H', kernel, 56, 0xffff)
        with self.assertRaisesRegex(ValueError, 'outside file'):
            load_segments(kernel)
        for offset, size in [(-1, 1), (0, -1), (7, 2), (1 << 64, 4)]:
            with self.assertRaises(ValueError):
                take(bytes(8), offset, size)


if __name__ == '__main__':
    unittest.main()
