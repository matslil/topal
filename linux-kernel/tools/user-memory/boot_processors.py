"""Independent ACPI processor discovery for TOPAL-SYSTEMS-BOOT-PROCESSOR-TOPOLOGY-001.

Firmware inventory and boundary snapshots grant no participation/lifetime authority.
"""
import hashlib
import struct
from kernel_image_plan import require

LIMIT=1<<30
RECORD_LENGTHS={0:8,1:12,2:10,3:8,4:6,5:12,9:16,10:12}


def retained_range(claims,base,length):
    require(0<len(claims)<=128 and all(0<=start<=last<1<<64 and 0<=kind<1<<32
                                      for start,last,kind in claims),'bounded complete firmware claims')
    end=base+length
    require(0<base<end<=LIMIT,'bounded firmware range')
    require(any(kind in [2,3,4] and start<=base and end<=last
                for start,last,kind in claims),'complete retained firmware provenance')
    require(all(kind in [2,3,4] or not (start<end and base<last)
                for start,last,kind in claims if start<last),'firmware overlaps allocator/unsupported memory')


def processors(madt,bsp):
    require(44<=len(madt)<=4096 and madt[:4]==b'APIC','complete bounded MADT')
    require(struct.unpack_from('<I',madt,4)[0]==len(madt) and sum(madt)%256==0,'MADT length/checksum')
    require(1<=madt[8]<=7,'supported MADT revision')
    local,flags=struct.unpack_from('<II',madt,36)
    require(local==0xfee00000 and flags<=1,'supported MADT controller/header flags')
    result=[]
    at=44
    while at<len(madt):
        require(at+2<=len(madt),'complete interrupt-record header')
        kind,length=madt[at:at+2]
        require(kind in RECORD_LENGTHS and length==RECORD_LENGTHS[kind]
                and at+length<=len(madt),'supported complete interrupt-record format')
        row=madt[at:at+length]
        if kind in [0,9]:
            flags=struct.unpack_from('<I',row,4 if kind==0 else 8)[0]
            require(flags<=2,'reserved processor flags or enabled/online-capable conflict')
            if flags:
                if kind==0: uid,identity=row[2:4]
                else:
                    require(row[2:4]==bytes(2),'x2APIC reserved field')
                    identity=struct.unpack_from('<I',row,4)[0]
                    uid=struct.unpack_from('<I',row,12)[0]
                require(len(result)<16,'processor inventory bound')
                require(all(uid!=p[1] and identity!=p[2] for p in result),'duplicate processor UID/controller identity')
                result.append([kind,uid,identity,flags])
        elif kind==5:
            require(row[2:4]==bytes(2) and struct.unpack_from('<Q',row,4)[0]==local,
                    'unsupported local-controller override')
        at+=length
    enabled=sum(p[3]==1 for p in result)
    require(result and enabled and sum(p[2]==bsp and p[3]==1 for p in result)==1,
            'bootstrap processor must identify exactly one enabled participant')
    return result,enabled


def discover(read,claims,bsp):
    def raw(base,length):
        require(0<=base<base+length<=LIMIT,'bounded physical read')
        data=read(base,length)
        require(len(data)==length,'complete physical read')
        return data

    def firmware(base,length):
        retained_range(claims,base,length)
        return raw(base,length)

    def scan(base,length):
        data=raw(base,length)
        for at in range(0,len(data)-19,16):
            if data[at:at+8]==b'RSD PTR ' and sum(data[at:at+20])%256==0:
                return base+at
        return 0

    def table(base):
        header=firmware(base,36)
        length=struct.unpack_from('<I',header,4)[0]
        require(36<=length<=4096,'bounded complete relevant SDT')
        data=firmware(base,length)
        require(struct.unpack_from('<I',data,4)[0]==length and sum(data)%256==0,
                'complete relevant retained SDT length/checksum')
        return data

    bda=raw(1038,2)
    ebda=int.from_bytes(bda,'little')<<4
    rsdp=0
    if ebda:
        require(0x80000<=ebda<=0x9fc00,'bounded legacy EBDA')
        retained_range(claims,ebda,1024)
        rsdp=scan(ebda,1024)
    if not rsdp: rsdp=scan(0xe0000,0x20000)
    require(rsdp!=0,'valid RSDP on supported legacy discovery path')
    limit=0x100000 if rsdp>=0xe0000 else ebda+1024
    first=raw(rsdp,20)
    revision=first[15]
    require(revision in [0,2],'supported RSDP revision')
    size,width=(20,4) if revision==0 else (36,8)
    require(rsdp+size<=limit,'RSDP fits selected search area')
    rsdp_bytes=raw(rsdp,size)
    require(rsdp_bytes[:8]==b'RSD PTR ' and rsdp_bytes[15]==revision
            and sum(rsdp_bytes[:20])%256==0,'retained RSDP signature/revision/checksum')
    if revision==2:
        require(struct.unpack_from('<I',rsdp_bytes,20)[0]==36
                and rsdp_bytes[33:36]==bytes(3) and sum(rsdp_bytes)%256==0,
                'complete extended RSDP length/reserved/checksum')
    root_base=struct.unpack_from('<I' if revision==0 else '<Q',rsdp_bytes,16 if revision==0 else 24)[0]
    root=table(root_base)
    require(root[:4]==(b'RSDT' if revision==0 else b'XSDT') and root[8]==1,'selected root signature/revision')
    payload=len(root)-36
    require(payload%width==0 and 0<payload//width<=64,'complete bounded root entries')
    count=payload//width
    retained=bytearray(bda+rsdp_bytes+root)
    regions=dict(bda=(1038,2),bios=(0xe0000,0x20000),rsdp=(rsdp,size),
                 root=(root_base,len(root)))
    if ebda: regions['ebda']=(ebda,1024)
    madt_base,madt=0,None
    for i in range(count):
        pointer=int.from_bytes(root[36+i*width:36+(i+1)*width],'little')
        header=firmware(pointer,36)
        regions['header-'+str(i)]=(pointer,36)
        retained+=header
        child_length=struct.unpack_from('<I',header,4)[0]
        require(child_length>=36 and pointer+child_length<=LIMIT,'referenced SDT header length/bounds')
        if header[:4]==b'APIC':
            require(madt is None,'one unambiguous MADT')
            madt_base,madt=pointer,table(pointer)
            require(madt[:36]==header,'retained MADT header copies agree')
            retained+=madt
    require(madt is not None,'MADT present in selected root')
    rows,enabled=processors(madt,bsp)
    regions['madt']=(madt_base,len(madt))
    require(len(retained)<=8192,'retained firmware byte bound')
    registers=[0]*128
    registers[:16]=[1,rsdp,revision,size,root_base,len(root),width,count,madt_base,len(madt),
                    0xfee00000,bsp,len(rows),enabled,len(retained),ebda]
    for i,row in enumerate(rows): registers[32+i*4:36+i*4]=row
    return dict(registers=registers,retained=bytes(retained),regions=regions,participants=rows)


def inspect(native,read):
    first=native['phases'][0]['processor_registers']
    require(len(first)==128 and all(type(v) is int and 0<=v<1<<64 for v in first),'complete bounded processor journal')
    require(all(p['processor_registers']==first for p in native['phases']),'retained processor journal equality')
    reads=native['boot_processor_reads']
    require(len(reads)==4 and reads[:2]==reads[2:],'independently observed identity reads at both boundaries')
    mmio,cpuid=reads[:2]
    require(mmio['kind']==1 and mmio['selector']==0xfee00020 and 0<=mmio['value']<1<<32
            and cpuid['kind']==2 and cpuid['selector']==1 and 0<=cpuid['value']<1<<32,
            'actual controller/CPUID identity read sites')
    bsp=mmio['value']>>24
    require(bsp==cpuid['value']>>24==first[11],'native controller/initial/bootstrap identities agree')
    captures=[]
    for phase in [native['phases'][0],native['phases'][-1]]:
        buffers={}
        ranges=[]
        for name,fact in phase['processor_snapshot'].items():
            require(fact['file']==f"firmware-{phase['phase']}-{name}.bin",'closed firmware filename')
            data=read(fact['file'])
            require(len(data)==fact['bytes'] and hashlib.sha256(data).hexdigest()==fact['sha256'],
                    'complete firmware capture/digest')
            buffers[name]=data
            ranges.append((fact['address'],data))

        def physical(base,length):
            matches=[data[base-start:base-start+length] for start,data in ranges
                     if start<=base and base+length<=start+len(data)]
            require(matches and all(data==matches[0] for data in matches),'complete consistent physical capture')
            return matches[0]

        require((phase['boot_registers'][6]>>24)&255==0,'UC local-controller view')
        result=discover(physical,native['phases'][0]['firmware_claims'],bsp)
        require(result['registers']==first,'native inventory matches independent complete discovery')
        expected=result['regions']
        require(set(buffers)==set(expected)|{'retained'},'complete closed discovery regions')
        for name,(address,length) in expected.items():
            fact=phase['processor_snapshot'][name]
            require(fact['address']==address and fact['bytes']==length,'actual discovery-region identity')
        retained=buffers['retained']
        require(len(retained)==8192 and retained[:len(result['retained'])]==result['retained']
                and not any(retained[len(result['retained']):]),'complete retained firmware copy and unused bytes')
        captures.append(retained)
        require(len(phase['boot_snapshot']['cpus'])==first[13],'enabled firmware inventory matches VM CPU count')
    require(captures[0]==captures[1],'every retained firmware byte unchanged at discard')
    return dict(passed=True,native_execution=False,inventory_only=True,participants=result['participants'],
                enabled=first[13],online_capable=first[12]-first[13],root_entries=first[7],
                madt_bytes=first[9],retained_bytes=first[14],boot_snapshots=2,independent_identity_reads=4,
                protected_context=False,production_qualified=False,
                scope='Actual bounded initial firmware processor inventory; no startup, parking, hotplug or lifetime authority')


def altered_record_controls(native,read):
    """Mutate replay inputs only; preserve original observations and guest files."""
    from copy import deepcopy
    rejected=[]
    first=native['phases'][0]
    bases=['inventory-count','bsp-identity','enabled-count','root-identity','retained-length',
           'reserved-journal','processor-identity','missing-madt','firmware-ram-conflict',
           'cache-encoding','vm-cpu-count','retained-copy','missing-identity-read',
           'changed-controller-read','changed-cpuid-read','reordered-identity-reads']
    madt_cases=['duplicate-uid','duplicate-controller','processor-flags','unsupported-record',
                'record-length','balanced-discard-change']
    for case in bases+madt_cases+['unsupported-rsdp-revision','bad-rsdp-checksum']:
        changed=deepcopy(native)
        overrides={}

        def replace(fact,data):
            overrides[fact['file']]=bytes(data)
            fact['sha256']=hashlib.sha256(data).hexdigest()

        def patch_physical(address,data,last_only=False):
            for phase in [changed['phases'][0],changed['phases'][-1]]:
                if last_only and phase['phase']==0: continue
                for fact in phase['processor_snapshot'].values():
                    start,length=fact['address'],fact['bytes']
                    lo,hi=max(start,address),min(start+length,address+len(data))
                    if lo<hi:
                        prior=bytearray(overrides.get(fact['file'],read(fact['file'])))
                        prior[lo-start:hi-start]=data[lo-address:hi-address]
                        replace(fact,prior)

        slots={'inventory-count':12,'bsp-identity':11,'enabled-count':13,'root-identity':4,
               'retained-length':14,'reserved-journal':16,'processor-identity':38}
        if case in slots:
            index=slots[case]
            for phase in changed['phases']: phase['processor_registers'][index]^=1
        elif case=='missing-madt': del changed['phases'][0]['processor_snapshot']['madt']
        elif case=='firmware-ram-conflict':
            base=first['processor_registers'][8]
            changed['phases'][0]['firmware_claims'].append([base,base+4,1])
        elif case=='cache-encoding': changed['phases'][0]['boot_registers'][6]^=1<<24
        elif case=='vm-cpu-count': changed['phases'][0]['boot_snapshot']['cpus'].pop()
        elif case=='missing-identity-read': changed['boot_processor_reads'].pop()
        elif case in ['changed-controller-read','changed-cpuid-read']:
            index=0 if case=='changed-controller-read' else 1
            for at in [index,index+2]: changed['boot_processor_reads'][at]['value']^=1<<24
        elif case=='reordered-identity-reads':
            changed['boot_processor_reads']=changed['boot_processor_reads'][1::-1]*2
        elif case=='retained-copy':
            fact=changed['phases'][0]['processor_snapshot']['retained']
            data=bytearray(read(fact['file'])); data[0]^=1
            replace(fact,data)
        elif case in madt_cases:
            fact=first['processor_snapshot']['madt']
            data=bytearray(read(fact['file']))
            cpus=[]
            at=44
            while at<len(data):
                kind,length=data[at:at+2]
                if kind in [0,9]: cpus.append((at,kind))
                at+=length
            require(len(cpus)>=2,'two actual VM processor records for offline controls')
            a,ka=cpus[0]; b,kb=cpus[1]
            if case=='duplicate-uid':
                uid=(data[a+2] if ka==0 else struct.unpack_from('<I',data,a+12)[0])
                if kb==0: data[b+2]=uid
                else: struct.pack_into('<I',data,b+12,uid)
            elif case=='duplicate-controller':
                identity=data[a+3] if ka==0 else struct.unpack_from('<I',data,a+4)[0]
                if kb==0: data[b+3]=identity
                else: struct.pack_into('<I',data,b+4,identity)
            elif case=='processor-flags': struct.pack_into('<I',data,b+(4 if kb==0 else 8),3)
            elif case=='unsupported-record': data[b]=7
            elif case=='record-length': data[b+1]=0
            elif case=='balanced-discard-change': data[10]^=1
            data[9]=0; data[9]=(-sum(data))&255
            patch_physical(fact['address'],data,last_only=case=='balanced-discard-change')
        else:
            fact=first['processor_snapshot']['rsdp']
            data=bytearray(read(fact['file']))
            if case=='unsupported-rsdp-revision':
                data[15]=1; data[8]=0; data[8]=(-sum(data[:20]))&255
            else: data[8]^=1
            patch_physical(fact['address'],data)
        altered_read=lambda name: overrides[name] if name in overrides else read(name)
        try: inspect(changed,altered_read)
        except (AssertionError,ValueError): rejected.append(case)
        else: raise AssertionError('altered processor discovery accepted: '+case)
    return rejected
