"""Saved-data checks of selected image-leaf publication and local invalidation.

TOPAL-SYSTEMS-USER-EXTENT-001. Records cannot create ownership or qualify
unobserved entries, CPUs, DMA, faults, asynchronous behavior or stable lifetime.
"""
PAGE=4096
BASE=0x5e5000
NX=1<<63


def owners(snapshot,bindings,kernel_spans,kind):
    ctx=snapshot['context']
    assert kind in ['relro','exec']
    generation=snapshot['image_generation']
    assert ctx['state']==1 and generation in [0,1]
    if kind=='relro':assert ctx['phase']==8
    elif generation==0:assert ctx['phase']==32
    else:assert 11<=ctx['phase']<=15
    assert snapshot['epoch'][0]>0 and snapshot['epoch'][1]==(10 if kind=='relro' else 59)
    assert ctx['address']==ctx['self']==snapshot['gs_base']
    assert snapshot['cr0']&0x80010001==0x80010001 and snapshot['cs']&3==0
    assert snapshot['cr4']&0x20 and snapshot['cr4']&((1<<7)|(1<<12)|(1<<17)|31592448)==0
    assert snapshot['flags']&0x600==0
    allocator=snapshot['allocator']
    assert len(allocator)==16 and allocator[:2]==[1,1] and 0<allocator[3]<=262144
    tokens=snapshot['root_tokens']
    roles=bindings['roles']
    assert len(tokens)*8==bindings['token_bytes'] and len(tokens)%3==0
    assert [(r['count'],r['pins']) for r in roles]==[(5,1),(1,1),(503,1),(2,2)]
    assert len({r['slot'] for r in roles})==len(snapshot['roles'])==4
    extents=[]
    ids=[]
    def allocation(row,token,count,pins):
        issuer,identity,index=token
        assert issuer==1 and 0<identity<allocator[2] and 0<=index<allocator[3]
        record,cells=row['record'],row['cells']
        assert len(record)==8 and record[:2]==token[:2]
        assert record[3:5]==[count,1] and record[5]>0 and record[6:]==[pins,1]
        assert 0<=record[2]<record[2]+count<=allocator[3]
        assert len(cells)==count*3 and cells[0::3]==[identity]*count and cells[1::3]==[record[5]]*count
        first=cells[2]
        assert 0<first<first+count<=1<<18 and cells[2::3]==list(range(first,first+count))
        span=[first*PAGE,(first+count)*PAGE]
        assert all(span[1]<=a or b<=span[0] for a,b in kernel_spans+extents)
        assert identity not in ids
        extents.append(span)
        ids.append(identity)
        return span[0]
    bases=[]
    for row,role in zip(snapshot['roles'],roles):
        slot=role['slot']
        assert 0<=slot<len(tokens)//3
        bases.append(allocation(row,tokens[slot*3:slot*3+3],role['count'],role['pins']))
    mm=snapshot['state']
    assert len(mm)==4 and mm[0]==1 and 0<mm[2]<=519 and mm[1]==0x5f7000+mm[2]*PAGE
    assert len(snapshot['heap_tokens'])==6 and snapshot['heap_tokens'][3:]==[0,0,0]
    heap=allocation(snapshot['heap'],snapshot['heap_tokens'][:3],mm[2],1)
    assert heap==mm[3] and snapshot['heap_tokens'][1] not in tokens[1::3]
    root,low,image,stack=bases
    assert ctx['root']==snapshot['cr3']==root and ctx['low']==low and ctx['stack_page']==stack+PAGE
    assert set(snapshot['tables'])=={str(p) for p in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,low]}
    assert all(len(words)==512 and all(type(w) is int and 0<=w<1<<64 for w in words) for words in snapshot['tables'].values())
    # Bind the effective ancestor path for the complete changed image interval.
    for table,index,next_table in [(root,0,root+PAGE),(root+PAGE,0,root+2*PAGE),(root+2*PAGE,2,low)]:
        assert snapshot['tables'][str(table)][index]&~0x60==next_table|7
    return image


def published(record,bindings,kernel_spans):
    before,after=record['before'],record['stored']
    kind=record['kind']
    image=owners(before,bindings,kernel_spans,kind)
    assert owners(after,bindings,kernel_spans,kind)==image
    assert {k:v for k,v in before.items() if k not in ['tables','flags']}=={k:v for k,v in after.items() if k not in ['tables','flags']}
    low=str(before['context']['low'])
    count=8 if kind=='relro' else 18
    selected=set(range(485,485+count))
    for table,words in before['tables'].items():
        for index,(old,new) in enumerate(zip(words,after['tables'][table])):
            if table==low and index in selected:
                page=index-485
                physical=image+(BASE-0x400000)+page*PAGE
                old_flags=7 if kind=='relro' or page>=8 else 5
                new_flags=5 if kind=='relro' else 7
                assert old&~0x60==physical|NX|old_flags,'old image leaf differs'
                assert new&~0x60==physical|NX|new_flags,'published image leaf differs'
            else:assert old&~0x60==new&~0x60 and old|new==new,'unrelated translation changed'
    assert record['publication_barrier_observed']
    return count


def check(record,bindings,kernel_spans):
    count=published(record,bindings,kernel_spans)
    before,after=record['before'],record['after']
    assert owners(after,bindings,kernel_spans,record['kind'])==owners(before,bindings,kernel_spans,record['kind'])
    assert {k:v for k,v in after.items() if k!='flags'}=={k:v for k,v in record['stored'].items() if k!='flags'}
    assert record['thread_before']==record['thread_stored']==record['thread_after']
    assert len(record['invalidations'])==count,'incomplete image invalidation interval'
    for index,event in enumerate(record['invalidations']):
        assert event['index']==count-index and event['address']==BASE+index*PAGE,'wrong image invalidation address/order'
        assert before['cr0']==event['cr0'] and before['cr3']==event['cr3'] and before['cr4']==event['cr4']
        assert event['cpu']+1==record['thread_before'][1] and record['thread_before'][0]==1
        assert event['flags']&0x600==0 and event['efer']&0xd00==0xd00
    assert record['final_barrier_observed']==(record['kind']=='relro')
    return dict(passed=True,pages_invalidated=count,
        scope='Selected actual image-owner boundaries, complete local leaf edits and ordered post-INVLPG states before reuse/readback. Exec has a publication barrier and no claimed extra final MFENCE. Not continuous mode, AP/DMA/asynchronous exclusion, arbitrary mapping editors or stable lifetime qualification')
