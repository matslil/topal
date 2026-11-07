"""Independent V6 consumer-oracle fixtures, including malformed live stores."""
import hashlib
from pathlib import Path
import unittest

import namespace_file_store_checks as CHECKS


class NamespaceFileStoreChecks(unittest.TestCase):
    def test_independent_slots_aliases_anonymous_lifetimes_and_corruptions(self):
        # TOPAL-SYSTEMS-NAMESPACE-OWNERSHIP-001; no native metadata validator.
        def put(store,at,value): store[at:at+8]=value.to_bytes(8,'little')
        def seed():
            store=bytearray(36864)
            for at,value in ((0,11),(32,0o41777),(48,100),(56,100)):
                put(store,at,value)
            for slot in range(4):
                at=256+128*slot
                for offset,value in ((0,1),(8,7+slot),(16,8),(24,0o100640),
                    (32,100),(40,100),(48,100),(56,1)):
                    put(store,at+offset,value)
            return store
        def entry(store,slot,identity,name):
            put(store,64+slot*16,identity)
            put(store,72+slot*16,len(name))
            store[1536+256*slot:1536+256*(slot+1)]=name+bytes(256-len(name))
        def valid(store,descriptions=None):
            original=bytes(store)
            result=CHECKS.inspect(store,descriptions or {},24)
            self.assertEqual(bytes(store),original)
            return result
        for rotation in range(4):
            for length in (1,255):
                store=seed()
                for slot in range(4):
                    entry(store,slot,7+(slot+rotation)%4,bytes([128+slot])*length)
                objects,pages=valid(store)
                self.assertEqual(objects,{identity:8 for identity in range(7,11)})
                self.assertEqual(pages,set())
        store=seed()
        for slot in range(4): entry(store,slot,7+slot,bytes([65+slot]))
        original=bytes(store)
        for at,value in ((0,7),(8,1),(24,2),(32,0),(40,1),(48,99),
            (64,11),(72,0),(72,256),(128,1),(255,1),(256,2),(264,11),
            (272,1<<63),(280,0o40777),(288,0),(312,0),(312,4),(320,1),
            (328,1),(383,1),(768,1),(1016,1),(1024,7),(1032,1),(1040,1),
            (1280,1),(1528,1),(1536,0),(1544,1),(2560,1),(3072,1),(4088,1),
            (4096,1),(36856,1)):
            changed=bytearray(original)
            put(changed,at,value)
            with self.subTest(offset=at,value=value), self.assertRaises(AssertionError):
                valid(changed)
        # Distinct entries may share an inode, but must never duplicate a name.
        aliases=bytearray(original)
        for slot in range(4):
            put(aliases,256+slot*128+56,4 if slot==0 else 0)
            entry(aliases,slot,7,bytes([65+slot]))
        valid(aliases,{8:1,9:1,10:1})
        duplicate=bytearray(aliases)
        entry(duplicate,3,7,b'A')
        with self.assertRaises(AssertionError): valid(duplicate,{8:1,9:1,10:1})
        # Unlinked/initial-anonymous objects consume no name slots; live
        # descriptions retain them. Retired names and record tails stay zero.
        anonymous=bytearray(original)
        put(anonymous,64,0); put(anonymous,72,0)
        anonymous[1536:1792]=bytes(256)
        put(anonymous,312,0); put(anonymous,320,1)
        valid(anonymous,{7:1})
        with self.assertRaises(AssertionError): valid(anonymous)
        orphan=bytearray(anonymous)
        put(orphan,320,0)
        valid(orphan,{7:1})
        # Sparse payload ownership, exact final-page tail and map association.
        payload=bytearray(original)
        payload[24]=1; put(payload,1024,7)
        payload[4096:4104]=b'payload!'
        self.assertEqual(valid(payload)[1],{(7,0)})
        for at in (24,1024,4104):
            changed=bytearray(payload)
            changed[at]^=1
            with self.subTest(payload_offset=at), self.assertRaises(AssertionError):
                valid(changed)
        profile=dict(schema='topal.namespace-entry-table.x86_64/1',entry_offset=64,
            entry_count=4,entry_bytes=16,inode_link_count_offset=56,
            inode_initial_link_intent_offset=64)
        sha=hashlib.sha256(Path(CHECKS.__file__).read_bytes()).hexdigest()
        CHECKS.validate_profile(profile,sha)
        with self.assertRaises(AssertionError): CHECKS.validate_profile(profile,'0'*64)
        with self.assertRaises(AssertionError): CHECKS.validate_profile(dict(profile,entry_bytes=32),sha)


if __name__=='__main__': unittest.main()
