"""Freeze staged-transfer evidence with explicit inherited source/artifact bytes."""
import gzip
import json
from pathlib import Path
import posixpath
import shutil
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

catalog=json.loads(Path(sys.argv[1]).read_text())
root=Path('linux-kernel/tools/user-memory/results/stage-3-staged-transfers')
root.mkdir(parents=True,exist_ok=True)
cache=Path('linux-kernel/.cache/x86_64')
prior=root.parent/'stage-3-copy-transfers'
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
        snapshot=Path(posixpath.normpath(str(Path('../stage-3-copy-transfers')/row['snapshot'])))
        assert row['sha256']==digest(data) and gzip.decompress((root/snapshot).read_bytes())==data
    selected[path]=dict(sha256=digest(data),snapshot=str(snapshot))
write('source-identity.json',dict(base_commit='2dd67214e75caf9979e49c59fc6736307e41687a',
    inherited_record='../stage-3-copy-transfers/source-identity.json',
    inherited_record_sha256=digest((prior/'source-identity.json').read_bytes()),
    selected_inputs=selected,scope='Selected authority/producer/reader/observer inputs, not the complete producer'))
artifacts=json.loads((prior/'artifact-identity.json').read_text())
for name,row in artifacts.items():
    row['snapshot']=posixpath.normpath(str(Path('../stage-3-copy-transfers')/row['snapshot']))
    assert digest((build/name).read_bytes())==row['sha256']
write('artifact-identity.json',artifacts)
compress(build/'staged-transfer-audit.json','staged-transfer-audit.json.gz')
data=(build/'staged-transfer-audit.json').read_bytes()
audit=json.loads(data)
write('staged-audit-identity.json',dict(bytes=len(data),sha256=digest(data),snapshot='staged-transfer-audit.json.gz'))
summary={key:value for key,value in audit.items() if key not in ['cases','locations']}
summary['source_locations']=len(audit['locations'])
summary['cases']=[dict({key:value for key,value in row.items() if key not in ['preparation','transfer','user_accesses']},
    preparation_instructions=len(row['preparation']),transfer_instructions=len(row['transfer']),
    user_access_instructions=len(row['user_accesses'])) for row in audit['cases']]
write('staged-summary.json',summary)
prior_audits={}
local_by_call={}
instructions=set()
for name,key,entry in [('fixed-transfer-audit.json','access_instructions','stage-3-fixed-transfers'),
                       ('copy-transfer-audit.json','copy_offset','stage-3-copy-transfers')]:
    data=(build/name).read_bytes()
    snapshot='../'+entry+'/'+name
    assert data==(root/snapshot).read_bytes()
    prior_audits[name]=dict(snapshot=snapshot,bytes=len(data),sha256=digest(data),native_generated_equal=True)
    for row in json.loads(data)['cases']:
        local_by_call[row['call_offset']]=dict(call_offset=row['call_offset'],name=row['name'],audit=name)
        if key=='copy_offset':
            instructions.add(row[key])
        else:
            instructions.update(r['offset'] for r in row[key])
for row in audit['cases']:
    local_by_call[row['call_offset']]=dict(call_offset=row['call_offset'],name=row['name'],audit='staged-transfer-audit.json.gz')
    instructions.update(r['offset'] for r in row['user_accesses'])
census=json.loads((build/'extent-call-inventory.json').read_text())
assert len(local_by_call)==40 and len(instructions)==249
write('prior-access-audits.json',prior_audits)
write('caller-access-index.json',dict(complete=False,kernel_sha256=audit['kernel_sha256'],
    emitted_helper_calls=len(census['calls']),calls_with_selected_local_intervals=len(local_by_call),
    distinct_user_access_instructions=len(instructions),
    selected_intervals=[local_by_call[at] for at in sorted(local_by_call)],
    calls_without_selected_local_intervals=[r for r in census['calls'] if r['offset'] not in local_by_call],
    scope='Identity-bound index of conditional local intervals only, not whole-caller coverage or safety/lifetime qualification; other accesses may remain after every listed interval'))
cohort_summary=[]
for cohort in catalog['cohorts']:
    label=cohort['label']
    b,r,readback=map(Path,[cohort['build'],cohort['run'],cohort['readback']])
    native=json.loads((r/'run.json').read_text())
    assert native['exit_code']==0 and not native['source_changes_during_run']
    assert native['modified_source_hashes_before']==run['modified_source_hashes_before']
    assert json.loads((readback/'readback.json').read_text())['passed']
    assert (b/'staged-transfer-audit.json').read_bytes()==(build/'staged-transfer-audit.json').read_bytes()
    guest=b/('first-image-exit' if label=='application' else 'source-user-extents')
    observed=json.loads((guest/'observations.json').read_text())
    assert observed['passed'] and observed['staged_transfer_observation']['passed']
    copy(r/'run.json',label+'/native-run.json')
    copy(r/'test.log',label+'/native-test.log')
    for name in ['gdb.stdout.log','gdb.stderr.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log','serial.log']:
        copy(guest/name,label+'/guest/'+name)
    compress(guest/'inspect.gdb',label+'/guest/inspect.gdb.gz')
    compress(guest/'observations.json',label+'/observations.json.gz')
    write(label+'/observation-identity.json',dict(bytes=(guest/'observations.json').stat().st_size,
        sha256=digest((guest/'observations.json').read_bytes())))
    copy(readback/'readback.json',label+'/readback.json')
    records=[]
    for path in sorted(readback.iterdir()):
        if path.name!='readback.json':
            data=path.read_bytes()
            records.append(dict(name=path.name,bytes=len(data),sha256=digest(data),text=data.decode()))
    bundle=gzip.compress((json.dumps(records,indent=2)+'\n').encode(),mtime=0)
    (root/label/'gnu-records.json.gz').write_bytes(bundle)
    assert json.loads(gzip.decompress(bundle))==records
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(b/('staged-transfer-audit.'+suffix),label+'/audit/'+suffix)
    image=(b/'static-elf-abi-fixture.elf').read_bytes()
    identity=dict(bytes=len(image),sha256=digest(image),boot_resource_sha256=digest((b/'initramfs.cpio').read_bytes()))
    if label=='storage':
        snapshot='../stage-3-copy-transfers/storage/source-user-image.elf.gz'
        assert gzip.decompress((root/snapshot).read_bytes())==image
        identity['first_party_snapshot']=snapshot
        copy(b/'source-component.json',label+'/source-component.json')
    write(label+'/image-identity.json',identity)
    details=observed['staged_transfer_observation']
    cohort_summary.append(dict(label=label,counts=details['counts'],unreached=details['unreached'],
        helper_first_reach=observed['extent_call_coverage']['reached_sites'],
        callback_returns=sum(x['count'] for x in observed['callback_observation']['counts']),
        preceding_rep_copies=sum(x['completed'] for x in observed['copy_transfer_observation']['counts'].values())))
write('native-summary.json',dict(passed=True,cohorts=cohort_summary))
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
for name in ['archive-staged-transfers','readback-staged-transfers','check-staged-transfers','check-staged-transfer-controls']:
    copy(cache/(name+'.py'),name+'.py')
for name in ['check-fixed-transfers','check-startup-source','run-source-storage']:
    copy(cache/(name+'.py'),name+'.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'native-application-runner.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'native-source-runner.py')
write('development-note.json',dict(initial_reader_exit_code=1,
    reported_error="unsupported GS preparation: 6548890425e8020000",
    correction='The existing preparation also stores RAX with MOV, in addition to immediate stores. Add its exact GS scalar effect before unit/native qualification',
    scope='Reader construction before source freeze/native validation; original failure reported in the session, not captured as a separate log or AI safeguard notice'))
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['native invalid-buffer/refusal cases for these staged regions',
        'all other access classes and complete syscall effect/error ordering',
        'complete GS/captured entry/counterpart/table/backing owners and continuous access lifetime',
        'remaining helper/root/callee/exception and native mode/stale-state obligations',
        'three fallback obligations, canonical binding and exact committed cold reconstruction',
        'the original full kernel/distribution/device/driver/container/VM-host requirements']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(selected),local_caller_intervals=len(local_by_call))))
