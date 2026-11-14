"""Retain final callback bindings and explicitly separate initial observer scope."""
import gzip
import json
from pathlib import Path
import shutil
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest
from inspect_callback_bindings import SLOTS

source,application,source_run,app_run,post,controls,readback=map(Path,sys.argv[1:8])
root=Path('linux-kernel/tools/user-memory/results/stage-3-callback-bindings')
root.mkdir(parents=True,exist_ok=True)
cache=Path('linux-kernel/.cache/x86_64')
def copy(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    if target.name in ['callback-bindings.json', 'observations.json']:
        target.with_suffix(target.suffix+'.gz').write_bytes(gzip.compress(Path(path).read_bytes(),mtime=0))
    else:
        shutil.copyfile(path,target)
def write(name,data): (root/name).write_text(json.dumps(data,indent=2)+'\n')
native=json.loads((source_run/'run.json').read_text())
external=json.loads((app_run/'run.json').read_text())
assert all(r['exit_code']==0 and not r['source_changes_during_run'] for r in [native,external])
assert native['modified_source_hashes_before']==external['modified_source_hashes_before']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
copy(source_run/'run.json','native-run.json')
copy(source_run/'test.log','native-test.log')
copy(app_run/'run.json','application-run.json')
copy(app_run/'test.log','application-test.log')
for label,build,work in [('source',source,'source-user-extents'),('application',application,'first-image-exit')]:
    for name in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log',
                 'serial.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log']:
        copy(build/work/name,label+'/guest/'+name)
    for name in ['callback-bindings.json','process-access-coverage.json','extent-call-inventory.json',
                 'extent-call-inspection.json','transfer-guard-audit.json']:
        copy(build/name,label+'/'+name)
    for stem in ['callback-bindings','callback-root-disassembly']:
        for suffix in ['command.json','stdout.log','stderr.log']:
            copy(build/(stem+'.'+suffix),label+'/audit/'+stem+'.'+suffix)
copy(source/'source-component.json','source/source-component.json')
copy(post/'validation.json','post-validation.json')
for path in post.glob('*.log'): copy(path,'post/'+path.name)
copy(readback/'readback.json','readback.json')
copy(controls/'controls.json','controls.json')
for path in controls.glob('*/rejection.json'):
    copy(path,'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log','controls/'+path.parent.name+'.log')
for name in ['readback-callback-bindings','archive-callback-bindings','check-callback-bindings',
             'check-callback-controls','run-storage-callbacks','run-source-callbacks']:
    copy(cache/(name+'.py'),name+'.py')
copy(cache/'check-startup-source.py','post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'source-native-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'application-native-runner.py')
artifact_paths=['kernel.elf','root.o','process-access-inventory.json','baseline/provenance.json']
for _,_,key in SLOTS:
    stem=key.replace('_','-')
    artifact_paths += ['baseline/'+stem+'.'+suffix for suffix in ['o','t','modules.json']]
artifacts={}
for name in artifact_paths:
    data=(source/name).read_bytes()
    target=root/'artifacts'/(name+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    artifacts[name]=dict(bytes=len(data),sha256=digest(data),snapshot=str(target.relative_to(root)),
                         snapshot_sha256=digest(target.read_bytes()))
write('artifact-identity.json',artifacts)

old_source=cache/'user-extent-source-poll-2281940-1794404256725054590'
old_app=cache/'user-extent-application-2282401-1794404441930574421'
old_observer=cache/'callback-observer-initial.py'
observer_path='linux-kernel/labs/qemu/x86_64/user_extent_callback_observer.py'
copy(old_observer,'initial/observer.py')
for label,build,run,work in [
    ('source',old_source,cache/'tk283-source-callback-run-1794404251126355740','source-user-extents'),
    ('application',old_app,cache/'tk283-application-run-1794404440954618658','first-image-exit')]:
    record=json.loads((run/'run.json').read_text())
    assert record['exit_code']==0 and not record['source_changes_during_run']
    for path,sha in record['modified_source_hashes_before'].items():
        data=old_observer.read_bytes() if path==observer_path else Path(path).read_bytes()
        assert digest(data)==sha,path
    copy(run/'run.json','initial/'+label+'/native-run.json')
    copy(run/'test.log','initial/'+label+'/native-test.log')
    copy(build/'callback-bindings.json','initial/'+label+'/callback-bindings.json')
    for name in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log','serial.log','qemu.command.json']:
        copy(build/work/name,'initial/'+label+'/guest/'+name)
write('initial/scope.json',dict(
    scope='Passing earlier source-poll and pinned-exit runs with the frozen initial observer; not evidence for final observer bytes',
    qualification='The source-poll cohort executes 120 source syscalls and five callback sites (1064 returns). Its eight-fact anonymous-create site is unobserved. The initial observer had a narrower unused five-bit policy assertion; source review corrected it to eight Boolean facts and the exact anonymous-create plan before the final all-six-site storage and pinned-application runs.',
    source_observer_sha256=digest(old_observer.read_bytes())))

previous=json.loads((root.parent/'stage-3-process-accesses/source-identity.json').read_text())
sources=set(previous['selected_inputs'])|set(native['modified_source_hashes_before'])|{
    'src/topal-compiler/src/systems_root_encoder_entropy_qualification.rs',
    'src/topal-compiler/src/systems_artifact_operations.rs',
    'src/topal-compiler/src/systems_policy_inputs.rs',
    'src/topal-compiler/src/systems_numeric_object.rs',
    'src/topal-compiler/src/systems_library_object.rs',
    'src/topal-compiler/src/systems_library.rs',
    'src/topal-compiler/src/systems_library_modules.rs',
    'src/topal-compiler/src/systems_numeric_library.rs',
    'src/topal-compiler/src/codegen/freestanding_numeric_library.rs',
    'linux-kernel/kernel/arch/x86_64/toolchain-gate.t',
    'linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t',
}
provenance=json.loads((source/'baseline/provenance.json').read_text())
for _,_,key in SLOTS:
    record=provenance[key]
    path='linux-kernel/kernel/'+record['source']
    assert digest(Path(path).read_bytes())==record['source_sha256']
    sources.add(path)
    stem=key.replace('_','-')
    for module in json.loads((source/'baseline'/(stem+'.modules.json')).read_text()):
        path='linux-kernel/kernel/'+module['source']
        assert Path(path).read_text()==module['text']
        sources.add(path)
identities={}
for path in sorted(sources):
    data=Path(path).read_bytes()
    if path in native['modified_source_hashes_before']:
        assert digest(data)==native['modified_source_hashes_before'][path],path
    target=root/'inputs'/(path+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[path]=dict(sha256=digest(data),snapshot=str(target.relative_to(root)))
write('source-identity.json',dict(base_commit='eb8389d19a42d526aecb1ffc35d7f6d94dc7860f',
    selected_inputs=identities,scope='Selected source/native/controller/root and authority snapshots; not complete producer or cold reconstruction'))
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['complete callee/exception/alternate-entry closure and ownership',
        'all direct/shared user-pointer classification, admission dominance and stable access lifetime',
        'all caller length/error/staging/cleanup and native mode/stale-state/ownership qualification',
        'three fallback coverage/reachability obligations',
        'canonical production binding and exact committed cold reconstruction',
        'all remaining original MM/distribution/device/driver/container/VM-host acceptance']))
write('exploration.json',dict(
    scope='Host-side development/readback corrections, separate from native results and AI safeguards',
    corrections=['The first offline relocation reader looked up anonymous section symbols by name; exact symbol-table indices now preserve their identity.',
        'A host readback comparison now separately verifies each cohort baseline provenance digest: application inputs differ although selected root, kernel, libraries and controller bindings match.',
        'A readback-only Python comparison required parentheses around a Boolean negation.'],
    native_failures=0,safeguard_denials=0))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(identities))))
