"""Saved process-MM publication checks, including equal-size owner changes.

TOPAL-SYSTEMS-PROCESS-MM-001. Records do not create ownership, inspect hardware
TLBs or prove unobserved mapping writers, continuous mode or AP/DMA exclusion.
"""
from pathlib import Path
import sys

if 'owner' not in globals():
    from process_native import owner, require
if 'slot' not in globals():
    sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'linux-kernel/tools/user-memory'))
    from translation_publication import HEAP, LEAF, PAGE, mode, slot


def bank(snapshot,name,state_name):
    state=snapshot[state_name]
    allocation=owner(snapshot[name],snapshot[name]['token'],snapshot['allocator'],False)
    require(len(state) in [4,7], 'publication bank width')
    if state==[0]*len(state):
        require(allocation is None, 'publication owner without bank')
        return None
    require(state[0]==1 and HEAP<=state[1]<=HEAP+519*PAGE
            and state[2]==(state[1]-HEAP+PAGE-1)//PAGE, 'publication byte break')
    if state[2]:
        require(allocation is not None and allocation['count']==state[2]
                and allocation['span'][0]==state[3] and allocation['pins']==1
                and allocation['record'][4]==1, 'publication bank backing/pin')
    else:require(allocation is None and state[3]==0, 'empty publication bank')
    return allocation


def owners(snapshot):
    allocator=snapshot['allocator']
    require(len(allocator)==16 and allocator[:2]==[1,1], 'publication allocator identity')
    values={name:bank(snapshot,name,state) for name,state in [
        ('active','state'),('inactive','inactive_state'),('prepared','prepared_state')]}
    values['candidate']=owner(snapshot['candidate'],snapshot['tokens'][3:],allocator,False)
    require(snapshot['tokens'][:3]==snapshot['active']['token'], 'publication active token')
    roots=[owner(row,row['token'],allocator,False) for row in snapshot['root_owners']]
    require(all(row is not None for row in roots), 'publication retained root allocation')
    require(any(row['span'][0]==snapshot['context']['root'] and row['count']==5 and row['pins']==1 for row in roots)
            and any(row['span'][0]==snapshot['context']['low'] and row['count']==1 and row['pins']==1 for row in roots),
            'publication owned translation storage')
    spans=list(snapshot['kernel_spans']);identities=set()
    for allocation in list(values.values())+roots:
        if allocation is None:continue
        a,b=allocation['span'];identity=tuple(allocation['token'][:2])
        require(identity not in identities and all(b<=x or y<=a for x,y in spans),
                'publication backing alias')
        spans.append([a,b]);identities.add(identity)
    return values


def target(record,values):
    before=record['before'];operation=record['operation']
    ids=before['ids']
    require(len(ids)==7 and ids[0]>0 and 0<ids[1]<ids[6], 'publication active MM identity')
    if operation==5:return values['candidate']
    if operation in [6,7]:return None
    if operation==10:
        state=before['prepared_state']
        require(state[0]==1 and before['prepared_owner']==ids[:3]
                and state[4:]==[before['context']['root'],before['context']['address'],before['epoch'][0]],
                'publication prepared receipt')
        require(ids[0]==1 and ids[3:6]==[0]*3 and 2<=before['transition'][0]<=2147483647
                and before['transition'][1]!=ids[1] and 0<before['transition'][1]+1==ids[6]
                and before['transition'][2]==ids[2], 'publication creation identity')
        return values['prepared']
    require(operation in [11,12], 'operation cannot publish process heap')
    require(ids[3]>0 and ids[3]!=ids[0] and 0<ids[4]<ids[6] and ids[4]!=ids[1]
            and before['inactive_state'][0]==1, 'publication inactive MM identity')
    if operation==12:require(ids[0]>=2 and ids[3]==1, 'publication child-to-parent return')
    return values['inactive']


def check(record):
    before,stored,after=[record[key] for key in ['before','stored','after']]
    old,new,base=[record[key] for key in ['old_count','new_count','new_base']]
    detached=record.get('detachment')
    require(0<=old<=519 and 0<=new<=519 and old==before['state'][2], 'publication page geometry')
    require(detached is None or record['operation']==6, 'publication detached operation')
    require(mode(before,detached)==mode(stored,detached)==mode(after,detached), 'publication processor mode')
    require(set(before)==set(stored)==set(after), 'publication snapshot closure')
    for key in before:
        if key not in ['tables','flags']:
            require(before[key]==stored[key]==after[key], 'publication changed '+key)
    values=owners(before);selected=target(record,values)
    require(base%PAGE==0 and 0<=base<=base+new*PAGE<=1<<30, 'publication physical geometry')
    if selected is None:require(new==base==0, 'publication without target owner')
    else:require(selected['pins']==1 and selected['count']==new and selected['span'][0]==base,
                 'publication target extent')
    if detached is not None:require(old>0 and new==0, 'detached whole-heap publication')
    root=before['context']['root']
    tables={str(t) for t in [root,root+PAGE,root+2*PAGE,root+3*PAGE,root+4*PAGE,before['context']['low']]}
    require(set(before['tables'])==set(stored['tables'])==set(after['tables'])==tables,
            'publication complete translation graph')
    changed={slot(before,i):((base+i*PAGE)|LEAF if i<new else 0) for i in range(max(old,new))}
    for table,words in before['tables'].items():
        actual=stored['tables'][table]
        require(len(words)==len(actual)==512, 'publication complete table')
        for index,(a,b) in enumerate(zip(words,actual)):
            require(type(a) is int and type(b) is int and 0<=a<1<<64 and 0<=b<1<<64,
                    'publication table word')
            if (table,index) in changed:require(b&~0x60==changed[(table,index)], 'publication selected leaf')
            else:require(a&~0x60==b&~0x60 and (a|b)==b, 'publication unrelated translation')
    for i in range(519):
        table,index=slot(before,i)
        expected=((before['state'][3]+i*PAGE)|LEAF) if i<old else 0
        require(before['tables'][table][index]&~0x60==expected, 'publication old owner mappings')
    count=max(old,new)
    require(record['published_before_invalidations'] and record['final_barrier_observed'],
            'publication barrier observations')
    require(len(record['invalidations'])==count, 'publication complete invalidation interval')
    for index,event in enumerate(record['invalidations']):
        require(event['index']==index and event['address']==HEAP+index*PAGE
                and event['thread']==before['thread'], 'publication invalidation order/CPU')
    require(after['tables']==stored['tables'], 'publication translation changed during invalidation')
    require(record['return_stack']==record['entry_stack']
            and record['return_address']==record['entry_return_address'], 'publication return identity')
    return dict(passed=True,pages_invalidated=count,operation=record['operation'],
        source_token=before['active']['token'],target_token=selected['token'] if selected is not None else [0]*3,
        scope='Saved selected process publication and local invalidation boundaries; not TLB introspection, continuous lifetime, all mapping writers, AP/DMA/async exclusion or public activation')


def retirement(record,publications):
    """Check one observed heap unpin/erase/release against recorded local flushes."""
    import hashlib
    before=record['before'];values=owners(before)
    operation,role=record['operation'],record['bank']
    require((operation==13 and role=='inactive' and before['ids'][0]==1)
            or (operation in [5,6,7,12] and role=='active'), 'retirement process role')
    allocation=values[role]
    require(allocation is not None and record['owner']==before[role], 'retirement original owner')
    token=allocation['token'];index=record['publication_index']
    require(0<=index<len(publications), 'retirement missing completed publication')
    results=[check(row) for row in publications]
    require(results[index]['source_token']==token
            and all(row['target_token']!=token for row in results[index:]), 'retirement source remains unmapped')
    if operation!=13:require(index==len(publications)-1
            and publications[index]['operation']==operation, 'retirement current publication')
    last=publications[-1]
    require(mode(before,record.get('detachment'))==mode(last['after'],last.get('detachment'))
            and before['thread']==last['after']['thread'], 'retirement publication CPU/mode')
    require(all(before['context'][key]==last['after']['context'][key]
                for key in ['address','root','low','bootstrap']), 'retirement publication context')
    require(set(before['tables'])==set(last['after']['tables']), 'retirement complete tables')
    for table,words in last['after']['tables'].items():
        current=before['tables'][table]
        require(len(current)==len(words)==512 and all(a&~0x60==b&~0x60 and a|b==b for a,b in zip(words,current)),
                'retirement mapping changed after publication')
    a,b=allocation['span']
    for i in range(519):
        table,position=slot(before,i);pte=before['tables'][table][position]
        if pte&1:require(not a<=(pte&0x000ffffffffff000)<b, 'retirement still mapped heap')
    require(record['unpin_succeeded'] is True, 'retirement unpin result')
    unpinned=record['unpinned']
    expected=allocation['record'][:]
    expected[6]=0
    require(unpinned['record']==expected and unpinned['token']==token
            and unpinned['cells']==allocation['cells'] and unpinned['bytes']=='', 'retirement unpin effect')
    erased=record['erase'];release=record['release']
    require(erased==dict(base=a,count=allocation['count']), 'retirement erase arguments')
    size=allocation['count']*PAGE;zero=hashlib.sha256(bytes(size)).hexdigest()
    require(release==dict(base=a,count=allocation['count'],erased_bytes=size,erased_sha256=zero),
            'retirement complete erasure before release')
    after=record['released']
    cells=[value for i in range(allocation['count']) for value in [0,allocation['cells'][i*3+1],allocation['cells'][i*3+2]]]
    require(record['release_succeeded'] is True and after['token']==[0]*3
            and after['record']==[0]*8 and after['cells']==cells
            and after['erased_sha256']==zero, 'retirement consumed owner/free cells')
    return dict(passed=True,erased_bytes=size,token=token,publication_index=index,
        scope='Saved local publication before heap unpin, complete erasure before release and consumed record; no continuous exclusion, other mapping writers, AP/DMA/async or fatal-path qualification')
