"""TOPAL-SYSTEMS-PROCESS-MM-001 model controls; no native authority."""
from copy import deepcopy
import hashlib
import unittest

from publication import check,retirement,HEAP,PAGE,LEAF,slot


def allocation(identity,index,start,count,physical):
    if not count:return dict(token=[0]*3,record=[],cells=[],bytes='')
    return dict(token=[1,identity,index],record=[1,identity,start,count,1,1,1,1],
        cells=[value for i in range(count) for value in [identity,1,physical//PAGE+i]],bytes='')


def fixture(operation=10,old=34,new=34):
    root,low=0x2000000,0x2100000
    active=allocation(2,0,0,old,0x3000000)
    selected=allocation(3,1,600,new,0x3400000)
    empty=allocation(0,0,0,0,0)
    before=dict(allocator=[1,1,20,2048]+[0]*12,
        context=dict(address=0x120000,root=root,low=low,state=1,bootstrap=0x100000),
        state=[1,HEAP+old*PAGE,old,0x3000000 if old else 0],
        inactive_state=[0]*4,prepared_state=[0]*7,prepared_owner=[0]*3,
        ids=[1,1,0,0,0,0,3],transition=[0]*3,epoch=[14,57],
        tokens=active['token']+[0]*3,active=active,candidate=deepcopy(empty),
        inactive=deepcopy(empty),prepared=deepcopy(empty),
        root_owners=[allocation(6,6,1200,5,root),allocation(7,7,1300,1,low)],
        kernel_spans=[[0,0x1000000]],thread=[1,1,1],cr0=0x80010011,cr3=root,cr4=0x620,cs=16,flags=2,
        tables={str(t):[0]*512 for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,low]})
    state=[1,HEAP+new*PAGE,new,0x3400000 if new else 0]
    if operation==10:
        before.update(prepared=selected,prepared_state=state+[root,0x120000,14],
            prepared_owner=[1,1,0],transition=[2,2,0])
    elif operation in [11,12]:
        before.update(inactive=selected,inactive_state=state,ids=[1,1,0,2,2,0,3])
        if operation==12:before['ids']=[2,2,0,1,1,0,3]
    elif operation==5:
        before.update(candidate=selected,tokens=active['token']+selected['token'])
    elif operation not in [6,7]:raise ValueError(operation)
    for i in range(old):
        table,index=slot(before,i)
        before['tables'][table][index]=(before['state'][3]+i*PAGE)|LEAF|0x60
    stored=deepcopy(before)
    base=0x3400000 if new else 0
    for i in range(max(old,new)):
        table,index=slot(before,i)
        stored['tables'][table][index]=((base+i*PAGE)|LEAF) if i<new else 0
    return dict(operation=operation,old_count=old,new_count=new,new_base=base,
        before=before,stored=stored,after=deepcopy(stored),detachment=None,
        invalidations=[dict(index=i,address=HEAP+i*PAGE,thread=[1,1,1]) for i in range(max(old,new))],
        published_before_invalidations=True,final_barrier_observed=True,
        entry_stack=0x110000,return_stack=0x110000,entry_return_address=0x110080,return_address=0x110080)


class ProcessPublicationTests(unittest.TestCase):
    def test_equal_size_process_backing_and_empty_identity_transitions(self):
        for operation in [10,11,12]:
            for count in [0,1,34,519]:
                with self.subTest(operation=operation,count=count):
                    result=check(fixture(operation,count,count))
                    self.assertEqual(result['pages_invalidated'],count)
                    if count:self.assertNotEqual(result['source_token'],result['target_token'])

    def test_different_process_geometry_and_original_resize_exec(self):
        for operation,old,new in [(10,67,67),(11,67,101),(11,101,0),(12,0,519),
                                  (5,0,1),(5,67,1),(7,67,0)]:
            with self.subTest(operation=operation,old=old,new=new):
                self.assertEqual(check(fixture(operation,old,new))['pages_invalidated'],max(old,new))

    def test_exact_table_pins_do_not_constrain_other_allocation_pin_counts(self):
        record=fixture()
        extra=allocation(8,8,1400,2,0x2200000);extra['record'][6]=2
        for name in ['before','stored','after']:record[name]['root_owners'].append(deepcopy(extra))
        self.assertTrue(check(record)['passed'])
        for index in [0,1]:
            changed=deepcopy(record)
            for name in ['before','stored','after']:changed[name]['root_owners'][index]['record'][6]=0
            with self.assertRaisesRegex(ValueError,'owned translation storage'):check(changed)

    def test_target_pin_receipt_and_private_disjointness(self):
        for message,change in [
                ('bank backing/pin',lambda r:r['before']['prepared']['record'].__setitem__(6,0)),
                ('prepared receipt',lambda r:r['before']['prepared_owner'].__setitem__(2,1)),
                ('prepared receipt',lambda r:r['before']['prepared_state'].__setitem__(6,15)),
                ('creation identity',lambda r:r['before']['transition'].__setitem__(2,1))]:
            record=fixture();change(record)
            for name in ['stored','after']:
                tables=record[name]['tables']
                record[name]=deepcopy(record['before']);record[name]['tables']=tables
            with self.assertRaisesRegex(ValueError,message):check(record)
        record=fixture()
        before=record['before']
        before['prepared']['cells'][2::3]=before['active']['cells'][2::3]
        before['prepared_state'][3]=before['state'][3]
        record['new_base']=before['state'][3]
        for name in ['stored','after']:record[name]=deepcopy(before)
        with self.assertRaisesRegex(ValueError,'backing alias'):check(record)

    def test_stale_inactive_identity_and_changed_processor_mode(self):
        record=fixture(11)
        for name in ['before','stored','after']:record[name]['ids'][4]=record[name]['ids'][1]
        with self.assertRaisesRegex(ValueError,'inactive MM identity'):check(record)
        for field,value in [('cr0',0x80000011),('cr4',0x20620),('flags',0x202),('thread',[1,2,2])]:
            record=fixture();record['stored'][field]=value
            with self.subTest(field=field):
                with self.assertRaises((AssertionError,ValueError)):check(record)

    def test_missing_or_reordered_invalidations_and_barriers(self):
        for change in [lambda r:r['invalidations'].pop(),
                lambda r:r['invalidations'][9].update(index=8),
                lambda r:r['invalidations'][9].update(thread=[1,2,2]),
                lambda r:r.update(published_before_invalidations=False),
                lambda r:r.update(final_barrier_observed=False)]:
            record=fixture();change(record)
            with self.assertRaises(ValueError):check(record)


def retired(publication,inactive=False):
    before=deepcopy(publication['after'])
    role='active';operation=publication['operation']
    if inactive:
        role='inactive';operation=13
        before['active'],before['inactive']=before['inactive'],before['active']
        before['state'],before['inactive_state']=before['inactive_state'],before['state']
        before['tokens']=before['active']['token']+[0]*3
        before['ids']=[1,1,0,2,2,0,3]
    original=before[role]
    unpinned=deepcopy(original);unpinned['record'][6]=0
    base=original['cells'][2]*PAGE;count=original['record'][3]
    zero=hashlib.sha256(bytes(count*PAGE)).hexdigest()
    free=[value for i in range(count) for value in [0,original['cells'][i*3+1],original['cells'][i*3+2]]]
    return dict(operation=operation,bank=role,before=before,owner=deepcopy(original),publication_index=0,
        detachment=None,unpin_succeeded=True,unpinned=unpinned,
        erase=dict(base=base,count=count),
        release=dict(base=base,count=count,erased_bytes=count*PAGE,erased_sha256=zero),
        release_succeeded=True,released=dict(token=[0]*3,record=[0]*8,cells=free,erased_sha256=zero))


class ProcessRetirementTests(unittest.TestCase):
    def test_active_child_retirement_and_inactive_parent_cancellation(self):
        publication=fixture(12,67,101)
        self.assertEqual(retirement(retired(publication),[publication])['erased_bytes'],67*PAGE)
        publication=fixture(11,101,67)
        publication['before']['ids']=[2,2,0,1,1,0,3]
        for name in ['stored','after']:publication[name]['ids']=publication['before']['ids'][:]
        self.assertEqual(retirement(retired(publication,True),[publication])['erased_bytes'],101*PAGE)

    def test_unpin_requires_completed_demap_and_must_not_reselect_old_backing(self):
        publication=fixture(12,67,101);record=retired(publication)
        with self.assertRaisesRegex(ValueError,'missing completed publication'):retirement(record,[])
        changed=deepcopy(publication);changed['invalidations'].pop()
        with self.assertRaisesRegex(ValueError,'complete invalidation'):retirement(record,[changed])
        changed=deepcopy(record)
        changed['before']['tables'][str(changed['before']['context']['low'])][503]=0x3000000|LEAF
        with self.assertRaisesRegex(ValueError,'mapping changed'):retirement(changed,[publication])

    def test_missing_unpin_dirty_erasure_and_retained_release_are_rejected(self):
        publication=fixture(12,67,101)
        for message,change in [
                ('unpin result',lambda r:r.update(unpin_succeeded=False)),
                ('unpin effect',lambda r:r['unpinned']['record'].__setitem__(6,1)),
                ('erase arguments',lambda r:r['erase'].update(count=66)),
                ('complete erasure',lambda r:r['release'].update(erased_sha256='0'*64)),
                ('consumed owner',lambda r:r['released']['record'].__setitem__(7,1)),
                ('consumed owner',lambda r:r['released']['cells'].__setitem__(0,2))]:
            record=retired(publication);change(record)
            with self.assertRaisesRegex(ValueError,message):retirement(record,[publication])

    def test_no_unrelated_store_or_owner_change_during_publication(self):
        for change in [lambda r:r['stored']['tables'][str(r['before']['context']['root'])].__setitem__(3,7),
                lambda r:r['after']['ids'].__setitem__(1,8),
                lambda r:r.update(return_address=0)]:
            record=fixture();change(record)
            with self.assertRaises(ValueError):check(record)


if __name__=='__main__':unittest.main()
