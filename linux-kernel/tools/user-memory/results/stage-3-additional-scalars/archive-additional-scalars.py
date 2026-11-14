"""Freeze the additional scalar proof and explicitly inherited source/artifacts."""
import gzip
import hashlib
import json
from pathlib import Path
import posixpath
import shutil
import sys

catalog=json.loads(Path(sys.argv[1]).read_text())
root=Path('linux-kernel/tools/user-memory/results/stage-3-additional-scalars')
prior=root.parent/'stage-3-staged-refusals'
build,run,post,readback,controls=map(Path,[catalog[k] for k in ['build','run','post','readback','controls']])
native=json.loads((run/'run.json').read_text())
assert native['exit_code']==0 and not native['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
assert json.loads((controls/'controls.json').read_text())['passed']
root.mkdir(parents=True,exist_ok=True)
digest=lambda data:hashlib.sha256(data).hexdigest()
def copy(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(path,target)
def write(name,value):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_text(json.dumps(value,indent=2)+'\n')
def compress(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(Path(path).read_bytes(),mtime=0))
previous=json.loads((prior/'source-identity.json').read_text())['selected_inputs']
selected={}
for path in sorted(set(previous)|set(native['modified_source_hashes_before'])):
    data=Path(path).read_bytes()
    if path in native['modified_source_hashes_before']:
        assert digest(data)==native['modified_source_hashes_before'][path]
        snapshot='inputs/'+path+'.gz'
        compress(path,snapshot)
    else:
        row=previous[path]
        snapshot=posixpath.normpath('../stage-3-staged-refusals/'+row['snapshot'])
        assert digest(data)==row['sha256'] and gzip.decompress((root/snapshot).read_bytes())==data
    selected[path]=dict(sha256=digest(data),snapshot=snapshot)
write('source-identity.json',dict(base_commit='582dace41a508702d77ad3dbc7360f949136349b',
    inherited_record='../stage-3-staged-refusals/source-identity.json',
    inherited_record_sha256=digest((prior/'source-identity.json').read_bytes()),selected_inputs=selected,
    scope='Selected authority/producer/reader/observer inputs, not a full producer reconstruction'))
artifacts=json.loads((prior/'artifact-identity.json').read_text())
for name,row in artifacts.items():
    row['snapshot']=posixpath.normpath('../stage-3-staged-refusals/'+row['snapshot'])
    assert digest((build/name).read_bytes())==row['sha256']
write('artifact-identity.json',artifacts)
data=(build/'additional-scalar-audit.json').read_bytes()
audit=json.loads(data)
compress(build/'additional-scalar-audit.json','additional-scalar-audit.json.gz')
write('audit-identity.json',dict(bytes=len(data),sha256=digest(data),snapshot='additional-scalar-audit.json.gz'))
summary={k:v for k,v in audit.items() if k!='cases'}
summary['cases']=[dict({k:v for k,v in r.items() if k not in ['prelude','preparation','local_instructions','user_accesses']},
    scalar_accesses=len(r['user_accesses']),preparation_instructions=len(r['preparation'])) for r in audit['cases']]
write('summary.json',summary)
old_index=root.parent/'stage-3-staged-transfers/caller-access-index.json'
index=json.loads(old_index.read_text())
selected_calls={r['call_offset']:dict(r,prior_index='../stage-3-staged-transfers/caller-access-index.json') for r in index['selected_intervals']}
accesses=set()
for name,field in [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),('staged-transfer-audit.json','user_accesses')]:
    for case in json.loads((build/name).read_text())['cases']:
        accesses.update([case[field]] if field=='copy_offset' else [a['offset'] for a in case[field]])
assert len(accesses)==249 and len(selected_calls)==40
for case in audit['cases']:
    assert case['call_offset'] not in selected_calls
    selected_calls[case['call_offset']]=dict(name=case['name'],call_offset=case['call_offset'],audit='additional-scalar-audit.json.gz')
    accesses.update(a['offset'] for a in case['user_accesses'])
census=json.loads((build/'extent-call-inventory.json').read_text())
assert len(selected_calls)==73 and len(accesses)==300
write('caller-access-index.json',dict(complete=False,kernel_sha256=audit['kernel_sha256'],
    emitted_helper_calls=91,calls_with_selected_local_intervals=73,distinct_user_access_instructions=300,
    selected_intervals=[selected_calls[at] for at in sorted(selected_calls)],
    calls_without_selected_local_intervals=[r for r in census['calls'] if r['offset'] not in selected_calls],
    scope='Conditional selected local intervals only, not whole callers or all accesses; other accesses and complete owners/lifetime remain unqualified'))
copy(run/'run.json','native-run.json')
copy(run/'test.log','native-test.log')
copy(post/'validation.json','post-validation.json')
for path in post.glob('*.log'): copy(path,'post/'+path.name)
copy(readback/'readback.json','readback.json')
records=[]
for path in sorted(readback.iterdir()):
    if path.name!='readback.json':
        data=path.read_bytes()
        records.append(dict(name=path.name,bytes=len(data),sha256=digest(data),text=data.decode()))
(root/'gnu-records.json.gz').write_bytes(gzip.compress((json.dumps(records,indent=2)+'\n').encode(),mtime=0))
copy(controls/'controls.json','controls.json')
for path in controls.glob('*/rejection.json'):
    copy(path,'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log','controls/'+path.parent.name+'.log')
guest=build/'first-image-exit'
for name in ['gdb.stdout.log','gdb.stderr.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log','serial.log']:
    copy(guest/name,'guest/'+name)
for name in ['inspect.gdb','observations.json']: compress(guest/name,'guest/'+name+'.gz')
image=(build/'static-elf-abi-fixture.elf').read_bytes()
write('image-identity.json',dict(bytes=len(image),sha256=digest(image),boot_resource_sha256=digest((build/'initramfs.cpio').read_bytes())))
for name in ['command.json','stdout.log','stderr.log']: copy(build/('additional-scalar-audit.'+name),'audit/'+name)
for name in ['archive-additional-scalars','readback-additional-scalars','check-additional-scalar-controls','check-additional-scalars',
    'check-staged-transfers','check-fixed-transfers','check-startup-source','replay-additional-scalars','finalize-additional-scalars']:
    copy(Path('linux-kernel/.cache/x86_64')/(name+'.py'),name+'.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'native-application-runner.py')
copy(Path(sys.argv[1]),'catalog.json')
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(selected),local_intervals=73,distinct_accesses=300)))
