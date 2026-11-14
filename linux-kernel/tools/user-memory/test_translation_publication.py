"""Model-level negative controls for complete local publication ordering."""
import copy
import unittest

from translation_publication import check,slot,LEAF,PAGE


def fixture(old,new):
    root,low=0x2000000,0x2100000
    before=dict(context=dict(root=root,low=low,state=1,bootstrap=0x100000),state=[1,0x5f7000+old*PAGE,old,0x3000000 if old else 0],
        tokens=[1,2,3,1,4,5],thread=[1,1,1],cr0=0x80010011,cr3=root,cr4=0x620,cs=16,flags=2,
        tables={str(t):[0]*512 for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,low]})
    for i in range(old):
        table,index=slot(before,i)
        before['tables'][table][index]=(before['state'][3]+i*PAGE)|LEAF|0x60
    after=copy.deepcopy(before)
    base=0x3400000 if new else 0
    for i in range(max(old,new)):
        table,index=slot(before,i)
        after['tables'][table][index]=((base+i*PAGE)|LEAF) if i<new else 0
    return dict(old_count=old,new_count=new,new_base=base,before=before,stored=after,
        after=copy.deepcopy(after),invalidations=[dict(index=i,address=0x5f7000+i*PAGE,thread=[1,1,1]) for i in range(max(old,new))],
        published_before_invalidations=True,final_barrier_observed=True,
        entry_stack=0x100000,return_stack=0x100000,entry_return_address=0x101000,return_address=0x101000)


class TranslationPublicationTests(unittest.TestCase):
    def test_detached_cleanup_requires_actual_non_global_non_pcid_root_switch(self):
        record=fixture(67,0)
        before=copy.deepcopy(record['before'])
        after=copy.deepcopy(before)
        after['cr3']=after['context']['bootstrap']
        record.update(operation=6,detachment=dict(before=before,after=after))
        for name in ['before','stored','after']:
            record[name]['cr3']=record[name]['context']['bootstrap']
            record[name]['context']['state']=2
        self.assertTrue(check(record)['passed'])
        for change in [lambda r:r.update(detachment=None),lambda r:r.update(operation=5),
                lambda r:r['detachment']['before'].update(cr4=0x6a0),
                lambda r:r['detachment']['after'].update(cr3=r['before']['context']['root']),
                lambda r:r['before']['context'].update(state=1)]:
            value=copy.deepcopy(record)
            change(value)
            with self.assertRaises(AssertionError):check(value)

    def test_growth_shrink_zero_and_full_capacity(self):
        for old,new in [(0,1),(1,9),(9,10),(10,67),(67,1),(67,0),(0,519),(519,0)]:
            with self.subTest(old=old,new=new):self.assertEqual(check(fixture(old,new))['pages_invalidated'],max(old,new))

    def test_missing_or_wrong_invalidation(self):
        for change in [lambda r:r['invalidations'].pop(),
                lambda r:r['invalidations'][9].update(address=0x5f7000),
                lambda r:r['invalidations'][9].update(index=8),
                lambda r:r['invalidations'][9].update(thread=[1,2,2])]:
            record=fixture(10,67)
            change(record)
            with self.assertRaises(AssertionError):check(record)

    def test_shrink_must_invalidate_the_removed_tail(self):
        record=fixture(67,1)
        record['invalidations']=record['invalidations'][:1]
        with self.assertRaisesRegex(AssertionError,'incomplete invalidation'):check(record)

    def test_publish_all_before_invalidating_and_barrier_before_return(self):
        for field in ['published_before_invalidations','final_barrier_observed']:
            record=fixture(1,67)
            record[field]=False
            with self.assertRaises(AssertionError):check(record)

    def test_backing_permissions_and_unrelated_mapping_changes(self):
        for mask in [2,4096]:
            record=fixture(1,67)
            table,index=slot(record['before'],66)
            record['stored']['tables'][table][index]^=mask
            with self.assertRaisesRegex(AssertionError,'published heap leaf'):check(record)
        record=fixture(1,67)
        root=str(record['before']['context']['root'])
        record['stored']['tables'][root][8]=7
        with self.assertRaisesRegex(AssertionError,'unrelated translation'):check(record)

    def test_disabled_mode_or_changed_root_or_cpu(self):
        for field,value in [('cr4',0x20620),('cr4',0x6a0),('cr4',0x1620),('cr3',0x1000),('cr0',0x80000011),('cs',0x23),('flags',0x202),('thread',[1,2,2])]:
            record=fixture(1,67)
            record['stored'][field]=value
            with self.assertRaises(AssertionError):check(record)

    def test_table_or_mode_changes_during_invalidation_and_wrong_return(self):
        for change in [lambda r:r['after'].update(cr4=0x600),lambda r:r.update(return_stack=0),
                lambda r:r.update(return_address=0),lambda r:r['after']['tables'][str(r['before']['context']['root'])].__setitem__(4,7)]:
            record=fixture(1,67)
            change(record)
            with self.assertRaises(AssertionError):check(record)


if __name__=='__main__':unittest.main()
