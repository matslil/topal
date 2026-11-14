"""TK-DEC-285 portable model checks; no native/root authority is constructed."""
from dataclasses import replace
from itertools import product
import unittest

from kernel_image_plan import CODE, IMMUTABLE, MUTABLE, Geometry, Part, Plan, Region, Rights, View, check_views, partition


class ImagePlans(unittest.TestCase):
    def test_exhaustive_small_geometry_against_page_oracle(self):
        # An independent per-cell oracle for all 256 role/hole patterns.
        for roles in product([None, 'code', 'immutable', 'mutable'], repeat=4):
            if all(role is None for role in roles):
                with self.assertRaises(ValueError):
                    partition([], Geometry(4, 16, 64))
                continue
            regions = [Region(i * 4, (i + 1) * 4, role) for i, role in enumerate(roles) if role]
            plan = partition(regions, Geometry(4, 16, 64))
            expected_split = any(role is None for role in roles) or len(set(roles)) != 1
            self.assertEqual(plan.split_blocks, (0,) if expected_split else (), roles)

    def test_current_x86_layout_needs_three_split_regions(self):
        regions = [Region(0x200000, 0x200238, 'immutable'),
                   Region(0x800000, 0x2ae4000, 'mutable'),
                   Region(0x2ae4000, 0x2aeb5c8, 'immutable'),
                   Region(0x2aec5d0, 0x2c231cd, 'code'),
                   Region(0x2c241d0, 0x2c25000, 'relro', True),
                   Region(0x2c251e0, 0x2c251e8, 'mutable')]
        self.assertEqual(partition(regions, Geometry(4096, 2 << 20, 1 << 30)).split_blocks,
                         (0x200000, 0x2a00000, 0x2c00000))
        # This x86 layout cannot silently become a larger-granule layout.
        for minimum, large in [(16384, 32 << 20), (65536, 512 << 20)]:
            with self.subTest(minimum=minimum), self.assertRaises(ValueError):
                partition(regions, Geometry(minimum, large, 1 << 30))

    def test_provider_geometries_do_not_define_a_portable_table_count(self):
        for minimum, large in [(4096, 2 << 20), (16384, 32 << 20), (65536, 512 << 20)]:
            regions = [Region(0, large, 'code'), Region(large, large + minimum, 'immutable')]
            plan = partition(regions, Geometry(minimum, large, 1 << 40))
            self.assertEqual(plan.split_blocks, (large,))
            self.assertEqual(plan.parts[0].rights, CODE)

    def test_huge_uniform_extents_stay_symbolic(self):
        plan = partition([Region(0, 1 << 63, 'mutable')], Geometry(4096, 2 << 20, 1 << 64))
        self.assertEqual(len(plan.parts), 1)
        self.assertEqual(plan.split_blocks, ())

    def test_page_rounding_conflicts_and_uninitialized_relro_refused(self):
        for regions in [[Region(0, 1, 'code'), Region(1, 2, 'immutable')],
                        [Region(0, 4, 'relro')], [Region(0, 4, 'unknown')]]:
            with self.assertRaises(ValueError):
                partition(regions, Geometry(4, 16, 64))
        plan = partition([Region(0, 2, 'immutable'), Region(2, 4, 'relro', True)], Geometry(4, 16, 64))
        self.assertEqual(plan.parts[0].rights, IMMUTABLE)

    def test_bounds_and_geometry_are_checked_before_planning(self):
        for values in [(0, 16, 64), (3, 16, 64), (16, 4, 64), (4, 12, 64), (True, 16, 64)]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                Geometry(*values)
        for start, end, limit in [(-1, 4, 64), (4, 4, 64), (0, 65, 64), (0, 5, 5), (False, 4, 64)]:
            with self.subTest(start=start, end=end), self.assertRaises(ValueError):
                partition([Region(start, end, 'immutable')], Geometry(4, 16, limit))

    def setup_views(self):
        plan = partition([Region(0, 4, 'code'), Region(4, 8, 'immutable'), Region(8, 12, 'mutable')], Geometry(4, 16, 256))
        views = [View(root, 64 + part.start, part.start, part.end - part.start, part.rights, True)
                 for root in ['process', 'bootstrap'] for part in plan.parts]
        return plan, views

    def test_primary_views_cover_both_roots_without_identity_assumption(self):
        plan, views = self.setup_views()
        self.assertIsNone(check_views(plan, ['process', 'bootstrap'], views))

    def test_writable_executable_user_and_memory_kind_aliases_refused(self):
        plan, views = self.setup_views()
        for extra in [View('process', 128, 0, 4, MUTABLE, False),
                      View('process', 128, 4, 4, CODE, False),
                      View('process', 128, 8, 4, CODE, False),
                      View('process', 128, 4, 4, Rights(True, False, False, True), False),
                      View('process', 128, 4, 4, IMMUTABLE, False, 'device')]:
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                check_views(plan, ['process', 'bootstrap'], views + [extra])
        check_views(plan, ['process', 'bootstrap'], views + [View('process', 128, 0, 4, IMMUTABLE, False)])

    def test_missing_bootstrap_holes_unknown_roots_and_primary_execute_refused(self):
        plan, views = self.setup_views()
        changes = [views[:3], views[1:], [replace(views[0], rights=IMMUTABLE)] + views[1:],
                   [replace(views[0], root='unregistered')] + views[1:]]
        for changed in changes:
            with self.assertRaises(ValueError):
                check_views(plan, ['process', 'bootstrap'], changed)

    def test_alias_alignment_overflow_and_duplicate_root_refused(self):
        plan, views = self.setup_views()
        for changed in [replace(views[0], virtual=65), replace(views[0], physical=1),
                        replace(views[0], size=3), replace(views[0], virtual=256),
                        replace(views[0], size=True)]:
            with self.assertRaises(ValueError):
                check_views(plan, ['process', 'bootstrap'], [changed] + views[1:])
        with self.assertRaises(ValueError):
            check_views(plan, ['process', 'process'], views)

    def test_conflicting_virtual_alias_cannot_supply_physical_coverage(self):
        plan, views = self.setup_views()
        # All physical bytes still have the stated rights, but one virtual page
        # cannot simultaneously name code and mutable backing in the same root.
        changed = views[:2] + [replace(views[2], virtual=views[0].virtual)] + views[3:]
        with self.assertRaises(ValueError):
            check_views(plan, ['process', 'bootstrap'], changed)

    def test_descriptive_plan_cannot_relabel_user_or_writable_code_rights(self):
        plan, views = self.setup_views()
        for parts in [(Part(0, 4, Rights(True, True, True)),),
                      (Part(0, 4, Rights(True, False, True, True)),),
                      (Part(1, 4, CODE),), (Part(4, 8, CODE), Part(0, 4, CODE)), ()]:
            with self.subTest(parts=parts), self.assertRaises(ValueError):
                check_views(Plan(plan.geometry, parts, ()), ['process', 'bootstrap'], views)
