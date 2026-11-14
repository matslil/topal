"""Independent replay of retained boot bytes and actual entry observations.

TOPAL-SYSTEMS-KERNEL-IMAGE-BOOT-BINDING-001. Snapshots grant no activation,
all-CPU/writer lifetime or production publication authority.
"""
import hashlib
import re
import struct
from kernel_image_plan import require

ROOTS = [0x101000,0x102000,0x103000,0x107000]
GDT = [0,0x00cf9a000000ffff,0x00af9a000000ffff,0x00cf92000000ffff,
       0x00cff2000000ffff,0x00affa000000ffff]
HANDLERS = ['_topal_kernel_entry','topal_x86_systems_debug_break_entry',
            'topal_x86_systems_local_notification_entry','topal_x86_systems_deadline_entry',
            'topal_x86_systems_user_copy_page_fault']


def symbols(kernel,extra=()):
    require(kernel[:6]==b'\x7fELF\x02\x01','ELF64 little-endian kernel required')
    shoff=struct.unpack_from('<Q',kernel,40)[0]
    size,count=struct.unpack_from('<HH',kernel,58)
    require(size==64 and shoff+count*size<=len(kernel),'bounded section headers')
    sections=[struct.unpack_from('<IIQQQQIIQQ',kernel,shoff+i*size) for i in range(count)]
    result={}
    for section in sections:
        if section[1]!=2: continue
        strings=sections[section[6]]
        names=kernel[strings[4]:strings[4]+strings[5]]
        require(section[9]==24,'ELF64 symbol size')
        for at in range(section[4],section[4]+section[5],24):
            name,info,other,index,value,length=struct.unpack_from('<IBBHQQ',kernel,at)
            if not index: continue
            end=names.find(b'\0',name)
            require(0<=name<=end<len(names),'bounded symbol name')
            label=names[name:end].decode()
            if label not in set(HANDLERS)|{'topal_x86_kernel_reservation_end_v1'}|set(extra): continue
            require(label not in result,'duplicate global symbol')
            result[label]=dict(address=value,size=length,type=info&15)
    return result


def entry_words():
    words=[0]*2048
    words[0]=ROOTS[1]|3
    words[512]=ROOTS[2]|3
    words[515]=ROOTS[3]|3
    for i in range(512): words[1024+i]=(i<<21)|0x83
    words[1536+502],words[1536+503]=0xfec0009b,0xfee0009b
    return words


def check_pages(data):
    require(len(data)==16384,'complete four entry tables')
    actual=struct.unpack('<2048Q',data)
    for i,(a,b) in enumerate(zip(actual,entry_words())):
        mask=(96 if i>=1024 else 32) if b else 0
        require(a&~mask==b,'entry topology, attributes or unused word changed')


def check_gdt(data):
    require(len(data)==48,'complete initial GDT')
    require([v&~(1<<40) if i else v for i,v in enumerate(struct.unpack('<6Q',data))]==GDT,'GDT descriptor mismatch')


def check_idt(data,handlers):
    require(len(data)==4096,'complete initial IDT')
    expected=[0]*512
    for vector,symbol in [(3,HANDLERS[1]),(14,HANDLERS[4]),(241,HANDLERS[2]),(242,HANDLERS[3])]:
        target=handlers[symbol]
        expected[2*vector]=(target&65535)|(16<<16)|((0x8f if vector==3 else 0x8e)<<40)|((target&0xffff0000)<<32)
        expected[2*vector+1]=target>>32
    require(list(struct.unpack('<512Q',data))==expected,'IDT gate, handler or unused word mismatch')


def check_idt_pointer(data):
    require(data==struct.pack('<HQ',4095,0x105000),'retained IDT pointer mismatch')


def check_registers(r,context,boot_params,image_end):
    require(len(r)==128 and all(type(x) is int and 0<=x<1<<64 for x in r),'bounded complete register record')
    require(r[0]==1 and r[1]&0xe0000001==0x80000001 and r[2]==ROOTS[0] and r[3]==32,'supported entry paging state')
    require(r[4]&0x500==0x500 and r[4]&~0xd01==0 and r[5]==0xfee00900,'long mode and BSP xAPIC')
    require(r[7]&255==r[28]<=16 and r[27]==r[7]&256,'advertised MTRR bounds')
    require(r[9:13]==[0x104000,47,0x105000,4095],'actual descriptor registers')
    require(0x109040<=r[13]<0x180000 and r[14]==16 and r[15]&0x700==0,'privileged bounded critical stack')
    require(r[16]==boot_params and 0<r[16]<=0xfff000 and r[19]==image_end,'actual handoff/image identity')
    require(image_end<=r[17]<r[18]<=1<<30 and r[23]==context[3],'actual boot resource and normalized capacity')
    require(r[20]&70248==70248 and r[22]&537919488==537919488,'optional reads require advertised capabilities')
    require(r[24:26]==[2048,512] and 0<r[26]<=4096,'complete native check counts')
    used=set(range(29))|set(range(64,64+2*r[28]))
    if r[27]: used.update(range(40,51))
    require(all(value==0 for i,value in enumerate(r) if i not in used),'unadvertised/reserved observations')


def cache_index(word,large):
    return ((word>>3)&1)|(((word>>4)&1)<<1)|(((word>>(12 if large else 7))&1)<<2)


def check_cache_indices(table_bytes,base,process=False):
    words=struct.unpack('<'+'Q'*(len(table_bytes)//8),table_bytes)
    for i in range(512):
        if process and i in [2,3]: continue  # distinct, presently empty USER topology
        entry=words[1024+i]
        if entry&128:
            require(cache_index(entry,True)==0,'changed direct large-leaf cache index')
        else:
            pointer=entry&0x000ffffffffff000
            offset=pointer-base
            require(0<=offset and offset+4096<=len(table_bytes) and offset%4096==0,'owned kernel subdivision')
            require(all(cache_index(w,False)==0 for w in words[offset//8:offset//8+512]),'changed subdivided cache index')
    require(all(cache_index(words[1536+i],True)==3 for i in [502,503]),'changed MMIO cache index')


def inspect(native,contract,kernel,boot_image,read):
    digest=lambda data:hashlib.sha256(data).hexdigest()
    require(contract['kernel_sha256']==digest(kernel) and contract['boot_image_sha256']==digest(boot_image),'actual kernel/adapter hashes')
    require(contract['entry_root']==ROOTS[0] and contract['entry_table_pages']==ROOTS,'supported adapter geometry')
    actual_symbols=symbols(kernel)
    handlers={h['symbol']:h['address'] for h in contract['handlers']}
    require(set(handlers)==set(HANDLERS),'complete handler identities')
    require(all(actual_symbols[n]['address']==a and actual_symbols[n]['type']==2 for n,a in handlers.items()),'handlers must be actual kernel functions')
    phoff=struct.unpack_from('<Q',kernel,32)[0]
    phsize,phcount=struct.unpack_from('<HH',kernel,54)
    require(phsize==56 and phoff+phsize*phcount<=len(kernel),'bounded ELF64 loads')
    loads=[struct.unpack_from('<II6Q',kernel,phoff+i*phsize) for i in range(phcount)]
    require(all(any(p[0]==1 and p[1]&1 and p[3]==p[4] and p[3]<=a<p[3]+p[6] for p in loads)
                for a in handlers.values()),'handler must remain in actual executable identity load')
    image_end=actual_symbols['topal_x86_kernel_reservation_end_v1']['address']
    first=native['phases'][0]
    registers=first['boot_registers']
    check_registers(registers,first['context'],native['boot_params'],image_end)
    require(first['reservations']==[[0,image_end],[registers[16],registers[16]+4096],
                                  [registers[17],registers[18]]],'independently read boot reservations')
    expected=[dict(number=n,value=registers[i]) for n,i in [(0xc0000080,4),(27,5),(631,6),(254,7),(767,8)]]
    expected += [dict(number=512+i,value=registers[64+i]) for i in range(registers[28]*2)]
    if registers[27]:
        expected += [dict(number=n,value=registers[40+i]) for i,n in enumerate([592,600,601,*range(616,624)])]
    require(native['boot_msr_reads']==expected*2,'every advertised MSR read observed at both boundaries')
    for phase in native['phases']:
        require(phase['boot_registers']==registers,'retained boot/mode/attribute state changed')
        require(registers[1:4]==[phase['controls'][n] for n in ['cr0','cr3','cr4']],
                'independently observed control registers')
    for phase in [native['phases'][0],native['phases'][-1]]:
        snap=phase['boot_snapshot']
        require(len(snap['cpus'])==2 and len({tuple(c['ptid']) for c in snap['cpus']})==2,'all two VM CPUs captured')
        bsp=[c['registers'] for c in snap['cpus'] if c['registers']['cr3']==ROOTS[0] and c['registers']['cs']==16]
        require(len(bsp)==1 and [bsp[0][n] for n in ['cr0','cr3','cr4']]==registers[1:4],
                'unique BSP independently matches entry controls')
        require(0x109040<=bsp[0]['rsp']<0x180000 and bsp[0]['eflags']&0x700==0,
                'independent BSP stack and critical flags')
        efer=re.search(r'\bEFER=([0-9a-fA-F]+)',snap['monitor'])
        require(efer and int(efer.group(1),16)==registers[4],'independent monitor EFER observation')
        buffers={}
        for name,fact in snap['regions'].items():
            require(fact['file']==f"boot-{phase['phase']}-{name}.bin",'closed boot data filename')
            data=read(fact['file'])
            require(len(data)==fact['bytes'] and digest(data)==fact['sha256'],'complete boot bytes/digest')
            buffers[name]=data
        require(set(buffers)=={'entry-tables','gdt','idt','idt-pointer','transition'},'complete retained boot byte regions')
        check_pages(buffers['entry-tables']); check_gdt(buffers['gdt']); check_idt(buffers['idt'],handlers)
        check_idt_pointer(buffers['idt-pointer'])
        transition=buffers['transition']
        require(len(transition)==registers[26] and transition==boot_image[2560:2560+len(transition)],'actual complete adapter handoff bytes')
        require(transition[-12:-10]==b'\x48\xb8' and transition[-2:]==b'\xff\xe0' and
                int.from_bytes(transition[-10:-2],'little')==handlers[HANDLERS[0]],'closed jump handoff to actual entry')
    constructed=native['phases'][2]['banks']
    for i,bank in enumerate(constructed[:2]): check_cache_indices(read(bank['data']),bank['base'],process=i==1)
    return dict(passed=True,native_execution=False,entry_table_words=2048,idt_words=512,gdt_words=6,idt_pointer_bytes=10,
                variable_mtrrs=registers[28],fixed_mtrrs=11 if registers[27] else 0,
                efer=registers[4],cpus_observed=2,boot_snapshots=2,cache_indices_preserved=True,
                independently_observed_msr_reads=len(expected)*2,
                protected_context=False,production_qualified=False,
                scope='Actual retained boot bytes, capabilities and complete advertised memory-attribute state; snapshots do not qualify CPU/writer lifetime or activation')


def altered_record_controls(native,contract,kernel,boot_image,read):
    """Alter saved observations only; do not modify guest files or VM state."""
    from copy import deepcopy
    rejected=[]
    cases=['cr4','efer','capabilities','mtrr-count','reserved-observation','resource',
           'missing-msr','reordered-msr','changed-msr','retained-state','cpu-count',
           'cpu-identity','bsp-control','monitor-efer','handler','adapter-hash']
    for case in cases:
        changed,selected=deepcopy(native),deepcopy(contract)
        journals=[p['boot_registers'] for p in changed['phases']]
        if case in ['cr4','efer','capabilities','mtrr-count','reserved-observation','resource']:
            index,value={'cr4':(3,32|128),'efer':(4,0),'capabilities':(20,0),
                         'mtrr-count':(28,17),'reserved-observation':(29,1),
                         'resource':(17,journals[0][17]+4096)}[case]
            for journal in journals: journal[index]=value
        elif case=='missing-msr': changed['boot_msr_reads'].pop()
        elif case=='reordered-msr': changed['boot_msr_reads'][0:2]=changed['boot_msr_reads'][1::-1]
        elif case=='changed-msr': changed['boot_msr_reads'][2]['value']^=1
        elif case=='retained-state': journals[-1][6]^=1
        elif case=='cpu-count': changed['phases'][0]['boot_snapshot']['cpus'].pop()
        elif case=='cpu-identity':
            cpus=changed['phases'][0]['boot_snapshot']['cpus']
            cpus[1]['ptid']=cpus[0]['ptid'][:]
        elif case=='bsp-control':
            for cpu in changed['phases'][0]['boot_snapshot']['cpus']:
                if cpu['registers']['cr3']==ROOTS[0]: cpu['registers']['cr4']|=128
        elif case=='monitor-efer': changed['phases'][0]['boot_snapshot']['monitor']='EFER=00000000'
        elif case=='handler': selected['handlers'][0]['address']+=1
        elif case=='adapter-hash': selected['boot_image_sha256']='0'*64
        try:
            inspect(changed,selected,kernel,boot_image,read)
        except (AssertionError,ValueError): rejected.append(case)
        else: raise AssertionError('altered boot record accepted: '+case)
    for name,offset,bit,label in [('entry-tables',0,6,'nonleaf-dirty'),
                                  ('entry-tables',8,5,'absent-accessed'),
                                  ('gdt',0,40,'null-descriptor-accessed'),
                                  ('gdt',16,41,'code-descriptor-type'),
                                  ('idt',32,0,'unused-idt-gate'),
                                  ('idt-pointer',2,0,'retained-idt-pointer'),
                                  ('transition',-10,0,'handoff-entry')]:
        changed=deepcopy(native)
        fact=changed['phases'][0]['boot_snapshot']['regions'][name]
        data=bytearray(read(fact['file']))
        if offset<0: offset+=len(data)
        data[offset+bit//8]^=1<<(bit%8)
        fact['sha256']=hashlib.sha256(data).hexdigest()
        altered_read=lambda path: bytes(data) if path==fact['file'] else read(path)
        try:
            inspect(changed,contract,kernel,boot_image,altered_read)
        except (AssertionError,ValueError): rejected.append(label)
        else: raise AssertionError('altered and rehashed boot bytes accepted: '+label)
    return rejected
