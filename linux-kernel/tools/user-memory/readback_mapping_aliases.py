"""Independently flatten the fixed boot topology in saved native observations.

TOPAL-SYSTEMS-USER-EXTENT-001. No recursive walker, owner model or interval model
is imported. This second algorithm covers the recorded six-table topology only.
It reads data and ELF identity bytes; it executes no guest program.
"""
import argparse
import hashlib
import json
from pathlib import Path


PAGE=4096
ADDRESS=((1<<52)-1)&-PAGE
NX=1<<63


def digest(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':')).encode()


def flatten(snapshot):
    """Use direct fixed array positions rather than a descriptor graph walk."""
    root=snapshot['context']['root']
    low=snapshot['context']['low']
    tables=snapshot['tables']
    assert set(tables)=={str(t) for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,low]}
    assert all(len(t)==512 and all(type(w) is int and 0<=w<1<<64 for w in t) for t in tables.values())
    l4,l3,main,apic,high,small=[tables[str(t)] for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,low]]
    assert [(i,w&~0x20) for i,w in enumerate(l4) if w]==[(0,root+PAGE|7)],'unexpected L4 topology'
    assert [(i,w&~0x20) for i,w in enumerate(l3) if w]==[(0,root+2*PAGE|7),(3,root+3*PAGE|3)],'unexpected L3 topology'
    assert main[2]&~0x20==low|7 and main[3]&~0x20==root+4*PAGE|7
    assert [(i,w&~0x60) for i,w in enumerate(apic) if w]==[(502,0xfec00000|0x9b),(503,0xfee00000|0x9b)]
    visits=[dict(table=t,level=level,prefix=prefix) for t,level,prefix in [
        (root,4,0),(root+PAGE,3,0),(root+2*PAGE,2,0),(low,1,0x400000),
        (root+4*PAGE,1,0x600000),(root+3*PAGE,2,0xc0000000)]]
    entries=[]
    leaves=[]
    def entry(table,level,index,virtual):
        entries.append(dict(table=table,level=level,index=index,word=tables[str(table)][index],virtual=virtual))
    def leaf(table,index,virtual,size,user):
        word=tables[str(table)][index]
        entries.append(dict(table=table,level=1 if size==PAGE else 2,index=index,word=word,virtual=virtual))
        leaves.append(dict(virtual=virtual,physical=(word&ADDRESS)&-size,size=size,
            user=user,writable=bool(word&2),executable=not bool(word&NX),table=table,index=index))
    entry(root,4,0,0)
    entry(root+PAGE,3,0,0)
    expected={}
    for start,row,count in [(0x400000,snapshot['roles'][2],503),
            (0x7fe000,snapshot['roles'][3],2),(0x5f7000,snapshot['heap'],snapshot['state'][2])]:
        assert len(row['cells'])==count*3
        for i,frame in enumerate(row['cells'][2::3]):expected[start+i*PAGE]=frame*PAGE
    assert len(expected)==572
    floor=0x5ed000 if snapshot['context']['phase']>=9 else 0x5e5000
    for index,word in enumerate(main):
        virtual=index<<21
        if index in [2,3]:
            table=low if index==2 else root+4*PAGE
            entry(root+2*PAGE,2,index,virtual)
            for j,w in enumerate(small if index==2 else high):
                address=virtual+j*PAGE
                if address not in expected:
                    assert w==0,'unexpected small-page mapping'
                    continue
                assert w&5==5 and w&~(ADDRESS|NX|0x67)==0
                assert w&ADDRESS==expected[address],'wrong fixed-topology backing'
                assert not w&2 or address>=floor or address>=0x5f7000
                leaf(table,j,address,PAGE,True)
        else:
            assert word&~0x60==(index<<21)|0x83,'unexpected supervisor identity leaf'
            leaf(root+2*PAGE,index,virtual,1<<21,False)
    entry(root+PAGE,3,3,0xc0000000)
    for index in [502,503]:leaf(root+3*PAGE,index,0xc0000000+(index<<21),1<<21,False)
    assert len(entries)==1089 and len(leaves)==1084
    return dict(leaves=leaves,entries=entries,visits=visits)


def protected_ranges(snapshot,bindings,addresses):
    """Reconstruct the independent range list, without owner_interval.regions."""
    ranges=[]
    def add(name,start,size):ranges.append(dict(name=name,start=start,size=size))
    for table in sorted(map(int,snapshot['tables'])):add('translation-'+str(table),table,PAGE)
    for name,size in [('root_tokens',bindings['token_bytes']),('mm_tokens',48),('context',128),('mm_state',32),('mm_epoch',16)]:
        add(name,addresses[name],size)
    for name,offset in [('process-state',0),('process-root',24),('process-low',112),('process-self',256),('process-stack-page',912)]:
        add(name,snapshot['context']['address']+offset,8)
    for i,row in enumerate(snapshot['roles']+[snapshot['heap']]):
        slot=bindings['roles'][i]['slot'] if i<4 else None
        token=snapshot['root_tokens'][slot*3:slot*3+3] if i<4 else snapshot['heap_tokens'][:3]
        add('registry-'+str(i),addresses['registry']+token[2]*64,64)
        add('cells-'+str(i),addresses['table']+128+row['record'][2]*24,len(row['cells'])*8)
    assert len(ranges)==26
    return ranges


def readback(build,report):
    build=Path(build)
    raw=(build/'source-user-extents/observations.json').read_bytes()
    observation=json.loads(raw)
    assert observation['passed']
    assert report['passed'] and report['observation_sha256']==digest(raw)
    assert report['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
    assert report['source_bindings_sha256']==digest((build/'user-owners.json').read_bytes())
    owner=observation['owner_boundary_observation']
    interval=observation['owner_interval_observation']
    bindings=owner['profile']['bindings']
    assert bindings==json.loads((build/'user-owners.json').read_bytes())
    snapshots=dict(owner['snapshots'])
    for key,snapshot in interval['snapshots'].items():
        assert key not in snapshots or snapshots[key]==snapshot
        snapshots[key]=snapshot
    assert all(digest(canonical(s))==key for key,s in snapshots.items())
    calls=[c for row in observation['executions'] for c in row['calls'] if c['number'] in [5,18]]
    assert len(calls)==len(owner['records'])==len(interval['records'])==52
    expected_events=[]
    used=set()
    used_snapshots=set()
    for index,(call,admission,copy) in enumerate(zip(calls,owner['records'],interval['records'])):
        assert call['args']==admission['args']==copy['args'] and call['result']>=0
        assert call['number']==admission['number']==copy['number']
        assert admission['before']==copy['before']
        assert admission['start']==copy['start']==call['args'][1]
        assert admission['direction']==copy['direction']==int(call['number']==5)
        assert admission['length']==copy['length']==(144 if call['number']==5 else 4)
        for boundary,key in [('helper-entry',admission['before']),('helper-return',admission['after']),('copy-complete',copy['after'])]:
            event=report['events'][len(expected_events)]
            inventory_id=event['inventory_sha256']
            result=report['inventories'][inventory_id]
            assert digest(canonical(result))==inventory_id
            expected_events.append(dict(transfer=index,number=call['number'],boundary=boundary,
                snapshot_sha256=key,inventory_sha256=inventory_id))
            snapshot=snapshots[key]
            flat=flatten(snapshot)
            assert flat==result['inventory'],'flat descriptor inventory disagrees'
            protected=protected_ranges(snapshot,bindings,interval['addresses'])
            assert protected==copy['ranges']
            routes=[]
            for region in protected:
                low,high=region['start'],region['start']+region['size']
                pieces=[]
                for leaf in flat['leaves']:
                    a,b=max(low,leaf['physical']),min(high,leaf['physical']+leaf['size'])
                    if a<b:
                        assert not leaf['user'] and leaf['writable'] and leaf['virtual']==leaf['physical']
                        pieces.append([a,b])
                pieces.sort()
                assert pieces and pieces[0][0]==low and pieces[-1][1]==high
                assert all(a[1]==b[0] for a,b in zip(pieces,pieces[1:]))
                routes.append(dict(name=region['name'],start=low,size=high-low,identity_segments=pieces))
            assert result['passed'] and result['protected_routes']==routes,'protected inverse routes disagree'
            assert result['user_pages']==572 and result['supervisor_leaves']==512
            used.add(inventory_id)
            used_snapshots.add(key)
    assert report['events']==expected_events and len(expected_events)==156
    assert used==set(report['inventories']) and len(used)==6
    assert used_snapshots==set(snapshots) and len(snapshots)==116
    for field,value in [('transfers',52),('boundary_observations',156),('distinct_snapshots',116),
            ('distinct_inventories',6),('user_pages_per_inventory',572),('supervisor_leaves_per_inventory',512),
            ('leaves_per_inventory',1084),('present_entries_per_inventory',1089),
            ('table_visits_per_inventory',6),('protected_ranges_per_inventory',26)]:assert report[field]==value
    return dict(passed=True,observation_sha256=digest(raw),kernel_sha256=report['kernel_sha256'],
        alias_report_sha256=digest(canonical(report)),reader_sha256=digest(Path(__file__).read_bytes()),
        transfers=52,boundaries=156,distinct_snapshots=116,distinct_inventories=6,
        exact_flat_topology_and_inverse_routes=True,
        scope='Independent fixed-topology saved-data reader; no recursive or owner/interval model imports, new VM execution, live TLB/DMA/asynchronous exclusion or general stable lifetime proof')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('report',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    result=readback(args.build,json.loads(args.report.read_bytes()))
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result))
