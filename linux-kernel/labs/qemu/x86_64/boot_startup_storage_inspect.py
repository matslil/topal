"""Read-only startup-storage observation, with no processor-control injection."""
GDB_HELPERS = r'''
def capture_startup(label):
    base=observed['boot_params']
    assert base>=4096 and base%4096==0 and base+8192<=0xa0000
    row=dict(boot_params=base,ebda_segment=int.from_bytes(memory(0x40e,2),'little'),
        journal=words(address('topal_x86_boot_startup_storage_v1'),32),regions={})
    for name,offset in [('parameters',0),('page',4096)]:
        data=memory(base+offset,4096)
        path='startup-'+label+'-'+name+'.bin'
        with open(os.path.join(os.path.dirname(observation_path),path),'wb') as f:f.write(data)
        row['regions'][name]=dict(start=base+offset,bytes=4096,file=path,sha256=hashlib.sha256(data).hexdigest())
    return row

class StartupCheckpoint(gdb.Breakpoint):
    def __init__(self):
        super().__init__('*topal_x86_boot_startup_observe_v1',type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def stop(self):
        try:
            assert reg('cs')&3==0 and reg('eflags')&0x600==0
            events=observed.setdefault('startup_storage_events',[])
            row=capture_startup('claim-'+str(len(events)))
            assert row['journal'][0]==len(events)+1<=3
            row['controls']={name:reg(name) for name in ['cr0','cr3','cr4']}
            events.append(row)
            if len(events)==3:
                self.enabled=False
                RetainedEvent()
            return False
        except Exception:fail()
'''


if __name__=='__main__':
    import image_backing_inspect
    image_backing_inspect.main(tables=True,retained=True,startup_storage=True)
