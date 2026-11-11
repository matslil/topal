"""Freeze the selected emission/access index, source bytes and native witness."""
import gzip
import json
import os
from pathlib import Path
import shutil
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build,run,post,controls,readback=map(Path,sys.argv[1:6])
root=Path('linux-kernel/tools/user-memory/results/stage-3-process-origins')
root.mkdir(parents=True,exist_ok=True)
cache=Path('linux-kernel/.cache/x86_64')
def copy(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(path,target)
def write(name,data): (root/name).write_text(json.dumps(data,indent=2)+'\n')
native=json.loads((run/'run.json').read_text())
assert native['exit_code']==0 and not native['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((controls/'controls.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
joined=json.loads((build/'process-access-origins.json').read_text())
copy(run/'run.json','native-run.json')
copy(run/'test.log','native-test.log')
for name in ['inspect.gdb','gdb.stdout.log','gdb.stderr.log','serial.log',
             'qemu.command.json','qemu.stdout.log','qemu.stderr.log']:
    copy(build/'first-image-exit'/name,'guest/'+name)
for name in ['extent-call-inventory.json','extent-call-inspection.json',
             'transfer-guard-audit.json','process-access-coverage.json']:
    copy(build/name,name)
for stem in ['process-access-origins','process-access-inventory']:
    for suffix in ['command.json','stdout.log','stderr.log']:
        copy(build/(stem+'.'+suffix),'audit/'+stem+'.'+suffix)
artifacts={}
for name in ['provider.o','kernel.elf','process-access-inventory.json',
             'process-byte-origins.json','process-access-origins.json',
             'callback-bindings.json','first-image-exit/observations.json']:
    data=(build/name).read_bytes()
    target=root/'artifacts'/(name+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    artifacts[name]=dict(bytes=len(data),sha256=digest(data),snapshot=str(target.relative_to(root)),
                         snapshot_sha256=digest(target.read_bytes()))
write('artifact-identity.json',artifacts)
copy(post/'validation.json','post-validation.json')
for path in post.glob('*.log'): copy(path,'post/'+path.name)
copy(controls/'controls.json','controls.json')
for path in controls.glob('*/rejection.json'):
    copy(path,'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log','controls/'+path.parent.name+'.log')
copy(readback/'readback.json','readback.json')
for name in ['archive-process-origins','readback-process-origins',
             'check-process-origins','check-process-origin-controls']:
    copy(cache/(name+'.py'),name+'.py')
copy(cache/'check-startup-source.py','post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'native-runner.py')
previous=json.loads((root.parent/'stage-3-callback-bindings/source-identity.json').read_text())
sources=set(previous['selected_inputs'])|set(native['modified_source_hashes_before'])|set(joined['source_hashes'])
identities={}
for path in sorted(sources):
    data=Path(path).read_bytes()
    if path in native['modified_source_hashes_before']:
        assert digest(data)==native['modified_source_hashes_before'][path],path
    if path in joined['source_hashes']:
        assert digest(data)==joined['source_hashes'][path],path
    target=root/'inputs'/(path+'.gz')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[path]=dict(sha256=digest(data),snapshot=str(target.relative_to(root)))
write('source-identity.json',dict(base_commit='e7a30a29613df55fbacbb814398d78cb2333c05a',
    selected_inputs=identities,scope='Selected emitter/reader/caller/authority bytes and all listed writer files; not complete producer or cold reconstruction'))
summary={key:value for key,value in joined.items() if key not in ['locations','accesses']}
summary['locations']=len(joined['locations'])
write('origin-summary.json',summary)
lines=['# Selected writer index','',
    'Opcode-writer counts for the selected direct-control graph. Categories are',
    'syntactic and overlap; they are not a memory-authority or admission proof.',
    'Full spans and source excerpts are retained in compressed artifacts.','',
    '| Emitter source | Potential-access instructions | General addressed | GS-based | Stack-base | Implicit stack |',
    '| --- | ---: | ---: | ---: | ---: | ---: |']
for row in joined['groups']:
    kinds=row['access_kind_counts']
    path=row['file']
    link=os.path.relpath(path,root)
    lines.append('| ['+Path(path).name+']('+link+') | '+str(row['primary_access_instructions'])+' | '+
        ' | '.join(str(kinds.get(kind,0)) for kind in ['general-address-operand','gs-based','stack-base-operand','implicit-stack'])+' |')
(root/'writer-index.md').write_text('\n'.join(lines)+'\n')
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['semantic classification and actual pointer/owner origins for every access',
        'complete admission dominance, direction/length/error/staging/cleanup and lifetime proof',
        'other roots/callees/exception entry and unvisited instruction coverage',
        'three fallback coverage/reachability and native ownership/mode/stale-state obligations',
        'canonical production binding and exact committed cold reconstruction',
        'all remaining original MM/distribution/device/driver/container/VM-host acceptance']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(identities))))
