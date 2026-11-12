"""Read back selected live allocation evidence at one native boundary.

TOPAL-SYSTEMS-USER-EXTENT-001. Saved numeric records describe observations;
they cannot construct kernel owners or establish a continuous observation loan.
"""
import struct

PAGE=4096
ADDRESS=((1<<52)-1)&-PAGE


def words(raw):
    return list(struct.unpack('<'+'Q'*(len(raw)//8),raw))


def separate(a,b):
    return a[1]<=b[0] or b[1]<=a[0]


def check(snapshot,bindings,kernel_spans,start,length,write):
    """Verify live selected owners and flatten each covered translation path.

    This checks the successful subset sampled by the ordinary metadata test.
    It neither assigns authority to records nor models every refusal format.
    """
    allocator=snapshot['allocator']
    assert len(allocator)==16 and allocator[:2]==[1,1]
    issued,n=allocator[2:4]
    assert 0<issued<1<<64 and 0<n<=262144
    tokens=snapshot['root_tokens']
    assert len(tokens)*8==bindings['token_bytes'] and len(tokens)%3==0
    assert len(bindings['roles'])==4 and len({r['slot'] for r in bindings['roles']})==4
    assert [(r['count'],r['pins']) for r in bindings['roles']]==[(5,1),(1,1),(503,1),(2,2)]
    owners=[]
    def owner(row,token,count,pins):
        issuer,identity,index=token
        assert issuer==1 and 0<identity<issued and 0<=index<n
        record,cells=row['record'],row['cells']
        assert len(record)==8 and record[:2]==token[:2]
        assert record[3:5]==[count,1] and record[5]>0 and record[6:]==[pins,1]
        assert 0<=record[2]<record[2]+count<=n
        assert len(cells)==count*3
        first=cells[2]
        assert 0<=first<first+count<=1<<18
        assert cells[0::3]==[identity]*count and cells[1::3]==[record[5]]*count
        assert cells[2::3]==list(range(first,first+count))
        span=(first*PAGE,(first+count)*PAGE)
        assert all(separate(span,other) for other in kernel_spans)
        assert all(separate(span,old['span']) and identity!=old['identity'] for old in owners)
        result=dict(identity=identity,span=span)
        owners.append(result)
        return result
    assert len(snapshot['roles'])==4
    for row,role in zip(snapshot['roles'],bindings['roles']):
        at=role['slot']*3
        owner(row,tokens[at:at+3],role['count'],role['pins'])
    state=snapshot['state']
    assert len(state)==4 and state[0]==1 and 0<state[2]<=519
    assert state[1]==0x5f7000+state[2]*PAGE
    heap_tokens=snapshot['heap_tokens']
    assert len(heap_tokens)==6
    heap=owner(snapshot['heap'],heap_tokens[:3],state[2],1)
    assert heap['span'][0]==state[3]
    assert heap['identity'] not in tokens[1::3]
    context=snapshot['context']
    assert context['state']==1 and context['self']==context['address']==snapshot['gs_base']
    root=owners[0]['span'][0]
    assert context['root']==snapshot['cr3']==root
    assert context['low']==owners[1]['span'][0]
    assert context['stack_page']==owners[3]['span'][0]+PAGE
    assert snapshot['epoch'][0]>0 and snapshot['epoch'][1] in [5,18]
    assert snapshot['cs']&3==0 and snapshot['flags']&0x600==0
    assert snapshot['cr0']&0x80010001==0x80010001
    assert snapshot['cr4']&31592448==0
    tables=snapshot['tables']
    table_roles={root:4,root+PAGE:3,root+2*PAGE:2,root+3*PAGE:2,
        root+4*PAGE:1,owners[1]['span'][0]:1}
    assert set(tables)=={str(at) for at in table_roles}
    assert all(len(tables[str(at)])==512 for at in table_roles)
    assert 0<length<=144 and 0<=start<start+length<=1<<47 and write in [0,1]
    pages=[]
    for virtual in range(start&-PAGE,start+length,PAGE):
        table=root
        path=[]
        for level,shift in [(4,39),(3,30),(2,21),(1,12)]:
            assert table_roles.get(table)==level
            entry=tables[str(table)][(virtual>>shift)&511]
            assert entry&~(ADDRESS|(1<<63)|0x67)==0
            assert entry&(7 if write else 5)==(7 if write else 5)
            path.append(dict(table=table,level=level,entry=entry))
            table=entry&ADDRESS
        if 0x400000<=virtual<0x5f7000:
            selected=owners[2]
            offset=virtual-0x400000
            assert not write or virtual>=(0x5ed000 if context['phase']>=9 else 0x5e5000)
        elif 0x7fe000<=virtual<0x800000:
            selected=owners[3]
            offset=virtual-0x7fe000
        else:
            assert 0x5f7000<=virtual<state[1]
            selected=heap
            offset=virtual-0x5f7000
        assert table==selected['span'][0]+offset
        assert table not in table_roles
        pages.append(dict(virtual=virtual,physical=table,identity=selected['identity'],path=path))
    return dict(owners=owners,pages=pages,write=write,start=start,length=length,
        scope='Live selected allocation/translation evidence at this boundary; no continuous lifetime, all-caller or production qualification')
