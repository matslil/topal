"""Read-only observation of actual namespace refusal/no-op ownership and cleanup."""
import hashlib
import json
from pathlib import Path
import traceback

import gdb

NUMBERS = (82, 86, 264, 265, 316)
MAX = (1 << 64)-1


def reg(name):
    return int(gdb.parse_and_eval('$'+name)) & MAX


def memory(start, size):
    return bytes(gdb.selected_inferior().read_memory(start,size))


class Observer:
    def __init__(self,config):
        assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['physical_sha256']
        self.config = config
        self.output = Path(config['output'])
        self.geometry = config['artifact']['namespace_runtime']
        self.binding = self.geometry['invocation']
        self.files = config['artifact']['volatile_files']
        self.owner = self.binding['controller_owner']['address']
        self.owner_bytes = self.binding['controller_owner']['bytes']
        self.storage = self.binding['storage']
        self.history = next(s for s in self.storage if s['symbol'].endswith('_history'))
        self.current = None
        self.observations = []
        self.generation = 0
        self.expected = sum(case['number'] in NUMBERS for case in config['cases'])
        import namespace_source_physical
        assert hashlib.sha256(Path(namespace_source_physical.__file__).read_bytes()).hexdigest() == config['source_physical_sha256']
        self.loan = namespace_source_physical.LoanObserver(self)
        observer = self

        class Apply(gdb.Breakpoint):
            def stop(self):
                try:
                    if observer.current:
                        assert reg('cs') & 3 == 0
                        assert reg('rdi') == observer.owner
                        assert reg('rsi') == observer.owner+observer.geometry['metadata_offset']
                        observer.current['actions'] += 1
                        observer.loan.begin()
                    return False
                except BaseException:
                    return observer.failed()

        apply = next(c for c in self.binding['code'] if c['symbol'].endswith('_apply_v1'))
        self.apply = Apply('*'+hex(apply['address']),internal=True)

    def failed(self):
        (self.output/'failed.txt').write_text(traceback.format_exc())
        return True

    def watch(self,case):
        if case['number'] not in NUMBERS:
            return
        observer = self

        class Enter(gdb.Breakpoint):
            def stop(self):
                try:
                    assert observer.current is None
                    assert reg('cs') & 3 == 3
                    assert memory(case['returned']-2,2) == b'\x0f\x05'
                    assert reg('rax') == case['number']
                    for name,value in zip(('rdi','rsi','rdx','r10','r8','r9'),case['arguments']):
                        assert reg(name) == value & MAX, (name,reg(name),value)
                    observer.current = dict(case=case,actions=0,
                        before=memory(observer.owner,observer.owner_bytes),
                        history=memory(observer.history['address'],observer.history['bytes']))
                    assert int.from_bytes(observer.current['history'],'little') == observer.generation, 'namespace history rewound or advanced outside observed actions'
                    self.enabled = False
                    return False
                except BaseException:
                    return observer.failed()

        Enter('*'+hex(case['returned']-2),internal=True)

    def returned(self,case):
        if case['number'] not in NUMBERS:
            return
        current = self.current
        assert current and current['case'] == case
        unchanged = case['expected'] < 0 or case.get('no_namespace_action',False)
        if unchanged:
            assert current['actions'] == 0, ('refusal/no-op entered namespace source',case)
            assert memory(self.owner,self.owner_bytes) == current['before'], ('owner bytes changed',case)
            assert memory(self.history['address'],self.history['bytes']) == current['history']
        else:
            assert current['actions'] == 1, ('mutation must use one actual source action',case)
            assert current.get('source_effect'), ('source effect not observed',case)
            assert current['source_effect']['generation'] == self.generation+1
            self.generation += 1
        assert int.from_bytes(memory(self.history['address'],8),'little') == self.generation
        for storage in self.storage:
            if storage != self.history:
                assert memory(storage['address'],storage['bytes']) == bytes(storage['bytes']), storage
        for offset,size in ((4096,4096),(self.files['transfer_staging_offset'],256),
                            (self.files['filesystem_offset']+self.files['directory_staging_offset'],512)):
            assert memory(self.owner+offset,size) == bytes(size), (offset,size,case)
        self.observations.append(dict(case=case,source_actions=current['actions'],
            complete_owner_preserved=unchanged,owner_bytes=self.owner_bytes,
            owner_sha256=hashlib.sha256(current['before']).hexdigest(),
            source_private_and_input_staging_erased=True,
            admission_stack_erased=self.loan.retired_stack(),
            source_effect=current.get('source_effect')))
        self.current = None
        record = dict(schema='topal.namespace-syscall-physical-refusal/1',
            result='pass' if len(self.observations) == self.expected else 'in-progress',
            expected_calls=self.expected,
            calls=self.observations,scope='actual user entries: complete owner/generation preservation on refusal/no-op, ordinary source store replay and exact complete owner delta, GPR/SIMD preservation, source and admission stack/buffer retirement; fault/lifetime qualification remains separate')
        (self.output/'refusal-physical.json').write_text(json.dumps(record,indent=2)+'\n')
