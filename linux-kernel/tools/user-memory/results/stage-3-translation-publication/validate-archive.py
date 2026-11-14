"""Validate the selected local-publication source/evidence delta and replay."""
import gzip
import json
from pathlib import Path
import re
import sys

from records import digest,inputs,read,unpack

root=Path(__file__).resolve().parent
repo=root.parents[4]
sources,artifacts=inputs(root)
source_bytes={name:unpack(root,row) for name,row in sources.items()}
artifact_bytes={name:unpack(root,row) for name,row in artifacts.items()}
assert (len(sources),len(artifacts))==(192,94)
assert all((repo/name).read_bytes()==raw for name,raw in source_bytes.items())
kernel=digest(artifact_bytes['kernel.elf'])
native={name:unpack(root,row) for name,row in read(root/'native/files.json').items()}
observation=json.loads(native['observations.json'])
assert observation['passed'] and len(observation['executions'])==4
run=read(root/'native/run.json')
assert run['exit_code']==0 and not run['source_changes_during_run'] and not run['artifacts_changed']
assert run['source_hashes_before']==run['source_hashes_after']
assert len(run['source_hashes_before'])==7
assert all(digest(source_bytes[name])==sha for name,sha in run['source_hashes_before'].items())
assert run['artifact_hashes_before']==run['artifact_hashes_after']
assert len(run['artifact_hashes_before'])==12
assert all(digest(artifact_bytes[name])==sha for name,sha in run['artifact_hashes_before'].items())
assert run['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
command=json.loads(native['qemu.command.json'])
assert command[command.index('-d')+1]=='cpu,exec,tid'
assert command[command.index('-accel')+1]=='tcg,thread=multi'
assert command[command.index('-smp')+1]=='cpus=2,sockets=1,cores=2,threads=1'
assert command[command.index('-gdb')+1].startswith('tcp:127.0.0.1:')
assert command[command.index('-monitor')+1]=='none'
publication=read(root/'checks/publication.json')
assert publication['passed'] and publication['observation_sha256']==digest(native['observations.json'])
assert publication['kernel_sha256']==kernel
assert (publication['publication_records'],publication['memory_requests'],publication['post_invlpg_boundaries'],
    publication['old_backing_releases'],publication['old_backing_erased_bytes'])==(20,36,812,16,2228224)
assert (publication['active_publications'],publication['detached_whole_heap_retirements'],
    publication['observed_flushing_root_handoffs'],publication['cancelled_block_attempts'])==(19,1,1,2)
for field,name in [('model_sha256','translation_publication.py'),('reader_sha256','readback_translation_publication.py')]:
    assert publication[field]==digest(source_bytes['linux-kernel/tools/user-memory/'+name])
layout=read(root/'checks/layout.json')
assert layout['passed'] and layout['kernel_sha256']==kernel
assert layout['function']['size']==149 and layout['function_sha256']==publication['publisher_function_sha256']
assert layout['source_sha256']==digest(source_bytes['src/topal-compiler/src/runtime/kernel_frame_memory.ll'])
assert layout['dispatch_ir_sha256']==digest(artifact_bytes['dispatch.ll'])
assert layout['inspector_sha256']==digest(source_bytes['linux-kernel/tools/user-memory/inspect_translation_publication.py'])
progress=json.loads(native['observations.json.translation-progress.json'])
assert not progress['passed'] and not progress['full_guest_retirement_complete']
actual=observation['translation_publication_observation']
assert progress['records']==actual['records'] and progress['requests']==actual['requests']
assert actual['profile']==layout and actual['passed']
assert actual['model_sha256']==publication['model_sha256']
assert actual['trace_model_sha256']==digest(source_bytes['linux-kernel/tools/user-memory/qemu_publication_trace.py'])
assert len(actual['root_switches'])==1
for name in ['owners.json','metadata/readback.json']:
    report=read(root/'checks'/name)
    assert report['passed'] and report['kernel_sha256']==kernel
    assert report['observation_sha256']==digest(native['observations.json'])
assert read(root/'checks/owners.json')['selected_native_helpers']==52
metadata=read(root/'checks/metadata/readback.json')
assert (metadata['syscalls'],metadata['modeled_calls'],metadata['stat_outputs'],metadata['clock_returns'])==(136,124,44,44)
qualification=read(root/'checks/qualification.json')
assert qualification['passed'] and (qualification['models'],qualification['offline_controls'])==(177,18)
assert qualification['complete_console_equals_preceding_native'] and qualification['serial_bytes']==7606
assert digest(native['serial.log'])==qualification['serial_sha256']
assert qualification['serial_sha256']=='5daa5b998dba1c694da64d9aef89eb7fcea653367ed424d90829488ce067b435'
assert len(qualification['commands'])==6
for row in qualification['commands']:
    assert row['exit_code']==0 and digest((root/'checks'/(row['label']+'.log')).read_bytes())==row['log_sha256']
assert b'Ran 177 tests' in (root/'checks/models.log').read_bytes()
controls=read(root/'checks/controls/controls.json')
assert controls['passed'] and len(controls['controls'])==18 and all(row['rejected'] for row in controls['controls'])
assert controls['observation_sha256']==digest(native['observations.json'])
assert controls['runner_sha256']==digest(source_bytes['linux-kernel/tools/user-memory/control_translation_publication.py'])
failure_versions={}
for label in ['per-page-breakpoints','global-nochain','cancelled-block-reader','detached-cleanup-reader']:
    history=root/'history'/label
    failed=read(history/'run.json')
    assert failed['exit_code']==1 and not failed['source_changes_during_run'] and not failed['artifacts_changed']
    assert failed['source_hashes_before']==failed['source_hashes_after']
    assert failed['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in failed['resources']['memory.events'].splitlines())
    assert failed['artifact_hashes_before']==failed['artifact_hashes_after']
    assert failed['artifact_hashes_before']['kernel.elf']==kernel
    for name,row in read(history/'inputs.json').items():
        raw=unpack(root,row)
        assert digest(raw)==failed['source_hashes_before'][name]
        failure_versions[digest(raw)]=True
    saved={name:unpack(root,row) for name,row in read(history/'files.json').items()}
    if label in ['per-page-breakpoints','global-nochain']:
        assert b'timed out (600s)' in gzip.decompress((history/'test.log.gz').read_bytes())
    elif label=='cancelled-block-reader':
        assert b'incomplete invalidation interval' in saved['gdb.stderr.log']
    else:
        assert saved['gdb.stdout.log'].count(b'SOURCE_METADATA_CASES_PASS')==4
        assert b'line 13, in mode' in saved['gdb.stderr.log']
for label in ['nochain','default-chain']:
    files={name:unpack(root,row) for name,row in read(root/'pilots'/label/'files.json').items()}
    observed=json.loads(files['observations.json'])
    assert observed['passed'] and not observed['full_guest_retirement_complete']
    if label=='default-chain':
        pilot=read(root/'pilots/default-chain/readback.json')
        assert pilot['passed'] and pilot['post_invlpg_states']==35 and pilot['publications']==2
        assert pilot['parser_sha256'] in failure_versions
        for row in pilot['logs']:
            assert len(files[row['file']])==row['bytes'] and digest(files[row['file']])==row['sha256']
replay=read(root/'replay.json')
assert replay['passed'] and not replay['new_native_execution']
assert (replay['selected_sources'],replay['artifacts'],replay['models'])==(192,94,177)
failure=read(root/'history/first-frozen-replay/status.json')
assert not failure['passed']
assert digest((root/'history/first-frozen-replay/inputs-before.json').read_bytes())==failure['initial_inputs_sha256']
assert digest((root/'history/first-frozen-replay/layout.log').read_bytes())==failure['log_sha256']
assert all(replay[field] for field in ['exact_layout_readback_equal','exact_publication_readback_equal',
    'exact_owner_readback_equal','exact_metadata_readback_equal','exact_eighteen_controls_equal'])
review=read(root/'review.json')
assert review['risk']=='high' and review['passed'] and not review['executable_kernel_changed']
assert not review['core_language_or_authority_meaning_changed'] and review['safeguard_notices']==[]
links=[]
for doc in [root/'README.md',repo/'linux-kernel/README.md',repo/'linux-kernel/step-4-status.md',
        repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
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
summary=lambda rows:digest(json.dumps({name:row['sha256'] for name,row in sorted(rows.items())},separators=(',',':')).encode())
manifest=dict(passed=True,files=files,selected_source_count=len(sources),artifact_count=len(artifacts),
    selected_source_set_sha256=summary(sources),artifact_set_sha256=summary(artifacts),local_links=links,
    scope='Current evidence delta, merged packed input identities, actual local native observations, separate failed scopes, pilot records and exact Python replay; no complete compiler/VM producer, all-CPU stable lifetime or production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,selected_sources=len(sources),artifacts=len(artifacts),archive_files=len(files),
    publications=20,post_invlpg_states=812,observed_root_handoffs=1)))
