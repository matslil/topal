from copy import deepcopy
import struct
import unittest

from calendar_data import YEAR_BYTES, calendar_payload
from inspect_calendar_placement import MONTH, YEAR, layout


def fixture():
    payload = calendar_payload()
    begin, end = 64, 64 + len(payload)
    old = bytearray(b'\x90' * begin + payload + b'\xc3' * 8)
    old[0:5] = b'\xe9' + struct.pack('<i', end - 5)
    for at, target in [(8, begin), (16, begin + YEAR_BYTES)]:
        old[at:at + 7] = bytes.fromhex('4c8d2d') + struct.pack('<i', target - at - 7)
    old[29:34] = b'\xe8' + bytes(4)
    new = bytearray(old[:begin] + old[end:])
    struct.pack_into('<i', new, 1, begin - 5)
    new[11:15] = new[19:23] = bytes(4)
    before = dict(labels={'entry': 0, 'after': end, 'realtime-year-table': begin,
                          'realtime-month-table': begin + YEAR_BYTES})
    after = dict(labels={'entry': 0, 'after': begin}, fixups=[[1, 'after']])
    old_relocations = {30: ('external', 4, -4)}
    new_relocations = {30: ('external', 4, -4), 11: (YEAR, 2, -4), 19: (MONTH, 2, -4)}
    return [old, new, before, after, old_relocations, new_relocations]


class CalendarPlacementTests(unittest.TestCase):
    def test_exact_layout_transformation(self):
        result = layout(*fixture())
        self.assertEqual((result['removed_bytes'], result['local_fixups'], result['retained_external_relocations']), (4864, 1, 1))

    def test_unplanned_instruction_changes_cannot_hide_in_moved_data(self):
        for at in [1, 25, 40, 65]:
            args = fixture()
            args[1][at] ^= 1
            with self.subTest(at=at), self.assertRaises(ValueError):
                layout(*args)

    def test_missing_fixup_wrong_label_and_changed_data_refused(self):
        for case in ['fixup', 'label', 'data']:
            args = fixture()
            if case == 'fixup':
                args[3]['fixups'] = []
            elif case == 'label':
                args[3]['labels']['after'] += 1
            else:
                args[0][64 + 105 * 16] ^= 1
            with self.subTest(case=case), self.assertRaises(ValueError):
                layout(*args)

    def test_external_bindings_are_complete_and_typed(self):
        for case in ['missing', 'duplicate', 'addend', 'kind', 'retained']:
            args = deepcopy(fixture())
            if case == 'missing':
                del args[5][19]
            elif case == 'duplicate':
                args[5][19] = args[5][11]
            elif case == 'addend':
                args[5][11] = (YEAR, 2, 0)
            elif case == 'kind':
                args[5][11] = (YEAR, 4, -4)
            else:
                args[5][30] = ('wrong', 4, -4)
            with self.subTest(case=case), self.assertRaises(ValueError):
                layout(*args)
