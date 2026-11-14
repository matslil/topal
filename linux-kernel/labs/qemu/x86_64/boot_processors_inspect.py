"""Read-only firmware capture fragment for the authorized QEMU/GDB observer."""

GDB_HELPERS = r'''
def capture_boot_processors(row):
    registers=words(address('topal_x86_boot_processors_v1'),128)
    assert registers[0]==1 and 0<registers[12]<=16 and 0<registers[13]<=registers[12]
    assert 0<registers[7]<=64 and 44<=registers[9]<=4096 and registers[14]<=8192
    if observed['phases']:
        assert registers==observed['phases'][0]['processor_registers']
    row['processor_registers']=registers
    if row['phase'] not in boot_capture_phases: return
    root=memory(registers[4],registers[5])
    regions=[('bda',1038,2),('bios',0xe0000,0x20000),
             ('rsdp',registers[1],registers[3]),('root',registers[4],registers[5]),
             ('madt',registers[8],registers[9]),
             ('retained',address('topal_x86_boot_firmware_bytes_v1'),8192)]
    if registers[15]: regions.append(('ebda',registers[15],1024))
    for i in range(registers[7]):
        at=36+i*registers[6]
        pointer=int.from_bytes(root[at:at+registers[6]],'little')
        assert 0<pointer and pointer+36<=1<<30
        regions.append(('header-'+str(i),pointer,36))
    snapshot={}
    for name,base,length in regions:
        data=memory(base,length)
        filename='firmware-%d-%s.bin'%(row['phase'],name)
        with open(os.path.join(os.path.dirname(observation_path),filename),'wb') as stream:
            stream.write(data)
        snapshot[name]=dict(file=filename,address=base,bytes=length,sha256=hashlib.sha256(data).hexdigest())
    row['processor_snapshot']=snapshot
'''
