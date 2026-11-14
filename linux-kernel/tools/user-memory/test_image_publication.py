"""Saved-model counterexamples for image mapping edits; no guest execution."""
import copy
import unittest

from image_publication import BASE,NX,PAGE,check
from test_owner_snapshot import fixture as owner_fixture


def fixture(kind):
    before,bindings,spans=owner_fixture()
    before['image_generation']=0
    before['context']['phase']=8 if kind=='relro' else 32
    before['epoch']=[1,10 if kind=='relro' else 59]
    low=str(before['context']['low'])
    image=before['roles'][2]['cells'][2]*PAGE
    count=8 if kind=='relro' else 18
    for i in range(18):
        old_flags=7 if kind=='relro' or i>=8 else 5
        before['tables'][low][485+i]=(image+BASE-0x400000+i*PAGE)|NX|old_flags|0x60
    stored=copy.deepcopy(before)
    for i in range(count):
        stored['tables'][low][485+i]=(image+BASE-0x400000+i*PAGE)|NX|(5 if kind=='relro' else 7)
    events=[dict(index=count-i,address=BASE+i*PAGE,cpu=0,cr0=before['cr0'],cr3=before['cr3'],
        cr4=before['cr4'],flags=2,efer=0xd00,cancelled=False) for i in range(count)]
    row=dict(kind=kind,before=before,stored=stored,after=copy.deepcopy(stored),invalidations=events,
        thread_before=[1,1,0],thread_stored=[1,1,0],thread_after=[1,1,0],
        publication_barrier_observed=True,final_barrier_observed=kind=='relro')
    return row,bindings,spans


class ImagePublicationTests(unittest.TestCase):
    def test_exec_generation_binds_its_existing_distinct_lifecycle(self):
        for phase in range(11,16):
            row,bindings,spans=fixture('exec')
            for name in ['before','stored','after']:
                row[name]['image_generation']=1
                row[name]['context']['phase']=phase
            self.assertTrue(check(row,bindings,spans)['passed'])
        for generation,phase in [(0,11),(1,32),(1,10),(1,16),(2,11)]:
            row,bindings,spans=fixture('exec')
            row['before']['image_generation']=generation
            row['before']['context']['phase']=phase
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_both_actual_publication_shapes(self):
        for kind,count in [('relro',8),('exec',18)]:
            row,bindings,spans=fixture(kind)
            self.assertEqual(check(row,bindings,spans)['pages_invalidated'],count)

    def test_missing_removed_or_unordered_invalidation(self):
        for change in [lambda r:r['invalidations'].pop(),
                lambda r:r['invalidations'][1].update(index=1),
                lambda r:r['invalidations'][1].update(address=BASE)]:
            row,bindings,spans=fixture('exec')
            change(row)
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_permissions_physical_identity_and_exec_tail(self):
        for kind,index,mask in [('relro',485,2),('relro',485,4096),('exec',502,2),('exec',502,NX)]:
            row,bindings,spans=fixture(kind)
            row['stored']['tables'][str(row['before']['context']['low'])][index]^=mask
            with self.assertRaisesRegex(AssertionError,'published image leaf'):check(row,bindings,spans)

    def test_owner_provenance_and_pins(self):
        for change in [lambda s:s['roles'][2]['record'].__setitem__(6,0),
                lambda s:s['roles'][2]['cells'].__setitem__(1,0),
                lambda s:s['root_tokens'].__setitem__(7,99),
                lambda s:s['heap_tokens'].__setitem__(3,1)]:
            row,bindings,spans=fixture('relro')
            change(row['before'])
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_wrong_entry_mode_and_cpu_are_not_admitted(self):
        for change in [lambda r:r['before']['context'].update(phase=9),
                lambda r:r['before'].update(epoch=[1,5]),
                lambda r:r['before'].update(cr4=0x20620),
                lambda r:r['stored'].update(cr3=0),
                lambda r:r['invalidations'][0].update(cpu=1),
                lambda r:r['invalidations'][0].update(cr0=0),
                lambda r:r.update(thread_after=[1,2,0])]:
            row,bindings,spans=fixture('relro')
            change(row)
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_unrelated_tables_and_ancestor_permissions(self):
        for change in [lambda r:r['stored']['tables'][str(r['before']['context']['root'])].__setitem__(7,7),
                lambda r:r['before']['tables'][str(r['before']['context']['root'])].__setitem__(0,0)]:
            row,bindings,spans=fixture('exec')
            change(row)
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_barrier_claims_match_the_actual_distinct_layouts(self):
        for kind,field,value in [('relro','publication_barrier_observed',False),
                ('relro','final_barrier_observed',False),('exec','final_barrier_observed',True)]:
            row,bindings,spans=fixture(kind)
            row[field]=value
            with self.assertRaises(AssertionError):check(row,bindings,spans)

    def test_post_publication_mutations_cannot_hide_at_finish(self):
        for field,value in [('epoch',[2,59]),('cr4',0x600),('flags',0x202)]:
            row,bindings,spans=fixture('exec')
            row['after'][field]=value
            with self.assertRaises(AssertionError):check(row,bindings,spans)


if __name__=='__main__':unittest.main()
