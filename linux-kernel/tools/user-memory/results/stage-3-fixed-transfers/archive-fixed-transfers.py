"""Freeze the conditional scalar proof; reference unchanged prior snapshots."""
import gzip
import json
from pathlib import Path
import shutil
import sys
sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build, run, post, controls, readback = map(Path, sys.argv[1:6])
root = Path('linux-kernel/tools/user-memory/results/stage-3-fixed-transfers')
root.mkdir(parents=True, exist_ok=True)
prior = root.parent/'stage-3-process-origins'
cache = Path('linux-kernel/.cache/x86_64')
def copy(path, name):
    target = root/name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, target)
def write(name, data):
    (root/name).write_text(json.dumps(data, indent=2)+'\n')

native = json.loads((run/'run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
for path in [post/'validation.json', controls/'controls.json', readback/'readback.json']:
    assert json.loads(path.read_text())['passed']
selected = {}
previous = json.loads((prior/'source-identity.json').read_text())['selected_inputs']
for path in sorted(set(previous) | set(native['modified_source_hashes_before'])):
    data = Path(path).read_bytes()
    if path in native['modified_source_hashes_before']:
        assert digest(data) == native['modified_source_hashes_before'][path]
        snapshot = Path('inputs')/(path+'.gz')
        target = root/snapshot
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(gzip.compress(data, mtime=0))
    else:
        row = previous[path]
        assert row['sha256'] == digest(data)
        snapshot = Path('../stage-3-process-origins')/row['snapshot']
        assert gzip.decompress((root/snapshot).read_bytes()) == data
    selected[path] = dict(sha256=digest(data), snapshot=str(snapshot))
write('source-identity.json', dict(base_commit='2bae26819166cd747a4bdd8e6e99c923d3b1b9be',
    inherited_record='../stage-3-process-origins/source-identity.json',
    inherited_record_sha256=digest((prior/'source-identity.json').read_bytes()),
    selected_inputs=selected, scope='Selected producer/authority/reader inputs; unchanged snapshots are explicitly inherited, not a complete producer/cold reconstruction'))
artifacts = {}
for name in ['provider.o', 'kernel.elf', 'process-access-inventory.json',
             'process-byte-origins.json', 'process-access-origins.json']:
    row = json.loads((prior/'artifact-identity.json').read_text())[name]
    assert digest((build/name).read_bytes()) == row['sha256']
    row['snapshot'] = str(Path('../stage-3-process-origins')/row['snapshot'])
    artifacts[name] = row
data = (build/'extent-call-inventory.json').read_bytes()
assert data == (prior/'extent-call-inventory.json').read_bytes()
artifacts['extent-call-inventory.json'] = dict(bytes=len(data), sha256=digest(data),
    snapshot='../stage-3-process-origins/extent-call-inventory.json', compressed=False)
write('artifact-identity.json', artifacts)
copy(build/'fixed-transfer-audit.json', 'fixed-transfer-audit.json')
for suffix in ['command.json', 'stdout.log', 'stderr.log']:
    copy(build/('fixed-transfer-audit.'+suffix), 'audit/'+suffix)
copy(run/'run.json', 'native-run.json')
copy(run/'test.log', 'native-test.log')
for name in ['inspect.gdb', 'gdb.stdout.log', 'gdb.stderr.log', 'serial.log',
             'qemu.command.json', 'qemu.stdout.log', 'qemu.stderr.log']:
    copy(build/'first-image-exit'/name, 'guest/'+name)
data = (build/'first-image-exit/observations.json').read_bytes()
(root/'observations.json.gz').write_bytes(gzip.compress(data, mtime=0))
write('observation-identity.json', dict(bytes=len(data), sha256=digest(data)))
copy(post/'validation.json', 'post-validation.json')
for path in post.glob('*.log'):
    copy(path, 'post/'+path.name)
copy(controls/'controls.json', 'controls.json')
for path in controls.glob('*/rejection.json'):
    copy(path, 'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log', 'controls/'+path.parent.name+'.log')
copy(readback/'readback.json', 'readback.json')
for path in readback.iterdir():
    if path.name != 'readback.json':
        copy(path, 'gnu/'+path.name)
for path in (cache/'fixed-transfer-readback').iterdir():
    copy(path, 'readback-initial/'+path.name)
write('readback-initial/failure.json', dict(passed=False, exit_code=1,
    cause='GNU default seven-byte instruction display wrapped longer encodings, so the readback parser incorrectly treated continuation rows as instructions',
    correction='Explicit --insn-width=16 preserves one display row per x86 instruction; final GNU reader passes with unchanged native artifacts',
    scope='Initial retained GNU outputs and command records; Python traceback was reported in the session, not captured as a log. Reader failure, not a native kernel or safeguard failure'))
for name in ['archive-fixed-transfers', 'readback-fixed-transfers', 'check-fixed-transfers', 'check-fixed-transfer-controls']:
    copy(cache/(name+'.py'), name+'.py')
copy(cache/'check-startup-source.py', 'post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'), 'native-runner.py')
write('scope.json', dict(stage_complete=False, production_activation=False,
    remaining=['all other semantic user-access regions and complete syscall effects/error order',
        'shared helper soundness, native owner/mode/stale-state and stable access lifetime',
        'complete roots/callee/exception contracts and three fallback reachability obligations',
        'canonical production binding and exact committed cold reconstruction',
        'the complete original distribution/device/driver/container/VM-host requirements']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root), selected_inputs=len(selected),
    inherited_inputs=sum(r['snapshot'].startswith('../') for r in selected.values()))))
