"""Stream and bind actual full-workload records to their retained summaries.

This checks record correspondence, not a new permission/ownership model or a
second guest execution. Complete native assertions belong to the saved harness.
"""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
digest = lambda data: hashlib.sha256(data).hexdigest()
observation_bytes = gzip.decompress((root/'full-shared/observations.json.gz').read_bytes())
observation = json.loads(observation_bytes)
assert observation['passed'] and observation['process_cleanup_checked']
for key in ['shared_caller_registers_and_retirement_checked','startup_outputs_checked',
            'shell_outputs_checked','stat_outputs_checked','statat_paths_checked']:
    assert observation[key]
for key,count in [('write_fixture_cases_checked',20),('file_fixture_cases_checked',11),
                  ('shell_io_fixture_cases_checked',15),('command_fixture_cases_checked',5),
                  ('statat_fixture_cases_checked',6),('stat_fixture_cases_checked',4)]:
    assert observation[key] == list(range(count))
assert len(observation['memory']) >= 83 and len(observation['helper_calls']) >= 4
summaries = observation['user_extents']
records_path = root/'full-shared/observations.json.extent-records.jsonl.gz'
count = 0
maximum_record_bytes = 0
with gzip.open(records_path,'rb') as stream:
    while True:
        line = stream.readline(1048577)
        if not line:
            break
        assert line.endswith(b'\n') and len(line) <= 1048576, 'incomplete or oversized record'
        assert count < len(summaries), 'extra full record'
        raw = line[:-1]
        record = json.loads(raw)
        summary = summaries[count]
        before = summary['before']
        assert before['archive_record'] == count and before['record_sha256'] == digest(raw)
        assert before['byte_scope'] in ['whole-backing','covered-bytes']
        assert {k:v for k,v in record.items() if k!='before'} == {k:v for k,v in summary.items() if k!='before'}
        assert isinstance(record['before'],dict)
        maximum_record_bytes = max(maximum_record_bytes,len(line))
        count += 1
assert count == len(summaries) and count > 0
comparison = json.loads((root/'method/full-shared-input-comparison.json').read_text())
assert comparison['passed']
assert comparison['debugger_program_equal_after_build_path_and_loopback_port_normalization']
assert set(comparison['artifacts']) == {'kernel.elf','static-elf-abi-fixture.elf','initramfs.cpio',
    'caller-observation.json','caller-fixture-sites.json','user-owners.json'}
for name,row in comparison['artifacts'].items():
    data = gzip.decompress((root/'full-shared/inputs'/(name+'.gz')).read_bytes())
    assert row['equal'] and digest(data) == row['before_sha256'] == row['after_sha256']
debugger_files = ['method/before-full-workload-budget-inspect.gdb.gz','full-shared/inspect.gdb.gz']
normalized = []
for name,identity in zip(debugger_files,comparison['debugger_programs'],strict=True):
    data = gzip.decompress((root/name).read_bytes())
    assert digest(data) == identity['raw_sha256']
    text = data.decode()
    build_paths = re.findall(r'^file "(.+)/kernel\.elf"$',text,re.M)
    assert len(build_paths) == 1
    text = text.replace(build_paths[0],'BUILD_ROOT')
    ports = re.findall(r'^target remote 127\.0\.0\.1:[0-9]+$',text,re.M)
    assert len(ports) == 1 and ports[0] == identity['loopback_connection_line']
    text = re.sub(r'^target remote 127\.0\.0\.1:[0-9]+$','target remote 127.0.0.1:DEBUG_PORT',text,flags=re.M)
    assert digest(text.encode()) == identity['normalized_sha256']
    normalized.append(text)
assert normalized[0] == normalized[1]
records_digest = hashlib.sha256()
with records_path.open('rb') as stream:
    while chunk := stream.read(65536):
        records_digest.update(chunk)
report = dict(passed=True,input_kernel_sha256=comparison['artifacts']['kernel.elf']['after_sha256'],
              observations_sha256=digest(observation_bytes),
              compressed_records_sha256=records_digest.hexdigest(),
              streamed_records=count,summary_records=len(summaries),
              maximum_record_bytes=maximum_record_bytes,
              complete_gzip_stream=True,all_record_hashes_and_non_snapshot_fields_equal=True,
              identical_six_full_workload_inputs=True,normalized_debugger_programs_equal=True,
              scope='Streaming correspondence of archived full native records, actual summaries and compared inputs/programs. Not another guest run, a new permission/owner model, complete lifetime qualification or an old-budget performance pass.')
(root/'full-shared-record-readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
