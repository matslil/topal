"""Check current selected image-publication evidence and its limited replay."""
import json
from pathlib import Path
import re
import sys

from records import digest,inputs,read,unpack

root=Path(__file__).resolve().parent
repo=root.parents[4]
sources,artifacts=inputs(root)
assert (len(sources),len(artifacts))==(201,94)
source_bytes={name:unpack(root,row) for name,row in sources.items()}
artifact_bytes={name:unpack(root,row) for name,row in artifacts.items()}
for name,raw in source_bytes.items():assert (repo/name).read_bytes()==raw
native={name:unpack(root,row) for name,row in read(root/'native/files.json').items()}
observed=json.loads(native['observations.json'])
assert observed['passed'] and len(observed['executions'])==4
run=read(root/'native/run.json')
assert run['exit_code']==0 and not run['source_changes_during_run'] and not run['artifacts_changed']
assert run['source_hashes_before']==run['source_hashes_after'] and len(run['source_hashes_before'])==12
assert all(digest(source_bytes[name])==sha for name,sha in run['source_hashes_before'].items())
assert run['artifact_hashes_before']==run['artifact_hashes_after'] and len(run['artifact_hashes_before'])==14
assert all(digest(artifact_bytes[name])==sha for name,sha in run['artifact_hashes_before'].items())
assert run['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
kernel=digest(artifact_bytes['kernel.elf'])
assert kernel=='47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f'
images=read(root/'checks/images.json')
layout=read(root/'checks/layout.json')
assert images['passed'] and images['kernel_sha256']==kernel
assert (images['publication_records'],images['relro_publications'],images['exec_publications'],images['post_invlpg_states'])==(7,4,3,86)
assert images['selected_owner_boundaries']==21 and images['all_selected_image_headers_covered_once']
assert layout['passed'] and layout['profiles']==images['layout']
assert layout['kernel_sha256']==kernel
assert [layout['profiles'][name]['bytes'] for name in ['relro','exec']]==[100,638]
actual=observed['image_publication_observation']
assert actual['passed'] and actual['profile']==layout
command=json.loads(native['qemu.command.json'])
assert command[command.index('-d')+1]=='cpu,exec,tid'
assert command[command.index('-accel')+1]=='tcg,thread=multi'
assert command[command.index('-smp')+1]=='cpus=2,sockets=1,cores=2,threads=1'
assert command[command.index('-gdb')+1].startswith('tcp:127.0.0.1:')
assert command[command.index('-monitor')+1]=='none'
sites={row['sites']['invalidated'] for row in layout['profiles'].values()}
sites.add(observed['translation_publication_observation']['profile']['sites']['invalidated'])
assert command[command.index('-dfilter')+1]==','.join(hex(site)+'..'+hex(site) for site in sorted(sites))
for field,name in [('model_sha256','image_publication.py'),('reader_sha256','readback_image_publication.py'),
        ('trace_model_sha256','qemu_publication_trace.py')]:
    assert images[field]==digest(source_bytes['linux-kernel/tools/user-memory/'+name])
assert layout['inspector_sha256']==digest(source_bytes['linux-kernel/tools/user-memory/inspect_image_publication.py'])
assert layout['origins_sha256']==digest(artifact_bytes['process-byte-origins.json'])
assert layout['graph_sha256']==digest(artifact_bytes['process-access-inventory.json'])
for name,row in images['raw_logs'].items():
    assert len(native[name])==row['bytes'] and digest(native[name])==row['sha256']
progress=json.loads(native['observations.json.image-progress.json'])
assert not progress['passed'] and not progress['full_guest_retirement_complete']
assert progress['records']==actual['records']
heap=read(root/'checks/heap.json')
assert heap['passed'] and heap['kernel_sha256']==kernel
assert (heap['publication_records'],heap['post_invlpg_boundaries'],heap['old_backing_releases'],heap['old_backing_erased_bytes'])==(20,812,16,2228224)
for report in [images,heap,read(root/'checks/owners.json'),read(root/'checks/metadata/readback.json')]:
    assert report['passed'] and report['observation_sha256']==digest(native['observations.json'])
assert read(root/'checks/owners.json')['selected_native_helpers']==52
metadata=read(root/'checks/metadata/readback.json')
assert (metadata['syscalls'],metadata['modeled_calls'],metadata['stat_outputs'],metadata['clock_returns'])==(136,124,44,44)
qualification=read(root/'checks/qualification.json')
assert qualification['passed'] and (qualification['models'],qualification['image_offline_controls'],qualification['heap_offline_controls'])==(196,16,18)
assert qualification['complete_console_equals_preceding_native'] and qualification['serial_bytes']==7606
assert digest(native['serial.log'])==qualification['serial_sha256']=='5daa5b998dba1c694da64d9aef89eb7fcea653367ed424d90829488ce067b435'
assert len(qualification['commands'])==8
for row in qualification['commands']:
    assert row['exit_code']==0 and digest((root/'checks'/(row['label']+'.log')).read_bytes())==row['log_sha256']
assert b'Ran 196 tests' in (root/'checks/models.log').read_bytes()
for name,count in [('image-controls.json',16),('heap-controls/controls.json',18)]:
    controls=read(root/'checks'/name)
    assert controls['passed'] and len(controls['controls'])==count and all(r['rejected'] for r in controls['controls'])
    assert controls['observation_sha256']==digest(native['observations.json'])
for label in ['loop-head-entry','later-exec-generation']:
    folder=root/'history'/label
    failed=read(folder/'run.json')
    assert failed['exit_code']==1 and not failed['source_changes_during_run'] and not failed['artifacts_changed']
    assert failed['source_hashes_before']==failed['source_hashes_after']
    assert failed['artifact_hashes_before']==failed['artifact_hashes_after']
    assert failed['artifact_hashes_before']['kernel.elf']==kernel
    assert failed['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in failed['resources']['memory.events'].splitlines())
    for name,row in read(folder/'inputs.json').items():assert digest(unpack(root,row))==failed['source_hashes_before'][name]
    saved={name:unpack(root,row) for name,row in read(folder/'files.json').items()}
    assert not json.loads(saved['observations.json'])['passed']
    assert b'AssertionError' in saved['gdb.stderr.log']
lock=read(root/'history/wrapper-lock.json')
assert lock['exit_code']==2 and not lock['host_security_relaxed'] and not lock['new_guest_execution']
replay=read(root/'replay.json')
assert replay['passed'] and not replay['new_native_execution']
assert (replay['selected_sources'],replay['artifacts'],replay['models'])==(201,94,196)
assert all(replay[key] for key in ['exact_layout_equal','exact_image_readback_equal','exact_heap_readback_equal',
    'exact_owner_readback_equal','exact_metadata_readback_equal','exact_sixteen_image_controls_equal','exact_eighteen_heap_controls_equal'])
assert b'Ran 196 tests' in (root/'replay-checks/models.log').read_bytes()
review=read(root/'review.json')
assert review['review_complete'] and review['risk']=='high'
assert not review['kernel_behavior_changed'] and not review['core_language_meaning_changed']
assert review['model_provider_safeguard_notices']==[]
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
    data=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(data),bytes=len(data))
manifest=dict(passed=True,files=files,selected_source_count=len(sources),artifact_count=len(artifacts),local_links=links,
    scope='Selected image/heap native records, source/artifact identities, failed attempts and exact frozen reader replay. Not all mapping editors, AP/DMA/asynchronous or continuous stable lifetime, full source producer or production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),selected_sources=201,artifacts=94,
    image_publications=7,image_post_invlpg_states=86,models=196)))
