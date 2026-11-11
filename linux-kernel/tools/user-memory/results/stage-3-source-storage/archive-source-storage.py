"""Retain selected source-built storage evidence and input identities."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import sys

build,run,post,readback=map(Path,sys.argv[1:5])
root=Path('linux-kernel/tools/user-memory/results/stage-3-source-storage')
root.mkdir(parents=True,exist_ok=True)
cache=Path('linux-kernel/.cache/x86_64')
digest=lambda data:hashlib.sha256(data).hexdigest()
def copy(source,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source,target)
def write(name,data): (root/name).write_text(json.dumps(data,indent=2)+'\n')
record=json.loads((run/'run.json').read_text())
assert record['exit_code']==0 and not record['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
for name in ['extent-call-inventory.json','extent-call-inspection.json','source-component.json']:
    copy(build/name,name)
copy(run/'run.json','native-run.json')
copy(run/'test.log','native-test.log')
for name in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log',
             'serial.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log']:
    copy(build/'source-user-extents'/name,'guest/'+name)
copy(post/'validation.json','post-validation.json')
for path in post.glob('*.log'): copy(path,'post/'+path.name)
copy(readback/'readback.json','readback.json')
for name in ['readback-source-storage','archive-source-storage','check-source-storage','run-source-storage']:
    copy(cache/(name+'.py'),name+'.py')
copy(cache/'check-startup-source.py','post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'native-runner-base.py')
for stem in ['source','startup','file','shell','command','storage']:
    for suffix in ['c','su']: copy(build/f'{stem}-cases.{suffix}',f'{stem}-cases.{suffix}')
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(build/f'{stem}-cases-compile.{suffix}',f'{stem}-cases-compile.{suffix}')
for stem in ['source-link','source-cc-version']:
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(build/f'{stem}.{suffix}',f'{stem}.{suffix}')
copy(build/'source-user.ld','source-user.ld')
previous=json.loads((root.parent/'stage-3-source-pipes/source-identity.json').read_text())
sources=set(previous['selected_inputs'])|set(record['modified_source_hashes_before'])|{
    f'src/topal-compiler/src/{name}.rs' for name in ['systems_volatile_files','systems_volatile_create',
                                                 'systems_namespace_entries','systems_ram_page_region']}
identities={}
for source in sorted(sources):
    data=Path(source).read_bytes()
    if source in record['modified_source_hashes_before']:
        assert digest(data)==record['modified_source_hashes_before'][source],source
    target=root/'inputs'/(source+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[source]=dict(sha256=digest(data),snapshot=str(target.relative_to(root)))
write('source-identity.json',dict(base_commit='3423d2c40ce374fb3acb9ae5a4f4abce209a4078',
    selected_inputs=identities,scope='Selected fixture, observer, helper/caller and authority snapshots; not the complete producer or cold reconstruction'))
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['four sites unobserved in the selected three-cohort union',
        'complete direct/shared user-access inventory and dominance',
        'every caller length/direction/error/lifetime obligation',
        'native ownership/mode/stability qualification',
        'canonical production binding and exact committed cold reconstruction',
        'all remaining original kernel acceptance']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(root)
