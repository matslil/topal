#!/usr/bin/env python3
"""Qualify real private process exchanges, live wait and whole-pair teardown."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import tempfile

LAB=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('children_gate',LAB/'children_gate.py')
CHILDREN=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(CHILDREN)
GATE=CHILDREN.GATE

def script(elf,debug,result,failed,entry,external,corrupt,terminate=False):
    setup=f'''
import gdb,json,traceback
from pathlib import Path
pending={{}}
records=[]
context=table=None
shared=None
local_yields=0
blocking_waits=0
second_bank_retained=False
external_exec=False
injected_yield=False
terminated_live=False
initial_entropy=None
entropy_advanced=False
registers=("rbx","rcx","rdx","rsi","rdi","rbp","rsp","r8","r9","r10","r11","r12","r13","r14","r15")
def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
def memory(address,size):return bytes(gdb.selected_inferior().read_memory(address,size))
def word(address):return int.from_bytes(memory(address,8),"little")
def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
def gp():return {{name:reg(name) for name in registers}}
def extended():return ([[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)],reg("mxcsr"),reg("fctrl"),reg("fstat"),reg("ftag"),[str(gdb.parse_and_eval("$st%d"%i)) for i in range(8)])
def private_table():return memory(table,320)+memory(table+1344,8192-1344)
def descriptions():return memory(table+320,1024)+memory(table+0x3d000,1024)
def history():return memory(context+184,40)+memory(context+264,8)
def capture():
    mask=0xffffffffff000
    root=word(context+24)
    backings=[(word(word(context+112)+0xf28)&mask,0x12000),(word(context+120),0x22000)]
    backings += [(word(root+offset)&mask,4096) for offset in (0x4ff0,0x4ff8)]
    leaves=[word(context+112),root+0x4000]
    return dict(gp=gp(),extended=extended(),fs=reg("fs_base"),backings=backings,
        private=[memory(a,n) for a,n in backings],table=private_table(),
        leaves=[[word(a+i)&~0x60 for i in range(0,4096,8)] for a in leaves],
        clear_tid=word(context+152),robust=word(context+160))
class Checked(gdb.Breakpoint):
    def stop(self):
        try:return self.inspect()
        except BaseException:
            Path({str(failed)!r}).write_text(traceback.format_exc())
            return True
class Rejected(Checked):
    def inspect(self):
        assert reg("rax")==0x12345679 and reg("r11")==0x12345678 and reg("eflags")&64==0
        assert memory(reg("rip"),2)==bytes.fromhex("75fe")
        Path({str(result)!r}).write_text(json.dumps(dict(result="expected-pair-restoration-rejection",
            snapshot_byte_injected=True,observed=reg("rax"),expected=reg("r11"),
            exact_user_comparison=True,execution="stopped before completion after independent rejection")))
        return True
class Returned(Checked):
    def __init__(self,address,pid,value,initial=False,exiting=False,kind="yield"):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.pid,self.value,self.initial,self.exiting,self.kind=pid,value,initial,exiting,kind
    def inspect(self):
        global shared,injected_yield,terminated_live,second_bank_retained
        self.enabled=False
        assert word(table+56)==self.pid and reg("rax")==self.value
        if self.initial:
            assert gp()==pending[1]["gp"] and extended()==pending[1]["extended"] and reg("fs_base")==pending[1]["fs"]
            if {external!r}:
                # Explicit test input: issue one extra native yield at the
                # child's clone-return SYSCALL site. Code bytes stay unchanged;
                # its eventual zero return is the original clone child result.
                gdb.execute("set $rax=24");gdb.execute("set $rip=$rcx-2")
                injected_yield=True
            return False
        target=pending.pop(self.pid)
        assert gp()==target["gp"] and extended()==target["extended"] and reg("fs_base")==target["fs"]
        assert history()==shared["history"]
        assert private_table()==target["table"]
        assert word(context+152)==target["clear_tid"] and word(context+160)==target["robust"]
        assert [[word(a+i)&~0x60 for i in range(0,4096,8)] for a in (word(context+112),word(context+24)+0x4000)]==target["leaves"]
        if {corrupt!r} and self.pid==1:
            assert word(0x5f2000)==0x12345679
            pattern=bytes.fromhex("48a100205f000000000049bb78563412000000004c39d875fe")
            code=memory(0x4f8000,0x8000);assert code.count(pattern)==3
            Rejected("*"+hex(0x4f8000+code.index(pattern)+len(pattern)-2),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
            return False
        assert [memory(a,n) for a,n in target["backings"]]==target["private"]
        if self.exiting:
            assert word(table+0x2000)==2
            assert memory(table+0x2020,0x3d000-0x2020)==bytes(0x3d000-0x2020)
            counts={{}}
            for i in range(16):
                value=word(table+64+16*i)
                if value:counts[value]=counts.get(value,0)+1
            for i in range(1,33):
                base=table+(320+64*(i-1) if i<=16 else 0x3d000+64*(i-17))
                assert word(base+8)==counts.get(i,0)
            if any(i>16 for i in counts):second_bank_retained=True
        else:
            assert descriptions()==shared["descriptions"]
            assert word(table+0x2000)==1
            assert memory(table+0x2100+184,40)==bytes(40)
            assert word(table+0x2100+264)==0
            assert memory(table+0x2a00,512)==bytes(512) and word(table+0x2070)==0
        records.append(dict(pid=self.pid,continuation=self.kind,native_return=self.value,
            registers_preserved=True,extended_state_preserved=True,tls_preserved=True,
            private_backings_preserved=True,private_table_preserved=True,
            mapping_permissions_preserved=True,thread_registrations_preserved=True,
            shared_history_preserved=True,description_references_preserved=True,
            retired_snapshot_erased=self.exiting))
        if {terminate!r} and self.pid==1 and self.kind=="fork" and self.value==8:
            assert self.pid==1 and not self.exiting and pending
            gdb.execute("set $rax=231");gdb.execute("set $rdi=0x23");gdb.execute("set $rip=$rcx-2")
            terminated_live=True
        return False
class Local(Checked):
    def __init__(self,address,before):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.before=before
    def inspect(self):
        global local_yields
        self.enabled=False
        assert word(table+56)!=1 and word(table+0x2050)==1 and reg("rax")==0
        assert capture()==self.before
        local_yields+=1
        return False
class Incoming(Checked):
    def inspect(self):
        global context,table,shared,blocking_waits,external_exec,initial_entropy,entropy_advanced
        number=reg("rax")&0xffffffff
        context=reg("k_gs_base");table=word(context+928)
        if not table or word(table)!=1:return False
        in_fixture=0x4f8000<=reg("rcx")<0x500000
        if not {external!r} and not in_fixture:return False
        state,pid=word(table+0x2000),word(table+56)
        if number in (56,57) and state==0:
            if number==56 and ((reg("rdi")&0xffffffff)&~0x1200000!=17 or reg("rsi")!=0):return False
            assert not pending and pid==1
            pending[1]=capture()
            initial_entropy=memory(context+192,32)
            child=word(table+0x2008) or 2
            pending[1].update(value=child,kind="fork")
            Returned(reg("rcx"),child,0,initial=True)
        elif number==24 and state==1:
            if pid!=1 and word(table+0x2050):
                Local(reg("rcx"),capture());return False
            pending[pid]=capture();pending[pid].update(value=0,kind="yield")
            target=word(table+0x2010) if pid==1 else 1
            saved=pending[target]
            shared=dict(history=history(),descriptions=descriptions())
            if {corrupt!r} and pid!=1:
                address=table+0x7000+(0x5f2000-0x5e5000)
                assert memory(address,1)==bytes([0x78])
                gdb.selected_inferior().write_memory(address,bytes([0x79]))
            Returned(saved["gp"]["rcx"],target,saved["value"],kind=saved["kind"])
        elif number==61 and state==1 and pid==1 and reg("rdx")==0:
            assert reg("rdi")&0xffffffff in (0xffffffff,0,word(table+0x2010)) and reg("r10")==0
            pending[1]=capture();pending[1]["gp"]["rcx"]-=2
            pending[1].update(value=61,kind="wait-retry")
            target=word(table+0x2010);saved=pending[target]
            shared=dict(history=history(),descriptions=descriptions())
            blocking_waits+=1
            Returned(saved["gp"]["rcx"],target,saved["value"],kind=saved["kind"])
        elif number==59 and state==1 and pid!=1:external_exec=True
        elif number in (60,231) and state==1 and pid!=1:
            entropy_advanced=memory(context+192,32)!=initial_entropy
            saved=pending[1]
            shared=dict(history=history(),descriptions=descriptions())
            Returned(saved["gp"]["rcx"],1,saved["value"],exiting=True,kind=saved["kind"])
        return False
class Released(Checked):
    def inspect(self):
        assert records
        assert memory(table,0x43000)==bytes(0x43000)
        assert word(context+152)==0 and word(context+160)==0
        assert all(word(context+i)==0 for i in (952,960,968))
        if {terminate!r}:assert terminated_live and pending and second_bank_retained
        else:assert not pending and blocking_waits=={1 if external else 2}
        if {external!r}:assert injected_yield and external_exec and entropy_advanced
        if not {external!r} and not {terminate!r}:assert second_bank_retained and local_yields==2
        Path({str(result)!r}).write_text(json.dumps(dict(result="pass",returns=records,
            local_yields_while_parent_waits=local_yields,blocking_waits=blocking_waits,
            second_bank_retained=second_bank_retained,external_exec_after_handoff=external_exec,
            entropy_history_advanced_across_child_exec=entropy_advanced,
            explicit_child_yield_injected=injected_yield,explicit_live_parent_exit_injected=terminated_live,
            terminal_storage_erased=True,terminal_thread_registrations_erased=True,
            terminal_library_bindings_erased=True),indent=2,sort_keys=True))
        return True
Incoming("*{entry}",type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Released("*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
'''
    return f'set pagination off\nset confirm off\nfile {elf}\ntarget remote {debug}\npython\n{setup}\nend\ncontinue\ndetach\n'

def verify(external_image,timeout=60,write_evidence=False):
    original_script,original_verify=CHILDREN.script,GATE.verify
    def observed(root,values,external=None,corrupt=False,terminate=False):
        CHILDREN.script=lambda *args:script(*args,terminate=terminate)
        if terminate:
            def terminated_runtime(*_args,**_kwargs):
                serial=GATE.observe_gate(values[0],values[1],root/'terminal-serial.log',root/'terminal-qemu.log',timeout,GATE.PROCESS_COMMAND)
                assert b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=23' in serial
                assert b'runtime-clean\n' not in serial
                return GATE.evidence_record(GATE.verify_host(),*values,serial)
            GATE.verify=terminated_runtime
        try:return CHILDREN.observe(root,values,external,corrupt,timeout)
        finally:GATE.verify=original_verify
    try:
        with tempfile.TemporaryDirectory(prefix='topal-pair-',dir='/tmp') as temporary:
            root=Path(temporary)
            clean=GATE.build_image(root/'clean-artifact')
            external=GATE.build_image(root/'external-artifact',external_image)
            # The shared observer recognizes its existing precise failure hook.
            record=dict(schema='topal-static-process-pair-qemu/1',harness_sha256=GATE.digest(Path(__file__)),
                fixture_sha256=GATE.digest(LAB/'static_pair_checks.py'),
                clean=observed(root/'clean',clean),
                external=observed(root/'external',external,external_image),
                termination=observed(root/'termination',clean,terminate=True),
                counterexample=observed(root/'corrupt',clean,corrupt=True))
            assert len({record[k]['linked_kernel_sha256'] for k in ('clean','external','termination','counterexample')})==1
        if write_evidence:(LAB/'results/topal-static-process-pair-gate.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
        print('PASS private pair handoff, live wait, both-bank lifetime, exec, teardown and independent rejection',flush=True)
        return record
    finally:CHILDREN.script,GATE.verify=original_script,original_verify

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external-static-user-image',type=Path,required=True)
    parser.add_argument('--timeout',type=float,default=60)
    parser.add_argument('--write-evidence',action='store_true')
    args=parser.parse_args();verify(args.external_static_user_image,args.timeout,args.write_evidence)
