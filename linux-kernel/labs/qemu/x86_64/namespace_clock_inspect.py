"""Observe RTC access ordering and force only a guest RTC read to remain busy."""
import hashlib
import json
from pathlib import Path
import traceback

import gdb

from namespace_source_physical import memory,reg,words

NUMBERS = (82,86,264,265,316)
MAX = (1 << 64)-1


def install(config):
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['inspector_sha256']
    output = Path(config['output'])
    binding = config['artifact']['namespace_runtime']
    invocation = binding['invocation']
    files = config['artifact']['volatile_files']
    owner = invocation['controller_owner']['address']
    owner_bytes = invocation['controller_owner']['bytes']
    history = next(s for s in invocation['storage'] if s['symbol'].endswith('_history'))
    probe = config['clock_probe']
    state = dict(current=None,verified=False,calls=[],returns=[])
    expected = sum(case['number'] in NUMBERS for case in config['cases'])

    def fail():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    class User(gdb.Breakpoint):
        def __init__(self,case):
            self.case = case
            super().__init__('*'+hex(case['returned']-2),internal=True)

        def stop(self):
            try:
                assert state['current'] is None
                assert reg('cs') & 3 == 3 and reg('rax') == self.case['number']
                assert memory(self.case['returned']-2,2) == b'\x0f\x05'
                for name,value in zip(('rdi','rsi','rdx','r10','r8','r9'),self.case['arguments']):
                    assert reg(name) == value & MAX
                if not state['verified']:
                    bounds = [probe]+invocation['code']+invocation['source_code']+invocation['source_data']+[invocation['controller_code']]
                    for bound in bounds:
                        assert hashlib.sha256(memory(bound['address'],bound['bytes'])).hexdigest() == bound['sha256'],bound['symbol']
                    assert memory(probe['first_update_read_return']-6,9).hex() == probe['instruction_prefix']
                    state['verified'] = True
                state['current'] = dict(case=self.case,before=memory(owner,owner_bytes),
                    history=memory(history['address'],8),rtc_reads=[],actions=0,context=None)
                self.enabled = False
                return False
            except BaseException:
                return fail()

    class Clock(gdb.Breakpoint):
        def stop(self):
            try:
                current = state['current']
                if current is None:
                    return False
                assert reg('cs') & 3 == 0 and not reg('eflags') & 0x200
                context = reg('gs_base')
                assert words(context+928,1) == (owner,)
                current['context'] = context
                observed = reg('rax') & 255
                current['rtc_reads'].append(observed)
                if current['case'].get('clock_failure'):
                    assert config['mode'] == 'busy-rtc' and current['actions'] == 0
                    assert memory(history['address'],8) == current['history']
                    # Test-only VM input fault at the actual IN return. The
                    # normal bounded retry loop, EIO and retirement must run.
                    gdb.execute('set $al = $al | 128')
                return False
            except BaseException:
                return fail()

    class Apply(gdb.Breakpoint):
        def stop(self):
            try:
                current = state['current']
                if current:
                    assert not current['case'].get('clock_failure'), 'clock failure issued a source action'
                    assert reg('cs') & 3 == 0 and reg('rdi') == owner
                    current['actions'] += 1
                return False
            except BaseException:
                return fail()

    class Returned(gdb.Breakpoint):
        def __init__(self,case):
            self.case = case
            super().__init__('*'+hex(case['returned']),internal=True)

        def stop(self):
            try:
                assert reg('cs') & 3 == 3 and reg('rax') == self.case['expected'] & MAX
                if self.case['number'] in NUMBERS:
                    current = state['current']
                    assert current and current['case'] == self.case
                    unchanged = self.case['expected'] < 0 or self.case.get('no_namespace_action',False)
                    if unchanged:
                        assert current['actions'] == 0
                        assert memory(owner,owner_bytes) == current['before']
                        assert memory(history['address'],8) == current['history']
                        assert len(current['rtc_reads']) == (1024 if self.case.get('clock_failure') else 0)
                    else:
                        assert current['actions'] == 1 and current['rtc_reads']
                    for storage in invocation['storage']:
                        if storage != history:
                            assert memory(storage['address'],storage['bytes']) == bytes(storage['bytes'])
                    for offset,size in ((4096,4096),(files['transfer_staging_offset'],256),
                            (files['filesystem_offset']+files['directory_staging_offset'],512)):
                        assert memory(owner+offset,size) == bytes(size)
                    stack_erased = None
                    if current['context']:
                        retirement = binding['admission_retirement']
                        stack = words(current['context']+retirement['kernel_stack_context_offset'],1)[0]
                        assert retirement['minimum_stack_pointer'] <= stack <= retirement['maximum_stack_pointer']
                        assert stack % retirement['stack_alignment'] == 0
                        stack_erased = retirement['retired_stack_bytes']
                        assert memory(stack-stack_erased,stack_erased) == bytes(stack_erased)
                    if self.case.get('clock_failure'):
                        assert stack_erased == 4096
                        (output/'owner-before-clock.bin').write_bytes(current['before'])
                        (output/'owner-after-clock.bin').write_bytes(memory(owner,owner_bytes))
                    state['calls'].append(dict(case=self.case,first_status_reads=len(current['rtc_reads']),
                        observed_status_bytes=current['rtc_reads'],source_actions=current['actions'],
                        complete_owner_preserved=unchanged,history_preserved=unchanged,
                        private_buffers_erased=True,admission_stack_erased=stack_erased))
                    state['current'] = None
                self.enabled = False
                state['returns'].append(self.case)
                (output/'syscall-returns.json').write_text(json.dumps(state['returns'],indent=2)+'\n')
                if len(state['returns']) == len(config['cases']):
                    assert len(state['calls']) == expected and state['current'] is None
                    record = dict(schema='topal.namespace-clock-ordering/1',result='pass',mode=config['mode'],
                        calls=state['calls'],all_bound_native_code_and_data_verified=state['verified'],
                        kernel_state_injections=(['RTC update-in-progress bit forced at actual IN return']
                            if config['mode'] == 'busy-rtc' else []))
                    (output/'clock-observation.json').write_text(json.dumps(record,indent=2)+'\n')
                    return True
                return False
            except BaseException:
                return fail()

    Clock('*'+hex(probe['first_update_read_return']),internal=True)
    Apply('*'+hex(next(c['address'] for c in invocation['code'] if c['symbol'].endswith('_apply_v1'))),internal=True)
    for case in config['cases']:
        Returned(case)
        if case['number'] in NUMBERS:
            User(case)
