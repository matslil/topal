"""Validate selected pipe-loop bytes, native/report bindings, index and links."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parent
repo=root.parents[4]
digest=lambda data:hashlib.sha256(data).hexdigest()
read=lambda path:json.loads(path.read_text())
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
references={}
def unpack(label,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed=path.read_bytes()
    if 'snapshot_sha256' in row: assert digest(packed)==row['snapshot_sha256']
    data=gzip.decompress(packed)
    assert digest(data)==row['sha256']
    if 'bytes' in row: assert len(data)==row['bytes']
    references[label]=dict(snapshot=row['snapshot'],sha256=digest(data),snapshot_sha256=digest(packed))
    return data
sources={name:unpack('sources/'+name,row) for name,row in identity['selected_inputs'].items()}
assert len(sources)==152 and all((repo/name).read_bytes()==data for name,data in sources.items())
artifacts={name:unpack('artifacts/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
assert len(artifacts)==35
kernel_sha=digest(artifacts['kernel.elf'])
origin=json.loads(artifacts['process-access-origins.json'])
assert origin['kernel_sha256']==kernel_sha and origin['origin_sha256']==digest(artifacts['process-byte-origins.json'])
assert all(digest(sources[name])==sha for name,sha in origin['source_hashes'].items())
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),('pipe-loop-audit.json','user_accesses')]
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
assert (len(selected),len(accesses),len(census['calls']))==(86,314,91)
assert {r['call_offset'] for r in index['selected_intervals']}==set(selected)
assert {r['offset'] for r in index['calls_without_selected_local_intervals']}=={r['offset'] for r in census['calls']}-set(selected)
assert len(index['calls_without_selected_local_intervals'])==5
for row in index['selected_intervals']:
    report=json.loads(gzip.decompress((root/row['audit']).read_bytes()))
    assert any(c['call_offset']==row['call_offset'] and c['name']==row['name'] for c in report['cases'])
audit_sha=digest(artifacts['pipe-loop-audit.json'])
frozen=None
for label in ['pipes','source','pinned','provider','fixtures']:
    run=read(root/'native'/label/'run.json')
    assert run['exit_code']==0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
    assert run['selected_tests']==dict(provider=3,fixtures=2).get(label,1)
    assert len(run['modified_source_hashes_before'])==9
    assert all(digest(sources[name])==sha for name,sha in run['modified_source_hashes_before'].items())
    if frozen is None: frozen=run['modified_source_hashes_before']
    assert frozen==run['modified_source_hashes_before']
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
cohorts=read(root/'native-input-identities.json')
observations={}
for label in ['pipes','source','pinned']:
    assert cohorts[label]['kernel.elf']['sha256']==kernel_sha
    for name,_ in fields: assert cohorts[label][name]['sha256']==digest(artifacts[name])
    raw=gzip.decompress((root/'native'/label/'observation/observations.json.gz').read_bytes())
    observations[label]=json.loads(raw)
    assert observations[label]['passed'] and observations[label]['extent_call_coverage']['kernel_sha256']==kernel_sha
    if label=='pipes': pipe_observation_sha=digest(raw)
assert cohorts['pinned']['static-elf-abi-fixture.elf']['sha256']=='7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
native=read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256']==kernel_sha and native['audit_sha256']==audit_sha
assert native['observation_sha256']==pipe_observation_sha and native['source_syscalls']==176
assert len(native['loop_completions'])==40 and sum(r['size'] for r in native['loop_completions'])==65808
assert len(native['refusals'])==28 and native['wrapping_transfers']==16
assert len(native['zero_length_calls_without_transfer'])==8 and native['saved_observer_programs_present']
gnu=json.loads(gzip.decompress((root/'gnu-graph/gnu-verification.json.gz').read_bytes()))
transcript=gzip.decompress((root/'gnu-graph/gnu-decoder-transcript.json.gz').read_bytes())
assert gnu['passed'] and gnu['kernel_sha256']==kernel_sha
assert gnu['graph_sha256']==digest(artifacts['process-access-inventory.json'])
assert gnu['decoder_transcript_sha256']==digest(transcript)
assert (gnu['gnu_decoder_nodes'],gnu['gnu_graph_edges'])==(31024,36950)
assert gnu['exact_lengths_and_direct_edges_equal'] and gnu['root_closure_equal']
operands=read(root/'gnu-operands.json')
assert operands['passed'] and operands['audit_sha256']==audit_sha and operands['gnu_transcript_sha256']==digest(transcript)
assert operands['selected_instructions']==58 and operands['exact_operands_and_direct_targets_equal']
assert len(operands['reverse_guard_cuts'])==6
controls=read(root/'controls/controls.json')
assert controls['passed'] and controls['positive_report_sha256']==audit_sha
assert controls['reader_sha256']==digest(sources['linux-kernel/tools/user-memory/audit_pipe_loops.py'])
assert len(controls['controls'])==17
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
comparison=read(root/'emission-comparison.json')
assert comparison['passed'] and comparison['old_provider_sha256']==digest(artifacts['before/provider.o'])
assert comparison['new_provider_sha256']==digest(artifacts['provider.o'])
assert comparison['new_function_bytes']==comparison['old_function_bytes']+52
assert len(comparison['insertions'])==2 and len(comparison['adjusted_relative_displacements'])==219
assert len(comparison['unchanged_component_objects'])==14
for row in comparison['unchanged_component_objects']:
    assert digest(artifacts[row['path']])==row['sha256'] and len(artifacts[row['path']])==row['bytes']
functions=read(root/'provider-function-comparison.json')
assert functions['passed'] and functions['old_provider_sha256']==comparison['old_provider_sha256']
assert functions['new_provider_sha256']==comparison['new_provider_sha256'] and functions['functions']==163
assert len(functions['unchanged_functions'])==162 and len(functions['changed_functions'])==1
component=json.loads(gzip.decompress((root/'fixture/source-component.json.gz').read_bytes()))
assert component['conservative_user_stack_bytes']==1368<=2048
assert component['stack_frame_costs']['pipe-cases']==216
assert component['pipe_source_sha256']==digest(sources['linux-kernel/labs/qemu/x86_64/user_extent_pipe_cases.c'])
coverage=read(root/'native-coverage.json')
union=set().union(*[{r['offset'] for r in obs['extent_call_coverage']['reached']} for obs in observations.values()])
assert coverage['passed'] and coverage['kernel_sha256']==kernel_sha
assert union==set(coverage['reached']) and len(union)==coverage['matching_kernel_union']==81
assert {r['offset'] for r in coverage['unreached']}=={r['offset'] for r in census['calls']}-union
for label,obs in observations.items(): assert len(obs['extent_call_coverage']['reached'])==coverage['fresh_first_reach_sites'][label]
failure=read(root/'history/first-replay-optional-compression-hash/failure.json')
assert failure['exit_code']==1 and failure['exception']=='KeyError: snapshot_sha256'
replay=read(root/'replay.json')
assert replay['passed'] and replay['models']==94 and replay['selected_inputs']==152 and replay['artifacts']==35
assert all(replay[key] for key in ['exact_source_join_equal','exact_nine_reports_equal',
    'exact_native_record_readback_equal','exact_complete_body_comparison_equal',
    'exact_provider_function_comparison_equal','exact_saved_gnu_operand_readback_equal'])
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
    data=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(data),bytes=len(data))
manifest=dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected byte/reference/current-source/report/native-record/index/link bindings only; not full producer, actual owners/lifetime or canonical qualification')
if '--check' in sys.argv[1:]: assert read(root/'manifest.json')==manifest
else: (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),snapshot_references=len(references),
    selected_inputs=len(sources),links=len(links),conditional_sites=len(selected),access_instructions=len(accesses))))
