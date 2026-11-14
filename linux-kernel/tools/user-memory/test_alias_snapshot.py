"""Nonexecuted controls for complete descriptor and protected-alias analysis."""
import copy
import unittest

from alias_snapshot import audit,enumerate_leaves
from owner_interval import regions
from test_owner_snapshot import fixture as owner_fixture


def fixture():
    snapshot,bindings,spans=owner_fixture()
    root=snapshot['context']['root']
    directory=snapshot['tables'][str(root+8192)]
    for i in range(512):
        if i not in [2,3]:directory[i]=(i<<21)|0x83
    snapshot['tables'][str(root+4096)][3]=root+12288|3
    for i in [502,503]:snapshot['tables'][str(root+12288)][i]=(i<<21)|(3<<30)|0x9b
    addresses=dict(root_tokens=0x4000000,mm_tokens=0x4001000,context=0x4002000,
        mm_state=0x4003000,mm_epoch=0x4004000,registry=0x5000000,table=0x6000000)
    return snapshot,bindings,spans,regions(snapshot,bindings,addresses)


class AliasSnapshotTests(unittest.TestCase):
    def check(self,snapshot,bindings,spans,protected):
        return audit(snapshot,bindings,spans,0x638fc0,144,1,protected)

    def test_complete_user_and_supervisor_descriptor_walk(self):
        result=self.check(*fixture())
        self.assertEqual(result['user_pages'],572)
        self.assertEqual(result['supervisor_leaves'],512)
        self.assertEqual(len(result['inventory']['leaves']),1084)
        self.assertEqual(len(result['inventory']['entries']),1089)
        self.assertEqual(len(result['inventory']['visits']),6)
        self.assertEqual(len(result['protected_routes']),26)

    def test_upper_half_supervisor_alias_is_not_pruned(self):
        snapshot,bindings,spans,protected=fixture()
        root=snapshot['context']['root']
        snapshot['tables'][str(root)][256]=snapshot['tables'][str(root)][0]&~4
        leaves=enumerate_leaves(snapshot)['leaves']
        self.assertTrue(any(r['virtual']>=0xffff800000000000 for r in leaves))
        with self.assertRaisesRegex(AssertionError,'nonidentity route'):self.check(snapshot,bindings,spans,protected)

    def test_upper_half_user_alias_does_not_create_an_owner(self):
        snapshot,bindings,spans,protected=fixture()
        root=snapshot['context']['root']
        snapshot['tables'][str(root)][256]=snapshot['tables'][str(root)][0]
        with self.assertRaisesRegex(AssertionError,'outside the selected live user owners'):self.check(snapshot,bindings,spans,protected)

    def test_foreign_and_wrong_level_tables_are_not_fetched(self):
        original,_,_,_=fixture()
        root=original['context']['root']
        for target in [0x17000000,original['context']['low']]:
            with self.subTest(target=target):
                snapshot=copy.deepcopy(original)
                snapshot['tables'][str(root)][1]=target|3
                reads=[]
                with self.assertRaisesRegex(AssertionError,'wrong-level table'):enumerate_leaves(snapshot,reads=reads)
                self.assertFalse(any(p==target and level==3 for p,level in reads))

    def test_user_leaf_cannot_point_to_kernel_or_translation_storage(self):
        original,bindings,spans,protected=fixture()
        low=original['context']['low']
        for physical in [0x200000,original['context']['root']]:
            with self.subTest(physical=physical):
                snapshot=copy.deepcopy(original)
                snapshot['tables'][str(low)][503]=physical|7
                with self.assertRaisesRegex(AssertionError,'actual backing owner'):self.check(snapshot,bindings,spans,protected)

    def test_supervisor_ancestor_blocks_user_leaf_flags(self):
        snapshot,bindings,spans,protected=fixture()
        root=snapshot['context']['root']
        snapshot['tables'][str(root+12288)][502]|=4
        self.assertEqual(self.check(snapshot,bindings,spans,protected)['user_pages'],572)

    def test_user_large_leaf_and_large_leaf_reserved_address_bits_fail(self):
        original,bindings,spans,protected=fixture()
        root=original['context']['root']
        snapshot=copy.deepcopy(original)
        snapshot['tables'][str(root+8192)][30]=0x13000000|0x87
        with self.assertRaisesRegex(AssertionError,'large user mapping'):self.check(snapshot,bindings,spans,protected)
        snapshot=copy.deepcopy(original)
        snapshot['tables'][str(root+8192)][32]|=1<<13
        with self.assertRaisesRegex(AssertionError,'large leaf format'):self.check(snapshot,bindings,spans,protected)

    def test_missing_identity_route_and_missing_owned_user_page_fail(self):
        original,bindings,spans,protected=fixture()
        root=original['context']['root']
        snapshot=copy.deepcopy(original)
        snapshot['tables'][str(root+8192)][32]=0
        with self.assertRaisesRegex(AssertionError,'unmapped protected'):self.check(snapshot,bindings,spans,protected)
        snapshot=copy.deepcopy(original)
        snapshot['tables'][str(original['context']['low'])][503]=0
        with self.assertRaisesRegex(AssertionError,'missing live user mapping'):self.check(snapshot,bindings,spans,protected)

    def test_present_disabled_nonleaf_permissions_remain_effective(self):
        snapshot,_,_,_=fixture()
        root=snapshot['context']['root']
        snapshot['tables'][str(root+4096)][3]|=1<<63
        leaves=enumerate_leaves(snapshot)['leaves']
        apic=[r for r in leaves if r['virtual']==0xfee00000]
        self.assertEqual(len(apic),1)
        self.assertFalse(apic[0]['executable'])
        self.assertFalse(apic[0]['user'])


if __name__=='__main__':unittest.main()
