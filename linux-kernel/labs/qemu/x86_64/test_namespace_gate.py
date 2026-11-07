"""Publication obligations for the real shared namespace machine qualification."""
import gzip
import hashlib
import json
from pathlib import Path
import unittest
import sys

import topal_gate as GATE

LAB=Path(__file__).resolve().parent
sys.path.insert(0,str(GATE.LINUX_KERNEL/'tools/archive-publication'))
import verification_revision


class NamespaceEvidence(unittest.TestCase):
    def test_real_admission_effects_lifetimes_and_verbatim_machine_evidence(self):
        # TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,ABI,ACTION}-001.
        archive=LAB/'results/tk281/namespace-consumer'
        proof=json.loads((archive/'qualification.json').read_text())
        self.assertEqual((proof['schema'],proof['result']),
            ('topal.namespace-machine-qualification.x86_64/1','pass'))
        self.assertEqual((proof['steps'],proof['vm_observations']),(42,47))
        rust_path=GATE.LINUX_KERNEL/'tools/rust-qualification/results/tk281/final/qualification.json'
        self.assertEqual(GATE.digest(rust_path),proof['rust_record_sha256'])
        rust=json.loads(rust_path.read_text())
        self.assertTrue(rust['complete'] and rust['passed'])
        self.assertEqual((rust['head'],rust['tree']),
            (proof['qualified_code_commit'],proof['qualified_code_tree']))
        self.assertEqual(proof['archiver_sha256'],GATE.digest(LAB/'namespace_archive.py'))
        self.assertEqual(proof['archive_index_sha256'],GATE.digest(archive/'files.json'))
        index=json.loads((archive/'files.json').read_text())
        self.assertEqual(index['schema'],'topal.namespace-verbatim-file-index/1')
        self.assertEqual(len(index['files']),proof['archived_files'])
        verified=set()
        for entry in index['files'].values():
            if entry['sha256'] in verified:
                continue
            data=gzip.decompress((archive/entry['blob']).read_bytes())
            self.assertEqual(len(data),entry['bytes'])
            self.assertEqual(hashlib.sha256(data).hexdigest(),entry['sha256'])
            verified.add(entry['sha256'])
        self.assertEqual(len(verified),proof['unique_blobs'])

        def read(name):
            entry=index['files'][name]
            data=gzip.decompress((archive/entry['blob']).read_bytes())
            self.assertEqual((len(data),hashlib.sha256(data).hexdigest()),(entry['bytes'],entry['sha256']))
            return data

        def load(name):
            return json.loads(read(name))

        verification_revision.validate(proof['source_snapshots'],GATE.REPOSITORY,read,
            archive/'verification-revision.json')
        self.assertEqual(hashlib.sha256(read('cohort.json')).hexdigest(),proof['cohort_sha256'])
        cohort=load('cohort.json')
        self.assertTrue(cohort['complete'] and cohort['passed'])
        self.assertEqual(cohort['head'],proof['qualified_code_commit'])
        self.assertTrue(all(step['exit_code']==0 for step in cohort['steps']))
        selected=json.loads((LAB/'results/topal-toolchain-gate.json').read_text())
        self.assertEqual(proof['primary_kernel_sha256'],selected['linked_kernel_sha256'])
        for name,count in (('normal',465),('directory',64),('fork',37),('exec',39)):
            record=load(proof['proofs'][name])
            self.assertEqual((record['result'],record['independent_calls']),('pass',count))
            self.assertEqual(record['kernel_sha256'],proof['primary_kernel_sha256'])
        physical=load('n0/refusal-physical.json')
        self.assertEqual((physical['result'],physical['expected_calls']),('pass',66))
        self.assertEqual(sum(call['source_actions'] for call in physical['calls']),13)
        self.assertEqual(sum(bool(call['complete_owner_preserved']) for call in physical['calls']),53)
        self.assertEqual(sum(len((call['source_effect'] or {}).get('source_stores',[]))
            for call in physical['calls']),461)
        saved=read('n3/filesystem-first-exec-save.bin')
        self.assertEqual(len(saved),36864)
        self.assertEqual(saved,read('n3/filesystem-first-exec-compare.bin'))
        self.assertEqual(read('n3/serial.log').count(GATE.EXEC_COMMIT_MARKER),3)
        canceled=load(proof['proofs']['cancel'])
        state=canceled['observation']
        self.assertEqual((state['ordinary_returns'],state['source_actions']),(22,3))
        self.assertTrue(state['inactive_child_retained_at_parent_exit'] and state['process_returned'])
        self.assertEqual((state['history_before_final_release'],state['history_after_final_release']),(3,0))
        self.assertEqual(state['source_and_user_state_injections'],[])
        self.assertEqual(read('n4/owner-after-terminal.bin'),bytes(311296))
        self.assertNotEqual(read('n4/owner-before-terminal.bin'),bytes(311296))
        for mode in ('source-false','source-wrong-association','runtime-abort'):
            for operation in ('link','linkat','move','replace','exchange'):
                case=load(proof['proofs']['fatal-'+mode+'-'+operation])
                state=case['observation']
                self.assertEqual((case['result'],case['mode'],case['operation']),('pass',mode,operation))
                self.assertEqual((case['stopped_target']['running'],case['stopped_target']['status']),(False,'debug'))
                self.assertTrue(state['stores'] and state['whole_owner_exact_replayed_delta']
                    and state['persistent_effects_retained'] and state['actual_return_words_erased']
                    and state['all_private_staging_and_gprs_erased'])
                self.assertEqual((state['source_frame_and_reserve_erased'],state['admission_stack_erased']),(4280,4096))
        for mode,entries in (('bad-link-count',1),('overlap-namespace',0)):
            case=load(proof['proofs']['inode-admission-'+mode])
            state=case['observation']
            self.assertEqual((state['source_entries'],state['metadata_entries']),(0,entries))
            self.assertTrue(state['actual_owner_and_all_runtime_preserved'])
            self.assertEqual((case['stopped_target']['running'],case['stopped_target']['status']),(False,'debug'))
        observed=load('c0/clock-observation.json')
        self.assertEqual(observed['result'],'pass')
        self.assertEqual(observed['kernel_state_injections'],[])
        self.assertEqual(len(observed['calls']),66)
        self.assertEqual(sum(call['source_actions'] for call in observed['calls']),13)
        for call in observed['calls']:
            self.assertTrue(call['private_buffers_erased'])
            if call['source_actions']==0:
                self.assertIsNone(call['admission_stack_erased'])
                self.assertEqual(call['first_status_reads'],0)
                self.assertTrue(call['complete_owner_preserved'] and call['history_preserved'])
            else:
                self.assertEqual(call['admission_stack_erased'],4096)
                self.assertGreater(call['first_status_reads'],0)
        for clock_index,operation in enumerate(('link','linkat','move','replace','exchange'),1):
            failed=load('c'+str(clock_index)+'/clock-observation.json')
            self.assertTrue(failed['all_bound_native_code_and_data_verified'])
            self.assertTrue(failed['kernel_state_injections'])
            self.assertEqual(len(failed['calls']),1)
            call=failed['calls'][0]
            self.assertEqual((call['case']['expected'],call['first_status_reads'],call['source_actions']),(-5,1024,0))
            self.assertTrue(call['complete_owner_preserved'] and call['history_preserved']
                and call['private_buffers_erased'])
            self.assertEqual(call['admission_stack_erased'],4096)
        controls=('normal','runtime-abort','post-frame','post-control','source-false',
            'source-wrong-association','pre-owner','pre-view','pre-input','pre-generation',
            'pre-overlap-position','pre-overlap-page','pre-overlap-inode')
        for mode in controls:
            case=load(proof['proofs']['controller-'+mode])
            self.assertEqual(case['mode'],mode)
            self.assertEqual(case['result'],'pass-native-namespace-controller-component'
                if mode=='normal' else 'pass-native-namespace-fatal-component')
            self.assertTrue(case['observation']['all_source_code_and_data_verified'])
            self.assertEqual(len(case['observation']['calls']),4 if mode=='normal' else 1)
            self.assertEqual((case['stopped_target']['running'],case['stopped_target']['status']),(False,'debug'))
            for call in case['observation']['calls']:
                self.assertEqual(call['source_entries'],int(not mode.startswith('pre-')))
                self.assertEqual(call['stack_erased'],4272 if mode=='normal' else 4280)
                self.assertEqual(call['staging_erased'],512)
                self.assertTrue(call['erased_private'])
                if mode!='normal':
                    self.assertEqual((call['admission_stack_erased']['bytes'],
                        call['admission_stack_erased']['owning_frame_bytes']),(4096,8))
        utilities=load(proof['proofs']['utilities'])
        self.assertEqual(utilities['result'],'pass')
        self.assertEqual(utilities['application_sha256'],GATE.PINNED_STATIC_BUSYBOX_SHA256)
        self.assertEqual(len(utilities['workloads']),6)


if __name__=='__main__':
    unittest.main()
