"""Restore exact selected sources and replay saved evidence; execute no guest."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

archive, work = [Path(name).resolve() for name in sys.argv[1:]]
root = archive.parents[4]
assert work.is_relative_to(root / 'linux-kernel')
work.mkdir(parents=True, exist_ok=False)
report = json.loads((archive / 'report.json').read_text())
assert report['passed'] and report['schema'] == 'topal.calendar-data-placement/1'
manifest = json.loads((archive / 'manifest.json').read_text())
assert {path.relative_to(archive).as_posix() for path in archive.rglob('*') if path.is_file()} == set(manifest) | {'manifest.json'}


def fact(raw):
    return dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


for name, expected in manifest.items():
    path = (archive / name).resolve()
    assert path.is_relative_to(archive) and fact(path.read_bytes()) == expected
for name, row in report['sources'].items():
    base = archive if row['location'] == 'archive' else root
    path = (base / row['path']).resolve()
    assert path.is_relative_to(base)
    packed = path.read_bytes()
    assert hashlib.sha256(packed).hexdigest() == row['packed_sha256']
    raw = gzip.decompress(packed)
    assert fact(raw) == {key: row[key] for key in ['bytes', 'sha256']}
    target = (work / name).resolve()
    assert target.is_relative_to(work)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(raw)
for name in manifest:
    if not name.startswith(('native/', 'reference/')):
        continue
    assert name.endswith('.gz')
    target = (work / 'evidence' / name[:-3]).resolve()
    assert target.is_relative_to(work / 'evidence')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.decompress((archive / name).read_bytes()))
sys.path[:0] = [str(work / 'linux-kernel/tools/user-memory'),
                str(work / 'linux-kernel/tools/process-memory')]
from archive_publication import closed_archive
from inspect_calendar_placement import inspect
from readback_applications import facts, readback as app_readback
from readback_exec_failures import readback as exec_readback

dependency = report['dependency']
assert fact(closed_archive(root / dependency['path'])) == dependency['manifest']
q = report['qualification']
assert q['passed'] and q['source_hashes_before'] == q['source_hashes_after']
assert set(report['sources']) == set(q['source_hashes_before'])
assert all(report['sources'][name]['sha256'] == sha for name, sha in q['source_hashes_before'].items())
reference = work / 'evidence/reference'
for name, expected in q['reference_artifacts'].items():
    assert fact((reference / name).read_bytes()) == expected
placements = {}
for cohort, files in report['build_files'].items():
    build = work / 'evidence/native' / cohort
    assert {path.relative_to(build).as_posix() for path in build.rglob('*') if path.is_file()} == set(files)
    for name, expected in files.items():
        assert fact((build / name).read_bytes()) == expected
    actual = inspect(build, reference)
    assert actual == q['placement'][cohort]
    placements[cohort] = actual
child = work / 'evidence/native/child-exec'
result = exec_readback(json.loads((child / 'process-mm-users/observations.json').read_bytes()))
expected = json.loads(gzip.decompress((archive / 'qualification/child-exec-readback.json.gz').read_bytes()))
assert json.loads(json.dumps(result)) == expected
applications = work / 'evidence/native/applications'
batch = json.loads((applications / 'process-mm-applications/observations.json').read_bytes())
binding = facts(applications)
assert binding == q['applications']['facts']
assert batch['passed'] and batch['complete_corpus']
assert len(batch['selected']) == len(set(batch['selected'])) == 51
assert set(batch['selected']) == set(batch['workloads']) == set(q['applications']['summaries'])
assert batch['kernel_sha256'] == binding['kernel_sha256']
sys.path.insert(0, str(work / 'linux-kernel/labs/qemu/x86_64'))
import topal_gate
from user_extent_application_inspect import members
image = (applications / 'static-elf-abi-fixture.elf').read_bytes()
assert fact(image)['sha256'] == topal_gate.PINNED_STATIC_BUSYBOX_SHA256 == batch['image_sha256']
boot = members((applications / 'initramfs.cpio').read_bytes())
assert set(boot) == {'init', 'static-user.elf'} and boot['static-user.elf'] == image
assert set(batch['selected']) == set(topal_gate.PROCESS_WORKLOADS)
summaries = {}
for name in batch['selected']:
    folder = applications / 'process-mm-applications' / name
    user_raw = (folder / 'observations.json').read_bytes()
    mm_raw = (folder / 'process-mm-observations.json').read_bytes()
    serial = (folder / 'serial.log').read_bytes()
    command, output, status = topal_gate.PROCESS_WORKLOADS[name]
    mm = json.loads(mm_raw)
    assert batch['workloads'][name] == dict(passed=True, command_hex=command.hex(),
        output_hex=output.hex(), status=status, serial_sha256=fact(serial)['sha256'],
        mm_boundaries=len(mm['events']), observation_sha256=fact(user_raw)['sha256'],
        mm_observation_sha256=fact(mm_raw)['sha256'])
    summaries[name] = app_readback(json.loads(user_raw), mm, serial, name, binding)
    assert json.loads(json.dumps(summaries[name])) == q['applications']['summaries'][name]
    qemu = json.loads((folder / 'qemu.command.json').read_text())
    assert qemu[qemu.index('-m') + 1] == '640M'
    assert qemu[qemu.index('-smp') + 1].startswith('cpus=2,')
    assert qemu[qemu.index('-accel') + 1].startswith('tcg,')
    stdout = (folder / 'gdb.stdout.log').read_bytes()
    assert stdout.count(b'FRAME_ROOT_GDB_PASS') == 1
    assert stdout.count(b'FRAME_APPLICATION_GDB_PASS ' + name.encode() + b' ') == 1
    del user_raw, mm_raw, mm, serial
models = {}
for directory, count in [('user-memory', 259), ('process-memory', 82)]:
    result = subprocess.run(['rtk', 'proxy', 'python3', '-m', 'unittest', 'discover', '-s',
                             'linux-kernel/tools/' + directory, '-p', 'test_*.py'], cwd=work,
                            env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'), capture_output=True)
    (work / (directory + '.stdout.log')).write_bytes(result.stdout)
    (work / (directory + '.stderr.log')).write_bytes(result.stderr)
    assert result.returncode == 0 and ('Ran ' + str(count) + ' tests').encode() in result.stderr, result.stderr.decode()
    models[directory] = count
receipt = dict(passed=True, models=models, native_workload_records=51,
               placement=placements, child_exec_readback=True,
               saved_application_controls=sum(len(value['controls']) for value in summaries.values()),
               new_native_execution=False, hardware_permissions_proved=False,
               qualification_complete=False)
(work / 'replayed.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
print('Frozen calendar placement, child exec, 51 saved application records and 341 models passed')
