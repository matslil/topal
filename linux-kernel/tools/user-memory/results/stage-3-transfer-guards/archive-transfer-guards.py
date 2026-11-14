"""Archive selected guard-audit, unchanged application and resource evidence."""
import gzip
import json
from pathlib import Path
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build, run, post, readback = map(Path, sys.argv[1:5])
root = Path('linux-kernel/tools/user-memory/results/stage-3-transfer-guards')
root.mkdir(parents=True, exist_ok=True)
cache = Path('linux-kernel/.cache/x86_64')
def copy(source, name):
    target = root/name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
def write(name, data):
    (root/name).write_text(json.dumps(data, indent=2)+'\n')

native = json.loads((run/'run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
for name in ['extent-call-inventory.json','extent-call-inspection.json','transfer-guard-audit.json']:
    copy(build/name, name)
for stem in ['extent-call-disassembly','extent-call-inspection','transfer-guard-audit']:
    for suffix in ['command.json','stderr.log']:
        copy(build/f'{stem}.{suffix}', f'audit/{stem}.{suffix}')
    if stem != 'extent-call-disassembly':
        copy(build/f'{stem}.stdout.log', f'audit/{stem}.stdout.log')
artifacts = {}
for name in ['provider.o','extent-call-disassembly.stdout.log']:
    data = (build/name).read_bytes()
    path = root/'artifacts'/(name+'.gz')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(data, mtime=0))
    artifacts[name] = dict(bytes=len(data), sha256=digest(data),
        snapshot=str(path.relative_to(root)), snapshot_sha256=digest(path.read_bytes()))
write('artifact-identity.json', artifacts)
copy(run/'run.json','native-run.json')
copy(run/'test.log','native-test.log')
for name in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log',
             'serial.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log']:
    copy(build/'first-image-exit'/name, 'guest/'+name)
copy(post/'validation.json', 'post-validation.json')
for path in post.glob('*.log'):
    copy(path, 'post/'+path.name)
copy(readback/'readback.json', 'readback.json')
for path in (readback/'offline-controls').glob('*/rejection.json'):
    copy(path, 'offline-controls/'+path.parent.name+'.json')
for name in ['readback-transfer-guards','archive-transfer-guards','check-transfer-guards']:
    copy(cache/(name+'.py'), name+'.py')
copy(cache/'check-startup-source.py', 'post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'), 'native-runner.py')

# Retain the earlier checker assumption as an explicitly replayed offline
# failure. It is not a failed guest run or a reported model-safety denial.
old = cache/'guard-audit-before-encoding-fix.py'
copy(old, 'earlier-checker.py')
with (root/'earlier-checker-replay.log').open('w') as log:
    import os
    env = os.environ.copy()
    env['PYTHONPATH'] = str(Path('linux-kernel/tools/user-memory').resolve())
    result = subprocess.run([sys.executable,str(old),str(build)],
                            stdout=log,stderr=subprocess.STDOUT,env=env)
assert result.returncode != 0
write('earlier-checker-replay.json', dict(exit_code=result.returncode,
    scope='Replay of retained initial offline checker, not the original attempt; wrong C7 assumption versus actual movabs -14 encoding; no guest execution'))

previous = json.loads((root.parent/'stage-3-source-serial/source-identity.json').read_text())
sources = set(previous['selected_inputs']) | set(native['modified_source_hashes_before']) | {
    'src/topal-compiler/src/systems_process_entry_completion.rs',
    'src/topal-compiler/src/systems_process_entry_output.rs',
}
identities = {}
for source in sorted(sources):
    data = Path(source).read_bytes()
    if source in native['modified_source_hashes_before']:
        assert digest(data) == native['modified_source_hashes_before'][source], source
    target = root/'inputs'/(source+'.gz')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[source] = dict(sha256=digest(data), snapshot=str(target.relative_to(root)))
write('source-identity.json', dict(base_commit='511e347c3f6481c3f79b406715d827c32366ec76',
    selected_inputs=identities,
    scope='Selected guard/census/application observers, helper/caller and authority snapshots; not complete producer or cold reconstruction'))
write('scope.json', dict(stage_complete=False, production_activation=False,
    remaining=['three bounded fallback sites unobserved in matching-kernel native union',
        'complete reachable direct/shared user-access inventory and dominance',
        'indirect/exception entry classification and all caller length/error/lifetime obligations',
        'native ownership/mode/stability qualification',
        'canonical production binding and exact committed cold reconstruction',
        'full distribution/container/VM-host and remaining original kernel acceptance']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root), selected_inputs=len(identities))))
