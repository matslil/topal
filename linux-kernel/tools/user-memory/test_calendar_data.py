import struct
import unittest

from calendar_data import YEAR_BYTES, calendar_payload, graph_exclusion, guarded_entries, inspect, site


def fixture():
    base, table, lea, month = 0x1000, 128, 53, 90
    data = bytearray(b'\x90' * table + calendar_payload())
    begin = lea - 37
    guard = (bytes.fromhex('4881f8b20700000f82') + struct.pack('<i', -begin - 13)
             + bytes.fromhex('4881f8d50800000f87') + struct.pack('<i', -begin - 26)
             + bytes.fromhex('4881c04ef8ffff48c1e004'))
    data[begin:lea] = guard
    data[lea:lea + 7] = bytes.fromhex('4c8d2d') + struct.pack('<i', table - lea - 7)
    data[lea + 7:lea + 24] = bytes.fromhex('4901c54d8bbd000000004d8bb508000000')
    data[month:month + 7] = bytes.fromhex('4c8d2d') + struct.pack('<i', table + YEAR_BYTES - month - 7)
    return base, data


class CalendarDataTests(unittest.TestCase):
    def test_declared_guard_bypasses_cannot_hide_in_data_annotation(self):
        base, data = fixture()
        consumer = inspect(data, base)['year_consumer']
        begin = consumer['address']
        starts = [begin + i for i in [0, 7, 13, 20, 26, 33, 37, 44, 47, 54]]
        rows = [dict(address=a, size=n) for a, n in zip(starts, [7, 6, 7, 6, 7, 4, 7, 3, 7, 7], strict=True)]
        graph = dict(instructions=rows, roots=[dict(address=begin)], callbacks={},
                     edges=[dict(source=a, target=b, kind='fallthrough') for a, b in zip(starts, starts[1:])])
        self.assertEqual(guarded_entries(graph, consumer)['checked_inner_edges'], 9)
        from copy import deepcopy
        for label in ['interior-entry', 'interior-callback', 'direct-bypass', 'missing-instruction']:
            value = deepcopy(graph)
            if label == 'interior-entry':
                value['roots'].append(dict(address=consumer['lea']))
            elif label == 'interior-callback':
                value['callbacks']['bad'] = dict(target=consumer['lea'])
            elif label == 'direct-bypass':
                value['edges'].append(dict(source=base, target=consumer['lea'], kind='direct-transfer'))
            else:
                value['instructions'].pop()
            with self.subTest(label=label), self.assertRaises(ValueError):
                guarded_entries(value, consumer)

    def test_independent_calendar_payload_and_site(self):
        base, data = fixture()
        result = inspect(data, base)
        self.assertEqual((result['year_rows'], result['month_rows'], result['bytes']), (292, 12, 4864))
        chosen = site(data, base, base + 128 + 105 * 16, result)
        self.assertEqual((chosen['year'], chosen['days'], chosen['leap']), (2075, 38351, 0))
        payload = calendar_payload()
        self.assertEqual(struct.unpack_from('<QQ', payload, 30 * 16), (10957, 1))  # 2000
        self.assertEqual(struct.unpack_from('<Q', payload, 130 * 16 + 8)[0], 0)  # 2100

    def test_rejects_missing_corrupt_duplicate_tables(self):
        base, data = fixture()
        for label, value in [('missing', data[:128]), ('corrupt', bytearray(data)),
                             ('duplicate', data + calendar_payload())]:
            if label == 'corrupt':
                value[128 + 130 * 16 + 8] = 1
            with self.subTest(label=label), self.assertRaises(ValueError):
                inspect(value, base)

    def test_rejects_weak_bounds_bad_stride_and_wrong_load(self):
        base, original = fixture()
        for at in [16 + 3, 16 + 8, 16 + 16, 16 + 21, 16 + 29, 16 + 36, 53 + 10]:
            data = bytearray(original)
            data[at] ^= 1
            with self.subTest(at=at), self.assertRaises(ValueError):
                inspect(data, base)

    def test_rejects_reference_and_rejection_redirects(self):
        base, original = fixture()
        for label in ['year-reference', 'month-reference', 'reject-to-data']:
            data = bytearray(original)
            if label == 'reject-to-data':
                struct.pack_into('<i', data, 16 + 9, 128 - 16 - 13)
                struct.pack_into('<i', data, 16 + 22, 128 - 16 - 26)
            else:
                data[(53 if label == 'year-reference' else 90) + 3] ^= 1
            with self.subTest(label=label), self.assertRaises(ValueError):
                inspect(data, base)

    def test_site_must_be_actual_year_day_field(self):
        base, data = fixture()
        result = inspect(data, base)
        for address in [base, base + 128 + 105 * 16 + 1, result['month_start']]:
            with self.subTest(address=address), self.assertRaises(ValueError):
                site(data, base, address, result)

    def test_graph_bytes_and_overlap_remain_obligations(self):
        class Elf:
            sections = [(0, 0, 4, 0x1000, 0, 8)]
            def section_bytes(self, index):
                return bytes.fromhex('9090909090909090')
        rows = [dict(address=0x1000, size=2, bytes='9090')]
        self.assertFalse(graph_exclusion(rows, Elf(), 0x1004, 0x1008)['global_unreachability_proved'])
        for lo, hi, selected in [(0x1001, 0x1004, rows), (0x1004, 0x1008, []),
                                 (0x1004, 0x1008, [dict(address=0x1000, size=2, bytes='90c3')])]:
            with self.assertRaises(ValueError):
                graph_exclusion(selected, Elf(), lo, hi)
