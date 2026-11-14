"""Observe natural inactive-child cancellation; never modify guest state."""
import hashlib
import json
from pathlib import Path
import traceback

import gdb

from namespace_source_physical import memory,reg,words
from namespace_syscall_physical import Observer,MAX


def install(config):
    output=Path(config['output'])
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest()==config['inspector_sha256']
    artifact=config['artifact']
    binding=artifact['namespace_runtime']['invocation']
    owner=binding['controller_owner']['address']
    owner_bytes=binding['controller_owner']['bytes']
    profile=artifact['volatile_files']
    history=next(s for s in binding['storage'] if s['symbol'].endswith('_history'))
    cases=[c for c in config['cases'] if 'event' not in c]
    events=[c for c in config['cases'] if 'event' in c]
    physical_config=dict(config,cases=cases)
    physical=Observer(physical_config)
    seen=[]
    state=dict(events=[],finalize_entries=0,finalized=False,process_returned=False)

    def fail():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    def runtime_zero(include_history=True):
        for storage in binding['storage']:
            if include_history or storage!=history:
                assert memory(storage['address'],storage['bytes'])==bytes(storage['bytes']),storage

    class Returned(gdb.Breakpoint):
        def __init__(self,case):
            self.case=case
            super().__init__('*'+hex(case['returned']),internal=True)

        def stop(self):
            try:
                assert self.case==cases[len(seen)],('return order',self.case,len(seen))
                assert reg('cs')&3==3 and reg('rax')==self.case['expected']&MAX
                assert reg('r12')==0x51ab
                if 'namespace_epoch' in self.case:
                    assert words(history['address'],1)==(self.case['namespace_epoch'],)
                physical.returned(self.case)
                seen.append(self.case)
                (output/'syscall-returns.json').write_text(json.dumps(seen,indent=2)+'\n')
                self.enabled=False
                return False
            except BaseException:
                return fail()

    class Forbidden(gdb.Breakpoint):
        def stop(self):
            (output/'failed.txt').write_text('terminated parent or parked child resumed user code')
            return True

    class Event(gdb.Breakpoint):
        def __init__(self,case):
            self.case=case
            super().__init__('*'+hex(case['returned']-2),internal=True)

        def stop(self):
            try:
                case=self.case
                assert case==events[len(state['events'])]
                assert reg('cs')&3==3 and reg('rax')==case['number'] and reg('r12')==0x51ab
                assert memory(case['returned']-2,2)==b'\x0f\x05'
                for name,value in zip(('rdi','rsi','rdx','r10','r8','r9'),case['arguments']):
                    assert reg(name)==value&MAX
                assert words(history['address'],1)==(1 if case['event']=='fork' else 3,)
                runtime_zero(False)
                if case['event']=='parent-exit':
                    assert len(seen)==len(cases) and physical.generation==3
                    assert words(owner+0x2000,1)==(1,),'child must remain live'
                    assert words(owner+56,1)==(1,),'parent must be active'
                    fs=owner+profile['filesystem_offset']
                    # The shared table contains exactly c/d, both naming one
                    # live inode with two links. Names outlive active/inactive
                    # slots until the outer owner's terminal release.
                    entries=[]
                    for index in range(profile['namespace_entries']['entry_count']):
                        identity,length=words(fs+64+16*index,2)
                        if identity:
                            entries.append((identity,memory(fs+profile['name_offset']+index*256,length).decode()))
                    assert sorted(name for _,name in entries)==['c','d'],entries
                    assert len({identity for identity,_ in entries})==1
                    objects=[fs+profile['object_offset']+i*128 for i in range(profile['object_count'])]
                    live=[address for address in objects if words(address,1)==(1,)]
                    assert len(live)==1 and words(live[0]+8,1)==(entries[0][0],)
                    assert words(live[0]+56,1)==(2,) and words(live[0]+24,1)==(0o100600,)
                    (output/'owner-before-terminal.bin').write_bytes(memory(owner,owner_bytes))
                    state['inactive_child_retained_at_parent_exit']=True
                    state['live_names_at_parent_exit']=entries
                state['events'].append(case)
                self.enabled=False
                return False
            except BaseException:
                return fail()

    class Finalized(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['finalize_entries']==1 and not state['finalized']
                assert words(history['address'],1)==(0,)
                runtime_zero()
                assert memory(owner,owner_bytes)==bytes(owner_bytes)
                (output/'owner-after-terminal.bin').write_bytes(memory(owner,owner_bytes))
                state['finalized']=True
                self.enabled=False
                return False
            except BaseException:
                return fail()

    class Finalize(gdb.Breakpoint):
        def stop(self):
            try:
                assert state.get('inactive_child_retained_at_parent_exit')
                assert len(seen)==len(cases) and len(state['events'])==3
                assert reg('cs')&3==0 and not reg('eflags')&0x200
                assert words(history['address'],1)==(3,),'history must survive until final release'
                runtime_zero(False)
                assert memory(owner,owner_bytes)==bytes(owner_bytes),'all banks/names/inodes/pages must retire first'
                state['finalize_entries']+=1
                assert state['finalize_entries']==1
                Finalized('*'+hex(words(reg('rsp'),1)[0]),internal=True)
                return False
            except BaseException:
                return fail()

    class Finished(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['finalized'] and not state['process_returned']
                assert reg('cs')&3==0
                runtime_zero()
                assert memory(owner,owner_bytes)==bytes(owner_bytes)
                state.update(result='pass',schema='topal.namespace-natural-inactive-cancellation/1',
                    process_returned=True,ordinary_returns=len(seen),source_actions=physical.generation,
                    owner_bytes_erased=owner_bytes,history_before_final_release=3,history_after_final_release=0,
                    source_and_user_state_injections=[],
                    scope='actual fork, child yield and parent exit; alias/description continuity before exit, both banks and entire owner erased before namespace finalizer, history reset only at final release and process controller returns')
                (output/'observation.json').write_text(json.dumps(state,indent=2)+'\n')
                return True
            except BaseException:
                return fail()

    class Enter(gdb.Breakpoint):
        def stop(self):
            try:
                for bound in binding['code']+binding['source_code']+binding['source_data']+[binding['controller_code']]:
                    assert hashlib.sha256(memory(bound['address'],bound['bytes'])).hexdigest()==bound['sha256']
                Finished('*'+hex(words(reg('rsp'),1)[0]),internal=True)
                self.enabled=False
                return False
            except BaseException:
                return fail()

    for case in cases:
        Returned(case)
        physical.watch(case)
    for case in events:
        Event(case)
        if case['event']!='fork':
            Forbidden('*'+hex(case['returned']),internal=True)
    finalizer=next(c for c in binding['code'] if c['symbol'].endswith('_finalize_v1'))
    Finalize('*'+hex(finalizer['address']),internal=True)
    Enter('*'+hex(binding['controller_code']['address']),internal=True)
