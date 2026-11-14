"""Independent inactive boot-role walks, not hardware permission evidence.

TOPAL-SYSTEMS-KERNEL-IMAGE-BOOT-COVERAGE-001. Captured bytes are read only.
"""
import struct
from kernel_image_plan import require

PAGE, NX, ADDRESS = 4096, 1 << 63, 0x000ffffffffff000
FIXED = [('closed-handoff-code',0x100000,0x101000),
         ('retained-entry-tables',0x101000,0x104000),
         ('retained-gdt',0x104000,0x105000),('retained-idt',0x105000,0x106000),
         ('retained-idt-pointer',0x106000,0x107000),
         ('retained-mmio-table',0x107000,0x108000),
         ('bootstrap-stack',0x108000,0x180000)]
MMIO = [[0xfec00000,0xfee00000],[0xfee00000,0xff000000]]


def roles(contract, registers):
    require(contract['roles']==[dict(role=n,start=a,end=b) for n,a,b in FIXED],
            'complete supported adapter role identities')
    require(contract['mmio_windows']==MMIO,'supported complete device windows')
    require(contract['inactive_coverage']==dict(roles=11,journal_words=64,
            complete_page_walks=True,activation=False),'inactive coverage contract')
    require(0<registers[16]<=0xfff000 and 0<registers[17]<registers[18]<=1<<30,
            'bounded actual boot inputs')
    return [(a,b,0) for _,a,b in FIXED]+[
        (registers[16],registers[16]+PAGE,0),(registers[17],registers[18],0)
    ]+[(a,b,3) for a,b in MMIO]


def page(data, base, address, expected_cache):
    """Four-level interpreter with effective ancestor rights and owned reads."""
    require(type(base) is int and base%PAGE==0 and base>=0x1000000 and
            len(data)%PAGE==0 and base+len(data)<=1<<30,'bounded whole bank')
    table, write, user, nx = base, True, True, False
    for shift in [39,30,21,12]:
        offset=table-base
        require(0<=offset and offset%PAGE==0 and offset+PAGE<=len(data),'owned table read')
        entry=struct.unpack_from('<Q',data,offset+((address>>shift)&511)*8)[0]
        require(entry&1,'present boot coverage')
        write &= bool(entry&2)
        user &= bool(entry&4)
        nx |= bool(entry&NX)
        require(not entry&128 or shift in [21,12],'supported boot granule')
        if shift==12 or entry&128:
            require(write and not user and nx,'effective supervisor RW/NX boot coverage')
            mask=(1<<shift)-1
            physical=(entry&ADDRESS&~mask)|(address&mask)
            require(physical==address,'identity boot coverage')
            cache=((entry>>3)&3)|(((entry>>(7 if shift==12 else 12))&1)<<2)
            require(cache==expected_cache,'boot cache encoding preserved')
            return
        table=entry&ADDRESS
    raise ValueError('missing leaf')


def check_bank(data,base,extents):
    counts=[]
    for start,end,cache in extents:
        require(type(start) is int and type(end) is int and 0<=start<end<=1<<32,
                'bounded nonempty boot extent before rounding')
        lo,hi=start&~4095,(end+4095)&~4095
        counts.append((hi-lo)//PAGE)
        for address in range(lo,hi,PAGE): page(data,base,address,cache)
    return counts


def journal(extents,counts,completed):
    result=[0]*64
    result[:4]=[completed,len(extents),sum(counts),len(extents)*2]
    result[4:4+len(counts)]=counts
    for i,((start,end,cache),count) in enumerate(zip(extents,counts)):
        result[16+4*i:20+4*i]=[start,end,cache,count]
    return result


def inspect(native,contract,read):
    require([p['phase'] for p in native['phases']]==list(range(5)),'complete preparation protocol')
    extents=roles(contract,native['phases'][0]['boot_registers'])
    banks=native['phases'][2]['banks'][:2]
    counts=[check_bank(read(bank['data']),bank['base'],extents) for bank in banks]
    require(len(counts)==2 and counts[0]==counts[1],'both complete bank views')
    for p in native['phases']:
        expected=[0]*64 if p['phase']<2 else journal(extents,counts[0],1 if p['phase']==2 else 2)
        require(p['boot_coverage']==expected,'exact native coverage journal and completion order')
    return dict(passed=True,native_execution=False,inactive_only=True,roles=11,
                views=22,pages_per_bank=sum(counts[0]),pages_checked=sum(counts[0])*2,
                native_completed_checks=2,journal_words=64,protected_context=False,
                production_qualified=False,
                scope='Independent walks of actual inactive table bytes; counters do not establish effective hardware permissions, activation, CPU participation or lifetime')


def altered_record_controls(native,contract,read):
    from copy import deepcopy
    rejected=[]
    for name in ['missing-role','changed-role','changed-device','missing-completion',
                 'premature-completion','changed-count','changed-extent','unused-journal']:
        changed,selected=deepcopy(native),deepcopy(contract)
        if name=='missing-role': selected['roles'].pop()
        elif name=='changed-role': selected['roles'][0]['end']+=4096
        elif name=='changed-device': selected['mmio_windows'][0][1]-=4096
        else:
            phase,index,value={'missing-completion':(3,0,1),'premature-completion':(1,0,1),
                               'changed-count':(2,4,99),'changed-extent':(2,16,0),
                               'unused-journal':(4,63,1)}[name]
            changed['phases'][phase]['boot_coverage'][index]=value
        try: inspect(changed,selected,read)
        except (ValueError,AssertionError): rejected.append(name)
        else: raise AssertionError('altered boot coverage accepted: '+name)
    # Exercise this interpreter directly, independently of the exact-template
    # replay which also rejects these altered bytes. Guest files stay intact.
    for bank_index in [0,1]:
        bank=native['phases'][2]['banks'][bank_index]
        original=read(bank['data'])
        for name,offset,mask in [('absent',1024*8,1),('read-only',0,2),
                                 ('executable',1024*8,NX),('alias',1024*8,1<<21),
                                 ('cache',1024*8,8),('external-pointer',0,1<<30),
                                 ('unsupported-granule',512*8,128)]:
            data=bytearray(original)
            word=struct.unpack_from('<Q',data,offset)[0]
            struct.pack_into('<Q',data,offset,word^mask)
            altered=lambda name:bytes(data) if name==bank['data'] else read(name)
            try: inspect(native,contract,altered)
            except (ValueError,AssertionError): rejected.append(f'{bank_index}-{name}')
            else: raise AssertionError('altered boot table accepted: '+name)
    return rejected
