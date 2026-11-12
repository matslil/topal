"""Read-only actual owner evidence for ordinary metadata syscall admissions."""
from pathlib import Path

MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/owner_snapshot.py').read_text()
PROGRAM='owner_snapshot_model = {}\nexec('+repr(MODEL)+', owner_snapshot_model)\n'+r'''
owner_pending=None
owner_snapshots={}
owner_records=[]
owner_addresses={}

def owner_read(symbol,count):
    return words(owner_addresses[symbol],count)

def owner_capture():
    allocator=owner_read('context',16)
    assert allocator[:2]==[1,1] and 0<allocator[3]<=262144
    tokens=owner_read('root_tokens',owner_profile['bindings']['token_bytes']//8)
    def allocation(token,count):
        assert len(token)==3 and token[0]==1 and 0<token[1]<allocator[2] and token[2]<allocator[3]
        record=words(owner_addresses['registry']+token[2]*64,8)
        assert 0<count<=519 and 0<=record[2]<record[2]+count<=allocator[3]
        cells=words(owner_addresses['table']+128+record[2]*24,count*3)
        return dict(record=record,cells=cells)
    roles=[allocation(tokens[r['slot']*3:r['slot']*3+3],r['count']) for r in owner_profile['bindings']['roles']]
    root=roles[0]['cells'][2]*4096
    low=roles[1]['cells'][2]*4096
    mm=state()
    heap_tokens=owner_read('mm_tokens',6)
    return dict(allocator=allocator,root_tokens=tokens,roles=roles,state=mm,
        heap_tokens=heap_tokens,heap=allocation(heap_tokens[:3],mm[2]),
        context=dict(address=context,state=word(context),self=word(context+256),root=word(context+24),
            low=word(context+112),stack_page=word(context+912),phase=memory(context+104,1)[0]),
        tables={str(at):words(at,512) for at in [root,root+4096,root+8192,root+12288,root+16384,low]},
        epoch=owner_read('mm_epoch',2),cr0=reg('cr0'),cr3=reg('cr3'),cr4=reg('cr4'),
        gs_base=reg('gs_base'),cs=reg('cs'),flags=reg('eflags'))

def owner_check(snapshot,start,length,direction):
    return owner_snapshot_model['check'](snapshot,owner_profile['bindings'],
        owner_profile['kernel_spans'],start,length,direction)

def owner_remember(snapshot):
    raw=json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode()
    key=hashlib.sha256(raw).hexdigest()
    assert key not in owner_snapshots or owner_snapshots[key]==snapshot
    owner_snapshots[key]=snapshot
    return key

class OwnerBoundary(Checked):
    def __init__(self,direction,returning):
        name='topal_x86_user_extent_'+('writable' if direction else 'readable')+('_return_v1' if returning else '_v1')
        super().__init__('*'+name,internal=True)
        self.direction,self.returning=direction,returning

    def observe(self):
        global owner_pending
        if current is None or pending is None or pending['number'] not in [5,18]:return False
        assert (pending['number']==5)==bool(self.direction)
        if not self.returning:
            assert owner_pending is None
            start,length=reg('r8'),reg('r9')
            assert start==pending['args'][1] and length==(144 if pending['number']==5 else 4)
            captured=owner_capture()
            evidence=owner_check(captured,start,length,self.direction)
            owner_pending=dict(start=start,length=length,direction=self.direction,sp=reg('rsp'),
                site=word(reg('rsp')),snapshot=captured,evidence=evidence,
                number=pending['number'],args=list(pending['args']))
        else:
            assert owner_pending is not None and owner_pending['direction']==self.direction
            before=owner_pending
            assert reg('rsp')==before['sp'] and word(reg('rsp'))==before['site']
            assert reg('rax')==0 and reg('eflags')&1==0
            captured=owner_capture()
            evidence=owner_check(captured,before['start'],before['length'],self.direction)
            # Flags are transient arithmetic results. Every retained owner,
            # translation word, entry identity and mode must agree exactly.
            assert {k:v for k,v in captured.items() if k!='flags'}=={k:v for k,v in before['snapshot'].items() if k!='flags'}
            assert evidence==before['evidence']
            owner_records.append(dict(number=before['number'],args=before['args'],
                start=before['start'],length=before['length'],direction=self.direction,
                before=owner_remember(before['snapshot']),after=owner_remember(captured),evidence=evidence))
            owner_pending=None
        return False

def install_owner_observation():
    if owner_profile is None:return
    assert component['mode']=='metadata'
    for suffix in ['context','root_tokens','registry','table','mm_tokens','mm_epoch']:
        owner_addresses[suffix]=address('topal_x86_frame_'+suffix+'_v1')
    for direction in [0,1]:
        for returning in [False,True]:OwnerBoundary(direction,returning)

def owner_observation_complete():
    if owner_profile is None:return
    assert owner_pending is None and len(owner_records)==52
    assert sum(row['number']==5 for row in owner_records)==44
    assert sum(row['number']==18 for row in owner_records)==8
    observed['owner_boundary_observation']=dict(passed=True,records=owner_records,snapshots=owner_snapshots,
        profile=owner_profile,model_sha256=hashlib.sha256(owner_profile['model'].encode()).hexdigest(),
        scope='Selected stat and positional-write helper entry/return owner observations, not continuous lifetime or whole-caller qualification')
'''
