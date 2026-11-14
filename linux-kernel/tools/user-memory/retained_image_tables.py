"""Independent scoped inactive-owner replay; no guest mutation or activation."""
from copy import deepcopy
import hashlib
from kernel_image_plan import require
from inactive_image_tables import encode,page_count
from boot_coverage import roles,check_bank,journal as coverage_journal


def owner(bank,original,context,live):
    count=original['bytes']//4096
    require(bank['base']==original['base'] and bank['bytes']==original['bytes'], 'same whole retained extent')
    require(bank['base']%4096==0 and bank['base']>=0x1000000 and bank['base']+bank['bytes']<=1<<30,'bounded retained extent')
    token,record,cells=bank['token'],bank['record'],bank['cells']
    require(len(cells)==count*3,'complete retained frame cells')
    if live:
        require(token==original['token'] and record==original['record'] and cells==original['cells'],
                'exact retained identity generation and pins')
        require(token[0]==context[1] and 0<token[1]<context[2] and 0<=token[2]<context[3], 'actual issuer and live identity')
        require(record[:2]==token[:2] and record[3:5]==[count,1] and record[5]>0 and record[6:]==[1,1],
                'whole pinned live retained allocation')
        require(0<=record[2] and record[2]+count<=context[3],'bounded complete retained registry extent')
        require(cells[0::3]==[token[1]]*count and cells[1::3]==[record[5]]*count and
                cells[2::3]==list(range(bank['base']//4096,bank['base']//4096+count)), 'complete generation-bound physical cells')
    else:
        require(token==[0,0,0] and record==[0]*8 and cells[0::3]==[0]*count,
                'complete retained owner release')
        require(cells[1::3]==original['cells'][1::3] and cells[2::3]==original['cells'][2::3],
                'issued generation and physical history retained')


def retained_journal(first,retirement,completed):
    expected=[0]*32
    expected[0]=completed
    expected[1]=first['context'][1]
    expected[2]=first['context'][2]
    expected[4]=first['controls']['cr3']
    expected[6]=first['journal'][1]
    if retirement is not None:
        expected[3]=retirement['context'][2]
        expected[5]=retirement['controls']['cr3']
    for i,bank in enumerate(first['banks']):
        expected[8+4*i:12+4*i]=[bank['base'],bank['token'][1],bank['token'][2],bank['record'][5]]
    return expected


def inspect(native,layout,contract,ordinary,read):
    require(native['passed'] and native['target_state_injected'] is False,'actual successful read-only native observation')
    phases=native['phases']
    require([p['phase'] for p in phases]==list(range(7)),'complete retained lifecycle')
    first=phases[0]
    pages=page_count(layout)
    require(first['journal'][1]==pages and len(first['banks'])==3,'exact image-derived whole shape')
    extents=roles(contract,first['boot_registers'])
    counts=None
    for phase in phases:
        index=phase['phase']
        require(phase['journal'][0]==index and phase['journal'][1:]==first['journal'][1:],'stable preparation identity journal')
        ctx=phase['context']
        require(ctx[:2]==first['context'][:2] and ctx[3]==first['context'][3] and ctx[2]>=first['context'][2],
                'same issuer capacity and monotonic issued history')
        if index<=3:
            require(ctx==first['context'] and phase['controls']==first['controls'],'initial binding comparison window')
        else:
            require(ctx==phases[4]['context'] and phase['controls']==phases[4]['controls'],'stable complete retirement window')
            require(ctx[2]>first['context'][2] and phase['controls']['cr3']!=first['controls']['cr3'],
                    'ordinary allocation and actual root restoration occurred')
        require(len(phase['banks'])==3,'every whole bank and separate low leaf observed')
        for i,bank in enumerate(phase['banks']):
            original=first['banks'][i]
            require(bank['bytes']==(pages if i<2 else 1)*4096,'exact complete extent size')
            owner(bank,original,ctx,index<6)
            require(not bank['base']<=phase['controls']['cr3']&~4095<bank['base']+bank['bytes'],'retained bank remains inactive')
            require(bank['data']==f'bank-{index}-{i}.bin','closed phase data path')
            data=read(bank['data'])
            require(len(data)==bank['bytes'] and hashlib.sha256(data).hexdigest()==bank['data_sha256'],'complete captured bytes')
            if index==0:
                expected=b''.join((6510615555426900570^bank['base']^j).to_bytes(8,'little') for j in range(bank['bytes']//8))
            elif index in [2,3,4] and i<2:
                expected=encode(layout,bank['base'],first['journal'][10] if i==1 else None)
            else: expected=bytes(bank['bytes'])
            require(data==expected,'complete expected retained bank or zero leaf')
            if index in [2,3,4] and i<2:
                actual_counts=check_bank(data,bank['base'],extents)
                if counts is None: counts=actual_counts
                require(counts==actual_counts,'identical complete retained-role coverage')
        expected_coverage=([0]*64 if index<2 else coverage_journal(extents,counts,1 if index==2 else (2 if index==3 else 3)))
        require(phase['boot_coverage']==expected_coverage,'exact repeated coverage and completion journal')
        expected_retention=([0]*32 if index<3 else retained_journal(first,None if index==3 else phases[4],
                            1 if index in [3,4] else (2 if index==5 else 3)))
        require(phase['retained_tables']==expected_retention,'exact owner generation and retirement journal')
    for i,a in enumerate(first['banks']):
        lo,hi=a['base'],a['base']+a['bytes']
        require(any(k==1 and s<=lo and hi<=e for s,e,k in first['firmware_claims']),'actual allocator RAM provenance')
        require(all(k==1 or hi<=s or lo>=e for s,e,k in first['firmware_claims']),'no conflicting firmware claim')
        require(all(hi<=s or lo>=e for s,e in first['reservations']),'all boot reservations excluded')
        for b in first['banks'][:i]:
            require(hi<=b['base'] or lo>=b['base']+b['bytes'],'disjoint complete retained extents')
            require(a['token'][1]!=b['token'][1] and a['token'][2]!=b['token'][2],'distinct actual owners')
    events=native['retention_events']
    require(len(events)==native['retention_event_count'],'complete final observed event count')
    require(4<=len(events)<=64 and all(e['syscall'] in [59,60,231] for e in events)
            and sum(e['syscall']==59 for e in events)>=3 and events[-1]['syscall'] in [60,231],
            'actual exec attempts and final exit boundaries')
    require([e['ordinal'] for e in events]==list(range(len(events))),'complete ordered observed syscall events')
    last_next=first['context'][2]
    for number,event in enumerate(events):
        ctx=event['context']
        require(ctx[:2]==first['context'][:2] and ctx[3]==first['context'][3] and ctx[2]>=last_next and ctx[2]>first['context'][2],
                'ordinary allocation history while exact owners remain retained')
        last_next=ctx[2]
        require(event['retained_tables']==phases[3]['retained_tables'],'scoped owner remains retained during userspace')
        require(event['registers']['rax']==event['syscall'] and event['registers']['cs']&3==0,
                'actual syscall entry arguments and privilege')
        require(event['controls']['cr0']==first['controls']['cr0']|(1<<16)
                and event['controls']['cr4']==first['controls']['cr4']|0x600,
                'ordinary provider preserves controls while enabling WP and its declared SIMD context')
        require(len(event['banks'])==3,'every retained bank observed at actual syscall entry')
        for i,bank in enumerate(event['banks']):
            owner(bank,first['banks'][i],ctx,True)
            require(bank['data']==f'retention-{number}-bank-{i}.bin','closed live-event data path')
            data=read(bank['data'])
            expected=encode(layout,bank['base'],first['journal'][10] if i==1 else None) if i<2 else bytes(4096)
            require(data==expected and hashlib.sha256(data).hexdigest()==bank['data_sha256'],'unchanged banks at exec/exit')
        expected_owners=[ordinary['bootstrap'],*ordinary['roles']]
        require(len(event['ordinary_owners'])==5,'all five ordinary root/image/stack owners captured')
        for actual,expected in zip(event['ordinary_owners'],expected_owners):
            require(actual['owner']==expected,'freshly reconstructed ordinary owner role')
            bank=dict(actual,bytes=expected['count']*4096)
            require(expected['count'] in [1,2,5,503] and expected['alignment']==1,'unchanged ordinary topology')
            token,record,cells=bank['token'],bank['record'],bank['cells']
            require(token[0]==ctx[1] and 0<token[1]<ctx[2] and 0<=token[2]<ctx[3],'actual ordinary issuer')
            require(record[:2]==token[:2] and record[3:5]==[expected['count'],1] and record[5]>0 and record[6:]==[expected['pins'],1],
                    'complete ordinary count generation and pins')
            require(0<=record[2] and record[2]+expected['count']<=ctx[3],'bounded complete ordinary registry extent')
            require(len(cells)==expected['count']*3 and cells[0::3]==[token[1]]*expected['count'] and
                    cells[1::3]==[record[5]]*expected['count'] and
                    cells[2::3]==list(range(bank['base']//4096,bank['base']//4096+expected['count'])),'complete ordinary physical owner')
            require(all(bank['base']+bank['bytes']<=b['base'] or bank['base']>=b['base']+b['bytes'] for b in first['banks']),
                    'ordinary execution owners exclude every retained bank')
        require(event['controls']['cr3']==event['ordinary_owners'][1]['base'],'actual process root differs from retained banks')
    final=native['final_bootstrap']
    require(final['controls']==phases[4]['controls'] and final['base']==final['controls']['cr3'],'actual restored bootstrap retained through retirement')
    require(final['token'][0]==first['context'][1] and final['record'][:2]==final['token'][:2] and
            final['record'][3:5]==[5,1] and final['record'][6:]==[1,1],'unchanged exact ordinary bootstrap owner')
    data=read('final-bootstrap-bank.bin')
    require(len(data)==5*4096 and hashlib.sha256(data).hexdigest()==final['bank_sha256'],'complete actual final ordinary bank')
    return dict(passed=True,native_execution=False,inactive_only=True,phases=7,retention_events=len(events),exec_attempts=sum(e['syscall']==59 for e in events),
                banks=2,extents=3,pages_per_bank=pages,roles=11,page_views_per_check=sum(counts)*2,
                completed_coverage_checks=3,whole_retired_bytes=(2*pages+1)*4096,
                protected_context=False,production_qualified=False,
                scope='Independent actual-byte/owner replay through ordinary exec/exit and restored-root retirement; not effective protection, continuous exclusion or activation')


def altered_record_controls(native,layout,contract,ordinary,read):
    rejected=[]
    for name in ['lost-phase','retired-early','issuer','generation','unpinned','changed-cell',
                 'next-regressed','stale-context','retained-while-released','lost-event',
                 'wrong-syscall','event-generation','event-unpinned','event-alias',
                 'event-root','ordinary-owner','ordinary-pins','live-journal','unused-journal','event-ordinal','changed-bank']:
        n=deepcopy(native)
        if name=='lost-phase': n['phases'].pop(4)
        elif name=='retired-early': n['phases'][3]['retained_tables'][0]=3
        elif name=='issuer': n['phases'][4]['banks'][0]['token'][0]+=1
        elif name=='generation': n['phases'][4]['banks'][0]['record'][5]+=1
        elif name=='unpinned': n['phases'][4]['banks'][0]['record'][6]=0
        elif name=='changed-cell': n['phases'][4]['banks'][0]['cells'][1]+=1
        elif name=='next-regressed': n['phases'][4]['context'][2]=1
        elif name=='stale-context': n['phases'][4]['controls']=n['phases'][0]['controls'].copy()
        elif name=='retained-while-released': n['phases'][6]['banks'][0]['token'][0]=1
        elif name=='lost-event': n['retention_events'].pop()
        elif name=='wrong-syscall': n['retention_events'][0]['syscall']=39
        elif name=='event-generation': n['retention_events'][0]['banks'][0]['record'][5]+=1
        elif name=='event-unpinned': n['retention_events'][0]['banks'][0]['record'][6]=0
        elif name=='event-alias':
            actual=n['retention_events'][0]['ordinary_owners'][0]
            actual['base']=n['phases'][0]['banks'][0]['base']
            actual['cells'][2::3]=list(range(actual['base']//4096,actual['base']//4096+actual['record'][3]))
        elif name=='event-root': n['retention_events'][0]['controls']['cr3']=n['phases'][0]['banks'][0]['base']
        elif name=='ordinary-owner': n['retention_events'][0]['ordinary_owners'][0]['owner']['count']=8
        elif name=='ordinary-pins': n['retention_events'][0]['ordinary_owners'][0]['record'][6]=0
        elif name=='live-journal': n['retention_events'][0]['retained_tables'][0]=2
        elif name=='unused-journal': n['phases'][4]['retained_tables'][31]=1
        elif name=='event-ordinal': n['retention_events'][0]['ordinal']=1
        selected_read=read
        if name=='changed-bank':
            bank=n['retention_events'][0]['banks'][0]
            data=bytearray(read(bank['data']));data[0]^=1
            bank['data_sha256']=hashlib.sha256(data).hexdigest()
            selected_read=lambda path:bytes(data) if path==bank['data'] else read(path)
        try: inspect(n,layout,contract,ordinary,selected_read)
        except (ValueError,AssertionError): rejected.append(name)
        else: raise AssertionError('altered retained record accepted: '+name)
    return rejected
