"""Counterexamples to false acceptance of a complete native caller census."""
from copy import deepcopy
from unittest import TestCase
import json
import tempfile
from pathlib import Path

from caller_coverage import combine
from archive_caller_coverage import ROOT, fact, member_path, origin_report


class CallerCoverageTests(TestCase):
    def setUp(self):
        self.calls=[dict(offset=i,target='runtime-readable-extent') for i in [10,20,30]]
        self.inventory=dict(schema='topal.owned-extent-call-inventory/1',calls=self.calls,function_bytes=64)
        self.records={}
        for name,offsets in [('first',[10]),('second',[10,20])]:
            self.records[name]=dict(passed=True,extent_call_coverage=dict(observation_passed=True,
                kernel_sha256='kernel',inventory_sha256='inventory',total_sites=3,reached_sites=len(offsets),
                reached=[dict(offset=i,target='runtime-readable-extent',candidate=4096,length=8,phase=32) for i in offsets],
                unreached=[r for r in self.calls if r['offset'] not in offsets]))

    def result(self):
        return combine(self.inventory,'inventory','kernel',self.records)

    def test_union_retains_different_arguments_and_explicit_missing_site(self):
        self.records['second']['extent_call_coverage']['reached'][0]['length']=0
        result=self.result()
        self.assertEqual(result['reached_sites'],2)
        self.assertFalse(result['all_sites_sampled'])
        self.assertEqual(result['missing'],[self.calls[2]])
        self.assertEqual([r['length'] for r in result['sites'][0]['samples']],[8,0])

    def test_success_without_coverage_is_excluded_explicitly(self):
        self.records['legacy']=dict(passed=True)
        self.assertEqual(self.result()['excluded_without_coverage'],['legacy'])

    def test_duplicate_sample_cannot_fill_an_unobserved_site(self):
        coverage=self.records['first']['extent_call_coverage']
        coverage['reached'].append(deepcopy(coverage['reached'][0]));coverage['reached_sites']=2
        with self.assertRaisesRegex(ValueError,'duplicate'):self.result()

    def test_another_kernel_cannot_supply_a_missing_sample(self):
        self.records['second']['extent_call_coverage']['kernel_sha256']='other'
        with self.assertRaisesRegex(ValueError,'identity'):self.result()

    def test_wrong_inventory_and_unknown_or_retargeted_site_are_rejected(self):
        for key,value in [('inventory_sha256','other'),('total_sites',4)]:
            original=deepcopy(self.records)
            self.records['first']['extent_call_coverage'][key]=value
            with self.assertRaises(ValueError):self.result()
            self.records=original
        for key,value in [('offset',31),('target','runtime-writable-extent')]:
            original=deepcopy(self.records)
            self.records['first']['extent_call_coverage']['reached'][0][key]=value
            with self.assertRaises(ValueError):self.result()
            self.records=original

    def test_failed_or_incomplete_observation_cannot_contribute(self):
        self.records['first']['passed']=False
        with self.assertRaisesRegex(ValueError,'failed'):self.result()
        self.records['first']['passed']=True
        self.records['first']['extent_call_coverage']['observation_passed']=False
        with self.assertRaisesRegex(ValueError,'incomplete'):self.result()

    def test_native_width_and_phase_are_not_silently_truncated(self):
        for key,value in [('candidate',-1),('length',1<<64),('phase',256),('candidate',True)]:
            original=deepcopy(self.records)
            self.records['first']['extent_call_coverage']['reached'][0][key]=value
            with self.assertRaises(ValueError):self.result()
            self.records=original

    def test_missing_closure_and_duplicate_inventory_are_rejected(self):
        self.records['first']['extent_call_coverage']['unreached'].pop()
        with self.assertRaisesRegex(ValueError,'closure'):self.result()
        self.inventory['calls'].append(deepcopy(self.calls[0]))
        with self.assertRaisesRegex(ValueError,'duplicate'):self.result()

    def test_changed_report_cannot_be_legitimized_by_new_origin_metadata(self):
        parent=ROOT/'linux-kernel/.cache/x86_64';parent.mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='caller-origin-test-',dir=parent) as temp:
            base=Path(temp);original=json.dumps(dict(passed=True,kernel='original')).encode()
            manifest=json.dumps({'report.json':fact(original)}).encode()
            (base/'report.json').write_bytes(original);(base/'manifest.json').write_bytes(manifest)
            identity=dict(report=fact(original),manifest=fact(manifest))
            self.assertEqual(origin_report(base,identity)['kernel'],'original')
            changed=json.dumps(dict(passed=True,kernel='changed')).encode()
            (base/'report.json').write_bytes(changed);identity['report']=fact(changed)
            with self.assertRaisesRegex(ValueError,'report/manifest disagreement'):
                origin_report(base,identity)

    def test_archive_member_cannot_read_a_parent_or_absolute_path(self):
        base=ROOT/'linux-kernel/.cache/x86_64'
        for name in ['../report.json','/tmp/report.json','']:
            with self.assertRaises(ValueError):member_path(base,name)
        self.assertEqual(member_path(base,'evidence/report.json'),base/'evidence/report.json')
