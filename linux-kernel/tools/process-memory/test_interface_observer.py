"""Host-only controls for MM observer transport and coherent checkpoints.

TOPAL-SYSTEMS-PROCESS-MM-001: these checks model breakpoint scheduling and
record publication, not native register/memory observation or kernel authority.
"""
import json
from pathlib import Path
import sys
import tempfile
import unittest

LAB=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64'
sys.path.insert(0,str(LAB))
from process_mm_regression_observer import PROGRAM

TRANSPORT=r'''
observed=dict(passed=False,events=[],cases=[])
pending=case=refusal_attempt=None
registry=[]
request={}
class Gateway:
    def __init__(self,symbol,enter):
        self.enter=enter;self.enabled=True;registry.append(self)
    def observe(self):
        global pending
        if self.enter:
            assert pending is None
            if request['selected']:pending=dict(request)
        else:
            assert pending is not None
            observed['events'].append(pending);pending=None
            save()
        return False
'''


class InterfaceObserverControls(unittest.TestCase):
    def setup_observer(self):
        temporary=tempfile.TemporaryDirectory();self.addCleanup(temporary.cleanup)
        folder=Path(temporary.name)
        scope=dict(process_regression_profile=dict(process_memory=True,kernel_spans=[]),
            component={},token_bytes=24,observation_path=str(folder/'observations.json'),
            process_regression_boundary_program=TRANSPORT,json=json)
        exec(PROGRAM,scope)
        scope['install_process_regression_observation']()
        ns=scope['process_regression_namespace']
        enter=next(bp for bp in ns['registry'] if bp.enter)
        returned=next(bp for bp in ns['registry'] if not bp.enter)
        return folder,ns,enter,returned

    def request(self,ns,enter,returned,name,selected):
        ns['request'].update(name=name,selected=selected)
        enter.observe()
        self.assertEqual(returned.enabled,selected)
        if returned.enabled:returned.observe()
        self.assertFalse(returned.enabled)

    def test_interleaved_non_mm_requests_preserve_every_selected_return(self):
        _,ns,enter,returned=self.setup_observer()
        self.assertFalse(returned.enabled)
        requests=[('boot',False),('resize',True),('extent-read',False),('prepare',True),
                  ('extent-write',False),('commit',True),('handoff',True),('exit',True),
                  ('empty-extent',False),('cancel',True),('cleanup',True)]
        for name,selected in requests:self.request(ns,enter,returned,name,selected)
        self.assertEqual([r['name'] for r in ns['observed']['events']],
                         ['resize','prepare','commit','handoff','exit','cancel','cleanup'])
        self.assertIsNone(ns['pending'])

    def test_periodic_checkpoint_and_final_flush_retain_identical_records(self):
        folder,ns,enter,returned=self.setup_observer()
        for i in range(65):self.request(ns,enter,returned,str(i),True)
        target=folder/'process-mm-observations.json'
        partial=json.loads(target.read_text())
        self.assertEqual(len(partial['events']),64)
        self.assertFalse(partial['passed'])
        ns['observed']['unsigned_word']=(1<<64)-1
        ns['observed']['byte_string']='00ff'*4096
        ns['complete_save']()
        self.assertEqual(json.loads(target.read_text()),ns['observed'])
        self.assertEqual(len(ns['observed']['events']),65)
        self.assertFalse(target.with_suffix('.json.tmp').exists())

    def test_failure_checkpoint_records_the_actual_pending_transport(self):
        folder,ns,_,_=self.setup_observer()
        ns['pending']=dict(operation=9,partial=True)
        ns['observed']['failure']='modeled observer failure'
        ns['save']()
        actual=json.loads((folder/'process-mm-observations.json').read_text())
        self.assertFalse(actual['passed'])
        self.assertEqual(actual['pending'],dict(operation=9,partial=True))
        self.assertIsNone(actual['active_case'])
        self.assertIsNone(actual['refusal_pending'])

    def test_failed_encoding_does_not_publish_an_incomplete_checkpoint(self):
        folder,ns,_,_=self.setup_observer()
        ns['complete_save']()
        target=folder/'process-mm-observations.json';prior=target.read_bytes()
        ns['observed']['unencodable']=object()
        with self.assertRaises(TypeError):ns['complete_save']()
        self.assertEqual(target.read_bytes(),prior)
        self.assertFalse(json.loads(prior)['passed'])


if __name__=='__main__':unittest.main()
