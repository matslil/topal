"""Counterexamples for selector-bound warning admission and observation.

TOPAL-SYSTEMS-USER-EXTENT-001;
TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001.
"""
from pathlib import Path
import runpy
import unittest

from audit_warning_inputs import VARIANTS,warning_variant


class WarningInputControls(unittest.TestCase):
    def test_each_selector_requires_its_exact_quantum_including_larger_admissions(self):
        for state,length,accesses in VARIANTS:
            self.assertEqual(warning_variant(state,state,length),(length,accesses))
            for wrong in [0,length+1,(1<<64)-1]:
                with self.subTest(state=state,count=wrong),self.assertRaises(AssertionError):
                    warning_variant(state,state,wrong)

    def test_an_admitted_large_segment_cannot_authorize_a_different_selector(self):
        for before,_,_ in VARIANTS:
            for after,length,_ in VARIANTS:
                if before!=after:
                    with self.subTest(before=before,after=after),self.assertRaises(AssertionError):
                        warning_variant(before,after,length)

    def test_unknown_and_high_bit_selectors_fail_closed(self):
        for state in [0,0x03030101,0x100030101,(1<<64)-1]:
            with self.subTest(state=state),self.assertRaises(AssertionError):
                warning_variant(state,state,40)

    def test_staging_observer_rejects_wrong_bytes_and_out_of_order_loads(self):
        path=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_warning_observer.py'
        access=dict(offset=10,candidate_offset=0,width=4)
        variant=dict(selector=0x30101,access_instructions=[access])
        active=dict(admitted=True,variant=variant,candidate=0x1000,rsp=0x2000,
            selector=variant['selector'],captured=b'c'*120,size=4,payload=b'sh: ',accesses=[])
        ns=dict(Checked=type('Breakpoint',(),{}))
        exec(runpy.run_path(str(path))['PROGRAM'],ns)
        ns.update(warning_active=active,context=0x3000,
            reg=lambda name:dict(rsi=0x1000,rsp=0x2000)[name],
            word=lambda pointer:variant['selector'],
            memory=lambda pointer,size:b'c'*120 if pointer==0x3000+296 else b'sh: ')
        point=object.__new__(ns['WarningRead'])
        point.row,point.variant=access,variant
        self.assertFalse(point.observe())
        with self.assertRaises(AssertionError): point.observe()
        active['accesses']=[]
        ns['memory']=lambda pointer,size:b'c'*120 if pointer==0x3000+296 else b'bad!'
        with self.assertRaises(AssertionError): point.observe()


if __name__=='__main__': unittest.main()
