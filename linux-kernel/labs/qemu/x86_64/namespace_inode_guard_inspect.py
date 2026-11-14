"""Refuse inconsistent or overlapping inode-mode admission after real path lookup."""
import hashlib
import json
from pathlib import Path
import struct
import traceback

import gdb

from namespace_source_physical import REGISTERS,memory,reg,vectors,words


def install(config):
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest()==config['inspector_sha256']
    output=Path(config['output'])
    artifact=config['artifact']
    binding=artifact['inode_mode_runtime']
    assert binding['schema']=='topal.inode-region-invocation.x86_64/2'
    owner=binding['controller_owner']['address']
    size=binding['controller_owner']['bytes']
    mode=config['mode']
    target_index=next(i for i,c in enumerate(config['cases']) if c['number']==90)
    target=config['cases'][target_index]
    seen=[]
    state=dict(active=False,injected=False,source_entries=0,metadata_entries=0,metadata_returned=False)
    ns=artifact['namespace_runtime']['invocation']
    controls={name:int(gdb.parse_and_eval("&'topal_x86_systems_"+name+"_control'"))
              for name in ('file_position','ram_page','namespace','inode_mode')}

    def fail():
        (output/'failed.txt').write_text(traceback.format_exc())
        return True

    def runtime():
        return {s['symbol']:memory(s['address'],s['bytes']).hex() for s in binding['storage']+ns['storage']}

    class Returned(gdb.Breakpoint):
        def __init__(self,case):
            self.case=case
            super().__init__('*'+hex(case['returned']),internal=True)

        def stop(self):
            try:
                assert self.case is not target,'refused inode controller resumed user code'
                assert reg('cs')&3==3 and reg('rax')==self.case['expected']&((1<<64)-1)
                seen.append(self.case)
                self.enabled=False
                return False
            except BaseException:
                return fail()

    class User(gdb.Breakpoint):
        def stop(self):
            try:
                assert len(seen)==target_index and reg('cs')&3==3 and reg('rax')==90
                assert memory(target['returned']-2,2)==b'\x0f\x05'
                for bound in binding['code']+binding['source_code']+binding['source_data']+[binding['controller_code']]:
                    assert hashlib.sha256(memory(bound['address'],bound['bytes'])).hexdigest()==bound['sha256']
                state['active']=True
                self.enabled=False
                return False
            except BaseException:
                return fail()

    class Apply(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['active'] and not state['injected']
                assert reg('cs')&3==0 and reg('rdi')==owner and reg('rdx')==0
                record=reg('rsi')
                assert 0<=record-owner-binding['file_object_offset']<4*128
                assert (record-owner-binding['file_object_offset'])%128==0
                assert words(record+56,1)==(2,)
                if mode=='bad-link-count':
                    gdb.selected_inferior().write_memory(record+56,struct.pack('<Q',3))
                else:
                    assert mode=='overlap-namespace'
                    gdb.selected_inferior().write_memory(controls['namespace']+24,struct.pack('<Q',1))
                state['injected']=True
                state['bad_owner']=memory(owner,size)
                state['runtime']=runtime()
                state['controls']={name:memory(address,64).hex() for name,address in controls.items()}
                return False
            except BaseException:
                return fail()

    class Metadata(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['injected'] and mode=='bad-link-count'
                state['metadata_entries']+=1
                assert state['metadata_entries']==1
                assert reg('rdi')==owner+binding['namespace_admission']['metadata_offset']
                stack=reg('rsp')
                saved=[reg(name) for name in REGISTERS]
                simd=vectors()
                state['metadata_stack']=stack

                class MetadataReturned(gdb.Breakpoint):
                    def stop(self):
                        try:
                            self.enabled=False
                            assert reg('rax')==0 and reg('rsp')==stack+8
                            assert [reg(name) for name in REGISTERS]==saved and vectors()==simd
                            assert memory(stack-64,64)==bytes(64)
                            assert memory(owner,size)==state['bad_owner']
                            state['metadata_returned']=True
                            return False
                        except BaseException:
                            return fail()

                MetadataReturned('*'+hex(words(stack,1)[0]),internal=True)
                return False
            except BaseException:
                return fail()

    class Source(gdb.Breakpoint):
        def stop(self):
            state['source_entries']+=1
            (output/'failed.txt').write_text('invalid/overlapping inode admission entered ordinary source')
            return True

    class Halted(gdb.Breakpoint):
        def stop(self):
            try:
                assert state['injected'] and state['source_entries']==0
                assert reg('cs')&3==0 and not reg('eflags')&0x200
                assert memory(owner,size)==state['bad_owner']
                assert runtime()==state['runtime']
                assert {name:memory(address,64).hex() for name,address in controls.items()}==state['controls']
                assert state['metadata_entries']==int(mode=='bad-link-count')
                if mode=='bad-link-count':
                    assert state['metadata_returned']
                    assert memory(state['metadata_stack']-64,72)==bytes(72)
                (output/'bad-owner-before.bin').write_bytes(state['bad_owner'])
                (output/'owner-at-refusal.bin').write_bytes(memory(owner,size))
                record=dict(schema='topal.inode-namespace-pre-entry-refusal/1',result='pass',mode=mode,
                    preparation_returns=seen,case=target,source_entries=0,
                    metadata_entries=state['metadata_entries'],metadata_returned_false=state['metadata_returned'],
                    actual_owner_and_all_runtime_preserved=True,
                    popped_validation_spills_and_return_erased=72 if mode=='bad-link-count' else None,
                    injected_fault=('owned inode claims three names where the actual table has two'
                        if mode=='bad-link-count' else 'namespace loan control marked active after real path admission'),
                    scope='real child chmod path; test-only privileged fault after normal lookup, no source entry or owner/user resumption')
                (output/'observation.json').write_text(json.dumps(record,indent=2)+'\n')
                return True
            except BaseException:
                return fail()

    for case in config['cases'][:target_index+1]:
        Returned(case)
    User('*'+hex(target['returned']-2),internal=True)
    Apply('*'+hex(next(c['address'] for c in binding['code'] if c['symbol'].endswith('_apply_v1'))),internal=True)
    Metadata('*'+hex(next(c['address'] for c in binding['code'] if c['symbol'].endswith('_namespace_check_v1'))),internal=True)
    Source("*'"+artifact['inode_mode_library']['native_export']+"'",internal=True)
    abort=next(c['address'] for c in binding['code'] if c['symbol']=='topal_kernel_region_frame_inode_metadata_abort_v1')
    Halted('*'+hex(abort),internal=True)
