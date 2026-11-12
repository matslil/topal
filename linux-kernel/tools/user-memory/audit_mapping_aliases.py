"""Join complete saved paging graphs to actual admitted transfers and watches.

TOPAL-SYSTEMS-USER-EXTENT-001. Reads recorded data and ELF metadata only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

from alias_snapshot import audit
from inspect_call_inventory import Elf
from owner_interval import regions


def digest(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':')).encode()


def audit_build(build):
    build=Path(build)
    raw=(build/'source-user-extents/observations.json').read_bytes()
    observation=json.loads(raw)
    assert observation['passed'] and len(observation['executions'])==4
    owner=observation['owner_boundary_observation']
    interval=observation['owner_interval_observation']
    assert owner['passed'] and interval['passed']
    kernel=Elf(build/'kernel.elf')
    profile=owner['profile']
    assert profile['kernel_sha256']==digest(kernel.data)
    assert profile['bindings']==json.loads((build/'user-owners.json').read_bytes())
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
    addresses=interval['addresses']
    for suffix in ['context','root_tokens','registry','table','mm_tokens','mm_epoch','mm_state']:
        assert addresses[suffix]==kernel.symbols['topal_x86_frame_'+suffix+'_v1']['value']
    context=kernel.symbols['topal_bootstrap_storage']['value']+512
    snapshots={}
    for group in [owner['snapshots'],interval['snapshots']]:
        for key,snapshot in group.items():
            assert digest(canonical(snapshot))==key
            assert key not in snapshots or snapshots[key]==snapshot
            assert snapshot['context']['address']==context
            assert snapshot['allocator'][4]==addresses['table']
            assert snapshot['allocator'][6]==addresses['registry']
            snapshots[key]=snapshot
    calls=[call for row in observation['executions'] for call in row['calls'] if call['number'] in [5,18]]
    assert len(calls)==len(owner['records'])==len(interval['records'])==52
    inventories={}
    events=[]
    used=set()
    for index,(call,admission,copy) in enumerate(zip(calls,owner['records'],interval['records'])):
        assert call['number']==admission['number']==copy['number']
        assert call['args']==admission['args']==copy['args'] and call['result']>=0
        assert admission['before']==copy['before']
        assert (admission['start'],admission['length'],admission['direction'])==(copy['start'],copy['length'],copy['direction'])
        assert copy['start']==call['args'][1] and copy['direction']==int(call['number']==5)
        assert copy['length']==(144 if call['number']==5 else 4)
        for boundary,key in [('helper-entry',admission['before']),('helper-return',admission['after']),('copy-complete',copy['after'])]:
            snapshot=snapshots[key]
            used.add(key)
            assert snapshot['epoch'][1]==call['number']
            protected=regions(snapshot,profile['bindings'],addresses)
            assert protected==copy['ranges']
            result=audit(snapshot,profile['bindings'],profile['kernel_spans'],
                copy['start'],copy['length'],copy['direction'],protected)
            encoded=canonical(result)
            identity=digest(encoded)
            assert identity not in inventories or canonical(inventories[identity])==encoded
            inventories[identity]=result
            events.append(dict(transfer=index,number=call['number'],boundary=boundary,
                snapshot_sha256=key,inventory_sha256=identity))
    assert used==set(snapshots) and len(events)==156
    assert sum(call['number']==5 for call in calls)==44
    counts={(r['user_pages'],r['supervisor_leaves'],len(r['inventory']['leaves']),
        len(r['inventory']['entries']),len(r['inventory']['visits']),len(r['protected_routes'])) for r in inventories.values()}
    assert counts=={(572,512,1084,1089,6,26)}
    return dict(passed=True,kernel_sha256=digest(kernel.data),observation_sha256=digest(raw),
        alias_model_sha256=digest(Path(__file__).with_name('alias_snapshot.py').read_bytes()),
        reader_sha256=digest(Path(__file__).read_bytes()),
        source_bindings_sha256=digest((build/'user-owners.json').read_bytes()),
        transfers=52,boundary_observations=156,distinct_snapshots=len(snapshots),
        distinct_inventories=len(inventories),user_pages_per_inventory=572,supervisor_leaves_per_inventory=512,
        leaves_per_inventory=1084,present_entries_per_inventory=1089,table_visits_per_inventory=6,
        protected_ranges_per_inventory=26,events=events,inventories=inventories,
        scope='Complete recorded six-table descriptor graphs and inverse mappings at observed boundaries only. User mappings match live image/stack/heap owners; protected ranges have only supervisor writable identity routes. No stale TLB, DMA, continuous mode, asynchronous-entry or general lifetime exclusion; no new VM execution or production activation')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=audit_build(args.build)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({key:report[key] for key in ['passed','transfers','boundary_observations',
        'distinct_snapshots','distinct_inventories','user_pages_per_inventory','supervisor_leaves_per_inventory',
        'leaves_per_inventory','present_entries_per_inventory','protected_ranges_per_inventory']}))
