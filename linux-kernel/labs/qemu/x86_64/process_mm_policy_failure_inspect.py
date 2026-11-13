"""Read-only observation of source-built allocator failures in the process VM.

TOPAL-SYSTEMS-PROCESS-MM-001 / FRAME-POLICY-BINDING-001. Diagnostic
producers are never canonical allocator selections or successful cohorts.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys

import frame_root_inspect as harness
from process_mm_process_inspect import PROGRAM as ORDINARY
from process_mm_publication_observer import augment
from user_extent_application_inspect import invariant_halt

ROOT = Path(__file__).resolve().parents[3]
MODEL = (ROOT / 'tools/process-memory/policy_failure.py').read_text()
PROGRAM = ("failure_model=dict(model)\nexec(" + repr(MODEL) + ",failure_model)\n" + r'''
fault_request=None
fault_source=None
fault_effects=None

def failure_capture(initial=False):
    value=capture();allocator=value['allocator'];n=allocator[3]
    assert 0<n<=262144
    table,comparison,registry=allocator[4:7]
    raw=memory(table+128,n*24)
    result=dict(snapshot=value,header=words(table,16),
        catalog_sha256=hashlib.sha256(raw).hexdigest(),
        registry_sha256=hashlib.sha256(memory(registry,n*64)).hexdigest(),
        history=words(allocator[7],5))
    if initial:result['catalog']=list(struct.unpack('<'+'Q'*(n*3),raw))
    else:
        result.update(snapshot_header=words(comparison,16),
            snapshot_catalog_sha256=hashlib.sha256(memory(comparison+128,n*24)).hexdigest())
    return result

class Gateway(Gateway):
    def observe(self):
        global fault_request
        if not self.enter and fault_source is not None:
            raise AssertionError('diagnostic source failure resumed its caller')
        result=super().observe()
        if self.enter and pending is not None and pending['operation']==5:
            before=pending['before']
            if pending['request']==0x5f7000+101*4096-11 and before['ids'][0]>=2:
                assert fault_request is None and before['ids'][3]==1
                assert before['state'][2]==before['inactive_state'][2]==67
                fault_request=pending
        return result

class SourceEntered(Checked):
    def __init__(self,symbol):
        super().__init__("*'"+symbol+"'",internal=True)
    def observe(self):
        global fault_source
        if fault_request is None:return False
        assert pending is fault_request
        if fault_kind=='allocate-metadata':
            if reg('rsi')!=101:return False
            assert reg('rdx')==1 and reg('rcx')==address('topal_x86_frame_mm_tokens_v1')+24
        else:
            assert reg('rsi')==address('topal_x86_frame_mm_tokens_v1')
        assert reg('rdi')==address('topal_x86_frame_context_v1') and fault_source is None
        fault_source=failure_capture(True)
        observed['policy_failure']=dict(kind=fault_kind,request=fault_request,source_before=fault_source)
        print('PROCESS_MM_DIAGNOSTIC_SOURCE_ENTER',fault_kind,flush=True)
        return False

class FailureEntered(Checked):
    def observe(self):
        global fault_effects
        assert fault_source is not None and fault_effects is None and pending is fault_request
        assert reg('rdi')==address('topal_x86_frame_context_v1')
        fault_effects=failure_capture()
        observed['policy_failure']['effects']=fault_effects
        print('PROCESS_MM_DIAGNOSTIC_EFFECTS_RETAINED',fault_kind,flush=True)
        return False

class FailureRetired(Checked):
    def observe(self):
        assert fault_effects is not None and pending is fault_request
        assert publication_active is publication_switch is None
        value=observed['policy_failure'];terminal=failure_capture()
        terminal.update(rsp=reg('rsp'),callee={r:reg(r) for r in callee},
            volatile={r:reg(r) for r in ['rax','rcx','rdx','rsi','rdi','r8','r9','r10','r11']},
            rip=reg('rip'),retired={})
        domains=[('caller',pending['sp']-4000,4000),
            ('private',address('topal_x86_frame_private_v1'),65528),
            ('stack',address('topal_x86_frame_stack_v1'),4096),
            ('facts',address('topal_x86_frame_facts_v1'),24),
            ('output',address('topal_x86_frame_output_v1'),56),
            ('details',address('topal_x86_frame_details_v1'),32),
            ('recovery',address('topal_x86_frame_gateway_recovery_v1'),48),
            ('allocator_active',address('topal_frame_allocator_active_v1'),8),
            ('boundary_active',address('topal_x86_tail_boundary_active_v1'),8)]
        for name,at,size in domains:
            terminal['retired'][name]=dict(address=at,bytes=size,sha256=hashlib.sha256(memory(at,size)).hexdigest())
        value.update(terminal=terminal,open_retirement=publication_retiring)
        value['comparison']=failure_model['check'](value)
        assert not publication_data['passed']
        observed.update(passed=True,scope='Selected terminal diagnostic outcome; ordinary cohort intentionally incomplete')
        complete_save()
        print('PROCESS_MM_DIAGNOSTIC_FATAL_PASS',fault_kind,flush=True)
        return True

def install_policy_failure():
    SourceEntered('topal_frame_allocator_'+('allocate' if fault_kind=='allocate-metadata' else 'release')+'_v1')
    FailureEntered("*'f.fatal'",internal=True)
    FailureRetired("*'topal_x86_frame_gateway_fatal_reclaimed_v1'",internal=True)
''')


def program():
    ordinary = augment(ORDINARY)
    marker = 'try:\n    install_process_publication()'
    assert ordinary.count(marker) == 1
    return ordinary.replace(marker, PROGRAM + '\ntry:\n    install_policy_failure()\n    install_process_publication()')


def main():
    build = Path(sys.argv[1]).resolve()
    component = json.loads((build / 'source-component.json').read_text())
    diagnostic = json.loads((build / 'process-mm-policy-failure.json').read_text())
    assert component['mode'] == 'process-mm'
    assert diagnostic['schema'] == 'topal-process-mm-source-policy-failure-v1'
    assert diagnostic['kind'] in ['allocate-metadata', 'release-metadata']
    assert hashlib.sha256((build / 'diagnostic-policy.t').read_bytes()).hexdigest() == diagnostic['source_sha256']
    source = Path(__file__).with_name('user_extent_process_mm_cases.c')
    assert hashlib.sha256(source.read_bytes()).hexdigest() == component['process_mm_source_sha256']
    sys.path.insert(0, str(ROOT / 'tools/user-memory'))
    from inspect_translation_publication import inspect
    from inspect_call_inventory import Elf
    profile = inspect(build)
    elf = Elf(build / 'kernel.elf')
    profile.update(pin_entry=elf.symbols['topal_frame_allocator_pin_v1']['value'],
                   release_entry=elf.symbols['topal_frame_allocator_release_v1']['value'])
    (build / 'process-publication-layout.json').write_text(json.dumps(profile, indent=2)+'\n')
    kernel = (build / 'kernel.elf').read_bytes()
    phoff = struct.unpack_from('<Q', kernel, 32)[0]
    phsize, phcount = struct.unpack_from('<HH', kernel, 54)
    spans = []
    for i in range(phcount):
        p = struct.unpack_from('<IIQQQQQQ', kernel, phoff+i*phsize)
        if p[0] == 1:
            assert p[3] == p[4] and p[4]+p[6] <= 1 << 30
            spans.append([p[4], p[4]+p[6]])
    harness.GDB_PROGRAM = program()
    harness.main(work_name='process-mm-policy-failure', expect_backing=False,
        program_prefix=f'component={component!r}\nkernel_spans={spans!r}\nfault_kind={diagnostic["kind"]!r}\n'
                       f'native_halt={invariant_halt(build / "kernel.elf")["halt"]!r}\npublication_profile={profile!r}\n',
        input_scope='source-built invalid allocator delta, retained process owners/effects and declared fatal retirement',
        guest_timeout_seconds=600, cpu_trace_sites=(profile['sites']['invalidated'],))


if __name__ == '__main__':
    main()
