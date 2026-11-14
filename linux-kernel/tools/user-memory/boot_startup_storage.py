"""Independent byte replay of boot-owned setup storage, not CPU arrival proof."""
import hashlib
import struct
from kernel_image_plan import require
from boot_coverage import page as coverage_page

SEED=int.from_bytes(b'TOPAL-ST','little')
STRIDE=0x9e3779b97f4a7c15
MASK=0xa55a3cc3f00f9669


def page(probe=False):
    return b''.join((((SEED+i*STRIDE)&((1<<64)-1))^(MASK if probe else 0)).to_bytes(8,'little') for i in range(512))


def inspect(native,contract,image,read):
    require(contract.get('adapter_revision')=='topal.boot-adapter.linux-x86-protocol-2.15-q35-startup-storage/1',
            'explicit startup-storage adapter identity')
    profile=contract.get('boot_startup_storage')
    require(isinstance(profile,dict),'explicit startup-storage profile')
    expected=dict(schema='topal.boot-startup-storage.x86_64/1',setup_bytes=8192,setup_sectors=15,
        page_offset=4096,page_bytes=4096,seed=SEED,stride=STRIDE,probe_mask=MASK,
        page_sha256=hashlib.sha256(page()).hexdigest(),owner='boot-adapter-setup-instance',
        lifetime='kernel',reclamation=False,processor_startup=False,participant_arrival=False)
    require(profile==expected,'exact private startup-storage contract')
    require(all(type(profile[k]) is type(v) for k,v in expected.items()),'typed startup-storage contract')
    require(contract['boot_image_sha256']==hashlib.sha256(image).hexdigest(),'actual adapter identity')
    require(len(image)>8192 and image[497]==15 and image[4096:8192]==page(),'complete loaded startup page')
    phases=native['phases']
    require(len(phases)==7 and [p['phase'] for p in phases]==list(range(7)),'complete table lifetime')
    base=native['boot_params']
    require(type(base) is int and base>=4096 and base%4096==0 and base+8192<=0xa0000,'whole low setup extent')
    boot=phases[0]['boot_registers']
    require(boot[16]==base and base+8192<=boot[19]<=boot[17]<boot[18],'boot, image and resource exclusion')
    require([0,boot[19]] in phases[0]['reservations'],'ordinary allocator exclusion remains')
    metadata=None
    parameters=None
    checked=0

    def capture(row,label,state,probe=False):
        nonlocal metadata,parameters,checked
        require(isinstance(row,dict) and row['boot_params']==base,'same actual boot instance')
        require(set(row['regions'])=={'parameters','page'},'complete setup byte regions')
        buffers={}
        for name,offset in [('parameters',0),('page',4096)]:
            fact=row['regions'][name]
            require(fact['start']==base+offset and fact['bytes']==4096 and
                    fact['file']==f'startup-{label}-{name}.bin','actual whole owned setup view')
            raw=read(fact['file'])
            require(len(raw)==4096 and hashlib.sha256(raw).hexdigest()==fact['sha256'],'whole bytes and digest')
            buffers[name]=raw
        require(buffers['page']==page(probe),'complete expected loaded/probe/restored bytes')
        params=buffers['parameters']
        if parameters is None:parameters=params
        require(params==parameters,'complete retained boot-parameter/firmware bytes')
        require(params[497]==15 and params[514:518]==b'HdrS' and struct.unpack_from('<H',params,518)[0]==527,
                'actual loaded setup header')
        require(struct.unpack_from('<I',params,532)[0]==0x100000 and
                struct.unpack_from('<Q',params,592)[0]==0 and
                struct.unpack_from('<I',params,200)[0]==0 and
                struct.unpack_from('<H',params,566)[0]==0,'supported handoff and no extended inputs')
        cmd=struct.unpack_from('<I',params,552)[0]
        require(cmd==0 or cmd+2048<=base or cmd>=base+8192,'command-line exclusion')
        count=params[488]
        require(1<=count<=128 and 720+count*20<=4096,'bounded complete E820 table')
        claims=[]
        for i in range(count):
            start,length,kind=struct.unpack_from('<QQI',params,720+20*i)
            require(start+length<1<<64,'firmware extent overflow')
            claims.append([start,start+length,kind])
        require(claims==phases[0]['firmware_claims'],'same actual raw firmware claims')
        require(any(kind==1 and start<=base and base+8192<=end for start,end,kind in claims),'whole available RAM provenance')
        require(all(kind==1 or start==end or end<=base or start>=base+8192 for start,end,kind in claims),
                'conflicting firmware claim')
        journal=row['journal']
        require(len(journal)==32 and all(type(v) is int and 0<=v<1<<64 for v in journal),'complete native owner journal')
        if state==0:
            require(journal==[0]*32,'no premature ownership journal')
        else:
            ebda=journal[9]
            require(type(row['ebda_segment']) is int and 0<=row['ebda_segment']<=65535 and
                    ebda==row['ebda_segment']*16,'actual BDA/EBDA identity')
            require(ebda==0 or base+8192<=ebda<=0xa0000,'EBDA exclusion')
            binding=[base,base+4096,4096,8192,1,1,0,count,ebda,cmd,boot[2],boot[19],boot[17],boot[18],SEED,STRIDE,MASK,1,1]+[0]*12
            require(journal==[state]+binding,'exact retained boot-owner binding')
            if metadata is None:metadata=binding
            require(binding==metadata,'unchanged instance/generation/extent/lifetime')
        checked+=1

    for i,row in enumerate(phases):
        capture(row.get('startup_storage'),'table-'+str(i),0 if i<3 else (3 if i==3 else 4))
        for bank in row['banks']:
            require(bank['base']+bank['bytes']<=base+4096 or bank['base']>=base+8192,'startup/table owner separation')
        if i in [3,4]:
            require(len(row['banks'])==3,'both whole inactive banks and separate low leaf')
            for bank in row['banks'][:2]:
                data=read(bank['data'])
                require(len(data)==bank['bytes'] and hashlib.sha256(data).hexdigest()==bank['data_sha256'],
                        'complete actual bank bytes for startup coverage')
                coverage_page(data,bank['base'],base+4096,0)
    claims=native.get('startup_storage_events',[])
    require(len(claims)==3,'actual claim/write/restore checkpoints')
    for i,row in enumerate(claims):
        capture(row,'claim-'+str(i),i+1,probe=i==1)
        require(row['controls']==phases[0]['controls'],'no interpretation-control change during storage claim')
    events=native.get('retention_events',[])
    require(4<=len(events)<=64 and native['retention_event_count']==len(events),'actual ordinary execution checkpoints')
    for i,event in enumerate(events):
        capture(event.get('startup_storage'),'event-'+str(i),3)
        for owner in event['ordinary_owners']:
            end=owner['base']+owner['record'][3]*4096
            require(end<=base+4096 or owner['base']>=base+8192,'startup/ordinary owner separation')
    capture(native['final_bootstrap'].get('startup_storage'),'terminal',4)
    return dict(passed=True,owner='boot-adapter-setup-instance',page_base=base+4096,page_bytes=4096,
        loaded_setup_bytes=8192,whole_page_writes=2,checked_snapshots=checked,ordinary_entries=len(events),
        inactive_page_views=4,
        final_owner_retained=True,reclamation=False,processor_startup=False,participant_arrival=False,
        protected_context=False,production_qualified=False)
