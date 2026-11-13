"""Counterexamples for conditional saved-code edges; no guest execution.

TOPAL-SYSTEMS-USER-EXTENT-001. These tests grant no entry or memory authority.
"""
import struct
from types import SimpleNamespace
from unittest import TestCase

from callee_control import Unresolved,guarded_table,matched,merge_window,no_bypass,previous,transfer,window_rows
from inspect_callee_graph import payload


def row(address,raw,instruction):
    raw=bytes.fromhex(raw)
    return dict(address=address,bytes=raw.hex(),size=len(raw),instruction=instruction)


def chain(parts,start=0x100):
    result={}
    for raw,instruction in parts:
        result[start]=row(start,raw,instruction);start+=result[start]['size']
    return result


def rdi_table():
    rows=chain([('4883ff02','cmp rdi,0x2'),('771a','ja 0x120'),
        ('ff24fd00020000','jmp qword ptr [rdi*8+0x200]')])
    return rows,0x106


def saved_table():
    rows=chain([('89c1','mov ecx,eax'),('48894df8','mov qword ptr [rbp-0x8],rcx'),
        ('83e807','sub eax,0x7'),('7735','ja 0x140'),('488b45f8','mov rax,qword ptr [rbp-0x8]'),
        ('488b04c500020000','mov rax,qword ptr [rax*8+0x200]'),('ffe0','jmp rax')])
    return rows,0x117


class ConditionalCalleeControls(TestCase):
    def table(self,rows,at):
        def read(base,size):
            self.assertEqual(base,0x200);self.assertEqual(size%8,0)
            return struct.pack('<'+'Q'*(size//8),*[0x180+8*i for i in range(size//8)])
        return guarded_table(rows,at,read)

    def test_direct_destination_matches_native_displacement_in_both_syntaxes(self):
        for text in ['call 0x10a','call 10a <target>']:
            self.assertEqual(transfer(row(0x100,'e805000000',text)),0x10a)
        for text in ['call 0x10b','jmp 0x10a','call rax']:
            with self.assertRaises(ValueError):transfer(row(0x100,'e805000000',text))

    def test_rex_indirect_jump_is_distinct_from_call_or_direct_target(self):
        raw='42ff24f500020000'
        self.assertIsNone(transfer(row(0x100,raw,'jmp qword ptr [r14*8+0x200]')))
        for text in ['call qword ptr [r14*8+0x200]','jmp 0x200']:
            with self.assertRaises(ValueError):transfer(row(0x100,raw,text))
        with self.assertRaises(Unresolved):transfer(row(0x100,'ff18','call fword ptr [rax]'))

    def test_short_conditional_and_unconditional_edges_cannot_be_interchanged(self):
        for raw,right,wrong in [('771a','ja','jmp'),('eb1a','jmp','jne'),
            ('e21a','loop','jmp'),('e31a','jrcxz','jmp')]:
            self.assertEqual(transfer(row(0x100,raw,right+' 0x11c')),0x11c)
            with self.assertRaises(ValueError):transfer(row(0x100,raw,wrong+' 0x11c'))

    def test_prefix_fragments_require_exact_complete_combined_bytes(self):
        llvm=chain([('f0','lock'),('48','rex.W'),('830c2400','or qword ptr [rsp],0x0')])
        gnu={0x100:row(0x100,'f048830c2400','lock or qword ptr [rsp],0x0')}
        result=matched(0x100,llvm,gnu)
        self.assertEqual(result['llvm_fragments'],[0x100,0x101,0x102])
        self.assertEqual(result['mnemonic'],'or')
        missing=dict(llvm);missing.pop(0x102)
        with self.assertRaises(Unresolved):matched(0x100,missing,gnu)
        wrong=dict(llvm);wrong[0x102]=row(0x102,'830c2401','or qword ptr [rsp],0x1')
        with self.assertRaises(ValueError):matched(0x100,wrong,gnu)

    def test_equal_bytes_cannot_hide_decoder_control_disagreement(self):
        llvm={0x100:row(0x100,'c3','ret')};gnu={0x100:row(0x100,'c3','nop')}
        with self.assertRaises(ValueError):matched(0x100,llvm,gnu)
        # Ordinary operation aliases are not a full instruction-effect proof.
        llvm={0x100:row(0x100,'6690','nop')};gnu={0x100:row(0x100,'6690','xchg ax,ax')}
        self.assertEqual(matched(0x100,llvm,gnu)['size'],2)

    def test_llvm_direct_target_text_cannot_disagree_with_the_encoded_destination(self):
        llvm={0x100:row(0x100,'e805000000','call 0x10b')}
        gnu={0x100:row(0x100,'e805000000','call 10a <target>')}
        with self.assertRaises(ValueError):matched(0x100,llvm,gnu)

    def test_register_guard_reads_the_whole_bounded_table(self):
        rows,at=rdi_table();value=self.table(rows,at)
        self.assertEqual((value['table'],value['maximum'],value['index']),(0x200,2,'rdi'))
        self.assertEqual(value['targets'],[0x180,0x188,0x190])
        self.assertEqual(value['default'],0x120)

    def test_r14_guard_matches_the_extended_index_encoding(self):
        rows=chain([('4983fe02','cmp r14,0x2'),('7719','ja 0x11f'),
            ('42ff24f500020000','jmp qword ptr [r14*8+0x200]')])
        self.assertEqual(self.table(rows,0x106)['index'],'r14')
        rows[0x100]=row(0x100,'4983ff02','cmp r15,0x2')
        with self.assertRaises(Unresolved):self.table(rows,0x106)

    def test_guard_condition_must_match_exact_ja_encoding_and_both_decoders(self):
        for raw,text,llvm in [('721a','ja 0x120','ja'),('771a','jb 0x120','ja'),
                              ('771a','ja 0x120','jb')]:
            rows,at=rdi_table();rows[0x104]=dict(row(0x104,raw,text),llvm_mnemonic=llvm)
            with self.assertRaises(ValueError):self.table(rows,at)

    def test_sign_extended_negative_bound_is_not_a_small_positive_index(self):
        rows,at=rdi_table();rows[0x100]=row(0x100,'4883ffff','cmp rdi,0xffffffffffffffff')
        with self.assertRaises(ValueError):self.table(rows,at)

    def test_saved_eax_preserves_zero_extension_and_same_negative_stack_slot(self):
        rows,at=saved_table();value=self.table(rows,at)
        self.assertEqual((value['index'],value['maximum'],value['stack_slot']),
            ('zero-extended saved eax',7,-8))
        for at_changed,raw,text in [(0x100,'88c1','mov cl,al'),
            (0x10b,'488b45f0','mov rax,qword ptr [rbp-0x10]')]:
            altered=dict(rows);altered[at_changed]=row(at_changed,raw,text)
            with self.assertRaises(Unresolved):self.table(altered,at)
        altered=dict(rows);altered[0x102]=row(0x102,'48894d08','mov [rbp+8],rcx')
        altered[0x10b]=row(0x10b,'488b4508','mov rax,[rbp+8]')
        with self.assertRaises(ValueError):self.table(altered,at)

    def test_missing_or_ambiguous_predecessors_cannot_supply_a_guard(self):
        rows,at=rdi_table();missing=dict(rows);missing.pop(0x100)
        with self.assertRaises(Unresolved):previous(missing,at,2)
        rows[0x102]=row(0x102,'ff02','inc dword ptr [rdx]')
        with self.assertRaises(Unresolved):previous(rows,at,2)

    def test_declared_entries_and_transfers_cannot_bypass_the_guard(self):
        rows,at=rdi_table();value=self.table(rows,at)
        edges=[dict(source=0x100,target=0x104,kind='fallthrough'),
            dict(source=0x104,target=0x106,kind='fallthrough'),
            dict(source=0x104,target=0x120,kind='direct-transfer')]
        no_bypass(value,[0x100],edges)
        for target in [0x104,0x106]:
            with self.assertRaises(ValueError):no_bypass(value,[target],edges)
            with self.assertRaises(ValueError):no_bypass(value,[],edges+[
                dict(source=0x80,target=target,kind='direct-transfer')])
        with self.assertRaises(ValueError):no_bypass(value,[],edges+[
            dict(source=0x100,target=0x106,kind='call-return-assumption')])

    def test_table_payload_needs_one_complete_nonwritable_allocated_section(self):
        def elf(flags):
            section=(0,1,flags,0x200,0,24,0,0,8,0)
            return SimpleNamespace(sections=[section],section_bytes=lambda _:bytes(24))
        self.assertEqual(payload(elf(2),0x200,24,True),bytes(24))
        for flags in [0,3,6]:
            with self.assertRaises(ValueError):payload(elf(flags),0x200,24,True)
        for address,size in [(0x1ff,8),(0x200,25),(0x200,0),(1<<64,8)]:
            with self.assertRaises(ValueError):payload(elf(2),address,size,True)

    def test_recovery_window_rejects_missing_or_unbound_bytes(self):
        text='100: f0 lock\n101: 83 0c 24 00 or dword ptr [rsp],0\n'
        raw=bytes.fromhex('f0830c2400')
        self.assertEqual(set(window_rows(text,0x100,raw,0x100,0x105)),{0x100,0x101})
        for changed in [text.replace('100: f0 lock\n',''),text.replace('83 0c 24 00','83 0c 24 01'),
            text+'100: f0 lock\n']:
            with self.assertRaises(ValueError):window_rows(changed,0x100,raw,0x100,0x105)

    def test_stop_truncated_prefix_does_not_replace_a_complete_tail_instruction(self):
        complete=row(0x17e,'654c8b1c2530010000','mov r11,qword ptr gs:[0x130]')
        rows={0x17e:complete}
        recovered={0x100:row(0x100,'90','nop'),0x17e:row(0x17e,'65','gs')}
        self.assertEqual(merge_window(rows,recovered,0x100,0x180,0x200),0x171)
        self.assertEqual(rows[0x17e],complete);self.assertIn(0x100,rows)
        # Actual section-end bytes have no truncated instruction after them.
        end={0x17f:row(0x17f,'c3','ret')}
        self.assertEqual(merge_window(rows,end,0x100,0x180,0x180),0x180)
        self.assertEqual(rows[0x17f]['bytes'],'c3')
