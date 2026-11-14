"""Retain source-built FIFO evidence and superseded attempts distinctly."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import sys

build, run, post, readback = map(Path,sys.argv[1:5])
root=Path('linux-kernel/tools/user-memory/results/stage-3-source-pipes')
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
for name in ['readback-source-pipes','archive-source-pipes','check-source-pipes','run-source-pipes']:
    copy(cache/(name+'.py'),name+'.py')
copy(cache/'check-startup-source.py','post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'native-runner-base.py')
for stem in ['source','startup','file','shell','command','pipe']:
    for suffix in ['c','su']: copy(build/f'{stem}-cases.{suffix}',f'{stem}-cases.{suffix}')
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(build/f'{stem}-cases-compile.{suffix}',f'{stem}-cases-compile.{suffix}')
for stem in ['source-link','source-cc-version']:
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(build/f'{stem}.{suffix}',f'{stem}.{suffix}')
copy(build/'source-user.ld','source-user.ld')
attempts=[('compile','1794398636271155241',None),
          ('timestamp-observer','1794398678846975300','user-extent-source-pipes-2270196-1794398679803153819'),
          ('before-window-review','1794398763216392836','user-extent-source-pipes-2270539-1794398764192615193')]
for label,stamp,guest in attempts:
    earlier=cache/('tk283-source-pipes-run-'+stamp)
    data=json.loads((earlier/'run.json').read_text())
    assert not data['source_changes_during_run']
    assert (data['exit_code']==0)==(label=='before-window-review')
    copy(earlier/'run.json','earlier-attempts/'+label+'/run.json')
    copy(earlier/'test.log','earlier-attempts/'+label+'/test.log')
    if guest:
        for member in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log','serial.log']:
            copy(cache/guest/'source-user-extents'/member,'earlier-attempts/'+label+'/guest/'+member)
for name,label in [('pipe-observer-before-timestamp-fix.py','timestamp-observer'),
                   ('pipe-observer-before-window-review.py','before-window-review')]:
    copy(cache/name,'earlier-attempts/'+label+'/observer.py')
    earlier=json.loads((root/'earlier-attempts'/label/'run.json').read_text())
    assert digest((root/'earlier-attempts'/label/'observer.py').read_bytes())==earlier['modified_source_hashes_before']['linux-kernel/labs/qemu/x86_64/user_extent_pipe_observer.py']
copy(cache/'user-extent-source-pipes-2270049-1794398645244650647/pipe-cases.c',
     'earlier-attempts/compile/pipe-cases.c')
earlier=json.loads((root/'earlier-attempts/compile/run.json').read_text())
assert digest((root/'earlier-attempts/compile/pipe-cases.c').read_bytes())==earlier['modified_source_hashes_before']['linux-kernel/labs/qemu/x86_64/user_extent_pipe_cases.c']
previous=json.loads((root.parent/'stage-3-call-census/source-identity.json').read_text())
sources=set(previous['selected_inputs'])|set(record['modified_source_hashes_before'])|{
    'src/topal-compiler/src/systems_pipe_wait.rs'}
identities={}
for source in sorted(sources):
    data=Path(source).read_bytes()
    if source in record['modified_source_hashes_before']:
        assert digest(data)==record['modified_source_hashes_before'][source],source
    target=root/'inputs'/(source+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[source]=dict(sha256=digest(data),snapshot=str(target.relative_to(root)))
write('source-identity.json',dict(base_commit='090faa77c4d4708d344f7bac5eb724ff77f12174',
    selected_inputs=identities,scope='Selected fixture, observer, helper/caller and authority snapshots; not the complete producer or cold reconstruction'))
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['ten sites unobserved in the selected two-cohort union',
        'complete direct/shared user-access inventory and dominance',
        'every caller length/direction/error/lifetime obligation',
        'native ownership/mode/stability qualification',
        'canonical production binding and exact committed cold reconstruction',
        'all remaining original kernel acceptance']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(root)
