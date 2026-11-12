"""Small independent catalog cases, TOPAL-SYSTEMS-PROCESS-MM-001.

These validate saved-record comparison only, not native allocator authority.
"""
import unittest

from process_native import PAGE, firmware_frames, physical_catalog


def catalog(owned=False):
    allocator=[1,1,2,8]+[0]*12
    cells=[word for i in range(8) for word in [0,1,0x100+i]]
    value=dict(allocator=allocator,catalog=cells,registry=[0]*64,root_owners=[])
    for name in ['active','inactive','prepared','candidate']:
        value[name]=dict(token=[0]*3)
    if owned:
        record=[1,1,3,2,1,1,1,1]
        value['registry'][:8]=record
        for i in [3,4]:cells[i*3]=1
        value['active']=dict(token=[1,1,0],record=record,cells=cells[9:15])
    return value


class PhysicalCatalogTests(unittest.TestCase):
    def test_contiguous_free_frames(self):
        self.assertEqual(physical_catalog(catalog()),dict(frames=8,free=8,largest_run=8,free_records=8))

    def test_live_owner_splits_capacity(self):
        self.assertEqual(physical_catalog(catalog(True)),dict(frames=8,free=6,largest_run=3,free_records=7))

    def test_physical_gap_splits_a_free_catalog(self):
        value=catalog()
        for i in range(4,8):value['catalog'][i*3+2]+=4
        self.assertEqual(physical_catalog(value)['largest_run'],4)

    def test_provenance_boundary_splits_physically_adjacent_frames(self):
        value=catalog()
        for i in range(4,8):value['catalog'][i*3+1]=2
        self.assertEqual(physical_catalog(value)['largest_run'],4)

    def test_owned_cell_cannot_be_presented_as_free(self):
        value=catalog(True);value['catalog'][9]=0
        with self.assertRaisesRegex(ValueError,'owned span'):physical_catalog(value)

    def test_truncated_registry_is_not_absence_evidence(self):
        value=catalog();value['registry'].pop()
        with self.assertRaisesRegex(ValueError,'complete physical'):physical_catalog(value)

    def test_aliased_physical_frame_is_invalid(self):
        value=catalog();value['catalog'][5]=value['catalog'][2]
        with self.assertRaisesRegex(ValueError,'physical identity'):physical_catalog(value)

    def test_missing_catalog_is_not_a_refusal_proof(self):
        value=catalog();del value['catalog']
        with self.assertRaisesRegex(ValueError,'missing complete'):physical_catalog(value)

    def test_firmware_requires_whole_unreserved_pages(self):
        value=dict(e820=[[0,10*PAGE,1],[5*PAGE+20,1,2]],
            reservations=[[0,2*PAGE+1],[8*PAGE+9,9*PAGE+5]])
        self.assertEqual(firmware_frames(value),[3,4,6,7])

    def test_firmware_interval_must_not_wrap(self):
        value=dict(e820=[[(1<<64)-2,4,1]],reservations=[[0,PAGE],[8*PAGE,9*PAGE]])
        with self.assertRaisesRegex(ValueError,'firmware interval'):firmware_frames(value)


if __name__=='__main__':unittest.main()
