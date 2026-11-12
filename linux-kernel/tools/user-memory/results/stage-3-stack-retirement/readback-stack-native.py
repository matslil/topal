"""Join saved native cleanup records to exact reports and observer programs."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

mapping,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest=lambda raw:hashlib.sha256(raw).hexdigest()
builds={name:Path(value) for name,value in json.loads(mapping.read_text()).items()}
program=runpy.run_path('linux-kernel/labs/qemu/x86_64/user_extent_stack_observer.py')['PROGRAM']
cohorts={}
union=set()
sites=set()
reference=None
for label,build in builds.items():
    audit_bytes=(build/'stack-retirement-audit.json').read_bytes()
    if reference is None: reference=audit_bytes
    assert reference==audit_bytes
    audit=json.loads(audit_bytes)
    assert audit['passed'] and not audit['complete'] and audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
    path=build/('first-image-exit' if label=='pinned' else 'source-user-extents')
    raw=(path/'observations.json').read_bytes()
    observation=json.loads(raw)
    assert observation['passed'] and observation['extent_call_coverage']['kernel_sha256']==audit['kernel_sha256']
    native=observation['stack_retirement_observation']
    assert native['passed'] and native['audit_sha256']==digest(audit_bytes)
    assert program in (path/'inspect.gdb').read_text()
    blocks={row['name']:row for row in audit['blocks']}
    names=[row['name'] for row in native['records']]
    assert len(names)==len(set(names)) and set(names)<=set(blocks)
    assert set(native['unreached'])==set(blocks)-set(names)
    for row in native['records']:
        block=blocks[row['name']]
        assert row['completed']>0 and row['zero_bytes_observed']==row['completed']*8*row['words']
        assert all(row[key]==block[key] for key in ['start','end','words'])
    union.update(names)
    sites.update(row['offset'] for row in observation['extent_call_coverage']['reached'])
    cohorts[label]=dict(observation_sha256=digest(raw),program_sha256=digest((path/'inspect.gdb').read_bytes()),
        records=native['records'],unreached=native['unreached'],observer_program_present=True,
        completions=sum(row['completed'] for row in native['records']),
        zero_bytes_observed=sum(row['zero_bytes_observed'] for row in native['records']),
        helper_sites_reached=observation['extent_call_coverage']['reached_sites'])
assert {'clock-leaf-return','clock-saved-registers','random-extract-return','random-saved-registers'}<=union
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(reference),cohorts=cohorts,
    reached_blocks=sorted(union),unreached_blocks=sorted(set(blocks)-union),
    fresh_helper_first_reach_union=sorted(sites),
    scope='Saved actual runtime assertion records and exact observer/report/kernel joins; not another native run, independent observation of every retired byte, ownership/nonalias/continuous lifetime, unreached or fatal/exception paths, or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,blocks_reached=len(union),total_blocks=len(blocks),helper_first_reach=len(sites),
    cohorts={k:dict(completions=v['completions'],zero_bytes=v['zero_bytes_observed']) for k,v in cohorts.items()})))
