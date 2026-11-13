"""Saved-byte reconstruction and normal caller-retirement checks.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
This model grants no native authority or continuous ownership guarantee.
"""
from copy import deepcopy
import hashlib

from process_native import require,snapshot
from policy_failure import tables


BANKS=['active','candidate','inactive','prepared']
REGISTERS=['rbx','rbp','r12','r13','r14','r15','rdx','rsi','rdi','r8','r9']
CALLEE=REGISTERS[:6]


def sha(raw):return hashlib.sha256(raw).hexdigest()


def result_carry(result,flags):
    # Existing target ABI: empty/admitted=0, range=1, absent=2,
    # permission=3, unsupported=4. CMP result,1; CMC clears CF only for zero.
    require(type(result) is int and result in [0,1,2,3,4]
            and type(flags) is int and 0<=flags<1<<64,'caller native status vocabulary')
    require(bool(flags&1)==bool(result),'caller admission/result carry')


def expand(value,blobs,used):
    result=deepcopy(value)
    for bank in BANKS:
        ref=result[bank]['bytes']
        require(isinstance(ref,dict) and set(ref)=={'sha256','bytes'},'caller heap reference shape')
        key=ref['sha256'];count=ref['bytes']
        require(isinstance(key,str) and type(count) is int and count>=0 and key in blobs,
                'caller missing/invalid heap blob')
        encoded=blobs[key]
        require(isinstance(encoded,str),'caller heap blob encoding')
        raw=bytes.fromhex(encoded)
        require(len(raw)==count and sha(raw)==key,'caller heap blob identity')
        used.add(key);result[bank]['bytes']=encoded
    snapshot(result)
    return result


def retained(first,last):
    require(set(first)==set(last),'caller MM snapshot closure')
    for key in first:
        if key not in ['flags','details','tables']:
            require(first[key]==last[key],'caller retained MM '+key)
    require(first['cs']&3==last['cs']&3==0
            and first['flags']&0x600==last['flags']&0x600==0,'caller critical mode')
    require(first['cr3']==first['context']['root'] and first['cr0']&(1<<16),
            'caller active paging mode/WP')
    tables(first['tables'],last['tables'])


def zero(at,size):return dict(address=at,bytes=size,sha256=sha(bytes(size)))


def checked(data,profile,kernel):
    from readback_publication import call
    require(data['schema']=='topal.process-mm.caller-retirement/1'
            and data['complete'] is True and data['pending'] is None,'caller completion')
    require(data['profile']==profile,'caller bound layout')
    sites={row['offset']:row for row in profile['calls']};seen=set();used=set();geometry={};statuses={}
    require(len(sites)==len(profile['calls'])>0 and bool(data['records']),'caller site/record closure')
    require(isinstance(data['blobs'],dict),'caller heap pool')
    for row in data['records']:
        site=row['site'];offset=site['offset']
        require(offset in sites and offset not in seen and site==sites[offset],'caller unknown/duplicate site')
        seen.add(offset);returned=row['return'];entered=row['gateway_entry'];done=row['gateway_return']
        require(row['rip']==site['call'] and returned['rip']==site['returned'],'caller actual native positions')
        require(set(row['arguments'])=={'r8','r9'} and all(type(n) is int and 0<=n<1<<64 for n in row['arguments'].values()),
                'caller native argument widths')
        require(set(row['registers'])==set(REGISTERS) and row['registers']==returned['registers'],
                'caller preserved registers')
        require(all(row['registers'][key]==row['arguments'][key] for key in ['r8','r9']),
                'caller argument/register binding')
        require(row['sp']==returned['sp'] and row['flags']&0x600==returned['flags']&0x600==0,
                'caller source stack/critical flags')
        require(entered['rip']==profile['gateway'] and entered['arguments']==dict(rdi=8,
            rsi=row['arguments']['r8'],rdx=row['arguments']['r9'],rcx=site['direction']),
            'caller actual gateway request')
        call(kernel,entered['return_pc'],profile['gateway'],'topal_x86_user_extent_call_v1')
        require(done['rip']==profile['gateway_return'] and entered['sp']==done['sp']
                and set(entered['callee'])==set(CALLEE) and entered['callee']==done['callee'],
                'caller gateway return/ABI')
        require(done['result']==returned['result'],'caller gateway/helper status binding')
        result_carry(returned['result'],returned['flags'])
        outer=row['sp']-entered['sp'];require(outer in [80,88],'caller outer adapter geometry')
        require(done['inner_caller']==zero(entered['sp']-4000,4000),'caller inner erasure')
        require(returned['caller']==zero(entered['sp']-4000,4000+outer),'caller full physical erasure')
        for value in [done,returned]:
            require(set(value['domains'])==set(profile['domains']),'caller private domain closure')
            for name,domain in profile['domains'].items():
                require(value['domains'][name]==zero(domain['address'],domain['bytes']),
                        'caller private erasure '+name)
        before=expand(row['before'],data['blobs'],used)
        after=expand(returned['after'],data['blobs'],used);retained(before,after)
        geometry[outer]=geometry.get(outer,0)+1
        statuses[returned['result']]=statuses.get(returned['result'],0)+1
    require(set(data['blobs'])==used,'caller unused heap blobs')
    missing=[row['offset'] for row in profile['calls'] if row['offset'] not in seen]
    return dict(passed=True,sampled_sites=len(seen),total_sites=len(sites),missing_sites=missing,
        outer_adapter_samples=dict(sorted(geometry.items())),native_results=dict(sorted(statuses.items())),
        both_alignments_observed=set(geometry)=={80,88},missing_alignments=sorted({80,88}-set(geometry)),
        heap_blobs=len(used),heap_blob_bytes=sum(len(bytes.fromhex(data['blobs'][key])) for key in used),
        scope='Actual saved normal-return samples with complete retained MM bytes and observed helper/private/caller retirement. No every argument/path, continuous lifetime, fatal/exception, global unreachability or production qualification.')
