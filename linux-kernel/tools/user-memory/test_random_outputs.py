"""Frame-retirement counterexamples for the bounded random output checker.

TOPAL-SYSTEMS-USER-EXTENT-001. Synthetic CFGs are model evidence only.
"""
from collections import defaultdict
import struct
import unittest

from audit_random_outputs import extract_contract,random_iteration


def fixture(change=None):
    rows=[]
    labels={}
    fixups=[]
    body=bytearray()
    def emit(raw,label=None):
        if label: labels[label]=len(body)
        raw=bytes.fromhex(raw)
        rows.append(dict(offset=len(body),size=len(raw),bytes=raw.hex()))
        body.extend(raw)
    def branch(target):
        fixups.append((len(body),target))
        emit('0f8400000000')
    for raw in ['4885ff','4885f6','4839f7']:
        emit(raw)
        branch('fail')
    emit('4989fa')
    emit('4989f3')
    emit('4881ec80000000')
    emit('31c0',label='after-allocation')
    branch('fail-stack')
    emit('4885c0')
    branch('fail-stack')
    def wipe(label):
        emit('31c0',label=label)
        for offset in range(0,128,8):
            emit('48890424' if offset==0 else '48894424'+format(offset,'02x'),label=label+'-'+str(offset))
    wipe('success-wipe')
    emit('4881c480000000',label='success-release')
    emit('b801000000')
    emit('c3')
    wipe('fail-stack')
    emit('4881c480000000',label='failure-release')
    emit('31c0',label='fail')
    emit('c3')
    for at,target in fixups:
        struct.pack_into('<i',body,at+2,labels[target]-at-6)
    if change: change(body,labels,fixups)
    nodes={}
    for row in rows:
        at,size=row['offset'],row['size']
        nodes[at]=dict(row,bytes=body[at:at+size].hex())
    successors,predecessors=defaultdict(set),defaultdict(set)
    for at,row in nodes.items():
        raw=bytes.fromhex(row['bytes'])
        if raw==b'\xc3': continue
        successors[at].add(at+len(raw))
        if raw[:2]==bytes.fromhex('0f84'):
            successors[at].add(at+6+struct.unpack_from('<i',raw,2)[0])
    for at,targets in successors.items():
        for target in targets: predecessors[target].add(at)
    return bytes(body),nodes,{0},successors,predecessors,0


class RandomOutputControls(unittest.TestCase):
    def test_both_allocated_paths_clear_the_entire_scratch_before_release(self):
        report=extract_contract(*fixture())
        self.assertTrue(report['scratch_zero_before_every_release'])
        self.assertEqual(report['borrowed_stack_bytes_including_return'],136)
        self.assertEqual(len(report['releases']),2)

    def test_a_duplicate_zero_store_cannot_hide_an_uncleared_final_word(self):
        for label in ['success-wipe-120','fail-stack-120']:
            def change(body,labels,fixups): body[labels[label]+4]=112
            with self.subTest(label=label),self.assertRaisesRegex(AssertionError,'uncleared scratch'):
                extract_contract(*fixture(change))

    def test_a_coherent_branch_cannot_bypass_the_wipe(self):
        def change(body,labels,fixups):
            at=labels['after-allocation']+2
            struct.pack_into('<i',body,at+2,labels['success-release']-at-6)
        with self.assertRaisesRegex(AssertionError,'uncleared scratch'):
            extract_contract(*fixture(change))

    def test_zeroing_a_different_register_does_not_clear_scratch_stores(self):
        def change(body,labels,fixups):
            # Ensure no reaching EAX definition is zero on the success path.
            for label in ['after-allocation','success-wipe']:
                body[labels[label]+1]=0xc9
        with self.assertRaisesRegex(AssertionError,'uncleared scratch'):
            extract_contract(*fixture(change))

    def test_every_bounded_loop_index_is_inside_the_admitted_range(self):
        for length in range(1,17):
            completed=0
            while completed<length:
                state=random_iteration(length,completed)
                self.assertLess(state['user_offset'],length)
                completed=state['next_completed']
            self.assertEqual(completed,length)
        for length,completed in [(0,0),(17,0),(8,-1),(8,8)]:
            with self.subTest(length=length,completed=completed),self.assertRaises(AssertionError):
                random_iteration(length,completed)


if __name__=='__main__': unittest.main()
