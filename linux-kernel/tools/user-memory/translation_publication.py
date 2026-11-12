"""Saved-data checks for the actual local heap publication/invalidation order.

TOPAL-SYSTEMS-USER-EXTENT-001. Numeric records describe observations; they do not
create mapping authority, inspect a TLB or establish AP/DMA/async exclusion.
"""
PAGE=4096
HEAP=0x5f7000
LEAF=(1<<63)|7


def controls(snapshot):
    assert snapshot['cr0']&0x80010001==0x80010001
    assert snapshot['cr4']&0x20 and snapshot['cr4']&((1<<7)|(1<<12)|(1<<17))==0
    assert snapshot['cs']&3==0 and snapshot['flags']&0x600==0
    return tuple(snapshot[k] for k in ['cr0','cr3','cr4','cs'])


def check_switch(record):
    before,after=record['before'],record['after']
    first,last=controls(before),controls(after)
    ctx=before['context']
    assert ctx==after['context'] and ctx['state']==1
    assert before['thread']==after['thread']
    assert all(0<ctx[k]<1<<30 and ctx[k]%PAGE==0 for k in ['root','bootstrap'])
    assert ctx['root']!=ctx['bootstrap']
    assert first[1]==ctx['root'] and last[1]==ctx['bootstrap']
    assert first[0]==last[0] and first[2:]==last[2:]
    return dict(passed=True,target_root=ctx['root'],bootstrap_root=ctx['bootstrap'],
        scope='Observed local CR3 replacement with PCID/global paging disabled; no other CPU or continuous-mode proof')


def mode(snapshot,detachment=None):
    current=controls(snapshot)
    ctx=snapshot['context']
    assert ctx['root']%PAGE==0
    if snapshot['cr3']==ctx['root']:
        assert ctx['state']==1 and detachment is None
    else:
        assert detachment is not None,'detached root requires its observed flushing handoff'
        check_switch(detachment)
        after=detachment['after']
        assert ctx['state']==2 and snapshot['cr3']==ctx['bootstrap']
        assert {k:v for k,v in ctx.items() if k!='state'}=={k:v for k,v in after['context'].items() if k!='state'}
        assert current==controls(after) and snapshot['thread']==after['thread']
    return current


def slot(snapshot,index):
    assert 0<=index<519
    ctx=snapshot['context']
    return (str(ctx['low']),503+index) if index<9 else (str(ctx['root']+16384),index-9)


def published(before,after,old,new,base,detachment=None):
    assert 0<=old<=519 and 0<=new<=519 and old!=new
    assert before['state'][2]==old
    assert base%PAGE==0 and (base>0 if new else base==0)
    assert 0<=base<=base+new*PAGE<=1<<30
    if detachment is not None:assert old>0 and new==0,'detached publication must revoke the whole heap'
    assert mode(before,detachment)==mode(after,detachment)
    assert before['thread']==after['thread'] and before['context']==after['context']
    assert before['state']==after['state'] and before['tokens']==after['tokens']
    root=before['context']['root']
    expected={str(t) for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,before['context']['low']]}
    assert set(before['tables'])==set(after['tables'])==expected
    selected={slot(before,i):((base+i*PAGE)|LEAF if i<new else 0) for i in range(max(old,new))}
    for table,words in before['tables'].items():
        actual=after['tables'][table]
        assert len(words)==len(actual)==512
        for index,(a,b) in enumerate(zip(words,actual)):
            assert type(a) is int and type(b) is int and 0<=a<1<<64 and 0<=b<1<<64
            if (table,index) in selected:
                assert b&~0x60==selected[(table,index)],'published heap leaf differs'
            else:
                assert a&~0x60==b&~0x60 and (a|b)==b,'unrelated translation changed'
    for i in range(519):
        table,index=slot(before,i)
        old_expected=((before['state'][3]+i*PAGE)|LEAF) if i<old else 0
        assert before['tables'][table][index]&~0x60==old_expected,'old heap leaf differs'
    return max(old,new)


def check(record):
    old,new,base=record['old_count'],record['new_count'],record['new_base']
    detachment=record.get('detachment')
    if detachment is not None:assert record['operation']==6
    count=published(record['before'],record['stored'],old,new,base,detachment)
    assert record['published_before_invalidations'],'missing publication barrier observation'
    assert len(record['invalidations'])==count,'incomplete invalidation interval'
    for index,event in enumerate(record['invalidations']):
        assert event['index']==index and event['address']==HEAP+index*PAGE,'wrong invalidation page/order'
        assert event['thread']==record['before']['thread'],'local invalidation changed CPU'
    assert record['after']['tables']==record['stored']['tables'],'translations changed during invalidation'
    assert mode(record['after'],detachment)==mode(record['before'],detachment)
    assert record['after']['thread']==record['before']['thread']
    assert record['after']['context']==record['before']['context']
    assert record['after']['state']==record['before']['state']
    assert record['after']['tokens']==record['before']['tokens']
    assert record['return_stack']==record['entry_stack'] and record['return_address']==record['entry_return_address']
    assert record['final_barrier_observed'],'missing final publication barrier'
    return dict(passed=True,pages_invalidated=count,
        scope='Recorded active heap publication or whole detached cleanup bound to its observed flushing CR3 handoff, plus current-context post-INVLPG boundaries and final barrier. Not TLB introspection, continuous mode, AP/DMA/async or general stable ownership qualification')
