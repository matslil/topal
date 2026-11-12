"""Recheck selected admission-to-copy watch records and ordinary write controls."""
import hashlib
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf
from owner_interval import compare,regions

build,observation,output=map(Path,sys.argv[1:4])
sha=lambda raw:hashlib.sha256(raw).hexdigest()
obs=json.loads(observation.read_bytes())
assert obs['passed'] and len(obs['executions'])==4
owner=obs['owner_boundary_observation']
native=obs['owner_interval_observation']
profile=owner['profile']
kernel=Elf(build/'kernel.elf')
assert profile['kernel_sha256']==sha(kernel.data)
model=Path('linux-kernel/tools/user-memory/owner_interval.py').read_text()
assert native['model_sha256']==sha(model.encode())
embedded=model.replace('from owner_snapshot import check',"check=owner_snapshot_model['check']")
assert ('exec('+repr(embedded)+',owner_interval_model)') in (build/'source-user-extents/inspect.gdb').read_text()
addresses=native['addresses']
for suffix in ['context','root_tokens','registry','table','mm_tokens','mm_epoch','mm_state']:
    assert addresses[suffix]==kernel.symbols['topal_x86_frame_'+suffix+'_v1']['value']
snapshots=native['snapshots']
for key,snapshot in snapshots.items():
    assert sha(json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode())==key
assert native['passed'] and native['all_watchpoints_hardware'] and native['all_watchpoints_disabled']
assert native['inactive_watchpoints_deleted_after_cases']
created=native['created_watchpoints']
assert len(created)==native['watchpoints_created']
assert len({row['number'] for row in created})==len(created)
for point in created:
    assert point['type']==7
    assert point['expression']=='*(unsigned char (*)['+str(point['size'])+'])'+hex(point['start'])
point_ranges={(row['start'],row['size']) for row in created}
calls=[call for row in obs['executions'] for call in row['calls'] if call['number'] in [5,18]]
assert len(calls)==len(native['records'])==len(owner['records'])==52
function,body=kernel.function('topal_x86_systems_static_image_process_enter')
audits=[json.loads((build/name).read_bytes()) for name in ['stat-output-audit.json','file-input-audit.json']]
assert all(a['passed'] and a['kernel_sha256']==sha(kernel.data) for a in audits)
used=set()
table_sets=[]
for call,row,admission in zip(calls,native['records'],owner['records']):
    assert row['number']==call['number']==admission['number']
    assert row['args']==call['args']==admission['args']
    assert row['start']==admission['start'] and row['length']==admission['length'] and row['direction']==admission['direction']
    assert row['copy_completed'] and row['identity_address_cpu_store_changes']==0
    before,after=[snapshots[row[name]] for name in ['before','after']]
    used.update([row['before'],row['after']])
    assert before==owner['snapshots'][admission['before']]
    expected_ranges=regions(before,profile['bindings'],addresses)
    assert row['ranges']==expected_ranges and len(expected_ranges)==row['hardware_watchpoint_count']==26
    assert all((r['start'],r['size']) in point_ranges for r in expected_ranges)
    expected=compare(before,after,profile['bindings'],profile['kernel_spans'],row['start'],row['length'],row['direction'])
    assert row['comparison']==expected
    table_sets.extend(expected['table_status_sets'])
    audit=audits[0 if row['number']==5 else 1]
    case,=[r for r in audit['cases'] if r['call_offset']==row['call_offset']]
    assert case['copy_offset']==row['copy_offset'] and case['copy_size']==row['copy_size']==2
    assert body[row['copy_offset']:row['copy_offset']+2]==b'\xf3\xa4'
    target=row['call_offset']+5+struct.unpack_from('<i',body,row['call_offset']+1)[0]
    assert body[row['call_offset']]==0xe8 and body[target]==0xe9
assert used==set(snapshots)
controls=native['controls']
assert len(controls)==12
for i in range(4):
    group=controls[i*3:i*3+3]
    assert sorted(r['region']['name'] for r in group)==sorted(['control-translation-page','control-mm-state','control-heap-registry'])
    for row in group:
        assert row['kind']=='ordinary-brk-positive-control' and row['epoch'][1]==12 and row['cs']&3==0
        before,after=bytes.fromhex(row['before']),bytes.fromhex(row['after'])
        assert before!=after and len(before)==len(after)==row['region']['size']
        assert (row['region']['start'],row['region']['size']) in point_ranges
        assert row['thread'] and row['rip']>0
assert native['watchpoints_created']>=26
command=json.loads((build/'source-user-extents/qemu.command.json').read_text())
assert command[command.index('-accel')+1]=='tcg,thread=multi'
assert command[command.index('-smp')+1]=='cpus=2,sockets=1,cores=2,threads=1'
report=dict(passed=True,kernel_sha256=sha(kernel.data),observation_sha256=sha(observation.read_bytes()),
    owner_interval_model_sha256=sha(model.encode()),selected_intervals=52,protected_ranges_per_interval=26,
    native_positive_controls=12,interval_snapshots=len(snapshots),watchpoints_created=native['watchpoints_created'],
    observed_identity_address_cpu_store_changes=0,copy_endpoints_and_admissions_bound=True,
    table_status_sets=table_sets,table_status_set_events=len(table_sets),two_cpu_tcg_preserved=True,
    scope='Saved hardware-watch evidence at selected identity virtual addresses and exact admission/copy bindings, with ordinary brk write controls; not physical aliases, DMA, continuous control-register/asynchronous behavior or general stable lifetime')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
