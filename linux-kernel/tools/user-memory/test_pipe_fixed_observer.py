"""Exercise actual observer ordering and refusal checks with host-side models.

TOPAL-SYSTEMS-USER-EXTENT-001. Mock memory/register changes below simulate
observations; they neither access a guest nor prove native permissions/owners.
"""
from pathlib import Path
import runpy
import unittest


class PipeFixedObserverControls(unittest.TestCase):
    def namespace(self):
        class Breakpoint:
            def __init__(self,*args,**kwargs): self.enabled = True
        names = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15']
        registers = dict.fromkeys(names,0)
        registers.update(rsp=0x9000,r8=0x2000,r9=8,rax=0x44332211,eflags=0)
        data,translations = {},[]
        def store(pointer,payload):
            data.update((pointer+i,value) for i,value in enumerate(payload))
        def memory(pointer,size): return bytes(data.get(pointer+i,0) for i in range(size))
        def mapping(pointer,size,user,writable):
            translations.append((pointer,size,user,writable))
            return [dict(pointer=pointer,size=size,physical=pointer,path=[[0x3000,12,7]],
                         user=user,writable=True,leaf_shift=12)]
        store(0x7000+928,(0x10000).to_bytes(8,'little'))
        pair = dict(name='pipe-pair-output',call_offset=0,admitted_bytes=8,success_edge=[14,20],
            access_instructions=[dict(offset=30,bytes='49898000000000',action='write',
                width=8,candidate_offset=0,base_register=8),dict(offset=37,bytes='4d8b9800000000',
                action='read',width=8,candidate_offset=0,base_register=8)])
        count = dict(name='pipe-readable-count-output',call_offset=100,admitted_bytes=4,success_edge=[114,120],
            access_instructions=[dict(offset=130,bytes='418900',action='write',
                width=4,candidate_offset=0,base_register=8)])
        ns = dict(Checked=Breakpoint,context=0x7000,copy_register_names=names,
            copy_preserved=[2,3,5,6,7,8,9,12,13,14,15],reg=lambda name:registers[name],
            memory=memory,word=lambda pointer:int.from_bytes(memory(pointer,8),'little'),
            copy_mode=lambda before=None:[0x10000,0x3000,0],copy_range=mapping,
            selected_extent_observers=lambda:['pipe-fixed'],observed={},
            extent_inventory=dict(address=0,kernel_sha256='model',pipe_fixed_output_audit_sha256='model',
                pipe_fixed_output_audit=dict(passed=True,complete=False,kernel_sha256='model',cases=[pair,count])))
        path = Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_pipe_fixed_observer.py'
        exec(runpy.run_path(str(path))['PROGRAM'],ns)
        ns['install_pipe_fixed_observation']()
        ns.update(model_registers=registers,model_store=store,model_translations=translations)
        return ns

    def entered(self,ns):
        call = dict(number=293,args=[0x2000,0x800,0],phase_before=8)
        ns['pipe_fixed_syscall_entered'](call)
        ns['pipe_fixed_calls']['pipe-pair-output'].observe()
        return call

    def admitted(self,ns):
        call = self.entered(ns)
        ns['pipe_fixed_results']['pipe-pair-output'].observe()
        return call

    def complete_pair(self,ns):
        regs = ns['model_registers']
        ns['pipe_fixed_points'][30].observe()
        ns['model_store'](0x2000,regs['rax'].to_bytes(8,'little'))
        ns['pipe_fixed_points'][37].observe()
        regs['r11'] = regs['rax']
        ns['pipe_fixed_points'][44].observe()

    def test_shared_pc_finishes_the_store_before_starting_readback(self):
        ns = self.namespace()
        call = self.admitted(ns)
        self.assertEqual(len(ns['pipe_fixed_points'][37].finishes),1)
        self.assertEqual(len(ns['pipe_fixed_points'][37].starts),1)
        self.complete_pair(ns)
        ns['pipe_fixed_syscall_returned'](dict(**call,result=0,phase_after=8))
        self.assertIsNone(ns['pipe_fixed_scope'])
        self.assertEqual([r['action'] for r in ns['pipe_fixed_records'][0]['accesses']],['write','read'])

    def test_readback_cannot_complete_before_the_store_started(self):
        ns = self.namespace()
        self.admitted(ns)
        with self.assertRaises(AssertionError): ns['pipe_fixed_points'][37].observe()

    def test_clobbered_helper_candidate_is_rejected_even_on_refusal(self):
        for flags in [0,1]:
            ns = self.namespace()
            self.entered(ns)
            ns['model_registers'].update(r8=0x2001,eflags=flags)
            with self.subTest(flags=flags),self.assertRaises(AssertionError):
                ns['pipe_fixed_results']['pipe-pair-output'].observe()

    def test_refusal_never_walks_or_reads_the_candidate(self):
        ns = self.namespace()
        call = self.entered(ns)
        ns['model_registers']['eflags'] = 1
        ns['pipe_fixed_results']['pipe-pair-output'].observe()
        self.assertEqual(ns['model_translations'],[])
        ns['pipe_fixed_syscall_returned'](dict(**call,result=-14,phase_after=8))
        self.assertEqual(len(ns['pipe_fixed_refusals']),1)
        with self.assertRaises(AssertionError): ns['pipe_fixed_points'][30].observe()

    def test_refusal_requires_efault_and_preserved_phase(self):
        for result,phase in [(-22,8),(-14,9)]:
            ns = self.namespace()
            call = self.entered(ns)
            ns['model_registers']['eflags'] = 1
            ns['pipe_fixed_results']['pipe-pair-output'].observe()
            with self.subTest(result=result,phase=phase),self.assertRaises(AssertionError):
                ns['pipe_fixed_syscall_returned'](dict(**call,result=result,phase_after=phase))

    def test_a_changed_boundary_mapping_cannot_pass(self):
        ns = self.namespace()
        self.admitted(ns)
        original = ns['copy_range']
        def changed(pointer,size,user,writable):
            rows = original(pointer,size,user,writable)
            if user: rows[0]['physical'] += 4096
            return rows
        ns['copy_range'] = changed
        with self.assertRaises(AssertionError): ns['pipe_fixed_points'][30].observe()

    def test_changed_payload_or_readback_register_cannot_pass(self):
        for changed in ['payload','register']:
            ns = self.namespace()
            self.admitted(ns)
            ns['pipe_fixed_points'][30].observe()
            if changed == 'payload':
                ns['model_store'](0x2000,b'changed!')
                with self.assertRaises(AssertionError): ns['pipe_fixed_points'][37].observe()
            else:
                ns['model_store'](0x2000,ns['model_registers']['rax'].to_bytes(8,'little'))
                ns['pipe_fixed_points'][37].observe()
                ns['model_registers']['r11'] = 1
                with self.assertRaises(AssertionError): ns['pipe_fixed_points'][44].observe()

    def test_unrun_observations_cannot_be_published(self):
        ns = self.namespace()
        with self.assertRaises(AssertionError): ns['pipe_fixed_observation_complete']()


if __name__ == '__main__':
    unittest.main()
