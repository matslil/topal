"""Saved byte-pool controls; synthetic memory is not native ownership evidence.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
"""
from copy import deepcopy
import hashlib
from pathlib import Path
import sys
from unittest import TestCase

from caller_retirement import BANKS,expand,retained,result_carry
from process_native import snapshot,PAGE

# The shared fixture imports a model from the sibling user-memory directory.
# Keep that import's search-path changes local to this module's setup so full
# discovery still resolves this suite's own test_model.py.
_search_path=sys.path[:]
try:
    from test_publication import fixture
    sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'labs/qemu/x86_64'))
    from process_mm_caller_retirement_observer import PROGRAM,boundary_program
finally:
    sys.path[:]=_search_path


def state():
    value=fixture(7,1,0)['before'];value['details']=[0]*4
    value['active']['bytes']=(bytes(range(256))*16).hex()
    root=value['context']['root'];low=value['context']['low']
    for table,index,target in [(root,0,root+PAGE),(root+PAGE,0,root+2*PAGE),
            (root+2*PAGE,2,low),(root+2*PAGE,3,root+4*PAGE)]:
        value['tables'][str(table)][index]=target|7
    snapshot(value);return value


def encoder(value):
    scope=dict(capture=lambda:deepcopy(value),model=dict(snapshot=snapshot),hashlib=hashlib,
        caller_data=dict(blobs={}))
    source='def caller_snapshot():'+PROGRAM.split('def caller_snapshot():',1)[1].split('def caller_stable',1)[0]
    exec(source,scope)
    return scope


class CallerHeapPoolTests(TestCase):
    def test_boundary_composition_keeps_callbacks_and_completion_obligations(self):
        source=boundary_program();compile(source,'<caller-boundary>','exec')
        self.assertNotIn('class Complete(Complete):',source)
        self.assertIn(PROGRAM.split('class Complete(Complete):')[0],source)
        self.assertIn('def install_caller_retirement():',source)

    def test_completion_refuses_pending_sample_missing_records_and_live_callbacks(self):
        from types import SimpleNamespace
        source='def complete_caller_retirement():'+PROGRAM.split(
            'def complete_caller_retirement():',1)[1].split('class Complete(Complete):',1)[0]
        for kind in ['complete','pending','missing','gateway','return']:
            scope=dict(caller_pending=None,caller_data=dict(records=[{}],complete=False),
                caller_gateways={'entry':SimpleNamespace(enabled=False)},
                caller_return_points={1:SimpleNamespace(enabled=False)})
            exec(source,scope)
            if kind=='pending':scope['caller_pending']={}
            elif kind=='missing':scope['caller_data']['records']=[]
            elif kind=='gateway':scope['caller_gateways']['entry'].enabled=True
            elif kind=='return':scope['caller_return_points'][1].enabled=True
            if kind=='complete':
                scope['complete_caller_retirement']();self.assertTrue(scope['caller_data']['complete'])
            else:
                with self.assertRaises(AssertionError):scope['complete_caller_retirement']()
                self.assertFalse(scope['caller_data']['complete'])

    def test_all_qualified_refusals_set_carry_and_only_zero_is_admitted(self):
        for status in range(5):
            result_carry(status,2|int(status!=0))
            with self.assertRaises(ValueError):result_carry(status,2|int(status==0))
        for status in [-1,5,1<<64,True]:
            with self.assertRaises(ValueError):result_carry(status,3)

    def test_roundtrip_preserves_complete_heap_owners_and_tables(self):
        value=state();scope=encoder(value);encoded=scope['caller_snapshot']();used=set()
        self.assertEqual(expand(encoded,scope['caller_data']['blobs'],used),value)
        self.assertEqual(used,set(scope['caller_data']['blobs']))
        self.assertEqual(len(used),2)
        self.assertTrue(all(isinstance(encoded[key]['bytes'],dict) for key in BANKS))

    def test_repeated_equal_capture_shares_bytes_without_caching_live_reads(self):
        value=state();scope=encoder(value)
        first=scope['caller_snapshot']();again=scope['caller_snapshot']()
        self.assertEqual(first,again);self.assertEqual(len(scope['caller_data']['blobs']),2)
        value['active']['bytes']='ff'+value['active']['bytes'][2:]
        later=scope['caller_snapshot']();self.assertEqual(len(scope['caller_data']['blobs']),3)
        self.assertNotEqual(first['active']['bytes'],later['active']['bytes'])
        self.assertNotEqual(expand(first,scope['caller_data']['blobs'],set())['active']['bytes'],
            expand(later,scope['caller_data']['blobs'],set())['active']['bytes'])

    def test_missing_wrong_width_or_wrong_digest_is_refused(self):
        for kind in ['missing','width','digest','boolean-width']:
            with self.subTest(kind=kind):
                scope=encoder(state());encoded=scope['caller_snapshot']();blobs=scope['caller_data']['blobs']
                ref=encoded['active']['bytes']
                if kind=='missing':blobs.pop(ref['sha256'])
                elif kind=='width':ref['bytes']+=1
                elif kind=='boolean-width':ref['bytes']=True
                else:blobs[ref['sha256']]='00'*ref['bytes']
                with self.assertRaises(ValueError):expand(encoded,blobs,set())

    def test_fresh_bytes_do_not_hide_a_changed_owner_or_epoch(self):
        value=state();scope=encoder(value);encoded=scope['caller_snapshot']()
        before=expand(encoded,scope['caller_data']['blobs'],set())
        for field in ['ids','epoch','cr0','tables','active']:
            after=deepcopy(before)
            if field in ['ids','epoch']:after[field][0]+=1
            elif field=='cr0':after[field]^=1<<16
            elif field=='tables':after[field][str(after['context']['root'])][0]^=4
            else:after['active']['bytes']='ff'+after['active']['bytes'][2:]
            with self.subTest(field=field):
                with self.assertRaises(ValueError):retained(before,after)

    def test_monotonic_hardware_ad_bits_are_distinct_from_permission_changes(self):
        scope=encoder(state());encoded=scope['caller_snapshot']()
        before=expand(encoded,scope['caller_data']['blobs'],set());after=deepcopy(before)
        table=str(before['context']['root']);after['tables'][table][0]|=0x20
        retained(before,after)
        with self.assertRaises(ValueError):retained(after,before)
