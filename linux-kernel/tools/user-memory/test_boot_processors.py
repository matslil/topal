"""ACPI format counterexamples; these are models, not injected guest states."""
import struct
import unittest
from boot_processors import discover,processors,retained_range

BASE=0x10000000
ROOT=BASE
MADT=BASE+0x1000
OTHER=BASE+0x3000


def checksum(data,offset=9):
    result=bytearray(data)
    result[offset]=0
    result[offset]=(-sum(result))&255
    return bytes(result)


def table(signature,payload=b'',revision=1):
    result=bytearray(36)+payload
    result[:4]=signature
    struct.pack_into('<I',result,4,len(result))
    result[8]=revision
    return checksum(result)


def cpu(uid,identity,flags=1,extended=False):
    if extended: return struct.pack('<BBHIII',9,16,0,identity,flags,uid)
    return struct.pack('<BBBBI',0,8,uid,identity,flags)


def madt(rows):
    return table(b'APIC',struct.pack('<II',0xfee00000,1)+b''.join(rows),revision=7)


class Firmware:
    def __init__(self,extended=False,ebda=False):
        self.claims=[[BASE,BASE+0x20000,3],[0x9fc00,0xa0000,2]]
        self.memory={1038:struct.pack('<H',0x9fc0 if ebda else 0),
                     0x9fc00:bytes(1024),0xe0000:bytes(0x20000)}
        self.extended=extended
        self.rsdp=0x9fc00 if ebda else 0xe0010
        self.memory[ROOT]=table(b'XSDT' if extended else b'RSDT',
                                struct.pack('<QQ' if extended else '<II',OTHER,MADT))
        self.memory[OTHER]=table(b'FACP')
        self.memory[MADT]=madt([cpu(0,0),cpu(1,1)])
        r=bytearray(36 if extended else 20)
        r[:8]=b'RSD PTR '
        r[15]=2 if extended else 0
        struct.pack_into('<I',r,16,ROOT)
        r[:20]=checksum(r[:20],8)
        if extended:
            struct.pack_into('<I',r,20,36)
            struct.pack_into('<Q',r,24,ROOT)
            r=bytearray(checksum(r,32))
        self.set_rsdp(bytes(r))

    def set_rsdp(self,data):
        base=0x9fc00 if self.rsdp<0xe0000 else 0xe0000
        window=bytearray(self.memory[base])
        at=self.rsdp-base
        window[at:at+len(data)]=data
        self.memory[base]=bytes(window)

    def read(self,base,length):
        found=[data[base-start:base-start+length] for start,data in self.memory.items()
               if start<=base and base+length<=start+len(data)]
        if not found: raise ValueError('uncaptured firmware range')
        return found[0]

    def discover(self): return discover(self.read,self.claims,0)


class BootProcessorInventory(unittest.TestCase):
    def test_legacy_discovery_retains_complete_advertised_inventory(self):
        f=Firmware()
        f.memory[MADT]=madt([cpu(0,0),cpu(1,1,2),cpu(0,0,0),cpu(1000,5,1,True)])
        r=f.discover()
        self.assertEqual(r['participants'],[[0,0,0,1],[0,1,1,2],[9,1000,5,1]])
        self.assertEqual(r['registers'][12:14],[3,2])
        self.assertEqual(r['registers'][14],len(r['retained']))
        self.assertFalse(any(r['registers'][44:]))
        self.assertEqual(r['registers'][0],1)

    def test_extended_root_checks_both_checksums_and_exact_format(self):
        f=Firmware(extended=True)
        self.assertEqual(f.discover()['registers'][2:7],[2,36,ROOT,52,8])
        original=f.read(f.rsdp,36)
        for index in [8,32,33]:
            changed=bytearray(original); changed[index]^=1
            f.set_rsdp(bytes(changed))
            with self.subTest(index=index),self.assertRaises(ValueError): f.discover()
            f.set_rsdp(original)
        changed=bytearray(original); struct.pack_into('<Q',changed,24,0)
        f.set_rsdp(checksum(changed,32))
        with self.assertRaises(ValueError): f.discover()

    def test_ebda_priority_and_bad_signature_checksum_search(self):
        f=Firmware(ebda=True)
        self.assertEqual(f.discover()['registers'][1],0x9fc00)
        f.memory[0x9fc00]=bytes(1024)
        good=Firmware()
        f.memory[0xe0000]=good.memory[0xe0000]
        self.assertEqual(f.discover()['registers'][1],0xe0010)
        bad=bytearray(f.memory[0xe0000]); bad[:8]=b'RSD PTR '
        bad[8]=1
        f.memory[0xe0000]=bytes(bad)
        self.assertEqual(f.discover()['registers'][1],0xe0010)

    def test_relevant_identity_flags_and_bootstrap_binding_are_exact(self):
        for rows in [[cpu(0,0),cpu(0,1)],[cpu(0,0),cpu(1,0,2)],
                     [cpu(0,0),cpu(1,1,3)],[cpu(0,0,2)],[],[cpu(1,1)]]:
            with self.subTest(rows=rows),self.assertRaises(ValueError): processors(madt(rows),0)
        bad=bytearray(cpu(1,1,1,True)); bad[2]=1
        with self.assertRaises(ValueError): processors(madt([cpu(0,0),bytes(bad)]),0)
        for flags in [0,1,2]:
            rows,count=processors(madt([cpu(0,0),cpu(1,1,flags)]),0)
            self.assertEqual(len(rows),1+bool(flags))
            self.assertEqual(count,1+(flags==1))

    def test_record_walk_refuses_truncation_unknown_kinds_and_corrected_checksums(self):
        for row in [b'\x00',b'\x00\x00',b'\x00\xff',b'\x07\x02',b'\x0b\x02',
                    b'\x10\x02',b'\x18\x02',b'\x01\x08'+bytes(6)]:
            with self.subTest(row=row),self.assertRaises(ValueError): processors(madt([cpu(0,0),row]),0)
        data=bytearray(madt([cpu(0,0),cpu(1,1)])); data[-1]^=1
        with self.assertRaises(ValueError): processors(bytes(data),0)
        with self.assertRaises(ValueError): processors(madt([cpu(i,i) for i in range(17)]),0)

    def test_firmware_provenance_bounds_and_conflicts(self):
        f=Firmware(); f.discover()
        for claims in [[[BASE,BASE+0x20000,1]],[[BASE,BASE+0x100,3]],
                       f.claims+[[ROOT+16,ROOT+20,1]],f.claims+[[ROOT+16,ROOT+20,5]],
                       f.claims+[[1,(1<<64)+1,2]],f.claims*65]:
            with self.subTest(claims=claims),self.assertRaises(ValueError):
                discover(f.read,claims,0)
        for base,length in [(0,36),(1<<30,36),(BASE,0),(BASE,(1<<64)-BASE)]:
            with self.subTest(base=base,length=length),self.assertRaises(ValueError):
                retained_range(f.claims,base,length)

    def test_root_cardinality_and_complete_madt_checks(self):
        for pointers in [[],[OTHER],[MADT,MADT],[MADT]*65]:
            f=Firmware()
            f.memory[ROOT]=table(b'RSDT',b''.join(struct.pack('<I',p) for p in pointers))
            with self.subTest(pointers=len(pointers)),self.assertRaises(ValueError): f.discover()
        f=Firmware(); changed=bytearray(f.memory[MADT]); changed[9]^=1
        f.memory[MADT]=bytes(changed)
        with self.assertRaises(ValueError): f.discover()
        f=Firmware(); f.memory[MADT]=table(b'APIC',bytes(4096))
        with self.assertRaises(ValueError): f.discover()

    def test_discovery_refuses_inconsistent_retained_snapshots(self):
        # Model changing firmware views; no guest bytes or instructions change.
        f=Firmware()
        altered=bytearray(f.memory[MADT]); altered[54]=2
        altered=checksum(altered)
        def changed_madt(base,length):
            return altered if base==MADT and length==len(altered) else f.read(base,length)
        with self.assertRaisesRegex(ValueError,'header copies'):
            discover(changed_madt,f.claims,0)
        altered_root=bytearray(f.memory[ROOT])
        struct.pack_into('<I',altered_root,4,len(altered_root)+4)
        altered_root=checksum(altered_root)
        def changed_root(base,length):
            return altered_root if base==ROOT and length==len(altered_root) else f.read(base,length)
        with self.assertRaisesRegex(ValueError,'retained SDT length'):
            discover(changed_root,f.claims,0)
        reads=0
        original=f.read(f.rsdp,20)
        def changed_rsdp(base,length):
            nonlocal reads
            if base==f.rsdp and length==20:
                reads+=1
                if reads==2:
                    data=bytearray(original); data[15]=1
                    return checksum(data,8)
            return f.read(base,length)
        with self.assertRaisesRegex(ValueError,'retained RSDP'):
            discover(changed_rsdp,f.claims,0)


if __name__=='__main__': unittest.main()
