"""Validate frozen bounded-random sources, frame/loop reports and native records."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parent
repo=root.parents[4]
digest=lambda raw:hashlib.sha256(raw).hexdigest()
read=lambda path:json.loads(path.read_text())
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
references={}
def unpack(label,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed=path.read_bytes()
    if 'snapshot_sha256' in row: assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256']
    if 'bytes' in row: assert len(raw)==row['bytes']
    references[label]=dict(snapshot=row['snapshot'],sha256=digest(raw),snapshot_sha256=digest(packed))
    return raw
sources={name:unpack('sources/'+name,row) for name,row in identity['selected_inputs'].items()}
assert len(sources)==164 and all((repo/name).read_bytes()==raw for name,raw in sources.items())
assert sum(not row['snapshot'].startswith('../') for row in identity['selected_inputs'].values())==8
model_version=identity['nonexecuted_model_input_in_native_commands']
old_model=unpack('native-recorded-model-input',model_version)
assert model_version['path']=='linux-kernel/tools/user-memory/test_random_observer.py'
assert old_model.replace(b"ns=dict(Checked=type('Breakpoint',(),{}))",
    b"ns=dict(Checked=type('Breakpoint',(),{}),struct=struct)")==sources[model_version['path']]
assert b'unittest' not in sources['src/topal-compiler/src/systems_frame_root_tests.rs']
artifacts={name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts)==36
kernel_sha=digest(artifacts['kernel.elf'])
origin=json.loads(artifacts['process-access-origins.json'])
assert origin['kernel_sha256']==kernel_sha and origin['origin_sha256']==digest(artifacts['process-byte-origins.json'])
assert all(digest(sources[name])==sha for name,sha in origin['source_hashes'].items())
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),
    ('pipe-loop-audit.json','user_accesses'),('file-input-audit.json','copy_offset'),
    ('warning-input-audit.json','user_accesses'),('random-output-audit.json','user_accesses')]
selected,accesses={},set()
for name,field in fields:
    report=json.loads(artifacts[name])
    assert report['passed'] and report['kernel_sha256']==kernel_sha
    for case in report['cases']:
        assert case['call_offset'] not in selected
        selected[case['call_offset']]=case
        accesses.update([case[field]] if field=='copy_offset' else [r['offset'] for r in case[field]])
index=read(root/'caller-access-index.json')
census=json.loads(artifacts['extent-call-inventory.json'])
assert not index['complete'] and index['kernel_sha256']==kernel_sha
assert (len(selected),len(accesses),len(census['calls']))==(90,325,91)
assert index['calls_with_selected_local_intervals']==len(selected)
assert index['distinct_user_access_instructions']==len(accesses)
assert {r['call_offset'] for r in index['selected_intervals']}==set(selected)
assert {r['offset'] for r in index['calls_without_selected_local_intervals']}=={r['offset'] for r in census['calls']}-set(selected)
assert len(index['calls_without_selected_local_intervals'])==1
for row in index['selected_intervals']:
    report=json.loads(gzip.decompress((root/row['audit']).read_bytes()))
    assert any(c['call_offset']==row['call_offset'] and c['name']==row['name'] for c in report['cases'])
audit=json.loads(artifacts['random-output-audit.json'])
audit_sha=digest(artifacts['random-output-audit.json'])
assert len(audit['cases'])==1 and audit['user_access_instructions']==2
case=audit['cases'][0]
assert (case['minimum_bytes'],case['maximum_bytes'])==(1,16)
assert case['selected_switched_stack_bytes']==168 and case['outer_saved_words']==4
assert case['retired_outer_spill_erasure_verified'] is False
callee=audit['extract_contract']
assert len(callee['instructions'])==len(callee['effects'])==1843
assert callee['maximum_scratch_bytes']==128 and callee['borrowed_stack_bytes_including_return']==136
assert callee['scratch_zero_before_every_release'] and callee['argument_registers_preserved']==[6,7]
assert len(callee['allocations'])==1 and len(callee['releases'])==len(callee['normal_returns'])==2
frozen=None
for label in ['storage','source','pinned','fixtures']:
    run=read(root/'native'/label/'run.json')
    assert run['exit_code']==0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
    assert run['selected_tests']==(2 if label=='fixtures' else 1)
    assert len(run['modified_source_hashes_before'])==8
    for name,sha in run['modified_source_hashes_before'].items():
        raw=old_model if name==model_version['path'] else sources[name]
        assert digest(raw)==sha
    if frozen is None: frozen=run['modified_source_hashes_before']
    assert frozen==run['modified_source_hashes_before']
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
cohorts=read(root/'native-input-identities.json')
observations={}
status_sets=[]
for label in ['storage','source','pinned']:
    assert cohorts[label]['kernel.elf']['sha256']==kernel_sha
    for name,_ in fields: assert cohorts[label][name]['sha256']==digest(artifacts[name])
    raw=gzip.decompress((root/'native'/label/'observation/observations.json.gz').read_bytes())
    observations[label]=json.loads(raw)
    obs=observations[label]
    assert obs['passed'] and obs['extent_call_coverage']['kernel_sha256']==kernel_sha
    random=obs['random_output_observation']
    assert random['passed'] and random['audit_sha256']==audit_sha
    for record in random['records']:
        if not record['accepted']: continue
        assert record['only_admitted_destination_leaf_ad_sets']
        for change in record['table_status_sets']:
            before,after=change['before'],change['after']
            assert before&7==7 and not before&~after
            assert change['changed_bits']==before^after and change['changed_bits'] and not change['changed_bits']&~0x60
            status_sets.append(change)
    if label=='source': source_observation_sha=digest(raw)
assert status_sets
assert cohorts['pinned']['static-elf-abi-fixture.elf']['sha256']=='7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
native=read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256']==kernel_sha and native['audit_sha256']==audit_sha
assert native['observation_sha256']==source_observation_sha
assert (native['admissions'],native['refusals'],native['completions'],native['bytes'])==(48,40,8,64)
assert len(native['events'])==48 and len(native['early_results_without_admission'])==24
assert native['native_lengths']==[8] and (native['initial_completions'],native['reusable_completions'])==(1,7)
assert native['indexed_stores']==native['indexed_readbacks']==64
assert native['source_syscall_order_results_and_opaque_output_hashes_equal'] and native['saved_observer_program_present']
gnu_artifacts={name:unpack('gnu/'+name,row) for name,row in read(root/'gnu-identity.json').items()}
gnu=json.loads(gnu_artifacts['gnu-verification.json'])
transcript=gnu_artifacts['gnu-decoder-transcript.json']
assert gnu['passed'] and gnu['kernel_sha256']==kernel_sha
assert gnu['graph_sha256']==digest(artifacts['process-access-inventory.json'])
assert gnu['decoder_transcript_sha256']==digest(transcript)
assert (gnu['gnu_decoder_nodes'],gnu['gnu_graph_edges'])==(31024,36950)
assert gnu['exact_lengths_and_direct_edges_equal'] and gnu['root_closure_equal']
operands=read(root/'gnu-operands.json')
assert operands['passed'] and operands['kernel_sha256']==kernel_sha
assert operands['audit_sha256']==audit_sha and operands['gnu_transcript_sha256']==digest(transcript)
assert operands['graph_sha256']==digest(artifacts['process-access-inventory.json'])
assert operands['selected_instructions']==1900 and operands['extract_effect_instructions']==1843
assert operands['exact_gnu_register_memory_stack_effects_equal']
assert operands['exact_count_candidate_stack_switch_and_loop_operands_equal'] and len(operands['reverse_guard_cuts'])==3
controls=read(root/'controls/controls.json')
assert controls['passed'] and controls['positive_report_sha256']==audit_sha
assert controls['reader_sha256']==digest(sources['linux-kernel/tools/user-memory/audit_random_outputs.py'])
assert controls['effects_reader_sha256']==digest(sources['linux-kernel/tools/user-memory/random_extract_effects.py'])
assert len(controls['controls'])==23
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
comparison=read(root/'artifact-comparison.json')
assert comparison['passed'] and comparison['core_objects_images_per_cohort']==17
assert comparison['metadata_reports_per_cohort']==15 and comparison['other_inherited_storage_artifacts_equal']==33
for label,rows in comparison['comparisons'].items():
    assert len(rows)==32 and label in cohorts
    for row in rows:
        if row['path']=='bzImage': continue
        assert digest(artifacts[row['path']])==row['sha256'] and len(artifacts[row['path']])==row['bytes']
intermediate=read(root/'source-intermediate-comparison.json')
assert intermediate['passed'] and intermediate['old_sha256']==digest(artifacts['before/source-user.elf'])
assert intermediate['new_sha256']==digest(artifacts['source-user.elf'])
assert intermediate['elf_and_program_headers_equal'] and intermediate['section_headers_equal']
assert {r['section'] for r in intermediate['changed_sections']}<={'.debug_info','.debug_line','.debug_line_str'}
assert all(not r['flags']&2 for r in intermediate['changed_sections'])
assert intermediate['unchanged_load_segments'] and intermediate['unchanged_allocated_sections']
coverage=read(root/'native-coverage.json')
old_raw=(root/coverage['inherited_record']).read_bytes()
assert digest(old_raw)==coverage['inherited_record_sha256']
old=json.loads(old_raw)
assert old['passed'] and old['kernel_sha256']==kernel_sha
fresh=set().union(*[{r['offset'] for r in obs['extent_call_coverage']['reached']} for obs in observations.values()])
union=fresh|set(old['reached'])
assert coverage['passed'] and coverage['kernel_sha256']==kernel_sha
assert fresh==set(coverage['fresh_reached']) and len(fresh)==coverage['fresh_matching_kernel_union']==83
assert union==set(coverage['reached']) and len(union)==coverage['matching_kernel_union']==87
assert {r['offset'] for r in coverage['unreached']}=={r['offset'] for r in census['calls']}-union
for label,obs in observations.items(): assert len(obs['extent_call_coverage']['reached'])==coverage['fresh_first_reach_sites'][label]
review=read(root/'review.json')
assert review['passed'] and review['retired_clock_spill_erasure_verified'] is False
assert review['retired_random_outer_spill_erasure_verified'] is False
assert not review['executable_kernel_changed'] and not review['core_language_or_authority_meaning_changed']
assert review['safeguard_notices']==[] and len(review['failures'])==3
for name in review['failures']: assert read(root/name)['exit_code'] in [1,101]
diagnostic=read(root/'history/mapping-diagnostic/run.json')
assert diagnostic['exit_code']==1 and diagnostic['kernel_sha256']==kernel_sha
observed_diagnostic=json.loads(gzip.decompress((root/'history/mapping-diagnostic/observation/observations.json.gz').read_bytes()))
changes=observed_diagnostic['random_mapping_failure']['changes']
assert len(changes)==1 and changes[0]['changed_bits']==0x60
assert changes[0]['after']==changes[0]['before']|0x60
replay=read(root/'replay.json')
assert replay['passed'] and replay['models']==121 and replay['selected_inputs']==164 and replay['artifacts']==36
assert all(replay[key] for key in ['exact_source_join_equal','exact_twelve_reports_equal',
    'exact_native_record_readback_equal','exact_saved_gnu_operand_readback_equal','exact_source_intermediate_comparison_equal',
    'exact_recorded_nonexecuted_model_repair_equal'])
links=[]
for doc in [root/'README.md',repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'): continue
        assert (doc.parent/target.split('#',1)[0]).resolve().is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files={}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name=='manifest.json': continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    raw=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(raw),bytes=len(raw))
manifest=dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected byte/reference/current-source/report/native-record/index/link bindings and one explicit nonexecuted model-version repair only; not a complete clean producer, actual owners/lifetime/retired-spill cleanup or production qualification')
if '--check' in sys.argv[1:]: assert read(root/'manifest.json')==manifest
else: (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),snapshot_references=len(references),
    selected_inputs=len(sources),links=len(links),conditional_sites=len(selected),access_instructions=len(accesses))))
