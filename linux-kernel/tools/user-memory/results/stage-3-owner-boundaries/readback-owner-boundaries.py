"""Check saved actual-owner boundaries against linked ELF and ordinary calls."""
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf
from owner_snapshot import check

build,observations,output=map(Path,sys.argv[1:4])
obs=json.loads(observations.read_bytes())
assert obs['passed'] and len(obs['executions'])==4
native=obs['owner_boundary_observation']
assert native['passed']
profile=native['profile']
kernel=Elf(build/'kernel.elf')
sha=lambda data:hashlib.sha256(data).hexdigest()
assert profile['kernel_sha256']==sha(kernel.data)
model=Path('linux-kernel/tools/user-memory/owner_snapshot.py').read_text()
assert profile['model']==model and native['model_sha256']==sha(model.encode())
assert ('owner_snapshot_model = {}\nexec('+repr(model)+', owner_snapshot_model)') in (build/'source-user-extents/inspect.gdb').read_text()
assert profile['bindings']==json.loads((build/'user-owners.json').read_text())
phoff=struct.unpack_from('<Q',kernel.data,32)[0]
phsize,phcount=struct.unpack_from('<HH',kernel.data,54)
assert phsize==56
spans=[]
for i in range(phcount):
    kind,flags,offset,virtual,physical,size,memsize,align=struct.unpack_from('<II6Q',kernel.data,phoff+i*phsize)
    if kind==1:
        assert virtual==physical and memsize and physical+memsize<=1<<30
        spans.append([physical,physical+memsize])
assert spans==profile['kernel_spans']
target=(build/'target.ll').read_text()
body=re.search(r'define internal i64 @u.static_owner\(.*?\n\}',target,re.S).group()
for alias,role in zip(['h','l','m','s'],profile['bindings']['roles']):
    assert f'%{alias} = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {role["slot"]*24}' in body
    assert f'call i64 @u.owner(ptr %{alias},i64 {role["count"]},i64 {role["pins"]})' in body
context=kernel.symbols['topal_bootstrap_storage']['value']+512
snapshots=native['snapshots']
for key,snapshot in snapshots.items():
    assert sha(json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode())==key
    assert snapshot['context']['address']==context
    assert snapshot['allocator'][4]==kernel.symbols['topal_x86_frame_table_v1']['value']
    assert snapshot['allocator'][6]==kernel.symbols['topal_x86_frame_registry_v1']['value']
calls=[call for row in obs['executions'] for call in row['calls'] if call['number'] in [5,18]]
assert len(calls)==len(native['records'])==52
referenced=set()
pages=0
for call,row in zip(calls,native['records']):
    assert call['number']==row['number'] and call['args']==row['args'] and call['result']>=0
    assert row['direction']==int(call['number']==5)
    assert row['start']==call['args'][1] and row['length']==(144 if call['number']==5 else 4)
    before,after=[snapshots[row[k]] for k in ['before','after']]
    referenced.update([row['before'],row['after']])
    assert {k:v for k,v in before.items() if k!='flags'}=={k:v for k,v in after.items() if k!='flags'}
    for snapshot in [before,after]:
        assert snapshot['epoch'][1]==call['number']
        evidence=check(snapshot,profile['bindings'],spans,row['start'],row['length'],row['direction'])
        assert json.loads(json.dumps(evidence))==row['evidence']
    pages+=len(row['evidence']['pages'])
assert referenced==set(snapshots)
assert sum(row['number']==5 for row in native['records'])==44
assert sum(row['number']==18 for row in native['records'])==8
report=dict(passed=True,kernel_sha256=sha(kernel.data),observation_sha256=sha(observations.read_bytes()),
    owner_model_sha256=sha(model.encode()),selected_native_helpers=52,stat_helpers=44,pwrite_helpers=8,
    boundary_snapshots=len(snapshots),translated_pages=pages,before_after_owners_tables_entry_mode_equal=True,
    actual_elf_token_bindings_equal=True,source_calls_joined=True,
    scope='Exact saved boundary evidence and linked symbol/source bindings only; no continuous lifetime, complete callee/private-counterpart or production qualification')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
