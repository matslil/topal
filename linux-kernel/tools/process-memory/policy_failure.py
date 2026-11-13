"""Selected source-policy failures retain effects and retire the invocation.

TOPAL-SYSTEMS-PROCESS-MM-001. Diagnostic producers are not allocator
qualification. Records prove neither all fatal paths nor public activation.
"""
import hashlib
import struct

if 'snapshot' not in globals():
    from process_native import owner, require, snapshot


def sha_words(values):
    return hashlib.sha256(struct.pack('<'+'Q'*len(values),*values)).hexdigest()


def tables(first,last):
    require(set(first)==set(last),'failure complete table graph')
    for name,words in first.items():
        actual=last[name]
        require(len(words)==len(actual)==512 and all(a&~0x60==b&~0x60 and a|b==b for a,b in zip(words,actual)),
                'failure unexpected mapping change')


def unchanged(first,last,poison=False,cell_tag=None):
    require(set(first)==set(last),'failure snapshot closure')
    for name,value in first.items():
        if name=='tables':tables(value,last[name])
        elif name=='flags':require(last[name]&0x600==0,'failure interrupt/direction state')
        elif name=='allocator':
            expected=value[:];expected[0]=3 if poison else 2
            require(last[name]==expected,'failure allocator effects/issuance retention')
        elif name=='state' and poison:
            require(last[name]==[3]+value[1:],'failure MM poison and retained geometry')
        elif name=='details' and poison:require(last[name]==[0]*4,'failure details retirement')
        elif name=='active' and cell_tag is not None:
            expected=dict(value)
            expected['cells']=[v for i,word in enumerate(value['cells']) for v in [cell_tag if i%3==0 else word]]
            require(last[name]==expected,'failure retained active owner and source delta')
        else:require(last[name]==value,'failure changed '+name)


def first_fit(catalog,count):
    start=length=0;previous=None
    for i in range(len(catalog)//3):
        identity,tag,physical=catalog[i*3:i*3+3]
        if identity==0 and tag>0:
            if previous!=(tag,physical-1):start=i;length=0
            length+=1;previous=(tag,physical)
            if length==count:return start
        else:length=0;previous=None
    raise ValueError('diagnostic allocation had no actual source span')


def check(value):
    kind=value['kind'];require(kind in ['allocate-metadata','release-metadata'],'source failure kind')
    request=value['request'];before=request['before'];snapshot(before)
    require(request['operation']==5 and request['request']==0x5f7000+101*4096-11
            and before['ids'][0]>=2 and before['ids'][3]==1
            and before['state'][2]==before['inactive_state'][2]==67,'failure ordinary child resize')
    source=value['source_before'];effects=value['effects'];terminal=value['terminal']
    a=source['snapshot'];b=effects['snapshot'];c=terminal['snapshot']
    require(a['allocator'][0]==1 and a['ids']==before['ids'],'failure actual source entry')
    for bank in ['active','inactive']:
        require(a[bank]['token']==before[bank]['token'],'failure actual process owner')
    catalog=source['catalog'];n=a['allocator'][3]
    require(len(catalog)==n*3 and sha_words(catalog)==source['catalog_sha256'],'failure complete initial catalog')
    require(source['registry_sha256']==effects['registry_sha256']==terminal['registry_sha256'],
            'failure retained whole registry')
    require(effects['snapshot_catalog_sha256']==terminal['snapshot_catalog_sha256']==source['catalog_sha256'],
            'failure comparison snapshot identity')
    count=101 if kind=='allocate-metadata' else 67
    if kind=='allocate-metadata':
        require(a['candidate']['token']==[0]*3,'failure unpublished candidate token')
        start=first_fit(catalog,count);identity=a['allocator'][2];provenance=0;initial=0
        tag=identity+1
        require(a['active']==before['active'] and a['inactive']==before['inactive'],'failure prepublication process bytes')
        tables(before['tables'],a['tables'])
        require(request['publication_indices']==[] and value['open_retirement'] is None,
                'failure allocation unexpectedly published')
    else:
        active=owner(a['active'],a['active']['token'],a['allocator'])
        candidate=owner(a['candidate'],a['candidate']['token'],a['allocator'])
        require(active['count']==67 and active['pins']==0 and candidate['count']==101 and candidate['pins']==1,
                'failure published candidate and unmapped owner')
        require(not any(bytes.fromhex(a['active']['bytes'])),'failure old backing erasure before source release')
        old=bytes.fromhex(before['active']['bytes'])
        require(bytes.fromhex(a['candidate']['bytes'])==old+bytes(34*4096),'failure copied candidate bytes')
        require(a['inactive']==before['inactive'],'failure retained parent bytes')
        start=active['record'][2];identity=active['token'][1];provenance=active['record'][5];initial=start;tag=1
        require(len(request['publication_indices'])==1 and value['open_retirement'] is not None,
                'failure publication/retirement prefix')
        partial=value['open_retirement']
        require(partial['unpin_succeeded'] and partial['erase'] is not None and partial['release'] is not None
                and not partial['release_succeeded'],'failure release must remain incomplete')
    expected=catalog[:]
    for i in range(start,start+count):expected[i*3]=tag
    expected_sha=sha_words(expected)
    require(expected_sha!=source['catalog_sha256']
            and effects['catalog_sha256']==terminal['catalog_sha256']==expected_sha,
            'failure performed source effects retained without rollback')
    header=source['header'][:]
    header[1:12]=[count,0,identity,provenance,0 if kind=='allocate-metadata' else 2,initial,initial,0,0,0,0]
    require(effects['snapshot_header']==terminal['snapshot_header']==header,'failure actual comparison header')
    header[5:12]=[4,start+count,start,count,catalog[start*3+1] if kind=='allocate-metadata' else 0,
                  catalog[(start+count-1)*3+2],start+1]
    require(effects['header']==terminal['header']==header,'failure actual source completion')
    unchanged(a,b,cell_tag=tag if kind=='release-metadata' else None)
    unchanged(b,c,poison=True)
    history=effects['history'][:];history[3:5]=[0,1]
    require(terminal['history']==history,'failure source history retention and poison')
    require(terminal['rsp']==request['sp'] and terminal['callee']==request['callee'],
            'failure caller stack/register restoration')
    require(all(v==0 for v in terminal['volatile'].values()),'failure volatile retirement')
    sizes=dict(caller=4000,private=65528,stack=4096,facts=24,output=56,details=32,recovery=48,
               allocator_active=8,boundary_active=8)
    require(set(terminal['retired'])==set(sizes),'failure complete declared retirement domains')
    for name,size in sizes.items():
        row=terminal['retired'][name]
        require(row['bytes']==size and row['sha256']==hashlib.sha256(bytes(size)).hexdigest(),
                'failure incomplete physical retirement: '+name)
    return dict(passed=True,kind=kind,source_cells=count,source_identity=identity,
        active_pid=before['ids'][0],inactive_pid=before['ids'][3],retained_extents=2+int(kind=='release-metadata'),
        caller_retired_bytes=4000,
        scope='Selected source-built invalid delta, retained actual process owners/effects and declared gateway retirement; not every fatal path, outer caller, lifetime, AP/DMA/async or public activation')
