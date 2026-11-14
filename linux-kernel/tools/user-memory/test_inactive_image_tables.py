"""Independent word/geometry checks for TK-DEC-288; no guest or live owners."""
import unittest
from inactive_image_tables import PAGE, NX, encode, geometry, page_count, verify


def layout():
    return dict(minimum_granule=4096, large_granule=2 << 20, address_limit=1 << 30,
                spans=[dict(start=0x200000, end=0x201000, role='immutable'),
                       dict(start=0x800000, end=0x801000, role='code')])


class InactiveTables(unittest.TestCase):
    def test_exact_words_preserve_roles_and_separate_process_low_leaf(self):
        selected = layout()
        self.assertEqual(geometry(selected).split_blocks, (0x200000, 0x800000))
        self.assertEqual(page_count(selected), 7)
        data = encode(selected, 0x3000000, 0x3100000)
        word = lambda i: int.from_bytes(data[i*8:i*8+8], 'little')
        self.assertEqual(word(0), 0x3001007)
        self.assertEqual(word(1025), 0x3005003)
        self.assertEqual(word(5*512), NX | 0x200061)
        self.assertEqual(word(6*512), 0x800061)
        self.assertEqual(word(1026), 0x3100007)
        self.assertEqual(word(1027), 0x3004007)
        self.assertEqual(data[4*PAGE:5*PAGE], bytes(PAGE))
        verify(selected, 0x3000000, data, 0x3100000)

    def test_changed_permissions_pointers_and_unused_words_are_refused(self):
        data = encode(layout(), 0x3000000)
        for index, bit in [(0, 4), (1025, 12), (5*512, 1), (5*512, 63),
                           (6*512, 2), (4*512, 0), (512+2, 0)]:
            changed = bytearray(data)
            changed[index*8+bit//8] ^= 1 << (bit%8)
            with self.subTest(index=index, bit=bit), self.assertRaises(ValueError):
                verify(layout(), 0x3000000, bytes(changed))

    def test_size_alone_cannot_substitute_a_different_image_partition(self):
        selected = layout()
        data = encode(selected, 0x3000000)
        selected['spans'][0]['role'] = 'code'
        self.assertEqual(len(data), page_count(selected)*PAGE)
        with self.assertRaises(ValueError):
            verify(selected, 0x3000000, data)

    def test_invalid_geometry_aliasing_and_missing_image_are_refused(self):
        for base, low in [(0x3000001, None), (0x200000, None), (0x3000000, 0x3001000),
                          (0x3000000, 0x800000), (0x40000000, None)]:
            with self.subTest(base=base, low=low), self.assertRaises(ValueError):
                encode(layout(), base, low)
        selected = layout(); selected['spans'] = []
        with self.assertRaises(ValueError): geometry(selected)


if __name__ == '__main__':
    unittest.main()
