"""Counterexamples for retired layout, entry closure and read-only observations.

TOPAL-SYSTEMS-USER-EXTENT-001. Synthetic evidence grants no native owners.
"""
from collections import defaultdict
from pathlib import Path
import runpy
import struct
import unittest

from audit_stack_retirement import closed_block


def fixture(words=4,change=None):
    # Encode independently of the checker's encoding constructor.
    chunks=[bytes.fromhex('e800000000')]+[
        bytes.fromhex('48c78424')+struct.pack('<i',-8*i)+bytes(4)
        for i in range(1,words+1)]+[b'\xc3']
    body=bytearray(b''.join(chunks))
    if change: change(body)
    nodes={}
    at=0
    for chunk in chunks:
        nodes[at]=dict(offset=at,size=len(chunk),bytes=body[at:at+len(chunk)].hex())
        at+=len(chunk)
    predecessors=defaultdict(set)
    last=0
    for at in list(nodes)[1:]:
        predecessors[at]={last}
        last=at
    return bytes(body),nodes,{0},predecessors,5,words,0


class RetiredLayoutControls(unittest.TestCase):
    def test_return_and_both_saved_register_layouts_are_closed(self):
        for words in [1,4,12]:
            report=closed_block(*fixture(words))
            self.assertEqual((report['relative_start'],report['relative_end']),(-8*words,0))
            self.assertEqual(report['next_instruction']['bytes'],'c3')

    def test_duplicate_or_live_slot_is_rejected_even_with_coherent_graph_bytes(self):
        for offset in [-8,0,8,-40]:
            def change(body): struct.pack_into('<i',body,5+3*12+4,offset)
            with self.subTest(offset=offset),self.assertRaises(AssertionError): closed_block(*fixture(change=change))

    def test_wrong_base_and_nonzero_store_are_rejected(self):
        for offset,value in [(5+3,0x25),(5+8,1),(5+1,0xc6)]:
            def change(body): body[offset]=value
            with self.subTest(offset=offset),self.assertRaises(AssertionError): closed_block(*fixture(change=change))

    def test_midblock_entry_and_cleanup_bypass_are_rejected(self):
        for at in [5,17,53]:
            args=list(fixture())
            args[3][at].add(12345)
            with self.subTest(at=at),self.assertRaises(AssertionError): closed_block(*args)

    def test_independent_root_cannot_skip_stores(self):
        args=list(fixture())
        args[2].add(17)
        with self.assertRaises(AssertionError): closed_block(*args)

    def test_graph_instruction_cannot_disagree_with_actual_body(self):
        args=list(fixture())
        body=bytearray(args[0])
        body[13]=1
        args[0]=bytes(body)
        with self.assertRaises(AssertionError): closed_block(*args)


class RetiredObservationControls(unittest.TestCase):
    def fixture(self):
        path=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_stack_observer.py'
        class Breakpoint:
            def __init__(self,*args,**kwargs): pass
        ns=dict(Checked=Breakpoint)
        exec(runpy.run_path(str(path))['PROGRAM'],ns)
        registers={name:32+i for i,name in enumerate(ns['stack_retirement_registers'])}
        registers.update(rsp=256,eflags=0x43,cr0=1<<16,cs=16)
        data=bytearray([0xa5]*512)
        ns.update(reg=lambda name:registers[name],memory=lambda at,size:bytes(data[at:at+size]),
            extent_inventory=dict(address=0,stack_retirement_audit=dict(blocks=[]),stack_retirement_audit_sha256='model'),observed={})
        row=dict(name='model',start=5,end=53,bytes=32,words=4)
        ns['extent_inventory']['stack_retirement_audit']['blocks']=[row]
        start,end=ns['StackRetirementStart'](row),ns['StackRetirementEnd'](row)
        start.observe()
        data[224:256]=bytes(32)
        return ns,registers,data,start,end

    def test_zeroed_retired_bytes_and_live_guard_preservation_are_observed(self):
        ns,registers,data,start,end=self.fixture()
        end.observe()
        ns['stack_retirement_observation_complete']()
        report=ns['observed']['stack_retirement_observation']
        self.assertEqual(report['unreached'],[])
        self.assertEqual(report['records'][0]['zero_bytes_observed'],32)

    def test_each_gpr_and_carry_result_is_preserved(self):
        for name in ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi',
                     'r8','r9','r10','r11','r12','r13','r14','r15','eflags']:
            ns,registers,data,start,end=self.fixture()
            registers[name]^=1
            with self.subTest(name=name),self.assertRaises(AssertionError): end.observe()

    def test_incomplete_zeroing_and_adjacent_word_damage_are_rejected(self):
        for offset in [224,255,223,256]:
            ns,registers,data,start,end=self.fixture()
            data[offset]^=1
            with self.subTest(offset=offset),self.assertRaises(AssertionError): end.observe()

    def test_missing_end_and_overlapping_activation_are_rejected(self):
        ns,registers,data,start,end=self.fixture()
        with self.assertRaises(AssertionError): ns['stack_retirement_observation_complete']()
        with self.assertRaises(AssertionError): start.observe()

    def test_ending_another_block_cannot_consume_the_active_observation(self):
        ns,registers,data,start,end=self.fixture()
        end.row=dict(end.row,name='other')
        with self.assertRaises(AssertionError): end.observe()


if __name__=='__main__': unittest.main()
