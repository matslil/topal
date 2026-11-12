"""Counterexamples for the saved paging-control inventory.

TOPAL-SYSTEMS-USER-EXTENT-001. These are reader tests, not guest execution.
"""
import unittest

from inventory_paging_controls import candidates,classify,decode,join_controls


class PagingControls(unittest.TestCase):
    def test_control_register_encoding_and_rex(self):
        for raw,text,kind in [
            ('0f22d8','mov cr3, rax','control-register-write'),
            ('410f22d8','mov cr3, r8','control-register-write'),
            ('440f22c0','mov cr8, rax','control-register-write'),
            ('410f20c7','mov r15, cr0','control-register-read'),
        ]:
            self.assertEqual(classify(bytes.fromhex(raw),text),kind)
        for raw,text in [('0f22d8','mov rax, cr3'),('0f22d8','mov cr4, rax'),
                         ('440f22c0','mov cr0, rax'),('0f2218','mov cr3, rax')]:
            with self.assertRaises(AssertionError):classify(bytes.fromhex(raw),text)

    def test_invalidation_memory_group_is_not_swapgs(self):
        self.assertEqual(classify(bytes.fromhex('410f0138'),'invlpg byte ptr [r8]'),'page-invalidation')
        self.assertEqual(classify(bytes.fromhex('0f01f8'),'swapgs'),'gs-base-swap')
        with self.assertRaises(AssertionError):classify(bytes.fromhex('0f0138'),'nop')
        with self.assertRaises(AssertionError):classify(bytes.fromhex('0f01f8'),'invlpg byte ptr [rax]')

    def test_msr_direction_cannot_be_relabelled(self):
        self.assertEqual(classify(bytes.fromhex('0f30'),'wrmsr'),'msr-write')
        self.assertEqual(classify(bytes.fromhex('0f32'),'rdmsr'),'msr-read')
        for raw,text in [('0f30','rdmsr'),('0f32','wrmsr'),('90','wrmsr')]:
            with self.assertRaises(AssertionError):classify(bytes.fromhex(raw),text)

    def test_missing_middle_or_tail_cannot_establish_coverage(self):
        body=bytes.fromhex('900f22d890')
        listing='100: 90 nop\n101: 0f 22 d8 mov cr3, rax\n104: 90 nop\n'
        self.assertEqual(len(decode(listing,0x100,body)),3)
        for bad in [listing.replace('100: 90 nop\n',''),
                    listing.replace('101: 0f 22 d8 mov cr3, rax\n',''),
                    listing.replace('104: 90 nop\n','')]:
            with self.assertRaises(AssertionError):decode(bad,0x100,body)

    def test_linked_bytes_and_unique_row_starts(self):
        body=bytes.fromhex('0f22d8')
        for text in ['100: 0f 22 e0 mov cr4, rax\n',
                     '100: 0f 22 d8 mov cr3, rax\n100: 0f 22 d8 mov cr3, rax\n',
                     'ff: 90 nop\n100: 0f 22 d8 mov cr3, rax\n']:
            with self.assertRaises(AssertionError):decode(text,0x100,body)

    def test_separate_lock_prefix_retains_coverage(self):
        body=bytes.fromhex('f0830c2400')
        llvm=decode('100: f0 lock\n101: 83 0c 24 00 or dword ptr [rsp], 0x0\n',0x100,body)
        gnu=decode('100: f0 83 0c 24 00 lock or DWORD PTR [rsp],0x0\n',0x100,body)
        self.assertEqual(len(llvm),2)
        self.assertEqual(len(gnu),1)
        self.assertFalse(any(row['kind'] for row in llvm.values()))
        self.assertEqual(candidates(0x100,body,llvm),[])
        rex=decode('100: 49 rex.WB\n',0x100,bytes.fromhex('49'))
        self.assertIsNone(rex[0x100]['kind'])

    def test_padding_overlap_retains_both_interpretations(self):
        body=bytes.fromhex('005053')
        listing='100: 00 50 53 add byte ptr [rax+0x53],dl\n101: 50 push rax\n102: 53 push rbx\n'
        rows=decode(listing,0x100,body)
        self.assertEqual(len(rows),3)
        self.assertEqual(sum(row['size'] for row in rows.values()),5)

    def test_opcode_embedded_in_immediate_remains_a_candidate(self):
        body=bytes.fromhex('48b80f22d80000000000')
        rows=decode('100: 48 b8 0f 22 d8 00 00 00 00 00 movabs rax, 0xd8220f\n',0x100,body)
        retained=candidates(0x100,body,rows)
        self.assertEqual(len(retained),1)
        self.assertEqual(retained[0]['address'],0x102)
        self.assertFalse(retained[0]['at_listed_opcode'])
        self.assertEqual(retained[0]['containing_listing_rows'],[0x100])
        normal=bytes.fromhex('410f22d8')
        rows=decode('100: 41 0f 22 d8 mov cr3,r8\n',0x100,normal)
        self.assertTrue(candidates(0x100,normal,rows)[0]['at_listed_opcode'])

    def test_decoder_disagreement_is_preserved(self):
        first=decode('100: 00 00 add byte ptr [rax],al\n102: cf iretd\n',0x100,bytes.fromhex('0000cf'))
        second=decode('100: 00 00 add byte ptr [rax],al\n102: cf .byte 0xcf\n',0x100,bytes.fromhex('0000cf'))
        selected,disagreements=join_controls(first,second)
        self.assertEqual(disagreements,[0x102])
        self.assertEqual(selected[0x102]['gnu_instruction'],'.byte 0xcf')
        self.assertFalse(selected[0x102]['decoder_agreement'])
        matched,disagreements=join_controls(first,first)
        self.assertEqual(disagreements,[])
        self.assertTrue(matched[0x102]['decoder_agreement'])


if __name__=='__main__':unittest.main()
