"""Saved-record controls for live owner/translation evidence; no guest execution."""
import copy
import unittest

from owner_snapshot import check


def fixture():
    bindings=dict(token_bytes=14*24,roles=[dict(slot=i,count=n,pins=p)
        for i,(n,p) in enumerate([(5,1),(1,1),(503,1),(2,2)])])
    bases=[0x10000000,0x10010000,0x11000000,0x12000000,0x13000000]
    tokens=[]
    allocations=[]
    cell=0
    for i,(count,pins) in enumerate([(5,1),(1,1),(503,1),(2,2),(67,1)]):
        token=[1,i+1,i]
        tokens.append(token)
        cells=[value for j in range(count) for value in [i+1,99,bases[i]//4096+j]]
        allocations.append(dict(record=[1,i+1,cell,count,1,99,pins,1],cells=cells))
        cell+=count
    root=bases[0]
    tables={str(at):[0]*512 for at in [root,root+4096,root+8192,root+12288,root+16384,bases[1]]}
    tables[str(root)][0]=root+4096|7
    tables[str(root+4096)][0]=root+8192|7
    tables[str(root+8192)][2]=bases[1]|7
    tables[str(root+8192)][3]=root+16384|7
    for vbase,pbase,count,writable in [(0x400000,bases[2],503,False),
            (0x7fe000,bases[3],2,True),(0x5f7000,bases[4],67,True)]:
        for i in range(count):
            virtual=vbase+i*4096
            table=bases[1] if virtual>>21==2 else root+16384
            tables[str(table)][virtual//4096%512]=pbase+i*4096|(7 if writable else 5)
    snapshot=dict(allocator=[1,1,6,2048]+[0]*12,
        root_tokens=sum(tokens[:4],[])+[0]*30,roles=allocations[:4],
        state=[1,0x63a000,67,bases[4]],heap_tokens=tokens[4]+[0]*3,heap=allocations[4],
        context=dict(address=0x200000,state=1,self=0x200000,root=root,
            low=bases[1],stack_page=bases[3]+4096,phase=32),
        tables=tables,epoch=[1,5],cr0=0x80010011,cr3=root,cr4=0x20,
        gs_base=0x200000,cs=8,flags=0x46)
    return snapshot,bindings,[[0x100000,0x3000000]]


class OwnerSnapshotTests(unittest.TestCase):
    def test_split_page_output_binds_two_heap_pages(self):
        snapshot,bindings,spans=fixture()
        result=check(snapshot,bindings,spans,0x638fc0,144,1)
        self.assertEqual([p['virtual'] for p in result['pages']],[0x638000,0x639000])
        self.assertTrue(all(p['identity']==5 for p in result['pages']))
        self.assertEqual(len(result['owners']),5)

    def test_readonly_image_input_uses_image_owner(self):
        snapshot,bindings,spans=fixture()
        snapshot['epoch'][1]=18
        result=check(snapshot,bindings,spans,0x4e8000,4,0)
        self.assertEqual(result['pages'][0]['identity'],3)

    def test_readonly_image_cannot_be_output(self):
        snapshot,bindings,spans=fixture()
        with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x4e8000,4,1)

    def test_stale_or_unpinned_owner_records_are_rejected(self):
        original,bindings,spans=fixture()
        for field,index,value in [('record',0,2),('record',1,99),('record',3,1),
                ('record',5,0),('record',6,0),('record',7,0),('cells',0,99),
                ('cells',1,100),('cells',5,0)]:
            with self.subTest(field=field,index=index,value=value):
                snapshot=copy.deepcopy(original)
                snapshot['roles'][2][field][index]=value
                with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x638fc0,144,1)

    def test_physical_aliases_do_not_manufacture_backing(self):
        original,bindings,spans=fixture()
        for kind in ['kernel','table','image']:
            with self.subTest(kind=kind):
                snapshot=copy.deepcopy(original)
                selected={'kernel':0x200000,'table':0x10000000,'image':0x11000000}[kind]
                snapshot['state'][3]=selected
                snapshot['heap']['cells'][2::3]=list(range(selected//4096,selected//4096+67))
                with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x638fc0,144,1)

    def test_unowned_tables_and_forged_leaves_are_rejected(self):
        original,bindings,spans=fixture()
        for at,index,value in [(0x10000000,0,0x10010007),
                (0x10004000,0x638000//4096%512,0x10010007),
                (0x10000000,0,0x10001003),(0x10001000,0,0x10002005)]:
            with self.subTest(at=at,index=index,value=value):
                snapshot=copy.deepcopy(original)
                snapshot['tables'][str(at)][index]=value
                with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x638fc0,144,1)

    def test_wrong_entry_or_mode_cannot_qualify_snapshot(self):
        original,bindings,spans=fixture()
        for name,value in [('gs_base',0),('cr3',0),('cr0',0x80000011),
                ('cr4',1<<12),('cs',3),('flags',0x246),('epoch',[0,5])]:
            with self.subTest(name=name):
                snapshot=copy.deepcopy(original)
                snapshot[name]=value
                with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x638fc0,144,1)

    def test_heap_reuse_of_root_identity_is_rejected(self):
        snapshot,bindings,spans=fixture()
        snapshot['root_tokens'][13*3+1]=5
        with self.assertRaises(AssertionError):check(snapshot,bindings,spans,0x638fc0,144,1)


if __name__=='__main__':unittest.main()
