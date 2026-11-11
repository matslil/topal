"""Retain selected REP proof and ordinary native cohorts without snapshot copies."""
import gzip
import json
from pathlib import Path
import posixpath
import shutil
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

catalog=json.loads(Path(sys.argv[1]).read_text())
root=Path('linux-kernel/tools/user-memory/results/stage-3-copy-transfers')
root.mkdir(parents=True,exist_ok=True)
cache=Path('linux-kernel/.cache/x86_64')
def copy(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(path,target)
def write(name,data):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_text(json.dumps(data,indent=2)+'\n')
def compress(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(Path(path).read_bytes(),mtime=0))

first=catalog['cohorts'][0]
build=Path(first['build'])
run=json.loads((Path(first['run'])/'run.json').read_text())
prior=root.parent/'stage-3-fixed-transfers'
previous=json.loads((prior/'source-identity.json').read_text())['selected_inputs']
selected={}
for path in sorted(set(previous)|set(run['modified_source_hashes_before'])):
    data=Path(path).read_bytes()
    if path in run['modified_source_hashes_before']:
        assert digest(data)==run['modified_source_hashes_before'][path]
        snapshot=Path('inputs')/(path+'.gz')
        compress(path,str(snapshot))
    else:
        row=previous[path]
        snapshot=Path(posixpath.normpath(str(Path('../stage-3-fixed-transfers')/row['snapshot'])))
        assert row['sha256']==digest(data) and gzip.decompress((root/snapshot).read_bytes())==data
    selected[path]=dict(sha256=digest(data),snapshot=str(snapshot))
write('source-identity.json',dict(base_commit='59f1ab2168e434fac22e4e5d710ec5cd1b3a5533',
    inherited_record='../stage-3-fixed-transfers/source-identity.json',
    inherited_record_sha256=digest((prior/'source-identity.json').read_bytes()),
    selected_inputs=selected,scope='Selected producer/authority/observer/reader bytes with explicit inherited snapshots; not the complete producer'))
artifacts=json.loads((prior/'artifact-identity.json').read_text())
for name,row in artifacts.items():
    row['snapshot']=posixpath.normpath(str(Path('../stage-3-fixed-transfers')/row['snapshot']))
    assert digest((build/name).read_bytes())==row['sha256']
write('artifact-identity.json',artifacts)
copy(build/'copy-transfer-audit.json','copy-transfer-audit.json')
union=set()
native_summary=[]
for cohort in catalog['cohorts']:
    label=cohort['label']
    b,r,readback=map(Path,[cohort['build'],cohort['run'],cohort['readback']])
    native=json.loads((r/'run.json').read_text())
    assert native['exit_code']==0 and not native['source_changes_during_run']
    assert native['modified_source_hashes_before']==run['modified_source_hashes_before']
    assert json.loads((readback/'readback.json').read_text())['passed']
    assert (b/'copy-transfer-audit.json').read_bytes()==(build/'copy-transfer-audit.json').read_bytes()
    copy(r/'run.json',label+'/native-run.json')
    copy(r/'test.log',label+'/native-test.log')
    guest=b/('first-image-exit' if label=='application' else 'source-user-extents')
    observations=json.loads((guest/'observations.json').read_text())
    assert observations['passed'] and observations['copy_transfer_observation']['passed']
    observed=observations['copy_transfer_observation']
    image=(b/'static-elf-abi-fixture.elf').read_bytes()
    write(label+'/image-identity.json',dict(bytes=len(image),sha256=digest(image),
        boot_resource_sha256=digest((b/'initramfs.cpio').read_bytes()),
        scope='Actual unchanged application/boot identities; pinned third-party bytes remain outside Git'))
    if label!='application':
        component=json.loads((b/'source-component.json').read_text())
        assert component['image_sha256']==digest(image)
        copy(b/'source-component.json',label+'/source-component.json')
        compress(b/'static-elf-abi-fixture.elf',label+'/source-user-image.elf.gz')
    union.update(row['name'] for row in observed['samples'])
    for name in ['inspect.gdb','gdb.stdout.log','gdb.stderr.log','qemu.command.json',
                 'qemu.stdout.log','qemu.stderr.log','serial.log']:
        copy(guest/name,label+'/guest/'+name)
    compress(guest/'observations.json',label+'/observations.json.gz')
    write(label+'/observation-identity.json',dict(bytes=(guest/'observations.json').stat().st_size,
        sha256=digest((guest/'observations.json').read_bytes())))
    copy(readback/'readback.json',label+'/readback.json')
    for path in readback.iterdir():
        if path.name!='readback.json':
            copy(path,label+'/gnu/'+path.name)
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(b/('copy-transfer-audit.'+suffix),label+'/audit/'+suffix)
    native_summary.append(dict(label=label,counts=observed['counts'],unreached=observed['unreached'],
        native_extent_first_reach=observations['extent_call_coverage']['reached_sites'],
        callback_returns=sum(r['count'] for r in observations['callback_observation']['counts'])))
write('native-summary.json',dict(passed=True,cohorts=native_summary,reached_case_union=sorted(union),
    unreached_case_union=sorted({r['name'] for r in json.loads((build/'copy-transfer-audit.json').read_text())['cases']}-union)))
post=Path(catalog['post'])
assert json.loads((post/'validation.json').read_text())['passed']
copy(post/'validation.json','post-validation.json')
for path in post.glob('*.log'):
    copy(path,'post/'+path.name)
controls=Path(catalog['controls'])
assert json.loads((controls/'controls.json').read_text())['passed']
copy(controls/'controls.json','controls.json')
for path in controls.glob('*/rejection.json'):
    copy(path,'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log','controls/'+path.parent.name+'.log')
failed=catalog['failed']
fb,fr=map(Path,[failed['build'],failed['run']])
failure=json.loads((fr/'run.json').read_text())
assert failure['exit_code']!=0 and not failure['source_changes_during_run']
assert (fb/'kernel.elf').read_bytes()==(build/'kernel.elf').read_bytes()
copy(fr/'run.json','initial-observer/native-run.json')
copy(fr/'test.log','initial-observer/native-test.log')
copy(cache/'copy-observer-initial.py','initial-observer/observer.py')
for path,sha in failure['modified_source_hashes_before'].items():
    data=(cache/'copy-observer-initial.py').read_bytes() if path.endswith('/user_extent_copy_observer.py') else (
        (cache/'copy-tests-initial.py').read_bytes() if path.endswith('/test_copy_transfers.py') else Path(path).read_bytes())
    assert digest(data)==sha,path
for name in ['inspect.gdb','gdb.stdout.log','gdb.stderr.log','observations.json','serial.log','qemu.command.json']:
    copy(fb/'first-image-exit'/name,'initial-observer/guest/'+name)
write('initial-observer/failure.json',dict(passed=False,
    cause='REP start breakpoint was revisited between iterations while a copy was active',
    correction='Disable the start breakpoint after the initial snapshot and re-enable it at completion; no guest code/state or deadline change',
    scope='Observer failure, not kernel/resource failure or an AI safeguard notice; original source hashes, failed result and guest output retained'))
copy(cache/'copy-observer-boundaries-v2.py','history/observer-v2.py')
copy(cache/'copy-tests-initial.py','history/tests-v1.py')
for attempt in catalog['history']:
    b,r=map(Path,[attempt['build'],attempt['run']])
    label=attempt['label']
    guest=b/('first-image-exit' if attempt['application'] else 'source-user-extents')
    assert (b/'kernel.elf').read_bytes()==(build/'kernel.elf').read_bytes()
    copy(r/'test.log','history/'+label+'/native-test.log')
    if (r/'run.json').exists():
        record=json.loads((r/'run.json').read_text())
        for path,sha in record['modified_source_hashes_before'].items():
            data=(cache/'copy-observer-boundaries-v2.py').read_bytes() if path.endswith('/user_extent_copy_observer.py') else (
                (cache/'copy-tests-initial.py').read_bytes() if path.endswith('/test_copy_transfers.py') else Path(path).read_bytes())
            assert digest(data)==sha,path
        copy(r/'run.json','history/'+label+'/native-run.json')
    else:
        assert attempt['status']=='stopped'
    for name in ['inspect.gdb','gdb.stdout.log','gdb.stderr.log','serial.log','qemu.command.json']:
        if (guest/name).exists():
            copy(guest/name,'history/'+label+'/guest/'+name)
    if (guest/'observations.json').exists():
        compress(guest/'observations.json','history/'+label+'/observations.json.gz')
    write('history/'+label+'/scope.json',dict(status=attempt['status'],reason=attempt['reason'],
        process_exit_code=attempt.get('process_exit_code'),
        observer_override='../observer-v2.py',tests_override='../tests-v1.py',
        scope='Preceding diagnostic cohort, not final-source qualification; unchanged kernel bytes verified'))
initial_readback=cache/'copy-transfer-poll-readback'
for path in initial_readback.iterdir():
    copy(path,'initial-readback/'+path.name)
write('initial-readback/failure.json',dict(passed=False,exit_code=1,
    cause='The preceding source-poll build predates process-access-origins.json; the initial reader named that absent metadata file',
    correction='Compare mode-specific executable/application/boot/console bytes with their preceding source cohort and compare graph/origin metadata with the explicitly verified byte-identical common-kernel application baseline',
    scope='Readback path-selection failure, not native failure or a safeguard notice; original GNU outputs retained, Python traceback reported in the session but not captured as a log'))
for name in ['archive-copy-transfers','readback-copy-transfers','check-copy-transfers','check-copy-transfer-controls']:
    copy(cache/(name+'.py'),name+'.py')
for name in ['check-fixed-transfers','check-startup-source','run-source-poll','run-source-storage']:
    copy(cache/(name+'.py'),name+'.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'native-application-runner.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'native-source-runner.py')
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['all other scalar/indexed/string access regions and complete syscall effects/error ordering',
        'continuous check-to-copy lifetime, complete table/backing and counterparty ownership',
        'remaining helper/root/callee/exception and native mode/stale-state obligations',
        'three fallback obligations, canonical production binding and committed cold reconstruction',
        'the complete original kernel/distribution/device/driver/container/VM-host requirements']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
# Keep every original GNU command/output byte with per-entry identities while
# avoiding hundreds of duplicate small files in the review. Repacking removes
# only files just generated above and only after exact decompression equality.
for directory in [root/c['label']/'gnu' for c in catalog['cohorts']]+[root/'initial-readback']:
    paths=sorted(p for p in directory.iterdir() if p.name.endswith(('.command.json','.stdout.log','.stderr.log')))
    records=[dict(name=p.name,bytes=p.stat().st_size,sha256=digest(p.read_bytes()),text=p.read_bytes().decode()) for p in paths]
    target=directory.parent/(directory.name+'-records.json.gz')
    target.write_bytes(gzip.compress((json.dumps(records,indent=2)+'\n').encode(),mtime=0))
    recovered=json.loads(gzip.decompress(target.read_bytes()))
    for path,row in zip(paths,recovered,strict=True):
        data=row['text'].encode()
        assert data==path.read_bytes() and digest(data)==row['sha256'] and len(data)==row['bytes']
        path.unlink()
    if not list(directory.iterdir()):
        directory.rmdir()
for path in list(root.rglob('inspect.gdb')):
    data=path.read_bytes()
    target=path.with_suffix('.gdb.gz')
    target.write_bytes(gzip.compress(data,mtime=0))
    assert gzip.decompress(target.read_bytes())==data
    path.unlink()
print(json.dumps(dict(archive=str(root),selected_inputs=len(selected),reached_case_union=len(union))))
