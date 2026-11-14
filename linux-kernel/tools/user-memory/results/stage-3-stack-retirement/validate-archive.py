"""Verify frozen selected sources, changed kernel and cleanup evidence."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root=Path(__file__).resolve().parent
repo=root.parents[4]
read=lambda path:json.loads(gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes())
digest=lambda raw:hashlib.sha256(raw).hexdigest()
references={}
def unpack(label,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(repo)
    packed=path.read_bytes()
    if 'snapshot_sha256' in row:assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256']
    if 'bytes' in row:assert len(raw)==row['bytes']
    references[label]=dict(snapshot=row['snapshot'],sha256=digest(raw),snapshot_sha256=digest(packed))
    return raw
identity=read(root/'source-identity.json')
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
sources={name:unpack('source/'+name,row) for name,row in identity['selected_inputs'].items()}
assert all((repo/name).read_bytes()==raw for name,raw in sources.items())
artifacts={name:unpack('artifact/'+name,row) for name,row in read(root/'artifact-identity.json').items()}
kernel=digest(artifacts['kernel.elf'])
assert kernel!=digest(artifacts['before/kernel.elf'])
origin=json.loads(artifacts['process-access-origins.json'])
assert origin['kernel_sha256']==kernel and origin['origin_sha256']==digest(artifacts['process-byte-origins.json'])
assert all(digest(sources[name])==sha for name,sha in origin['source_hashes'].items())
reports={name:json.loads(raw) for name,raw in artifacts.items() if name.endswith('-audit.json') and not name.startswith('before/')}
assert len(reports)==13
assert all(report['passed'] and report['kernel_sha256']==kernel for report in reports.values())
stack=reports['stack-retirement-audit.json']
assert not stack['complete'] and len(stack['blocks'])==13
assert sum(row['words'] for row in stack['blocks'])==stack['zero_store_instructions']==27
assert stack['added_code_bytes']==324 and stack['maximum_clock_stack_bytes']==112 and stack['maximum_random_stack_bytes']==168
files=reports['file-input-audit.json']
random=reports['random-output-audit.json']
assert files['clock_contracts'][0]['retired_spill_erasure_verified']
assert random['cases'][0]['retired_outer_spill_erasure_verified']
assert random['extract_contract']['scratch_zero_before_every_release']
graph=json.loads(artifacts['process-access-inventory.json'])
assert graph['kernel_sha256']==kernel and len(graph['instructions'])==31051 and len(graph['edges'])==36977
gnu={name:unpack('gnu/'+name,row) for name,row in read(root/'gnu-identity.json').items()}
decoding=json.loads(gnu['gnu-verification.json'])
assert decoding['passed'] and decoding['kernel_sha256']==kernel
assert decoding['graph_sha256']==digest(artifacts['process-access-inventory.json'])
assert decoding['decoder_transcript_sha256']==digest(gnu['gnu-decoder-transcript.json'])
operands=read(root/'gnu-operands.json')
assert operands['passed'] and operands['kernel_sha256']==kernel and operands['selected_instructions']==27
assert operands['audit_sha256']==digest(artifacts['stack-retirement-audit.json'])
assert operands['gnu_transcript_sha256']==digest(gnu['gnu-decoder-transcript.json'])
emission=read(root/'emission-comparison.json.gz')
assert emission['passed'] and emission['added_bytes']==324 and len(emission['insertions'])==13
assert emission['old_provider_sha256']==digest(artifacts['before/provider.o'])
assert emission['new_provider_sha256']==digest(artifacts['provider.o'])
assert len(emission['unchanged_functions'])==162 and len(emission['unchanged_component_objects'])==14
assert emission['baseline_linear_transcript_sha256']==digest(gzip.decompress((root/'baseline-linear-transcript.json.gz').read_bytes()))
for row in emission['unchanged_component_objects']:
    assert artifacts[row['path']]==artifacts['before/'+row['path']]
    assert digest(artifacts[row['path']])==row['sha256']
native=read(root/'native-readback.json')
assert native['passed'] and native['kernel_sha256']==kernel
assert native['audit_sha256']==digest(artifacts['stack-retirement-audit.json'])
cohorts=read(root/'native-input-identities.json')
frozen=None
reached=set()
for label in ['source','storage','pinned','fixtures']:
    run=read(root/'native'/label/'run.json')
    assert run['exit_code']==0 and not run['source_changes_during_run']
    hashes=run['modified_source_hashes_before']
    assert hashes==run['modified_source_hashes_after'] and all(digest(sources[name])==sha for name,sha in hashes.items())
    if frozen is None:frozen=hashes
    assert hashes==frozen
    assert run['selected_tests']==(2 if label=='fixtures' else 1)
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
    if label=='fixtures':continue
    raw=gzip.decompress((root/'native'/label/'observation/observations.json.gz').read_bytes())
    observation=json.loads(raw)
    assert observation['passed'] and digest(raw)==native['cohorts'][label]['observation_sha256']
    actual=observation['stack_retirement_observation']
    assert actual['records']==native['cohorts'][label]['records'] and actual['unreached']==native['cohorts'][label]['unreached']
    assert actual['passed'] and actual['audit_sha256']==native['audit_sha256']
    reached.update(row['offset'] for row in observation['extent_call_coverage']['reached'])
    assert cohorts[label]['kernel.elf']['sha256']==kernel
    for name in reports:assert cohorts[label][name]['sha256']==digest(artifacts[name])
assert reached==set(native['fresh_helper_first_reach_union'])
assert cohorts['pinned']['static-elf-abi-fixture.elf']['sha256']=='7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
index=read(root/'caller-access-index.json')
assert index['kernel_sha256']==kernel and not index['complete']
assert (index['emitted_helper_calls'],index['calls_with_selected_local_intervals'],index['distinct_user_access_instructions'])==(91,90,325)
assert len({row['call_offset'] for row in index['selected_intervals']})==90
assert len(index['calls_without_selected_local_intervals'])==1
controls=read(root/'controls/controls.json')
assert controls['passed'] and len(controls['controls'])==16
assert controls['positive_report_sha256']==digest(artifacts['stack-retirement-audit.json'])
assert controls['reader_sha256']==digest(sources['linux-kernel/tools/user-memory/audit_stack_retirement.py'])
for row in controls['controls']:
    assert row['rejected'] and row['exit_code']!=0
    assert digest((root/'controls'/(row['name']+'.log')).read_bytes())==row['log_sha256']
checks=read(root/'checks/commands.json')
assert len(checks)==6 and all(row['exit_code']==0 for row in checks)
assert b'Ran 132 tests' in (root/'checks/models.log').read_bytes()
replay=read(root/'replay.json')
assert replay['passed'] and replay['models']==132 and replay['selected_sources']==len(sources) and replay['artifacts']==len(artifacts)
assert all(replay[key] for key in ['exact_source_join_equal','exact_thirteen_reports_equal',
    'exact_gnu_operand_readback_equal','exact_complete_emission_comparison_equal','exact_saved_native_readback_equal'])
review=read(root/'review.json')
assert review['passed'] and review['risk']=='high' and review['executable_kernel_changed']
assert review['safeguard_notices']==[] and not review['core_language_or_authority_meaning_changed']
links=[]
for doc in [root/'README.md',repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'):continue
        assert (doc.parent/target.split('#',1)[0]).resolve().is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files={}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name=='manifest.json':continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    raw=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(raw),bytes=len(raw))
manifest=dict(passed=True,files=files,snapshot_references=references,local_links=links,
    scope='Selected current-source/snapshot/artifact/report/native/model/replay/link bindings. Not complete producer, actual owners/nonalias/lifetime, unreached/fatal/exception paths or production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,sources=len(sources),artifacts=len(artifacts),archive_files=len(files),
    snapshot_references=len(references),native_blocks=len(native['reached_blocks']),fresh_helper_sites=len(reached))))
