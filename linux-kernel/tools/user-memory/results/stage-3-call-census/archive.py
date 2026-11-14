"""Retain selected census/reachability evidence without claiming reconstruction."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import sys

build, run, post, readback = map(Path, sys.argv[1:5])
root = Path('linux-kernel/tools/user-memory/results/stage-3-call-census')
root.mkdir(parents=True, exist_ok=True)
cache = Path('linux-kernel/.cache/x86_64')
digest = lambda data: hashlib.sha256(data).hexdigest()
def copy(source, name):
    target = root/name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
def write(name, data):
    (root/name).write_text(json.dumps(data, indent=2)+'\n')
record = json.loads((run/'run.json').read_text())
assert record['exit_code'] == 0 and not record['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
for name in ['extent-call-inventory.json', 'extent-call-inspection.json', 'source-component.json']:
    copy(build/name, name)
for name in ['command.json', 'stdout.log', 'stderr.log']:
    source = build/f'extent-call-disassembly.{name}'
    target = root/(source.name+'.gz')
    target.write_bytes(gzip.compress(source.read_bytes(), mtime=0))
copy(run/'run.json', 'native-run.json')
copy(run/'test.log', 'native-test.log')
for name in ['inspect.gdb', 'observations.json', 'gdb.stdout.log', 'gdb.stderr.log',
             'serial.log', 'qemu.command.json', 'qemu.stdout.log', 'qemu.stderr.log']:
    copy(build/'source-user-extents'/name, 'guest/'+name)
copy(post/'validation.json', 'post-validation.json')
for path in post.glob('*.log'):
    copy(path, 'post/'+path.name)
copy(readback/'readback.json', 'readback.json')
for source, name in [
    (cache/'readback-call-census.py', 'readback.py'),
    (cache/'archive-call-census.py', 'archive.py'),
    (cache/'check-call-census.py', 'post-runner.py'),
    (cache/'check-startup-source.py', 'post-runner-base.py'),
    (cache/'run-source-poll.py', 'native-runner.py'),
    (Path('linux-kernel/.cache/tk283-source-retry.py'), 'native-runner-base.py')]:
    copy(source, name)
for label, attempt in [('compile', 'tk283-source-poll-run-1794397811359165959'),
                       ('symbol-reader', 'tk283-source-poll-run-1794397835342304419')]:
    failure = json.loads((cache/attempt/'run.json').read_text())
    assert failure['exit_code'] != 0 and not failure['source_changes_during_run']
    copy(cache/attempt/'run.json', f'earlier-failures/{label}/run.json')
    copy(cache/attempt/'test.log', f'earlier-failures/{label}/test.log')
copy(cache/'call-census-inspector-before-symbol-fix.py', 'earlier-failures/symbol-reader/inspector.py')
failure = json.loads((cache/'tk283-source-poll-run-1794397835342304419/run.json').read_text())
assert digest((root/'earlier-failures/symbol-reader/inspector.py').read_bytes()) == failure['modified_source_hashes_before']['linux-kernel/tools/user-memory/inspect_call_inventory.py']
source = Path('src/topal-compiler/src/systems_user_extent_inventory.rs').read_text()
start = source.index('    let helpers = ')
end = source.index('    Some(serde_json::json!', start)
earlier = source[:start]+source[end:]
earlier = earlier.replace('"helpers": helpers,',
    '"helpers": ["runtime-readable-extent", "runtime-writable-extent"].into_iter()\n'
    '            .map(|name| (name.to_owned(), code.labels[name])).collect::<std::collections::BTreeMap<_, _>>(),')
failure = json.loads((cache/'tk283-source-poll-run-1794397811359165959/run.json').read_text())
assert digest(earlier.encode()) == failure['modified_source_hashes_before']['src/topal-compiler/src/systems_user_extent_inventory.rs']
(root/'earlier-failures/compile/inventory.rs').write_text(earlier)
previous = json.loads((root.parent/'stage-3-owned-poll/source-identity.json').read_text())
inspection = json.loads((root/'extent-call-inspection.json').read_text())
sources = (set(record['modified_source_hashes_before']) |
           set(previous['selected_executable_inputs']) | set(inspection['source_sha256']))
authority = set(previous['authority_inputs']) | {'AGENTS.md'}
identities = {}
for name in sorted(sources | authority):
    data = Path(name).read_bytes()
    if name in record['modified_source_hashes_before']:
        assert digest(data) == record['modified_source_hashes_before'][name], name
    target = root/'inputs'/(name+'.gz')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.compress(data, mtime=0))
    identities[name] = dict(sha256=digest(data), snapshot=str(target.relative_to(root)),
                            authority=name in authority)
write('source-identity.json', dict(base_commit='0b205e6e6d7243a5aa3c4ebe4889061522719938',
    selected_inputs=identities,
    scope='Selected changed inputs, immediate helper emitters, preceding source-fixture inputs and authority snapshots. Not a complete producer census, clean-checkout run or cold reconstruction.'))
write('scope.json', dict(stage_complete=False, production_activation=False, systems_only=True,
    remaining=['complete direct/shared caller and direct-user-access inventory with access dominance',
        'all length, direction and error paths; classification of unobserved fallback routes',
        'full native ownership/mode/stability qualification',
        'complete caller/source/native/load/boot binding and committed cold reconstruction',
        'canonical production activation', 'full original kernel goal']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(root)
