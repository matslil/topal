"""Replay selected frozen readers/models/native records without ELF execution."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
work = Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
source = work/'source-root'
identity = json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes()) == identity['inherited_record_sha256']
def unpack(row):
    packed = (root/row['snapshot']).read_bytes()
    if 'snapshot_sha256' in row:
        assert digest(packed) == row['snapshot_sha256']
    data = gzip.decompress(packed)
    assert digest(data) == row['sha256']
    if 'bytes' in row:
        assert len(data) == row['bytes']
    return data
for path,row in identity['selected_inputs'].items():
    target = source/path
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(unpack(row))
build = work/'build'
build.mkdir()
artifacts = json.loads((root/'artifact-identity.json').read_text())
for name,row in artifacts.items():
    (build/name).write_bytes(unpack(row))
observation = build/'source-user-extents'
observation.mkdir()
for path in (root/'native/observation').glob('*.gz'):
    (observation/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
baseline = source/'linux-kernel/.cache/x86_64/user-extent-source-pipes-2367848-1794444753502668964'
baseline.mkdir(parents=True)
for row in json.loads((root/'baseline-identities.json').read_text()):
    data = (build/row['path']).read_bytes()
    assert digest(data) == row['sha256'] and len(data) == row['bytes']
    # These equal selected historical bytes were directly compared before
    # freezing. Restoring them is not a new independent producer or execution.
    (baseline/row['path']).write_bytes(data)
previous = work/'preceding-source-user.elf'
previous.write_bytes(unpack(json.loads((root/'preceding-link-identity.json').read_text())))
readers = source/'linux-kernel/tools/user-memory'
expected_names = ['fixed-transfer-audit.json','copy-transfer-audit.json','staged-transfer-audit.json',
    'additional-scalar-audit.json','staging-copy-audit.json','pipe-fixed-output-audit.json']
expected = {name:(build/name).read_bytes() for name in expected_names}
def run(label,command):
    result = subprocess.run(command,cwd=source,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(label+'.log')).write_bytes(result.stdout)
    assert result.returncode == 0,result.stdout[-4000:].decode(errors='replace')
for stem in ['fixed_transfers','copy_transfers','staged_transfers','additional_scalars',
             'staging_copies','pipe_fixed_outputs']:
    run(stem,[sys.executable,str(readers/('audit_'+stem+'.py')),str(build)])
assert all((build/name).read_bytes() == data for name,data in expected.items())
run('models',[sys.executable,'-m','unittest','discover','-s',str(readers),'-v'])
assert b'Ran 69 tests' in (work/'models.log').read_bytes()
run('native-records',[sys.executable,str(root/'readback-fixed-pipe-native.py'),str(build),str(work/'native-readback')])
assert (work/'native-readback/readback.json').read_bytes() == (root/'native-readback.json').read_bytes()
run('source-link-sections',[sys.executable,str(root/'readback-source-link-sections.py'),
    str(previous),str(build/'source-user.elf'),str(work/'source-link-sections.json')])
assert (work/'source-link-sections.json').read_bytes() == (root/'source-link-section-difference.json').read_bytes()
report = dict(passed=True,selected_inputs=len(identity['selected_inputs']),artifacts=len(artifacts),
    exact_six_reports_equal=True,models=69,exact_native_record_readback_equal=True,
    exact_raw_source_link_section_readback_equal=True,
    scope='Frozen selected source/artifact and saved-native correspondence replay only. No ELF, compiler, GNU, GDB or VM execution, complete producer, actual owner/lifetime or canonical qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
