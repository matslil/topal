"""Read-only observation of normal source-bound inactive backing preparation.

Hardware breakpoints observe provider checkpoints and the ordinary final halt.
No registers, memory, firmware map or application instructions are injected.
"""
import hashlib
import json
from pathlib import Path
import sys

import frame_root_inspect as harness
import topal_gate


PROGRAM = r'''
import gdb, hashlib, json, os, traceback
tables=profile.get('tables',False)
retained=profile.get('retained',False)
startup_storage=profile.get('startup_storage',False)
released=6 if retained else (4 if tables else 3)
boot_capture_phases=[0,3] if retained else [0,released]
extent_owners=profile['bindings']['banks']+([profile['bindings']['low_user_leaf']] if tables else [])
observed = dict(passed=False, phases=[], target_state_injected=False,
                protected_context=False, native_execution_attempted=True)

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(p,n): return bytes(gdb.selected_inferior().read_memory(p,n))
def words(p,n):
    raw=memory(p,n*8)
    return [int.from_bytes(raw[i:i+8],'little') for i in range(0,len(raw),8)]
def fail():
    observed['failure']=traceback.format_exc()
    print(observed['failure'],flush=True)
    with open(observation_path,'w') as f: json.dump(observed,f,indent=2)
    gdb.execute('quit 1')

class Initialization(gdb.Breakpoint):
    def __init__(self):
        super().__init__('*topal_x86_frame_gateway_v1',type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def stop(self):
        try:
            assert reg('rdi')==0 and 'boot_params' not in observed
            observed['boot_params']=reg('rsi')
            observed['initial_stack']=reg('rsp')
            assert observed['initial_stack']-profile['component']['combined_boot_stack_bytes']>=0x109040
            self.enabled=False
            if startup_storage: StartupCheckpoint()
            elif retained: RetainedEvent()
            return False
        except Exception: fail()

class BootRead(gdb.Breakpoint):
    def __init__(self):
        super().__init__('*topal_x86_image_boot_read_observe_v1',type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def stop(self):
        try:
            kind=reg('rdi')
            assert kind in [0,1,2]
            if kind==0:
                observed.setdefault('boot_msr_reads',[]).append(dict(number=reg('rsi'),value=reg('rdx')))
                assert len(observed['boot_msr_reads'])<=96
            else:
                observed.setdefault('boot_processor_reads',[]).append(dict(kind=kind,selector=reg('rsi'),value=reg('rdx')))
                assert len(observed['boot_processor_reads'])<=4
            return False
        except Exception: fail()

class Checkpoint(gdb.Breakpoint):
    def __init__(self,symbol,final=False):
        super().__init__('*'+symbol,type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.final=final
    def stop(self):
        try:
            assert reg('cs')&3==0 and reg('eflags')&0x600==0
            ctx=words(address('topal_x86_frame_context_v1'),4)
            assert ctx[0]==1
            tokens=address('topal_x86_frame_root_tokens_v1')
            journal=words(address('topal_x86_image_tables_observation_v1' if tables else 'topal_x86_image_backing_observation_v1'),32)
            controls={name:reg(name) for name in ['cr0','cr3','cr4']}
            assert 0x109040<=reg('rsp')<=0x180000
            if self.final:
                if len(observed['phases'])!=released+1:
                    observed['halt_diagnostics']=dict(context=ctx,journal=journal,controls=controls,
                        tokens=words(tokens,profile['bindings']['token_bytes']//8),
                        stack=words(reg('rsp'),16),backtrace=gdb.execute('bt',to_string=True))
                    if tables:
                        observed['halt_diagnostics']['boot_registers']=words(address('topal_x86_image_boot_observation_v1'),128)
                        observed['halt_diagnostics']['processor_registers']=words(address('topal_x86_boot_processors_v1'),128)
                assert [row['phase'] for row in observed['phases']]==list(range(released+1))
                owner=profile['active_owners']['bootstrap']
                token=words(tokens+owner['slot']*24,3)
                record=words(address('topal_x86_frame_registry_v1')+token[2]*64,8)
                assert token[0]==record[0]==1 and token[1]==record[1]>0
                assert record[3:5]==[5,1] and record[6:]==[1,1]
                cells=words(address('topal_x86_frame_table_v1')+128+record[2]*24,15)
                assert cells[0::3]==[token[1]]*5
                assert cells[1::3]==[record[5]]*5
                assert cells[2::3]==list(range(cells[2],cells[2]+5))
                base=cells[2]*4096
                assert reg('cr3')==base
                data=memory(base,5*4096)
                with open(os.path.join(os.path.dirname(observation_path),'final-bootstrap-bank.bin'),'wb') as f: f.write(data)
                for slot in range(profile['bindings']['token_bytes']//24):
                    if slot!=owner['slot']: assert words(tokens+slot*24,3)==[0,0,0]
                observed['final_bootstrap']=dict(token=token,record=record,controls=controls,
                    base=base,bank_sha256=hashlib.sha256(data).hexdigest())
                if startup_storage: observed['final_bootstrap']['startup_storage']=capture_startup('terminal')
                if retained: observed['retention_event_count']=len(observed['retention_events'])
                observed['passed']=True
                with open(observation_path,'w') as f: json.dump(observed,f,indent=2)
                return True
            phase=journal[0]
            assert phase==len(observed['phases']) and phase<=released
            assert journal[1]==profile['pages']
            if not retained or phase<=3:
                assert controls['cr3']==journal[4]
                if observed['phases']: assert controls==observed['phases'][0]['controls']
            row=dict(phase=phase,journal=journal,context=ctx,controls=controls,stack=reg('rsp'),banks=[])
            if startup_storage: row['startup_storage']=capture_startup('table-'+str(phase))
            if tables:
                boot_registers=words(address('topal_x86_image_boot_observation_v1'),128)
                assert boot_registers[0]==1
                if not retained or phase<=3:
                    assert boot_registers[1:4]==[controls['cr0'],controls['cr3'],controls['cr4']]
                if observed['phases']: assert boot_registers==observed['phases'][0]['boot_registers']
                row['boot_registers']=boot_registers
                row['boot_coverage']=words(address('topal_x86_boot_coverage_v1'),64)
                if retained: row['retained_tables']=words(address('topal_x86_retained_tables_v1'),32)
                capture_boot_processors(row)
                if phase in boot_capture_phases:
                    original_thread=gdb.selected_thread()
                    cpus=[]
                    for thread in gdb.selected_inferior().threads():
                        thread.switch()
                        cpus.append(dict(thread=thread.num,ptid=list(thread.ptid),registers={n:reg(n) for n in ['cr0','cr3','cr4','cs','rip','rsp','eflags']}))
                    original_thread.switch()
                    monitor=gdb.execute('monitor info registers',to_string=True)
                    import re
                    efer=re.search(r'\bEFER=([0-9a-fA-F]+)',monitor)
                    assert efer and int(efer.group(1),16)==boot_registers[4]
                    captured=dict(cpus=cpus,monitor=monitor,regions={})
                    regions=[('entry-tables',None,None),('gdt',boot_registers[9],48),
                             ('idt',boot_registers[11],4096),('idt-pointer',0x106000,10),
                             ('transition',1048576,boot_registers[26])]
                    for name,base,length in regions:
                        data=(b''.join(memory(base,4096) for base in profile['boot_contract']['entry_table_pages'])
                              if name=='entry-tables' else memory(base,length))
                        filename=f'boot-{phase}-{name}.bin'
                        with open(os.path.join(os.path.dirname(observation_path),filename),'wb') as f: f.write(data)
                        captured['regions'][name]=dict(file=filename,bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
                    row['boot_snapshot']=captured
            for index,owner in enumerate(extent_owners):
                token=words(tokens+owner['slot']*24,3)
                if phase<released:
                    assert token[0]==1 and 0<token[1]<ctx[2] and token[2]<ctx[3]
                    record=words(address('topal_x86_frame_registry_v1')+token[2]*64,8)
                    assert record[:2]==token[:2] and record[3:5]==[owner['count'],1]
                    assert record[6:]==[1,1] and record[5]>0
                    start,count=record[2:4]
                    assert start+count<=ctx[3]
                    cells=words(address('topal_x86_frame_table_v1')+128+start*24,count*3)
                    assert cells[0::3]==[token[1]]*count
                    assert cells[1::3]==[record[5]]*count
                    assert cells[2::3]==list(range(cells[2],cells[2]+count))
                    base=cells[2]*4096
                    assert base==journal[[2,3,10][index]]
                    if phase==0:
                        boot=observed['boot_params']
                        size=memory(boot+488,1)[0]
                        claims=[]
                        for i in range(size):
                            begin,length=words(boot+720+20*i,2)
                            kind=int.from_bytes(memory(boot+736+20*i,4),'little')
                            claims.append([begin,begin+length,kind])
                        resource=int.from_bytes(memory(boot+536,4),'little')
                        length=int.from_bytes(memory(boot+540,4),'little')
                        reserved=[[0,address('topal_x86_kernel_reservation_end_v1')],
                                  [boot,boot+4096],[resource,resource+length]]
                        assert any(kind==1 and a<=base and base+count*4096<=b for a,b,kind in claims)
                        assert all(kind==1 or base+count*4096<=a or base>=b for a,b,kind in claims)
                        assert all(base+count*4096<=a or base>=b for a,b in reserved)
                        row['firmware_claims']=claims
                        row['reservations']=reserved
                    else:
                        previous=observed['phases'][0]['banks'][index]
                        assert token==previous['token'] and record==previous['record'] and cells==previous['cells']
                else:
                    previous=observed['phases'][0]['banks'][index]
                    assert token==[0,0,0]
                    record=words(address('topal_x86_frame_registry_v1')+previous['token'][2]*64,8)
                    assert record==[0]*8
                    start,count=previous['record'][2:4]
                    base=previous['base']
                    cells=words(address('topal_x86_frame_table_v1')+128+start*24,count*3)
                    assert cells[0::3]==[0]*count
                    assert cells[1::3]==previous['cells'][1::3] and cells[2::3]==previous['cells'][2::3]
                assert base+count*4096<=1<<30
                assert not base<=controls['cr3']&~4095<base+count*4096
                raw=memory(base,count*4096)
                if phase==0:
                    expected=[6510615555426900570^base^i for i in range(count*512)]
                    assert raw==b''.join(word.to_bytes(8,'little') for word in expected)
                elif tables and (phase==2 or retained and phase in [3,4]) and index<2:
                    expected=table_encode(profile['layout'],base,journal[10] if index==1 else None)
                    assert raw==expected
                else: assert raw==bytes(count*4096)
                name='bank-%d-%d.bin'%(phase,index)
                with open(os.path.join(os.path.dirname(observation_path),name),'wb') as f: f.write(raw)
                row['banks'].append(dict(token=token,record=record,cells=cells,base=base,
                    bytes=count*4096,data=name,data_sha256=hashlib.sha256(raw).hexdigest()))
            for index,a in enumerate(row['banks']):
                for b in row['banks'][:index]:
                    assert a['base']+a['bytes']<=b['base'] or b['base']+b['bytes']<=a['base']
            if not retained or phase<=3:
                assert ctx[2]==journal[9]
                if observed['phases']: assert ctx==observed['phases'][0]['context']
            else:
                assert ctx[:2]==observed['phases'][0]['context'][:2] and ctx[3]==observed['phases'][0]['context'][3]
                assert ctx[2]>=observed['phases'][0]['context'][2]
            observed['phases'].append(row)
            with open(observation_path,'w') as f: json.dump(observed,f,indent=2)
            print('IMAGE_TABLES_PHASE' if tables else 'IMAGE_BACKING_PHASE',phase,flush=True)
            return False
        except Exception: fail()

try:
    Initialization()
    if tables: BootRead()
    Checkpoint('topal_x86_image_tables_observe_v1' if tables else 'topal_x86_image_backing_observe_v1')
    Checkpoint('topal_x86_systems_fatal',True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception: fail()
'''


def main(*, tables=False, retained=False, startup_storage=False):
    build = Path(sys.argv[1]).resolve()
    assert build.is_relative_to(topal_gate.LINUX_KERNEL.resolve())
    group = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                 if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group and (resources / 'memory.swap.max').read_text().strip() == '0'
    role = 'image-tables' if tables else 'image-backing'
    plan = json.loads((build / (role+'-plan-binding.json')).read_text())
    assert plan.get('retained',False)==retained and (not retained or tables)
    assert plan.get('startup_storage',False)==startup_storage and (not startup_storage or retained)
    binding = json.loads((build / 'private-image-binding.json').read_text())
    kernel = topal_gate.digest(build / 'kernel.elf')
    assert plan['kernel_sha256'] == binding['layout']['kernel_sha256'] == kernel
    assert plan['layout'] == binding['layout']
    assert plan['matched_final_partition_and_template' if tables else 'preparation_sizing_matched'] is True
    assert plan['protected_context'] is plan['production_qualified'] is False
    owners = json.loads((build / 'image-backing-bindings.json').read_text())
    component = json.loads((build / (role+'-component.json')).read_text())
    assert component['object_sha256'] == topal_gate.digest(build / (role+'.o'))
    assert component['acyclic'] and component['combined_boot_stack_bytes']<=8192
    assert owners == plan['bindings']
    assert owners['root_object_sha256'] == topal_gate.digest(build / 'root.o')
    pages = 5 + plan['layout']['image_leaf_storage_bytes'] // 4096
    if tables:
        sys.path.insert(0,str(topal_gate.LINUX_KERNEL/'tools/user-memory'))
        from inactive_image_tables import page_count
        pages = page_count(plan['layout'])
        assert component['inactive_tables'] is True
        low = owners['low_user_leaf']
        assert (low['count'],low['alignment'],low['pins'],low['purpose']) == (1,1,1,'prepared-image-low-user-leaf')
        assert low['slot'] not in [b['slot'] for b in owners['banks']]
        boot_contract=json.loads((build/'kernel-image-boot-binding.json').read_text())
        assert boot_contract['kernel_sha256']==kernel
        assert boot_contract['boot_image_sha256']==topal_gate.digest(build/'bzImage')
    assert [row['count'] for row in owners['banks']] == [pages, pages]
    profile = dict(bindings=owners, pages=pages, tables=tables, retained=retained, layout=plan['layout'],
                   component=component,
                   active_owners=json.loads((build / 'root-backing-owners.json').read_text()))
    if startup_storage:
        assert 'boot_startup_storage' in boot_contract
        profile['startup_storage']=True
    if tables: profile['boot_contract']=boot_contract
    if retained:
        from boot_image_binding import symbols
        profile['process_entry']=symbols((build/'kernel.elf').read_bytes(),
            ['topal_x86_systems_static_image_process_enter'])['topal_x86_systems_static_image_process_enter']
        assert profile['process_entry']['type']==2 and 0<profile['process_entry']['size']<=1<<20
    result = dict(passed=False, native_execution=False, native_execution_attempted=False,
                  protected_context=False, production_qualified=False,
                  kernel_sha256=kernel, scope='Actual image-sized whole-owner reserve/zero/erase/release, with read-only hardware-breakpoint observations; no protected activation')
    try:
        result['qemu_version'] = topal_gate.verify_host()
        harness.GDB_PROGRAM = PROGRAM
        result['native_execution_attempted'] = True
        prefix=f'profile={profile!r}\n'
        if tables:
            prefix+=f'import sys\nsys.path.insert(0,{str(topal_gate.LINUX_KERNEL/"tools/user-memory")!r})\nfrom inactive_image_tables import encode as table_encode\n'
            from boot_processors_inspect import GDB_HELPERS
            prefix+=GDB_HELPERS
            if retained:
                from retained_tables_inspect import GDB_HELPERS as RETAINED_HELPERS
                prefix+=RETAINED_HELPERS
            if startup_storage:
                from boot_startup_storage_inspect import GDB_HELPERS as STARTUP_HELPERS
                prefix+=STARTUP_HELPERS
        harness.main(work_name=role+'-native', program_prefix=prefix,
                     guest_timeout_seconds=600, input_scope='ordinary source-built transfer workload')
        native = json.loads((build / (role+'-native/observations.json')).read_text())
        assert native['passed'] and [row['phase'] for row in native['phases']] == list(range(7 if retained else (5 if tables else 4)))
        serial = (build / (role+'-native/serial.log')).read_bytes()
        markers = ([b'TOPAL_KERNEL_IMAGE_TABLES_PREPARED',b'TOPAL_KERNEL_IMAGE_TABLES_DISCARDED'] if tables else
                   [b'TOPAL_KERNEL_IMAGE_BACKING_RESERVED', b'TOPAL_KERNEL_IMAGE_BACKING_DISCARDED'])
        if retained:
            markers=[b'TOPAL_KERNEL_IMAGE_TABLES_PREPARED',b'TOPAL_KERNEL_IMAGE_TABLES_RETAINED',b'TOPAL_KERNEL_IMAGE_RETAINED_TABLES_DISCARDED']
        for marker in markers:
            assert serial.count(marker) == 1
            serial = serial.replace(marker, b'')
        # This existing transfer fixture performs two checked console copies
        # in each of the initial image and three replacements. Preserve the
        # complete log and remove exactly those extension lines for the
        # unchanged baseline newline/output/status contract.
        marker = b'TOPAL_USER_EXTENT_WRITE\n'
        assert serial.count(marker) == 8
        serial = serial.replace(marker, b'')
        topal_gate.verify_process_serial(serial, external=False, process_workload='true')
        result.update(banks=2, pages_per_bank=pages,
                      completed_programs=4, checked_console_copies=8,
                      zeroed_bytes=(pages*2+int(tables))*4096, discarded_bytes=(pages*2+int(tables))*4096,
                      observations=role+'-native/observations.json',
                      actual_final_bootstrap=native['final_bootstrap'])
        if tables:
            result.update(constructed_table_bytes=pages*4096*2,low_user_leaf_bytes=4096,extents=3,
                          scope='Actual image-bound inactive bootstrap/process table construction and separate zero low USER leaf; whole erase/release, no activation')
        if retained:
            events=native['retention_events']
            assert 4<=len(events)<=64 and all(e['syscall'] in [59,60,231] for e in events)
            assert sum(e['syscall']==59 for e in events)>=3
            assert events[-1]['syscall'] in [60,231]
            assert (build/(role+'-native/serial.log')).read_bytes().count(b'TOPAL_KERNEL_PROCESS_EXEC_COMMITTED')==3
            result.update(retained=True,retention_events=len(events),exec_attempts=sum(e['syscall']==59 for e in events),successful_execs=3,
                          scope='Actual pinned inactive owners across initial user execution, three execs and ordinary root restoration; same-owner byte checks and whole retirement, no activation')
        if startup_storage:
            from boot_startup_storage import inspect as startup_inspect
            result['boot_startup_storage']=startup_inspect(native,boot_contract,(build/'bzImage').read_bytes(),
                lambda path:(build/(role+'-native')/path).read_bytes())
        result.update(passed=True,native_execution=True)
    finally:
        result['resources'] = {name: (resources / name).read_text().strip() for name in
                               ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (build / (role+'-observations.json')).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: result[key] for key in ['passed', 'native_execution', 'pages_per_bank', 'zeroed_bytes', 'protected_context']}))


if __name__ == '__main__':
    main()
