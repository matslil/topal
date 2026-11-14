"""Counterexamples preserving complete table checks around random output.

TOPAL-SYSTEMS-USER-EXTENT-001. Model evidence, not native mapping authority.
"""
from pathlib import Path
import runpy
import struct
import unittest


class RandomTableObserverControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_random_observer.py'
        ns=dict(Checked=type('Breakpoint',(),{}))
        exec(runpy.run_path(str(path))['PROGRAM'],ns)
        cls.check=staticmethod(ns['random_table_status_sets'])

    def fixture(self,entry=0x120007):
        table=bytearray(4096)
        struct.pack_into('<Q',table,3*8,entry)
        mappings=[dict(pointer=0x403000,leaf_shift=12,user=True,writable=True,
            path=[[0x1000,12,entry&~0x60]])]
        return {0x1000:bytes(table)},mappings

    def delta(self,before,after,index=3):
        return dict(table=0x1000,index=index,before=before,after=after,changed_bits=before^after)

    def test_equal_tables_and_monotonic_target_leaf_status_sets_are_accepted(self):
        tables,mappings=self.fixture()
        self.assertEqual(self.check(tables,[],mappings),[])
        for bits in [0x20,0x40,0x60]:
            changes=[self.delta(0x120007,0x120007|bits)]
            self.assertEqual(self.check(tables,changes,mappings),changes)

    def test_permissions_nx_addresses_and_page_size_bits_cannot_hide_behind_status(self):
        tables,mappings=self.fixture()
        for bit in [1,2,4,0x80,0x1000,1<<63]:
            with self.subTest(bit=bit),self.assertRaises(AssertionError):
                self.check(tables,[self.delta(0x120007,(0x120007^bit)|0x60)],mappings)

    def test_status_bit_clearing_is_rejected(self):
        tables,mappings=self.fixture(0x120067)
        for bit in [0x20,0x40,0x60]:
            with self.subTest(bit=bit),self.assertRaises(AssertionError):
                self.check(tables,[self.delta(0x120067,0x120067&~bit)],mappings)

    def test_unrelated_leaf_and_duplicate_transition_records_are_rejected(self):
        tables,mappings=self.fixture()
        with self.assertRaises(AssertionError):
            self.check(tables,[self.delta(0,0x60,index=4)],mappings)
        change=self.delta(0x120007,0x120067)
        with self.assertRaises(AssertionError): self.check(tables,[change,change],mappings)

    def test_cached_leaf_or_supplied_delta_cannot_disagree_with_actual_before_bytes(self):
        tables,mappings=self.fixture()
        mappings[0]['path'][-1][-1]^=0x1000
        with self.assertRaises(AssertionError): self.check(tables,[],mappings)
        tables,mappings=self.fixture()
        with self.assertRaises(AssertionError): self.check(tables,[self.delta(0x130007,0x130067)],mappings)

    def test_supervisor_readonly_and_huge_mappings_cannot_authorize_status_changes(self):
        for field,value in [('user',False),('writable',False),('leaf_shift',21)]:
            tables,mappings=self.fixture()
            mappings[0][field]=value
            with self.subTest(field=field),self.assertRaises(AssertionError): self.check(tables,[],mappings)

    def test_empty_or_incomplete_table_evidence_is_rejected(self):
        tables,mappings=self.fixture()
        with self.assertRaises(AssertionError): self.check(tables,[],[])
        tables[0x1000]=tables[0x1000][:-8]
        with self.assertRaises(AssertionError): self.check(tables,[],mappings)


if __name__=='__main__': unittest.main()
