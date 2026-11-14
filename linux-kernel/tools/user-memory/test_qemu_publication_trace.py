"""Independent saved-text controls for the installed QEMU CPU log shape."""
import unittest

from qemu_publication_trace import parse,completed


SITE=0x2b11703
def block(index=0,cpu=0):
    return (f'Trace {cpu}: 0x1234 [00000000/{SITE:016x}/0040c0b0/ff068201] \n'
        f'RAX={0x5f7000+index*4096:016x} RCX={index:016x}\n'
        f'RIP={SITE:016x} RFL=00000097 CPL=0\n'
        'CS =0010 0000000000000000 ffffffff 00a09b00\n'
        'CR0=80010011 CR3=0000000002e20000 CR4=00000620\n'
        'EFER=0000000000000d01\n').encode()


class QemuPublicationTraceTests(unittest.TestCase):
    def test_exact_cpu_register_observation(self):
        row,=parse(block(),SITE)
        self.assertEqual(row,dict(cpu=0,index=0,address=0x5f7000,cr0=0x80010011,
            cr3=0x2e20000,cr4=0x620,efer=0xd01,flags=0x97,cancelled=False))
        link=b'Linking TBs 0x1234 index 0 -> 0x5678\n'
        self.assertEqual(parse(link,SITE),[])
        self.assertEqual(parse(link+block()+link,SITE),[row])

    def test_multiple_iterations_and_cpu_identity(self):
        rows=parse(block(0,1)+block(1,1)+block(2,1),SITE)
        self.assertEqual([(r['cpu'],r['index'],r['address']) for r in rows],
            [(1,i,0x5f7000+i*4096) for i in range(3)])

    def test_wrong_site_user_privilege_or_missing_duplicate_register(self):
        for raw in [block().replace(b'RIP=0000000002b11703',b'RIP=0000000002b11700'),
                block().replace(b'CS =0010',b'CS =0023'),
                block().replace(b'CR3=0000000002e20000',b''),
                block()+b'CR3=0000000002e20000\n']:
            with self.assertRaises(AssertionError):parse(raw,SITE)
        with self.assertRaises(AssertionError):parse(block(),SITE+1)

    def test_empty_and_truncated_or_unparsed_text(self):
        self.assertEqual(parse(b'',SITE),[])
        for raw in [b'not a CPU trace',block().split(b'EFER=')[0],block()+block().split(b'RIP=')[0]]:
            with self.assertRaises(AssertionError):parse(raw,SITE)

    def test_explicit_cancelled_attempt_requires_matching_successful_retry(self):
        stopped=b'Stopped execution of TB chain before 0x1234 [0000000002b11703] \n'
        raw=block()+stopped+block()
        states,cancelled=completed(parse(raw,SITE))
        self.assertEqual(len(states),1)
        self.assertEqual(len(cancelled),1)
        self.assertTrue(cancelled[0]['cancelled'])
        for raw in [block()+stopped,block()+stopped+block(1),
                block()+stopped.replace(b'0x1234',b'0x5678')+block()]:
            with self.assertRaises(AssertionError):completed(parse(raw,SITE))
        # Unmarked duplicates remain present for the publication model to reject.
        self.assertEqual(len(completed(parse(block()+block(),SITE))[0]),2)


if __name__=='__main__':unittest.main()
