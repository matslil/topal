"""Retain a scoped ordinary-refusal run, including any preceding timeout."""
import gzip
import hashlib
import json
from pathlib import Path
import posixpath
import shutil
import sys

catalog = json.loads(Path(sys.argv[1]).read_text())
root = Path('linux-kernel/tools/user-memory/results/stage-3-staged-refusals')
prior = root.parent / 'stage-3-staged-transfers'
build, run, post = map(Path, [catalog['build'], catalog['run'], catalog['post']])
native = json.loads((run / 'run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert json.loads(Path(catalog['readback']).read_text())['passed']
assert json.loads((post / 'validation.json').read_text())['passed']
root.mkdir(parents=True, exist_ok=True)
digest = lambda data: hashlib.sha256(data).hexdigest()
def copy(path, name):
    target = root / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, target)
def compress(path, name):
    target = root / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.compress(Path(path).read_bytes(), mtime=0))
def write(name, value):
    target = root / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2) + '\n')
def inherit(reference):
    data = (root / reference).read_bytes()
    return dict(path=reference, bytes=len(data), sha256=digest(data))
previous = json.loads((prior / 'source-identity.json').read_text())['selected_inputs']
selected = {}
extra = set(catalog.get('additional_selected_inputs', []))
for path in sorted(set(previous) | set(native['modified_source_hashes_before']) | extra):
    data = Path(path).read_bytes()
    if path in native['modified_source_hashes_before'] or path in extra:
        if path in native['modified_source_hashes_before']:
            assert digest(data) == native['modified_source_hashes_before'][path], path
        snapshot = 'inputs/' + path + '.gz'
        compress(path, snapshot)
    else:
        row = previous[path]
        snapshot = posixpath.normpath('../stage-3-staged-transfers/' + row['snapshot'])
        assert digest(data) == row['sha256']
        assert gzip.decompress((root / snapshot).read_bytes()) == data
    selected[path] = dict(sha256=digest(data), snapshot=snapshot)
write('source-identity.json', dict(base_commit='6841141aded9b90ad9bcf9e685f99a3635dc0756',
    inherited_record='../stage-3-staged-transfers/source-identity.json',
    inherited_record_sha256=digest((prior / 'source-identity.json').read_bytes()),
    selected_inputs=selected, scope='Selected authority/producer/reader/observer inputs, not the full producer'))
artifacts = json.loads((prior / 'artifact-identity.json').read_text())
for name, row in artifacts.items():
    row['snapshot'] = posixpath.normpath('../stage-3-staged-transfers/' + row['snapshot'])
    assert digest((build / name).read_bytes()) == row['sha256']
write('artifact-identity.json', artifacts)
references = [inherit('../stage-3-owned-poll/regression/guest/observations.json')]
for name in ['staged-transfer-audit.json.gz', 'staged-audit-identity.json',
             'caller-access-index.json', 'staged-summary.json']:
    references.append(inherit('../stage-3-staged-transfers/' + name))
write('inherited-evidence.json', references)
compress(build / 'staged-transfer-audit.json', 'staged-transfer-audit.json.gz')
copy(prior / 'staged-audit-identity.json', 'staged-audit-identity.json')
copy(build / 'source-component.json', 'source-component.json')
compress(build / 'static-elf-abi-fixture.elf', 'source-user-image.elf.gz')
image = (build / 'static-elf-abi-fixture.elf').read_bytes()
write('image-identity.json', dict(bytes=len(image), sha256=digest(image),
    first_party_snapshot='source-user-image.elf.gz',
    boot_resource_sha256=digest((build / 'initramfs.cpio').read_bytes())))
for path, name in [(run / 'run.json', 'native-run.json'), (run / 'test.log', 'native-test.log'),
                   (Path(catalog['readback']), 'readback.json'),
                   (post / 'validation.json', 'post-validation.json')]:
    copy(path, name)
for path in post.glob('*.log'):
    copy(path, 'post/' + path.name)
guest = build / 'source-user-extents'
for name in ['gdb.stdout.log', 'gdb.stderr.log', 'qemu.command.json',
             'qemu.stdout.log', 'qemu.stderr.log', 'serial.log']:
    copy(guest / name, 'guest/' + name)
for name in ['inspect.gdb', 'observations.json']:
    compress(guest / name, 'guest/' + name + '.gz')
for name in ['callback-bindings.json', 'baseline/provenance.json']:
    compress(build / name, 'diagnostic/' + name + '.gz')
write('observation-identity.json', dict(bytes=(guest / 'observations.json').stat().st_size,
    sha256=digest((guest / 'observations.json').read_bytes())))
if catalog.get('pinned'):
    pinned = catalog['pinned']
    pb, pr = map(Path, [pinned['build'], pinned['run']])
    pn = json.loads((pr / 'run.json').read_text())
    assert pn['exit_code'] == 0 and not pn['source_changes_during_run']
    assert pn['modified_source_hashes_before'] == native['modified_source_hashes_before']
    assert json.loads(Path(pinned['readback']).read_text())['passed']
    for path, name in [(pr / 'run.json', 'native-run.json'), (pr / 'test.log', 'native-test.log'),
                       (Path(pinned['readback']), 'readback.json')]:
        copy(path, 'application/' + name)
    for name in ['gdb.stdout.log', 'gdb.stderr.log', 'qemu.command.json',
                 'qemu.stdout.log', 'qemu.stderr.log', 'serial.log']:
        copy(pb / 'first-image-exit' / name, 'application/guest/' + name)
    for name in ['inspect.gdb', 'observations.json']:
        compress(pb / 'first-image-exit' / name, 'application/guest/' + name + '.gz')
    for name in ['callback-bindings.json', 'baseline/provenance.json']:
        compress(pb / name, 'application/diagnostic/' + name + '.gz')
    pi = (pb / 'static-elf-abi-fixture.elf').read_bytes()
    write('application/image-identity.json', dict(bytes=len(pi), sha256=digest(pi),
        boot_resource_sha256=digest((pb / 'initramfs.cpio').read_bytes())))
configuration = Path('linux-kernel/.cache/x86_64/tk283-application-run-1794416915398821260')
for name in ['run.json', 'test.log']:
    copy(configuration / name, 'pinned-configuration-failure/' + name)
for name in ['initial-staged-refusal-readback-command.json', 'initial-staged-refusal-readback.log',
             'readback-staged-refusals-initial.py', 'staged-refusal-readback-controls.json']:
    copy(Path('linux-kernel/.cache/x86_64') / name, name)
copy(Path('linux-kernel/.cache/x86_64/staged-refusal-interface-readback/independent-readback.json'),
    'interface-readback.json')
for attempt in catalog.get('failed_attempts', []):
    initial = Path(attempt['run'])
    label = attempt['label']
    failed = json.loads((initial / 'run.json').read_text())
    assert failed['exit_code'] != 0 and not failed['source_changes_during_run']
    for name in ['run.json', 'test.log']:
        copy(initial / name, label + '/' + name)
    for path, sha in failed['modified_source_hashes_before'].items():
        source = Path(attempt['inputs']) / path
        assert digest(source.read_bytes()) == sha
        compress(source, label + '/inputs/' + path + '.gz')
    initial_guest = Path(attempt['build']) / 'source-user-extents'
    for name in ['gdb.stdout.log', 'gdb.stderr.log', 'qemu.command.json',
                 'qemu.stdout.log', 'qemu.stderr.log', 'serial.log']:
        copy(initial_guest / name, label + '/guest/' + name)
    compress(initial_guest / 'inspect.gdb', label + '/guest/inspect.gdb.gz')
for name in ['archive-staged-refusals', 'readback-staged-refusals', 'check-staged-refusals',
             'check-staged-transfers', 'check-fixed-transfers', 'check-startup-source',
             'readback-staged-refusal-pinned', 'replay-staged-refusals', 'finalize-staged-refusals']:
    copy(Path('linux-kernel/.cache/x86_64') / (name + '.py'), name + '.py')
copy(Path('linux-kernel/.cache/x86_64/check-staged-refusal-readback.py'), 'check-staged-refusal-readback.py')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'), 'native-source-runner.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'), 'native-application-runner.py')
copy(Path(sys.argv[1]), 'catalog.json')
(root / '.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root), selected_inputs=len(selected))))
