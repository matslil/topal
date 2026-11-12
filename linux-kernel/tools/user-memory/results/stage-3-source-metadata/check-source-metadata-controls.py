"""Offline saved-record counterexamples; no ELF, GDB or VM execution."""
import copy
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build,reader,output=map(Path,sys.argv[1:4])
build,reader=build.resolve(),reader.resolve()
output.mkdir(exist_ok=False)
original=json.loads((build/'source-user-extents/observations.json').read_text())
def first(obs,n,positive=True):
    return next(row for row in obs['executions'][0]['calls'] if row['number']==n and (row['result']>=0)==positive)
def clock(obs):return first(obs,257)['metadata_clocks'][0]
def replace_hex(row,key,byte):
    data=bytearray.fromhex(row[key])
    data[byte]^=1
    row[key]=data.hex()
def tail(obs):
    row=first(obs,77)
    deltas=row['metadata_filesystem_delta']
    assert any(delta['offset']==4098 for delta in deltas)
    row['metadata_filesystem_delta']=[delta for delta in deltas if delta['offset']!=4098]
def close(obs):
    row=[r for r in obs['executions'][0]['calls'] if r['number']==3][-1]
    assert row['metadata_filesystem_delta']
    row['metadata_filesystem_delta']=[]
def duplicate(obs):
    row=first(obs,257)
    row['metadata_filesystem_delta'].insert(1,copy.deepcopy(row['metadata_filesystem_delta'][0]))
controls=[('clock-seconds',lambda o:clock(o).__setitem__('seconds',clock(o)['seconds']+1)),
    ('clock-call-site',lambda o:clock(o).__setitem__('call_offset',clock(o)['call_offset']+1)),
    ('clock-carry',lambda o:clock(o).__setitem__('carry',1)),
    ('missing-clock',lambda o:first(o,257).__setitem__('metadata_clocks',[])),
    ('immutable-payload',lambda o:replace_hex(first(o,18),'metadata_payload',3)),
    ('immutable-path',lambda o:first(o,257)['metadata_paths'].__setitem__(0,b'/tmp/extent-z'.hex())),
    ('stat-mode',lambda o:replace_hex(first(o,5),'stat_output_hex',24)),
    ('stat-zero-field',lambda o:replace_hex(first(o,5),'stat_output_hex',32)),
    ('missed-close-table',lambda o:first(o,3).__setitem__('metadata_table_delta',[])),
    ('missed-page-tail-zero',tail),('missed-final-reclamation',close),('duplicate-delta',duplicate),
    ('missing-call',lambda o:o['executions'][0]['calls'].pop(4)),
    ('failed-native-cleanup',lambda o:o['stack_retirement_observation'].__setitem__('passed',False))]
rows=[]
for name,change in controls:
    work=output/name
    work.mkdir()
    for filename in ['kernel.elf','source-user.elf','static-elf-abi-fixture.elf','source-component.json','stack-retirement-audit.json']:
        (work/filename).symlink_to(build/filename)
    for path in build.glob('*.su'):(work/path.name).symlink_to(path)
    observation=work/'source-user-extents'
    observation.mkdir()
    (observation/'inspect.gdb').symlink_to(build/'source-user-extents/inspect.gdb')
    mutated=copy.deepcopy(original)
    change(mutated)
    raw=(json.dumps(mutated,indent=2)+'\n').encode()
    (observation/'observations.json').write_bytes(raw)
    result=subprocess.run([sys.executable,str(reader),str(work),str(work/'readback')],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode and b'AssertionError' in result.stdout,(name,result.stdout[-2000:])
    rows.append(dict(name=name,rejected=True,exit_code=result.returncode,
        observation_sha256=digest(raw),log_sha256=digest(result.stdout)))
report=dict(passed=True,reader_sha256=digest(reader.read_bytes()),
    original_observation_sha256=digest((build/'source-user-extents/observations.json').read_bytes()),controls=rows,
    scope='Offline mutations of saved first-party observation records only. No changed ELF, compiler, GNU decoder, GDB or VM execution; not authentication or native fault/owner/lifetime qualification')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(rows))))
