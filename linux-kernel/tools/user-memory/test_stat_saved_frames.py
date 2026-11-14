"""Saved-frame counterexamples using small host-side instruction/graph models.

TOPAL-SYSTEMS-USER-EXTENT-001. These models neither run native code nor
establish actual stack/metadata owners, exception effects or stable lifetime.
"""
from collections import defaultdict
import struct
import unittest

from audit_stat_outputs import closure,metadata_contract
from stat_instruction_effects import effects


class StatSavedFrameControls(unittest.TestCase):
    def fixture(self,extra=None,bypass=False):
        body = bytearray(128)
        nodes,fixups,labels = {},[],{}
        def emit(raw):
            at = len(body)
            raw = bytes.fromhex(raw)
            nodes[at] = dict(offset=at,size=len(raw),bytes=raw.hex())
            body.extend(raw)
        def label(name): labels[name] = len(body)
        def jump(op,target):
            at = len(body)
            emit(op+'00000000')
            fixups.append((at,len(bytes.fromhex(op)),target))
        for raw in ['51','52','56','57','4150','4151','4152','4153','4989f8']:
            emit(raw)
        if extra: emit(extra)
        emit('4881f800000000')
        jump('0f84','normal' if bypass else 'refused')
        emit('48b80100000000000000')
        jump('e9','restore')
        label('refused')
        emit('48b80000000000000000')
        label('restore')
        for raw in ['415b','415a','4159','4158','5f','5e','5a','59']: emit(raw)
        for offset in range(-64,0,8): emit('48c78424'+struct.pack('<i',offset).hex()+'00000000')
        for offset in range(-64,0,8):
            emit('4881bc24'+struct.pack('<i',offset).hex()+'00000000')
            jump('0f85','erasure-refused')
        label('normal')
        emit('c3')
        label('erasure-refused')
        emit('48b80000000000000000')
        emit('c3')
        for at,width,target in fixups:
            size = width+4
            struct.pack_into('<i',body,at+width,labels[target]-(at+size))
            nodes[at]['bytes'] = body[at:at+size].hex()
        successors,predecessors = defaultdict(set),defaultdict(set)
        for at,row in nodes.items():
            raw = bytes.fromhex(row['bytes'])
            effect = effects(raw)
            kind = effect.get('control')
            if kind == 'return': continue
            if kind in ['jump','conditional']:
                width = 1 if kind == 'jump' else 2
                successors[at].add(at+len(raw)+struct.unpack_from('<i',raw,width)[0])
            if kind != 'jump': successors[at].add(at+len(raw))
        summary = closure(128,body,nodes,successors)
        # An actual reachable-graph inventory omits unreachable diagnostic
        # instructions when a changed branch removes their only incoming path.
        active = {r['offset'] for r in summary['instructions']}
        nodes = {at:row for at,row in nodes.items() if at in active}
        for at in active:
            for target in successors[at]: predecessors[target].add(at)
        return summary,bytes(body),nodes,{128},successors,predecessors

    def test_every_normal_return_restores_eight_registers_and_retires_spills(self):
        contract = metadata_contract(*self.fixture())
        self.assertEqual(contract['saved_registers'],[1,2,6,7,8,9,10,11])
        self.assertEqual((contract['stack_bytes'],contract['spill_bytes']),(72,64))
        self.assertEqual(len(contract['restored_spill_erasure_offsets']),8)
        self.assertEqual(contract['normal_return_stack_delta'],8)
        self.assertNotIn(4,contract['preserved_registers'])

    def test_return_cannot_bypass_restoration_even_with_coherent_edges(self):
        with self.assertRaisesRegex(AssertionError,'return bypasses restoration'):
            metadata_contract(*self.fixture(bypass=True))

    def test_saved_candidate_word_cannot_be_written_in_the_body(self):
        # After eight pushes, saved R8 is at current RSP+24.
        with self.assertRaisesRegex(AssertionError,'body changes saved storage'):
            metadata_contract(*self.fixture(extra='48c784241800000000000000'))

    def test_body_cannot_adjust_stack_or_hide_nested_calls(self):
        for raw in ['4881c408000000','e800000000']:
            with self.subTest(raw=raw),self.assertRaises(AssertionError):
                metadata_contract(*self.fixture(extra=raw))

    def test_extra_push_cannot_change_the_saved_slot_layout(self):
        with self.assertRaises(AssertionError):
            metadata_contract(*self.fixture(extra='4150'))


if __name__ == '__main__':
    unittest.main()
