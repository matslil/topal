"""Real syscall fatal recovery: exact source effects, all erasures, no user return."""
import hashlib
import json
from pathlib import Path
import struct
import traceback

import gdb

from namespace_source_physical import LoanObserver, REGISTERS, memory, reg, words


def install(config):
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['inspector_sha256']
    import namespace_source_physical
    assert hashlib.sha256(Path(namespace_source_physical.__file__).read_bytes()).hexdigest() == config['source_physical_sha256']
    output = Path(config['output'])
    mode = config['fault_mode']
    seen = []
    target = config['cases'][-1]
    assert target['must_not_return']

    class Observer:
        def __init__(self):
            self.config = config
            self.geometry = config['artifact']['namespace_runtime']
            self.binding = self.geometry['invocation']
            self.files = config['artifact']['volatile_files']
            self.owner = self.binding['controller_owner']['address']
            self.owner_bytes = self.binding['controller_owner']['bytes']
            self.storage = self.binding['storage']
            self.current = None
            self.before_user = None
            self.aborted = False
            self.loan = LoanObserver(self)

        def failed(self):
            (output/'failed.txt').write_text(traceback.format_exc())
            return True

    observer = Observer()

    class Returned(gdb.Breakpoint):
        def __init__(self,case):
            self.case = case
            super().__init__('*'+hex(case['returned']),internal=True)

        def stop(self):
            try:
                assert not self.case.get('must_not_return'), 'fatal source returned to its user caller'
                assert reg('cs') & 3 == 3
                assert reg('rax') == self.case['expected'] & ((1 << 64)-1)
                seen.append(self.case)
                self.enabled = False
                return False
            except BaseException:
                return observer.failed()

    class User(gdb.Breakpoint):
        def stop(self):
            try:
                assert len(seen) == len(config['cases'])-1
                assert reg('cs') & 3 == 3 and reg('rax') == target['number']
                assert memory(target['returned']-2,2) == b'\x0f\x05'
                for name,value in zip(('rdi','rsi','rdx','r10','r8','r9'),target['arguments']):
                    assert reg(name) == value & ((1 << 64)-1)
                observer.before_user = memory(observer.owner,observer.owner_bytes)
                self.enabled = False
                return False
            except BaseException:
                return observer.failed()

    class Apply(gdb.Breakpoint):
        def stop(self):
            try:
                assert observer.before_user is not None and observer.current is None
                assert reg('cs') & 3 == 0
                assert (reg('rdi'),reg('rsi'),reg('rdx')) == (
                    observer.owner,observer.owner+observer.geometry['metadata_offset'],target['namespace_operation'])
                observer.current = dict(case=target)
                observer.loan.begin()
                current = observer.loan.current
                current['actual_return_words'] = words(observer.loan.stack()-16,2)
                assert all(current['actual_return_words'])
                assert current['generation'] == 1
                for offset,size in ((4096,4096),(observer.files['transfer_staging_offset'],256)):
                    assert memory(observer.owner+offset,size) == bytes(size)
                return False
            except BaseException:
                return observer.failed()

    class Aborted(gdb.Breakpoint):
        def stop(self):
            try:
                current = observer.loan.current
                assert current and not observer.aborted and current['source_entries'] == 1
                observer.aborted = True
                assert words(observer.loan.storage['recovery']['address'],4) == (
                    current['frame'],current['generation'],current['operation'],current['seconds'])
                if mode == 'runtime-abort':
                    # Exhaustion is injected after the first store; already
                    # constructed constant operands may allow another store
                    # before the next allocation. Every completed store must
                    # still survive recovery exactly, without rollback.
                    assert current.get('allocator_fault_after_store') and current['stores']
                    for name in REGISTERS[:6]:
                        gdb.execute('set $'+name+' = 123')
                else:
                    assert current['source_returned']
                    assert current['source_result'] == int(mode != 'source-false')
                return False
            except BaseException:
                return observer.failed()

    class Halted(gdb.Breakpoint):
        def stop(self):
            try:
                current = observer.loan.current
                assert current and observer.aborted and current['source_entries'] == 1 and current['stores']
                assert reg('cs') & 3 == 0 and not reg('eflags') & 0x200
                assert all(reg(name) == 0 for name in ('rax',)+REGISTERS)
                assert reg('rsp') == observer.loan.stack()
                expected = bytearray(current['replayed'])
                offset = observer.geometry['metadata_offset']
                expected[offset+3072:offset+3584] = bytes(512)
                actual = memory(observer.owner,observer.owner_bytes)
                assert actual == bytes(expected), 'persistent effects rolled back or extra owner bytes changed'
                assert actual[offset:offset+3072] != current['before'][offset:offset+3072], 'no persistent change preceded fatal recovery'
                assert actual[:offset] == observer.before_user[:offset]
                assert actual[offset+4096:] == observer.before_user[offset+4096:]
                for storage in observer.storage:
                    value = memory(storage['address'],storage['bytes'])
                    assert value == (struct.pack('<Q',current['generation'])
                        if storage['symbol'].endswith('_history') else bytes(storage['bytes'])), storage
                for name in ('file_position','ram_page','inode_mode'):
                    address = int(gdb.parse_and_eval("&'topal_x86_systems_"+name+"_control'"))
                    assert memory(address,64) == bytes(64)
                for delta,size in ((4096,4096),(observer.files['transfer_staging_offset'],256),
                        (observer.files['filesystem_offset']+observer.files['directory_staging_offset'],512)):
                    assert memory(observer.owner+delta,size) == bytes(size)
                assert memory(current['frame']-4096,4096+184) == bytes(4096+184)
                assert memory(observer.loan.stack()-4096,4096) == bytes(4096)
                assert words(observer.loan.stack()-16,2) == (0,0)
                (output/'owner-before-apply.bin').write_bytes(current['before'])
                (output/'owner-at-halt.bin').write_bytes(actual)
                record = dict(schema='topal.namespace-real-syscall-fatal/1',result='pass',
                    mode=mode,case=target,preparation_returns=seen,source_entries=1,
                    generation=current['generation'],operation=current['operation'],seconds=current['seconds'],
                    source_returned=current['source_returned'],source_result=current.get('source_result'),
                    stores=current['stores'],whole_owner_exact_replayed_delta=True,persistent_effects_retained=True,
                    source_frame_and_reserve_erased=4280,admission_stack_erased=4096,
                    original_owning_return_words=current['actual_return_words'],actual_return_words_erased=True,
                    all_private_staging_and_gprs_erased=True,
                    kernel_state_injections=(['arena exhaustion after first source store','callee-saved GPR clobber at actual abort hook']
                        if mode == 'runtime-abort' else []),
                    scope='real CPL3 Linux syscall and actual ordinary source; fatal boundary requires debugger disconnect plus QMP stopped confirmation')
                (output/'observation.json').write_text(json.dumps(record,indent=2)+'\n')
                return True
            except BaseException:
                return observer.failed()

    for case in config['cases']:
        Returned(case)
    User('*'+hex(target['returned']-2),internal=True)
    binding = observer.binding
    Apply('*'+hex(next(c['address'] for c in binding['code'] if c['symbol'].endswith('_apply_v1'))),internal=True)
    Aborted("*'topal_kernel_region_frame_namespace_metadata_abort_v1'",internal=True)
    halt = next(c['address'] for c in binding['code'] if c['symbol'].endswith('_halt_v1'))
    Halted('*'+hex(halt+observer.geometry['admission_retirement']['halt_instruction_offset']),internal=True)
