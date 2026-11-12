"""Nonexecuted controls for protected ranges and legal hardware status sets."""
import copy
import unittest

from owner_interval import compare,regions
from test_owner_snapshot import fixture


class OwnerIntervalTests(unittest.TestCase):
    def test_ranges_cover_complete_selected_owners_and_translation_pages(self):
        snapshot,bindings,_=fixture()
        addresses=dict(root_tokens=0x4000000,mm_tokens=0x4001000,context=0x4002000,
            mm_state=0x4003000,mm_epoch=0x4004000,registry=0x5000000,table=0x6000000)
        rows=regions(snapshot,bindings,addresses)
        self.assertEqual(len(rows),26)
        self.assertEqual(sum(r['size'] for r in rows if r['name'].startswith('translation-')),6*4096)
        self.assertEqual(next(r['size'] for r in rows if r['name']=='cells-2'),503*24)
        self.assertEqual(next(r['size'] for r in rows if r['name']=='cells-4'),67*24)

    def test_wrapped_or_overlapping_watch_ranges_fail(self):
        snapshot,bindings,_=fixture()
        addresses={name:0x4000000 for name in ['root_tokens','mm_tokens','context','mm_state','mm_epoch','registry','table']}
        with self.assertRaises(AssertionError):regions(snapshot,bindings,addresses)
        addresses['root_tokens']=(1<<64)-8
        with self.assertRaises(AssertionError):regions(snapshot,bindings,addresses)

    def test_output_allows_only_monotonic_covered_accessed_dirty_sets(self):
        before,bindings,spans=fixture()
        after=copy.deepcopy(before)
        root=before['context']['root']
        after['tables'][str(root)][0]|=0x20
        for virtual in [0x638000,0x639000]:after['tables'][str(root+16384)][virtual//4096%512]|=0x60
        result=compare(before,after,bindings,spans,0x638fc0,144,1)
        self.assertEqual(len(result['table_status_sets']),3)

    def test_permission_address_status_clear_and_unrelated_changes_fail(self):
        original,bindings,spans=fixture()
        root=original['context']['root']
        for table,index,mask in [(root,0,2),(root+16384,0x638000//4096%512,4096),
                (root+16384,0,0x20)]:
            with self.subTest(table=table,index=index,mask=mask):
                after=copy.deepcopy(original)
                after['tables'][str(table)][index]^=mask
                with self.assertRaises(AssertionError):compare(original,after,bindings,spans,0x638fc0,144,1)
        before=copy.deepcopy(original)
        before['tables'][str(root)][0]|=0x20
        with self.assertRaises(AssertionError):compare(before,original,bindings,spans,0x638fc0,144,1)

    def test_input_cannot_gain_dirty_status(self):
        before,bindings,spans=fixture()
        before['epoch'][1]=18
        after=copy.deepcopy(before)
        after['tables'][str(before['context']['low'])][0x4e8000//4096%512]|=0x40
        with self.assertRaises(AssertionError):compare(before,after,bindings,spans,0x4e8000,4,0)

    def test_temporary_arithmetic_flags_do_not_hide_mode_or_identity_changes(self):
        before,bindings,spans=fixture()
        after=copy.deepcopy(before)
        after['flags']^=0x40
        self.assertTrue(compare(before,after,bindings,spans,0x638fc0,144,1)['passed'])
        after['epoch'][0]+=1
        with self.assertRaises(AssertionError):compare(before,after,bindings,spans,0x638fc0,144,1)


if __name__=='__main__':unittest.main()
