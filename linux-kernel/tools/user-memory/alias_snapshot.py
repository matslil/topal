"""Enumerate the selected recorded LA48 descriptor graph and inverse mappings.

TOPAL-SYSTEMS-USER-EXTENT-001. This is saved-data analysis. It neither reads a
live TLB nor constructs kernel authority or excludes DMA/concurrent changes.
"""
from owner_snapshot import check

PAGE=4096
ADDRESS=((1<<52)-1)&-PAGE
NX=1<<63


def enumerate_leaves(snapshot,*,reads=None):
    """Walk every present descriptor, including supervisor and upper-half paths.

    The six table roles and 4-KiB/2-MiB formats are the current boot profile.
    The finite traversal budget bounds this offline reader, not kernel mappings.
    """
    root=snapshot['context']['root']
    roles={root:4,root+PAGE:3,root+2*PAGE:2,root+3*PAGE:2,
        root+4*PAGE:1,snapshot['context']['low']:1}
    tables=snapshot['tables']
    assert len(roles)==6 and set(tables)=={str(p) for p in roles}
    for table,words in tables.items():
        assert len(words)==512 and all(type(w) is int and 0<=w<1<<64 for w in words)
        assert 0<=int(table)<= (1<<30)-PAGE and int(table)%PAGE==0
    leaves=[]
    entries=[]
    visits=[]
    def walk(table,level,prefix,user,writable,nx):
        # Check the complete known table role before fetching even one word.
        assert roles.get(table)==level,'foreign or wrong-level table'
        assert len(visits)<4096,'offline descriptor traversal budget exceeded'
        visits.append(dict(table=table,level=level,prefix=prefix))
        if reads is not None:reads.append((table,level))
        shift=12+9*(level-1)
        for index,word in enumerate(tables[str(table)]):
            if not word&1:continue
            virtual=prefix|(index<<shift)
            if level==4 and index>=256:virtual|=((1<<64)-(1<<48))
            effective_user=user and bool(word&4)
            effective_write=writable and bool(word&2)
            effective_nx=nx or bool(word&NX)
            entries.append(dict(table=table,level=level,index=index,word=word,virtual=virtual))
            assert len(entries)<=4096,'offline descriptor traversal budget exceeded'
            if level==1:
                assert word&~(ADDRESS|NX|0x1ff)==0,'unsupported small leaf format'
                size=PAGE
                physical=word&ADDRESS
            elif level==2 and word&128:
                mask=ADDRESS&- (1<<21)
                assert word&~(mask|NX|0x11ff)==0,'unsupported large leaf format'
                size=1<<21
                physical=word&mask
            else:
                assert not word&128,'unqualified page-size format'
                assert word&~(ADDRESS|NX|0x3f)==0,'unsupported nonleaf format'
                walk(word&ADDRESS,level-1,virtual,effective_user,effective_write,effective_nx)
                continue
            assert virtual+size<=1<<64 and physical+size<=1<<52
            leaves.append(dict(virtual=virtual,physical=physical,size=size,user=effective_user,
                writable=effective_write,executable=not effective_nx,table=table,index=index))
    walk(root,4,0,True,True,False)
    assert {v['table'] for v in visits}==set(roles),'recorded table role is not reachable'
    return dict(leaves=leaves,entries=entries,visits=visits)


def audit(snapshot,bindings,kernel_spans,start,length,write,protected):
    owners=check(snapshot,bindings,kernel_spans,start,length,write)['owners']
    inventory=enumerate_leaves(snapshot)
    leaves=inventory['leaves']
    expected={}
    write_start=0x5ed000 if snapshot['context']['phase']>=9 else 0x5e5000
    for virtual,owner,count in [(0x400000,owners[2],503),(0x7fe000,owners[3],2),
            (0x5f7000,owners[4],snapshot['state'][2])]:
        for i in range(count):
            expected[virtual+i*PAGE]=(owner['span'][0]+i*PAGE,
                virtual!=0x400000 or virtual+i*PAGE>=write_start)
    user={}
    for leaf in leaves:
        if not leaf['user']:continue
        virtual=leaf['virtual']
        assert leaf['size']==PAGE,'unqualified large user mapping'
        assert virtual in expected and virtual not in user,'mapping outside the selected live user owners'
        physical,writable=expected[virtual]
        assert leaf['physical']==physical,'user mapping lacks its actual backing owner'
        assert not leaf['writable'] or writable,'user mapping exceeds owner write permission'
        user[virtual]=physical
    assert set(user)==set(expected),'missing live user mapping'
    assert len(set(user.values()))==len(user),'aliased selected user backing'
    routes=[]
    for region in protected:
        low,high=region['start'],region['start']+region['size']
        assert 0<low<high<=1<<30
        covered=[]
        for leaf in leaves:
            a,b=max(low,leaf['physical']),min(high,leaf['physical']+leaf['size'])
            if a>=b:continue
            virtual=leaf['virtual']+a-leaf['physical']
            assert virtual==a,'nonidentity route to protected physical storage'
            assert not leaf['user'] and leaf['writable'],'protected storage lacks writable supervisor identity mapping'
            covered.append((a,b))
        covered.sort()
        cursor=low
        for a,b in covered:
            assert a==cursor,'missing or duplicate protected physical route'
            cursor=b
        assert cursor==high,'unmapped protected physical range'
        routes.append(dict(name=region['name'],start=low,size=region['size'],identity_segments=covered))
    assert len(routes)==26
    return dict(passed=True,inventory=inventory,protected_routes=routes,
        user_pages=len(user),supervisor_leaves=sum(not leaf['user'] for leaf in leaves),
        scope='Complete present descriptor graph of this six-table LA48 snapshot, exact selected live user mappings and unique supervisor identity routes to protected physical ranges. No TLB, DMA, continuous mode/asynchronous or general lifetime proof')
