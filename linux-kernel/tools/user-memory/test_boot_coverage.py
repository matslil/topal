"""Boot coverage interpreter controls; no injected guest state."""
import struct
import unittest
from boot_coverage import PAGE,NX,check_bank,page,journal,roles,FIXED,MMIO
from inactive_image_tables import encode


BASE=0x5000000
LAYOUT=dict(minimum_granule=PAGE,large_granule=2<<20,address_limit=1<<30,
            spans=[dict(start=0x200000,end=0x201000,role='immutable')])


class BootCoverage(unittest.TestCase):
    def test_both_real_topologies_cover_all_supported_boot_roles(self):
        contract=dict(roles=[dict(role=n,start=a,end=b) for n,a,b in FIXED],
                      mmio_windows=MMIO,inactive_coverage=dict(roles=11,
                      journal_words=64,complete_page_walks=True,activation=False))
        registers=[0]*128
        registers[16:19]=[0x95001,0x1200001,0x1202001]
        extents=roles(contract,registers)
        counts=None
        for low in [None,0x4000000]:
            current=check_bank(encode(LAYOUT,BASE,low),BASE,extents)
            self.assertEqual(current,[1,3,1,1,1,1,120,2,3,512,512])
            counts=current
        self.assertEqual(journal(extents,counts,1)[:4],[1,11,sum(counts),22])
        self.assertEqual(len(journal(extents,counts,2)),64)

    def test_small_leaf_hole_inside_extent_is_not_hidden_by_valid_endpoints(self):
        data=bytearray(encode(LAYOUT,BASE))
        # Replace the first large leaf by an owned small table, using the
        # otherwise empty reserved high USER page in this model only.
        struct.pack_into('<Q',data,1024*8,BASE+4*PAGE+3)
        for i in range(512): struct.pack_into('<Q',data,4*PAGE+i*8,i*PAGE|NX|3)
        extent=[(0x100000,0x180000,0)]
        self.assertEqual(check_bank(data,BASE,extent),[128])
        middle=0x140000//PAGE
        struct.pack_into('<Q',data,4*PAGE+middle*8,0)
        page(data,BASE,0x100000,0)
        page(data,BASE,0x17f000,0)
        with self.assertRaises(ValueError): check_bank(data,BASE,extent)

    def test_effective_ancestor_rights_and_supported_granules(self):
        original=encode(LAYOUT,BASE)
        for index,mask in [(0,1),(0,2),(512,2),(512,128),(1024,1),
                           (1024,NX),(1024,8),(1024,1<<21),(0,1<<30)]:
            data=bytearray(original)
            value=struct.unpack_from('<Q',data,index*8)[0]
            struct.pack_into('<Q',data,index*8,value^mask)
            with self.subTest(index=index,mask=mask),self.assertRaises(ValueError):
                page(data,BASE,0x100000,0)
        data=bytearray(original)
        # All ancestors and leaf USER-accessible is invalid, even though the
        # identity, write and NX observations remain unchanged.
        value=struct.unpack_from('<Q',data,1024*8)[0]
        struct.pack_into('<Q',data,1024*8,value|4)
        with self.assertRaises(ValueError): page(data,BASE,0x100000,0)

    def test_extent_bounds_rounding_and_unsupported_inputs(self):
        data=encode(LAYOUT,BASE)
        for start,end in [(0,0),(2,1),(-1,1),(0,1<<64),(0,(1<<32)+1)]:
            with self.subTest(start=start,end=end),self.assertRaises(ValueError):
                check_bank(data,BASE,[(start,end,0)])
        self.assertEqual(check_bank(data,BASE,[(0x100001,0x102001,0)]),[3])
        with self.assertRaises(ValueError): check_bank(data,BASE,[(1<<30,(1<<30)+1,0)])


if __name__=='__main__': unittest.main()
