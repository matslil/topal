"""Public metadata preservation/lifetime laws over independent byte images.

TOPAL-SYSTEMS-USER-EXTENT-001; TOPAL-SYSTEMS-INODE-MODE-STATE-001;
TOPAL-SYSTEMS-NAMESPACE-TRANSACTION-001. Model evidence grants no native owners.
"""
import struct
import unittest

from metadata_model import FS_BYTES,TABLE_BYTES,description,entry,inode,transition,word


class MetadataEffects(unittest.TestCase):
    def fixture(self):
        fs,table=bytearray(FS_BYTES),bytearray(TABLE_BYTES)
        struct.pack_into('<Q',fs,0,7)
        struct.pack_into('<4Q',fs,32,0o41777,17,18,19)
        struct.pack_into('<Q',table,0,1)
        struct.pack_into('<Q',table,1664,0o022)
        for fd in range(3):
            struct.pack_into('<QQ',table,64+fd*16,fd+1,0)
            struct.pack_into('<8Q',table,320+fd*64,1,1,0,0,1,0,0,0)
        return bytes(fs),bytes(table)

    def created(self):
        return transition(*self.fixture(),257,[-100,0,0x42,0o666],3,[b'/tmp/a'],23)

    def test_creation_uses_independent_lowest_entry_inode_fd_and_description(self):
        fs,table=self.created()
        at=inode(fs,7)
        self.assertEqual(entry(fs,b'/tmp/a'),0)
        self.assertEqual(list(struct.unpack_from('<16Q',fs,at)),[1,7,0,0o100644,23,23,23,1]+[0]*8)
        self.assertEqual(word(fs,0),8)
        self.assertEqual(list(struct.unpack_from('<4Q',fs,32)),[0o41777,17,23,23])
        self.assertEqual(list(struct.unpack_from('<8Q',table,description(table,3))),[7,1,0x8002,0,7,0,0,0])

    def test_pwrite_preserves_position_type_links_and_unrelated_storage(self):
        fs,table=self.created()
        table=bytearray(table)
        struct.pack_into('<Q',table,description(table,3)+24,99)
        fs=bytearray(fs)
        fs[2000]=0x6b
        after,actual=transition(fs,table,18,[3,0,4,0],4,seconds=24,payload=b'data')
        self.assertEqual(actual,bytes(table))
        self.assertEqual(after[2000],0x6b)
        self.assertEqual(after[4096:4104],b'data'+bytes(4))
        self.assertEqual(word(after,inode(after,7)+16),4)
        self.assertEqual(list(struct.unpack_from('<4Q',after,inode(after,7)+24)),[0o100644,23,24,24])

    def test_mode_masking_changes_only_mode_and_ctime_without_umask(self):
        fs,table=self.created()
        after,actual=transition(fs,table,91,[3,(1<<63)|0o24777,0,0],0,seconds=25)
        expected=bytearray(fs)
        at=inode(fs,7)
        struct.pack_into('<Q',expected,at+24,0o104777)
        struct.pack_into('<Q',expected,at+48,25)
        self.assertEqual(after,bytes(expected))
        self.assertEqual(actual,table)

    def test_link_and_rename_preserve_contents_positions_and_shared_inode(self):
        fs,table=self.created()
        fs,table=transition(fs,table,18,[3,0,4,0],4,seconds=24,payload=b'data')
        linked,same=transition(fs,table,86,[0,0,0,0],0,[b'/tmp/a',b'/tmp/b'],25)
        renamed,still_same=transition(linked,same,82,[0,0,0,0],0,[b'/tmp/b',b'/tmp/c'],26)
        self.assertEqual(table,same)
        self.assertEqual(table,still_same)
        self.assertIsNone(entry(renamed,b'/tmp/b'))
        self.assertEqual(entry(renamed,b'/tmp/c'),1)
        self.assertEqual(word(renamed,inode(renamed,7)+56),2)
        self.assertEqual(renamed[4096:8192],fs[4096:8192])
        self.assertEqual(word(renamed,inode(renamed,7)+40),24)

    def test_truncation_retains_shared_identity_mode_and_atime(self):
        fs,table=self.created()
        fs,table=transition(fs,table,18,[3,0,4,0],4,seconds=24,payload=b'data')
        after,opened=transition(fs,table,257,[-100,0,0x202,0],4,[b'/tmp/a'],25)
        self.assertEqual(after[4096:8192],bytes(4096))
        self.assertEqual(after[1024:1056],bytes(32))
        self.assertEqual(after[24],0)
        self.assertEqual(list(struct.unpack_from('<7Q',after,inode(after,7))),[1,7,0,0o100644,23,25,25])
        self.assertEqual(word(opened,description(opened,3)+32),7)
        self.assertEqual(word(opened,description(opened,4)+32),7)
        self.assertEqual(word(after,0),8)

    def test_descriptor_shrink_erases_the_complete_tail_and_preserves_position(self):
        fs,table=self.created()
        fs,table=transition(fs,table,18,[3,0,4,0],4,seconds=24,payload=b'data')
        after,same=transition(fs,table,77,[3,2,0,0],0,seconds=25)
        self.assertEqual(same,table)
        self.assertEqual(after[4096:8192],b'da'+bytes(4094))
        self.assertEqual(after[1024:1056],fs[1024:1056])
        self.assertEqual(after[24],1)
        self.assertEqual(word(after,inode(after,7)+16),2)
        self.assertEqual(word(after,inode(after,7)+24),0o100644)
        self.assertEqual(word(after,inode(after,7)+32),23)

    def test_unlinked_inode_is_reclaimed_only_after_last_description(self):
        fs,table=self.created()
        fs,table=transition(fs,table,257,[-100,0,0x202,0],4,[b'/tmp/a'],24)
        fs,table=transition(fs,table,18,[3,0,4,0],4,seconds=25,payload=b'data')
        fs,table=transition(fs,table,87,[0,0,0,0],0,[b'/tmp/a'],26)
        first,table=transition(fs,table,3,[4,0,0,0],0)
        self.assertEqual(first,fs)
        self.assertEqual(first[4096:4100],b'data')
        final,table=transition(first,table,3,[3,0,0,0],0)
        self.assertEqual(final[64:128],bytes(64))
        self.assertEqual(final[256:],bytes(FS_BYTES-256))
        self.assertEqual(table,self.fixture()[1])
        self.assertEqual(word(final,0),8)

    def test_refusals_cannot_advance_clock_or_change_any_private_byte(self):
        fs,table=self.created()
        for result in [-14,-9,-17,-2]:
            with self.subTest(result=result):
                self.assertEqual(transition(fs,table,87,[0,0,0,0],result),(fs,table))
                with self.assertRaises(AssertionError):transition(fs,table,87,[0,0,0,0],result,seconds=99)

    def test_wrong_descriptor_and_unqualified_payload_layout_are_rejected(self):
        fs,table=self.created()
        for args,payload in [([16,0,4,0],b'data'),([3,0,5,0],b'data'),([3,0,4,4096],b'data')]:
            with self.subTest(args=args),self.assertRaises(AssertionError):
                transition(fs,table,18,args,4,seconds=24,payload=payload)
        with self.assertRaises(AssertionError):transition(fs[:-1],table,3,[3,0,0,0],0)


if __name__=='__main__': unittest.main()
