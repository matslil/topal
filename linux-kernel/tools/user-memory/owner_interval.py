"""Selected owner-record watch ranges and permitted hardware table status sets.

TOPAL-SYSTEMS-USER-EXTENT-001. Describes read-only debugger observations, not
kernel loans, physical watchpoints or an exclusion proof for all agents/aliases.
"""
from owner_snapshot import check


def regions(snapshot,bindings,addresses):
    result=[]
    def add(name,start,size):
        assert 0<start<start+size<=1<<30 and 0<size<=16384
        assert all(start+size<=r['start'] or r['start']+r['size']<=start for r in result)
        result.append(dict(name=name,start=start,size=size))
    for table in sorted(map(int,snapshot['tables'])):add('translation-'+str(table),table,4096)
    for name,size in [('root_tokens',bindings['token_bytes']),('mm_tokens',48),
            ('context',128),('mm_state',32),('mm_epoch',16)]:
        add(name,addresses[name],size)
    for name,offset in [('process-state',0),('process-root',24),('process-low',112),
            ('process-self',256),('process-stack-page',912)]:
        add(name,snapshot['context']['address']+offset,8)
    tokens=snapshot['root_tokens']
    selected=[tokens[r['slot']*3:r['slot']*3+3] for r in bindings['roles']]+[snapshot['heap_tokens'][:3]]
    allocations=snapshot['roles']+[snapshot['heap']]
    for i,(token,allocation) in enumerate(zip(selected,allocations)):
        add('registry-'+str(i),addresses['registry']+token[2]*64,64)
        add('cells-'+str(i),addresses['table']+128+allocation['record'][2]*24,len(allocation['cells'])*8)
    assert len(result)==26
    return result


def compare(before,after,bindings,kernel_spans,start,length,write):
    first=check(before,bindings,kernel_spans,start,length,write)
    last=check(after,bindings,kernel_spans,start,length,write)
    assert {k:v for k,v in before.items() if k not in ['tables','flags']}=={k:v for k,v in after.items() if k not in ['tables','flags']}
    allowed={}
    for page in first['pages']:
        for row in page['path']:
            shift={4:39,3:30,2:21,1:12}[row['level']]
            index=(page['virtual']>>shift)&511
            allowed[(str(row['table']),index)]=0x20|(0x40 if write and row['level']==1 else 0)
    changes=[]
    for table,words in before['tables'].items():
        assert len(words)==len(after['tables'][table])==512
        for index,(a,b) in enumerate(zip(words,after['tables'][table])):
            if a==b:continue
            mask=allowed.get((table,index),0)
            assert (a&~mask)==(b&~mask) and ((a&mask)|(b&mask))==(b&mask)
            changes.append(dict(table=int(table),index=index,before=a,after=b,allowed_set_mask=mask))
    assert [(r['virtual'],r['physical'],r['identity']) for r in first['pages']]==[(r['virtual'],r['physical'],r['identity']) for r in last['pages']]
    return dict(passed=True,table_status_sets=changes,
        scope='Equal selected owner/entry/mode records, with monotonic A/D sets on covered paths only; not continuous hardware mode, DMA, physical aliases or general lifetime qualification')
