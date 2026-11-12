"""Observe unchanged pinned applications on the experimental process-MM kernel.

TOPAL-SYSTEMS-PROCESS-MM-001; TOPAL-SYSTEMS-USER-EXTENT-001. The predecessor
application observer still checks its original boot/MM operations. Separate
namespaces check the owned-extent callbacks and every selected process-MM
boundary. None supplies guest state or activates a public compiler selection.
"""
import hashlib
import json
import os
from pathlib import Path
import struct
import sys

import frame_application_inspect
from process_mm_regression_observer import PROGRAM as MM_PROGRAM, BOUNDARY_PROGRAM
import topal_gate
from user_extent_application_inspect import PROGRAM as EXTENT_APPLICATION_PROGRAM, invariant_halt, members
from user_extent_call_coverage import linked_inventory

harness=frame_application_inspect.harness
LEGACY_PROGRAM=harness.GDB_PROGRAM
_EXTENT_END='def consumed_prelude():'
assert EXTENT_APPLICATION_PROGRAM.count(_EXTENT_END)==1
EXTENT_PROGRAM=EXTENT_APPLICATION_PROGRAM.split(_EXTENT_END)[0]

# All inherited legacy assertions remain, for the same operations 0..7.
# Operation 8 has its separate existing owned-extent observers. Operations
# 9..13 have the actual process-MM observer rather than the predecessor's
# single-active-MM protocol. The namespaces must never share pending state.
EXTRA_PROGRAM=MM_PROGRAM+r'''
extent_namespace=None

def install_application_observers():
    global extent_namespace
    install_process_regression_observation()
    ns=dict(observation_path=observation_path.rsplit('/',1)[0]+'/extent-observations.json',
            extent_inventory=application_extent_inventory)
    exec(application_extent_program,ns)
    extent_namespace=ns
    ns['install_extent_coverage']()

def process_application_complete():
    ns=process_regression_namespace
    assert ns is not None and ns['pending'] is None and ns['case'] is None
    assert 'failure' not in ns['observed'] and not ns['observed']['cases']
    assert words(address('topal_x86_frame_mm_tokens_v1'),6)==[0]*6
    assert words(address('topal_x86_frame_mm_state_v1'),4)==[0]*4
    final={}
    for suffix,count in [('ids',7),('inactive_token',3),('inactive_state',4),('prepared_token',3),
                         ('prepared_state',7),('prepared_owner',3),('transition',3)]:
        actual=words(address('topal_x86_process_mm_'+suffix+'_v1'),count)
        assert actual==[0]*count
        final[suffix]=actual
    events=ns['observed']['events']
    assert any(row['operation']==5 and row['request']>0 for row in events)
    assert any(row['operation']==7 for row in events)
    assert sum(row['operation']==6 for row in events)==1
    assert all('gateway_retirement' in row for row in events)
    allocator=address('topal_x86_frame_context_v1')
    count=word(allocator+24)
    assert 0<count<=262144 and token_bytes%24==0
    observed['final_physical']=dict(
        table_sha256=hashlib.sha256(memory(address('topal_x86_frame_table_v1')+128,count*24)).hexdigest(),
        registry_sha256=hashlib.sha256(memory(address('topal_x86_frame_registry_v1'),count*64)).hexdigest(),
        root_tokens=words(address('topal_x86_frame_root_tokens_v1'),token_bytes//8),cr3=value('$cr3'))
    extent_namespace['extent_coverage_complete']()
    extent_namespace['observed']['passed']=True
    with open(extent_namespace['observation_path'],'w') as stream:
        json.dump(extent_namespace['observed'],stream,indent=2)
    observed.update(extent_namespace['observed'])
    # The enclosing legacy Complete publishes passed only after its remaining
    # whole-catalog/root checks. Do not borrow the extent namespace's success.
    observed['passed']=False
    ns['observed'].update(passed=True,final=final,mode='pinned-application',
        workload=application_workload,kernel_sha256=process_regression_profile['kernel_sha256'])
    ns['complete_save']()

class LegacySubset(ApplicationGate):
    def stop(self):
        if self.role=='entry' and value('$rdi') in [8,9,10,11,12,13]:return False
        if self.role=='return' and pending is None:return False
        return super().stop()

def fail():
    traceback.print_exc()
    observed['passed']=False
    observed['observer_failure']=traceback.format_exc()
    registers={}
    for name in ['rip','rsp','cs','eflags','cr3','rax','rdi','rsi','rdx','rcx','r8','r9','r10','r11']:
        try:registers[name]=value('$'+name)&((1<<64)-1)
        except Exception as error:registers[name]=dict(read_failure=str(error))
    observed['failure_context']=dict(workload=application_workload,registers=registers)
    if pending and isinstance(pending.get('data_before'),bytes):
        pending['data_before_sha256']=hashlib.sha256(pending.pop('data_before')).hexdigest()
    observed['pending']=pending
    try:observed['backtrace']=gdb.execute('bt',to_string=True)
    except Exception as error:observed['backtrace_read_failure']=str(error)
    with open(observation_path,'w') as stream:json.dump(observed,stream,indent=2)
    gdb.execute('quit 1')

class ApplicationInvariant(gdb.Breakpoint):
    def stop(self):
        try:raise AssertionError('private process invariant halt')
        except Exception:fail()
        return True
'''

MM_RETIREMENT_PROGRAM=r'''
class Gateway(Gateway):
    def observe(self):
        count=len(observed['events'])
        result=super().observe()
        if not self.enter:
            assert len(observed['events'])==count+1
            # These are the same complete return-retirement windows checked
            # by the inherited application Gate for ordinary MM operations.
            spans=[('caller',reg('rsp')-4080,4080)]+[
                (name,address('topal_x86_frame_'+name+'_v1'),size)
                for name,size in [('private',65528),('stack',4096),('facts',24),
                                  ('output',56),('gateway_recovery',48)]]
            retired={}
            for name,pointer,size in spans:
                actual=memory(pointer,size)
                assert actual==bytes(size),(name,hex(pointer),size)
                retired[name]=dict(address=pointer,bytes=size,sha256=hashlib.sha256(actual).hexdigest())
            observed['events'][-1]['gateway_retirement']=retired
        return result
'''

PROGRAM=LEGACY_PROGRAM
_TERMINAL="            observed.update(passed=True,n=n,live=live,workload=application_workload,"
assert PROGRAM.count(_TERMINAL)==1
PROGRAM=PROGRAM.replace(_TERMINAL,'            process_application_complete()\n'+_TERMINAL)
_INSTALL="try:\n    ApplicationGate('topal_x86_frame_gateway_v1','entry')"
assert PROGRAM.count(_INSTALL)==1
PROGRAM=PROGRAM.replace(_INSTALL,EXTRA_PROGRAM+"\ntry:\n    install_application_observers()\n    ApplicationInvariant('*'+hex(application_invariant_halt))\n    LegacySubset('topal_x86_frame_gateway_v1','entry')")
_RETURN="    ApplicationGate('topal_x86_frame_gateway_return_reclaimed_v1','return')"
assert PROGRAM.count(_RETURN)==1
PROGRAM=PROGRAM.replace(_RETURN,"    LegacySubset('topal_x86_frame_gateway_return_reclaimed_v1','return')")
_SCOPE='Unchanged pinned BusyBox on the selected production frame profile; finite current MM geometry'
assert PROGRAM.count(_SCOPE)==1
PROGRAM=PROGRAM.replace(_SCOPE,'Unchanged pinned BusyBox on the experimental process-MM profile; no public activation')


def profile(build):
    marker=json.loads((build/'process-mm-application.json').read_text())
    assert marker['process_memory'] is True
    kernel=(build/'kernel.elf').read_bytes()
    phoff=struct.unpack_from('<Q',kernel,32)[0]
    phsize,phcount=struct.unpack_from('<HH',kernel,54)
    assert kernel[:6]==b'\x7fELF\x02\x01' and phsize==56
    spans=[]
    for index in range(phcount):
        kind,flags,offset,virtual,physical,filesize,memsize,alignment=struct.unpack_from('<IIQQQQQQ',kernel,phoff+index*phsize)
        if kind==1:
            assert virtual==physical and memsize and physical+memsize<=1<<30
            spans.append([physical,physical+memsize])
    assert spans
    return dict(process_memory=True,kernel_spans=spans,kernel_sha256=hashlib.sha256(kernel).hexdigest())


def main():
    build=Path(sys.argv[1]).resolve()
    assert build.is_relative_to(topal_gate.LINUX_KERNEL)
    selected=os.environ.get('TOPAL_TEST_PROCESS_MM_WORKLOADS')
    selected=selected.split(',') if selected is not None else list(topal_gate.PROCESS_WORKLOADS)
    assert selected and len(selected)==len(set(selected)) and set(selected)<=set(topal_gate.PROCESS_WORKLOADS)
    image=(build/'static-elf-abi-fixture.elf').read_bytes()
    image_sha256=hashlib.sha256(image).hexdigest()
    assert image_sha256==topal_gate.PINNED_STATIC_BUSYBOX_SHA256
    boot=members((build/'initramfs.cpio').read_bytes())
    assert set(boot)=={'init','static-user.elf'} and boot['static-user.elf']==image
    mm_profile=profile(build)
    inventory=linked_inventory(build)
    invariant=invariant_halt(build/'kernel.elf')['halt']
    folder=build/'process-mm-applications';folder.mkdir()
    report=dict(passed=False,complete_corpus=False,selected=selected,workloads={},
        kernel_sha256=mm_profile['kernel_sha256'],image_sha256=image_sha256)
    reached={};where={}
    for name in selected:
        command,output,status=topal_gate.PROCESS_WORKLOADS[name]
        harness.GDB_PROGRAM=PROGRAM
        prefix=(f'application_workload={name!r}\ncomponent={{"mode":"pinned-application"}}\n'
                f'process_regression_profile={mm_profile!r}\n'
                f'process_regression_boundary_program={BOUNDARY_PROGRAM+MM_RETIREMENT_PROGRAM!r}\n'
                f'application_extent_inventory={inventory!r}\napplication_extent_program={EXTENT_PROGRAM!r}\n'
                f'application_invariant_halt={invariant!r}\n')
        harness.main(work_name='process-mm-applications/'+name,process_command=command,
            program_prefix=prefix,input_scope='unchanged pinned BusyBox on experimental process-MM ('+name+')',
            guest_timeout_seconds=600)
        native=folder/name
        serial=(native/'serial.log').read_bytes()
        topal_gate.verify_process_serial(serial,external=True,process_workload=name)
        ordinary=json.loads((native/'observations.json').read_text())
        mm=json.loads((native/'process-mm-observations.json').read_text())
        assert ordinary['passed'] and mm['passed'] and mm['workload']==name
        coverage=ordinary['extent_call_coverage']
        assert coverage['observation_passed'] and coverage['kernel_sha256']==mm_profile['kernel_sha256']
        assert coverage['inventory_sha256']==inventory['inventory_sha256']
        for row in coverage['reached']:
            reached.setdefault(row['offset'],row)
            where.setdefault(row['offset'],[]).append(name)
        report['workloads'][name]=dict(passed=True,command_hex=command.hex(),output_hex=output.hex(),status=status,
            serial_sha256=hashlib.sha256(serial).hexdigest(),mm_boundaries=len(mm['events']),
            observation_sha256=hashlib.sha256((native/'observations.json').read_bytes()).hexdigest(),
            mm_observation_sha256=hashlib.sha256((native/'process-mm-observations.json').read_bytes()).hexdigest())
        (folder/'observations.json').write_text(json.dumps(report,indent=2)+'\n')
        print('PROCESS_MM_APPLICATION_PASS',name,len(mm['events']),flush=True)
    report.update(passed=True,complete_corpus=set(selected)==set(topal_gate.PROCESS_WORKLOADS))
    report['extent_call_coverage']=dict(observation_passed=True,total_sites=len(inventory['calls']),
        reached_sites=len(reached),reached=[dict(reached[offset],sample_workload=where[offset][0],
            observed_in_workloads=where[offset]) for offset in sorted(reached)],
        kernel_sha256=inventory['kernel_sha256'],inventory_sha256=inventory['inventory_sha256'],
        scope='Union of first-reach samples from the explicitly selected application runs; individual observations retain their own arguments and phases, no every-path/lifetime qualification')
    (folder/'observations.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':main()
