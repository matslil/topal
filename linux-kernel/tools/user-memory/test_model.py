"""Range, ownership and permission oracles for the first owned-extent model."""
from dataclasses import replace
from itertools import product
import unittest

from model import (ADDRESS_BITS, Backing, InvariantViolation, NX, Observation,
                   PAGE, PHYSICAL_END, Status, Table, USER_END, WORD_MAX, admit)


class Fixture:
    """Build sparse table bytes independently of the admission implementation."""
    def __init__(self):
        self.root = 0x100000
        self.tables = {self.root: [0] * 512}
        self.levels = {self.root: 4}
        self.backing = {}

    def map(self, virtual, physical, *, writable=True):
        assert virtual % PAGE == physical % PAGE == 0
        table = self.root
        for level, divisor in [(4, 512**3), (3, 512**2), (2, 512)]:
            index = (virtual // PAGE // divisor) % 512
            if self.tables[table][index] == 0:
                child = self.root + len(self.tables) * PAGE
                self.tables[child] = [0] * 512
                self.levels[child] = level - 1
                self.tables[table][index] = child | 7
            table = self.tables[table][index] & ADDRESS_BITS
        self.tables[table][virtual // PAGE % 512] = physical | NX | (7 if writable else 5)
        self.backing[virtual] = Backing(physical, writable=writable)

    def path(self, virtual):
        table = self.root
        path = []
        for divisor in [512**3, 512**2, 512, 1]:
            index = (virtual // PAGE // divisor) % 512
            path.append((table, index))
            table = self.tables[table][index] & ADDRESS_BITS
        return path

    def observe(self, **changes):
        observed = Observation(self.root, self.root, 1, 1,
                               {p: Table(self.levels[p], tuple(t)) for p, t in self.tables.items()},
                               self.backing, frozenset(self.tables))
        return replace(observed, **changes)


class OwnedExtentTests(unittest.TestCase):
    def single(self, *, writable=True):
        fixture = Fixture()
        fixture.map(0x700000, 0x20000000, writable=writable)
        return fixture

    def test_beyond_predecessor_and_unrelated_virtual_placements(self):
        for virtual in [0x619000, 0x700000, 0x180000000, 0x11000000000]:
            with self.subTest(virtual=hex(virtual)):
                fixture = Fixture()
                fixture.map(virtual, 0x20000000)
                for write in [False, True]:
                    result = admit(fixture.observe(), virtual + 11, PAGE - 11, write=write)
                    self.assertEqual((result.status, result.checked_pages), (Status.ADMITTED, 1))

    def test_cross_page_and_every_upper_table_boundary(self):
        for boundary in [1 << 21, 1 << 30, 1 << 39]:
            fixture = Fixture()
            fixture.map(boundary - PAGE, 0x20000000)
            fixture.map(boundary, 0x30000000)
            reads = []
            result = admit(fixture.observe(), boundary - 17, 34, write=True, reads=reads)
            self.assertEqual((result.status, result.checked_pages), (Status.ADMITTED, 2))
            self.assertEqual(len(reads), 8)

    def test_large_extent_has_no_34_page_quota(self):
        fixture = Fixture()
        for index in range(400):
            fixture.map(0x5f7000 + index * PAGE, 0x20000000 + index * PAGE)
        result = admit(fixture.observe(), 0x5f7000, 400 * PAGE, write=True)
        self.assertEqual((result.status, result.checked_pages), (Status.ADMITTED, 400))

    def test_all_8192_direction_and_path_permission_combinations(self):
        fixture = self.single()
        path = fixture.path(0x700000)
        original = [fixture.tables[table][index] for table, index in path]
        for flags in product(range(8), repeat=4):
            for (table, index), word, flag in zip(path, original, flags):
                fixture.tables[table][index] = (word & ~7) | flag
            observed = fixture.observe()
            for write in [False, True]:
                # Independent permission lattice: all entries present/user,
                # with all four write grants required only for output.
                expected = all(f & 1 and f & 4 and (not write or f & 2) for f in flags)
                actual = admit(observed, 0x700000, PAGE, write=write)
                self.assertEqual(actual.status is Status.ADMITTED, expected, (flags, write))

    def test_holes_at_every_level_stop_before_dereference(self):
        for stop in range(4):
            fixture = self.single()
            table, index = fixture.path(0x700000)[stop]
            fixture.tables[table][index] = 0
            reads = []
            result = admit(fixture.observe(), 0x700000, PAGE, reads=reads)
            self.assertEqual(result.status, Status.ABSENT)
            self.assertEqual(len(reads), stop + 1)

    def test_range_hole_cannot_be_covered_by_valid_endpoints(self):
        fixture = Fixture()
        fixture.map(0x700000, 0x20000000)
        fixture.map(0x702000, 0x20002000)
        result = admit(fixture.observe(), 0x700000, 3 * PAGE)
        self.assertEqual((result.status, result.checked_pages), (Status.ABSENT, 1))

    def test_readonly_mapping_and_ancestor_distinguish_direction(self):
        for level in range(4):
            fixture = self.single()
            table, index = fixture.path(0x700000)[level]
            fixture.tables[table][index] &= ~2
            observed = fixture.observe()
            self.assertEqual(admit(observed, 0x700000, PAGE).status, Status.ADMITTED)
            self.assertEqual(admit(observed, 0x700000, PAGE, write=True).status, Status.PERMISSION)
        fixture = self.single(writable=False)
        self.assertEqual(admit(fixture.observe(), 0x700000, PAGE).status, Status.ADMITTED)

    def test_mapping_owner_can_withhold_write_independently_of_pte(self):
        fixture = self.single()
        fixture.backing[0x700000] = replace(fixture.backing[0x700000], writable=False)
        self.assertEqual(admit(fixture.observe(), 0x700000, 1, write=True).status, Status.PERMISSION)

    def test_empty_has_no_table_or_context_or_buffer_access(self):
        for candidate in [0, 0x619000, USER_END, 0xffff800000000000, WORD_MAX]:
            reads = []
            self.assertEqual(admit(None, candidate, 0, reads=reads).status, Status.EMPTY)
            self.assertEqual(reads, [])

    def test_complete_word_range_overflow_and_canonical_boundaries(self):
        observed = self.single().observe()
        for start, length in [(-1, 1), (0, -1), (True, 1), (0, WORD_MAX + 1),
                              (WORD_MAX, 1), (WORD_MAX - 2, 4), (USER_END, 1),
                              (USER_END - 1, 2), (0xffff800000000000, PAGE)]:
            reads = []
            self.assertEqual(admit(observed, start, length, reads=reads).status, Status.RANGE)
            self.assertEqual(reads, [])
        fixture = Fixture()
        fixture.map(USER_END - PAGE, 0x20000000)
        self.assertEqual(admit(fixture.observe(), USER_END - 1, 1).status, Status.ADMITTED)

    def test_foreign_next_table_is_not_read_even_if_numerically_accessible(self):
        fixture = self.single()
        child = fixture.tables[fixture.root][0] & ADDRESS_BITS
        observed = fixture.observe(pinned_tables=frozenset(set(fixture.tables) - {child}))
        reads = []
        with self.assertRaises(InvariantViolation):
            admit(observed, 0x700000, PAGE, reads=reads)
        self.assertEqual(reads, [(fixture.root, 0)])

    def test_missing_wrong_role_or_out_of_window_table_is_not_read(self):
        for change in ['missing', 'role', 'window', 'unaligned-root']:
            fixture = self.single()
            child = fixture.tables[fixture.root][0] & ADDRESS_BITS
            observed = fixture.observe()
            if change == 'missing':
                observed = replace(observed, tables={p: t for p, t in observed.tables.items() if p != child})
            elif change == 'role':
                tables = dict(observed.tables)
                tables[child] = replace(tables[child], level=1)
                observed = replace(observed, tables=tables)
            elif change == 'window':
                fixture.tables[fixture.root][0] = PHYSICAL_END | 7
                observed = fixture.observe(pinned_tables=frozenset(set(fixture.tables) | {PHYSICAL_END}))
            else:
                observed = replace(observed, root=observed.root + 1, current_root=observed.root + 1)
            reads = []
            with self.assertRaises(InvariantViolation):
                admit(observed, 0x700000, PAGE, reads=reads)
            self.assertLessEqual(len(reads), 1)

    def test_stale_foreign_unpinned_or_table_alias_backing_is_not_authority(self):
        for change in ['missing', 'stale', 'unpinned', 'different', 'table-alias', 'window']:
            fixture = self.single()
            owner = fixture.backing[0x700000]
            if change == 'missing':
                fixture.backing.clear()
            elif change == 'stale':
                fixture.backing[0x700000] = replace(owner, live=False)
            elif change == 'unpinned':
                fixture.backing[0x700000] = replace(owner, pins=0)
            elif change == 'different':
                fixture.backing[0x700000] = replace(owner, physical_page=owner.physical_page + PAGE)
            else:
                physical = fixture.root if change == 'table-alias' else PHYSICAL_END
                table, index = fixture.path(0x700000)[-1]
                fixture.tables[table][index] = physical | NX | 7
                fixture.backing[0x700000] = replace(owner, physical_page=physical)
            with self.assertRaises(InvariantViolation):
                admit(fixture.observe(), 0x700000, PAGE)

    def test_stale_or_unstable_context_is_not_an_ordinary_pointer_miss(self):
        original = self.single().observe()
        for changes in [dict(current_root=original.root + PAGE), dict(entry=0),
                        dict(current_entry=2), dict(privileged=False),
                        dict(critical=False), dict(stable=False),
                        dict(entry=WORD_MAX + 1, current_entry=WORD_MAX + 1),
                        dict(entry=True, current_entry=True),
                        dict(root=-PAGE, current_root=-PAGE),
                        dict(root=WORD_MAX + 1, current_root=WORD_MAX + 1)]:
            reads = []
            with self.assertRaises(InvariantViolation):
                admit(replace(original, **changes), 0x700000, 1, reads=reads)
            self.assertEqual(reads, [])

    def test_huge_reserved_and_additional_feature_formats_are_not_small_tables(self):
        for level in range(4):
            for bit in [7, 9, 52, 59, 62]:
                fixture = self.single()
                table, index = fixture.path(0x700000)[level]
                fixture.tables[table][index] |= 1 << bit
                reads = []
                self.assertEqual(admit(fixture.observe(), 0x700000, PAGE, reads=reads).status,
                                 Status.UNSUPPORTED)
                self.assertEqual(len(reads), level + 1)
        for mode in ['la57', 'lam', 'pku', 'unknown']:
            reads = []
            self.assertEqual(admit(self.single().observe(mode=mode), 0x700000, PAGE, reads=reads).status,
                             Status.UNSUPPORTED)
            self.assertEqual(reads, [])

    def test_nx_and_hardware_accessed_dirty_do_not_prevent_data_access(self):
        fixture = self.single()
        for table, index in fixture.path(0x700000):
            fixture.tables[table][index] |= NX | 0x60
        self.assertEqual(admit(fixture.observe(), 0x700000, PAGE, write=True).status, Status.ADMITTED)

    def test_snapshot_and_admission_preserve_all_input_storage(self):
        fixture = self.single()
        observed = fixture.observe()
        before = (dict(observed.tables), dict(observed.backing), observed.pinned_tables)
        fixture.tables[fixture.root][0] = 0
        fixture.backing.clear()
        for write in [False, True]:
            self.assertEqual(admit(observed, 0x700000, PAGE, write=write).status, Status.ADMITTED)
        self.assertEqual((dict(observed.tables), dict(observed.backing), observed.pinned_tables), before)
        with self.assertRaises(TypeError):
            observed.backing[0x700000] = Backing(0)


if __name__ == '__main__':
    unittest.main()
