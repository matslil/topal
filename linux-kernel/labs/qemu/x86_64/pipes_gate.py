#!/usr/bin/env python3
"""Observe actual FIFO bytes, exact waits, native state and full reclamation."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import tempfile

LAB=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('children_gate',LAB/'children_gate.py')
CHILDREN=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(CHILDREN)
GATE=CHILDREN.GATE

def fatal_point(elf):
    lines=subprocess.check_output(['gdb','-q','-batch','-ex',f'file {elf}',
        '-ex','disassemble /r topal_x86_systems_static_image_process_enter'],text=True).splitlines()
    points=[int(re.search(r'0x([0-9a-f]+)\s+<',line)[1],16)
            for i,line in enumerate(lines[:-1]) if re.search(r'\bcli\s*$',line)
            and re.search(r'\bhlt\s*$',lines[i+1])]
    if len(points)!=1:raise GATE.GateError('cannot identify the unique private invariant-failure entry')
    return points[0]

def script(elf,debug,result,failed,entry,external,corrupt,terminate=False):
    fatal=fatal_point(elf)
    setup=f'''
import gdb,json,traceback
from pathlib import Path
context=table=None
pending={{}}
streams={{}}
locations={{}}
resumptions=[]
transfers=[]
created=[]
reclaimed=[]
blocks=[]
shared=None
unrelated_preserved=0
local_yields=0
injected=None
terminated=False
entropy_at_fork=None
entropy_advanced=False
registers=("rbx","rcx","rdx","rsi","rdi","rbp","rsp","r8","r9","r10","r11","r12","r13","r14","r15")
def reg(name):return int(gdb.parse_and_eval("$"+name))&((1<<64)-1)
def memory(address,size):return bytes(gdb.selected_inferior().read_memory(address,size)) if size else b""
def word(address):return int.from_bytes(memory(address,8),"little")
def symbol(name):return int(gdb.parse_and_eval("&'"+name+"'"))
def gp():return {{name:reg(name) for name in registers}}
def extended():return ([[int(gdb.parse_and_eval("$xmm%d.v2_int64[%d]"%(i,j))) for j in range(2)] for i in range(16)],reg("mxcsr"),reg("fctrl"),reg("fstat"),reg("ftag"),[str(gdb.parse_and_eval("$st%d"%i)) for i in range(8)])
def history():return memory(context+184,40)+memory(context+264,8)
def capture():
    root=word(context+24);mask=0xffffffffff000
    backings=[(word(word(context+112)+0xf28)&mask,0x12000),(word(context+120),0x22000)]
    backings += [(word(root+offset)&mask,4096) for offset in (0x4ff0,0x4ff8)]
    return dict(gp=gp(),extended=extended(),fs=reg("fs_base"),backings=backings,
        private=[memory(a,n) for a,n in backings],
        table=memory(table,320)+memory(table+1344,8192-1344),
        leaves=[[word(a+i)&~0x60 for i in range(0,4096,8)] for a in (word(context+112),root+0x4000)],
        clear_tid=word(context+152),robust=word(context+160))
def verify_private(saved):
    assert gp()==saved["gp"] and extended()==saved["extended"] and reg("fs_base")==saved["fs"]
    assert [memory(a,n) for a,n in saved["backings"]]==saved["private"]
    assert memory(table,320)+memory(table+1344,8192-1344)==saved["table"]
    assert [[word(a+i)&~0x60 for i in range(0,4096,8)] for a in (word(context+112),word(context+24)+0x4000)]==saved["leaves"]
    assert word(context+152)==saved["clear_tid"] and word(context+160)==saved["robust"]
def descriptor(identity):
    assert 1<=identity<=32
    return table+(320+(identity-1)*64 if identity<=16 else 0x3d000+(identity-17)*64)
def endpoint(fd):
    fd &= 0xffffffff
    if fd>=16:return None
    identity=word(table+64+16*fd)
    if not identity:return None
    address=descriptor(identity);kind=word(address)
    if kind not in (5,6):return None
    return address,kind,word(address+32)
def objects():
    return {{word(table+0x3e000+i*128+8):(table+0x3e000+i*128,table+0x3f000+i*4096)
            for i in range(4) if word(table+0x3e000+i*128)==1}}
def sync():
    live=objects()
    for generation in list(streams):
        if generation not in live:
            address,buffer=locations[generation]
            assert memory(address,128)==bytes(128) and memory(buffer,4096)==bytes(4096)
            reclaimed.append(generation);del streams[generation]
        else:
            address,buffer=live[generation];head,count=word(address+32),word(address+40)
            assert head<4096 and count==len(streams[generation])<=4096
            ring=memory(buffer,4096)
            assert bytes(ring[(head+i)%4096] for i in range(count))==bytes(streams[generation])
            occupied={{(head+i)%4096 for i in range(count)}}
            assert all(value==0 for i,value in enumerate(ring) if i not in occupied)
    counts={{}}
    bases=[table]
    if word(table+0x2000)==1:bases.append(table+0x3000)
    for base in bases:
        for i in range(16):
            identity=word(base+64+16*i)
            if identity:counts[identity]=counts.get(identity,0)+1
    for identity in range(1,33):
        address=descriptor(identity);kind=word(address)
        assert word(address+8)==counts.get(identity,0)
        if not kind:assert memory(address,64)==bytes(64)
        elif kind in (5,6):
            generation=word(address+32);assert generation in live
            assert word(live[generation][0]+(16 if kind==5 else 24))==identity
class Checked(gdb.Breakpoint):
    def stop(self):
        try:return self.inspect()
        except BaseException:
            Path({str(failed)!r}).write_text(traceback.format_exc())
            return True
class Created(Checked):
    def __init__(self,address,pointer):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True);self.pointer=pointer
    def inspect(self):
        self.enabled=False
        if reg("rax")!=0:return False
        values=memory(self.pointer,8);read,write=[int.from_bytes(values[i:i+4],"little") for i in (0,4)]
        r,w=endpoint(read),endpoint(write)
        assert r and w and r[1]==5 and w[1]==6 and r[2]==w[2]
        generation=r[2];assert generation not in created
        live=objects();address,buffer=live[generation]
        assert word(address+32)==0 and word(address+40)==0 and memory(buffer,4096)==bytes(4096)
        streams[generation]=bytearray();locations[generation]=(address,buffer);created.append(generation)
        sync();return False
class DataReturned(Checked):
    def __init__(self,address,operation,generation,pointer,count,source,before,unrelated):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.operation,self.generation,self.pointer,self.count=operation,generation,pointer,count
        self.source,self.before,self.unrelated=source,before,unrelated
    def inspect(self):
        global unrelated_preserved
        self.enabled=False
        actual=reg("rax");actual=actual-(1<<64) if actual>>63 else actual
        queue=streams[self.generation]
        if actual>0:
            assert actual<=self.count
            if self.operation==1:
                assert self.source is not None and actual<=len(self.source)
                queue.extend(self.source[:actual])
            else:
                assert actual<=len(queue) and memory(self.pointer,actual)==bytes(queue[:actual])
                del queue[:actual]
        elif actual<0:
            address=objects()[self.generation][0]
            assert memory(address,128)==self.before
        elif self.operation==0 and self.count:
            address=objects()[self.generation][0]
            assert not queue and word(address+24)==0
        transfers.append(dict(operation="write" if self.operation else "read",generation=self.generation,
            requested=self.count,actual=actual,ordered_bytes_verified=actual>0))
        if self.unrelated:
            assert (word(table+0x2060),word(table+0x2078))==self.unrelated
            unrelated_preserved+=1
        sync();return False
class Closed(Checked):
    def inspect(self):self.enabled=False;sync();return False
class Handoff(Checked):
    def __init__(self,address,pid,value,initial=False,exiting=False):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.pid,self.value,self.initial,self.exiting=pid,value,initial,exiting
    def inspect(self):
        global terminated
        self.enabled=False
        assert word(table+56)==self.pid and reg("rax")==self.value
        if self.initial:
            assert gp()==pending[1]["gp"] and extended()==pending[1]["extended"] and reg("fs_base")==pending[1]["fs"]
            sync();return False
        saved=pending.pop(self.pid);verify_private(saved);assert history()==shared
        assert word(table+0x2070)==0 and memory(table+0x2a00,512)==bytes(512)
        if self.exiting:assert memory(table+0x2020,0x3d000-0x2020)==bytes(0x3d000-0x2020)
        resumptions.append(dict(pid=self.pid,continuation=saved["kind"],native_return=self.value,
            private_backings_preserved=True,private_table_preserved=True,mapping_permissions_preserved=True,
            registers_preserved=True,tls_preserved=True,extended_state_preserved=True,
            thread_registrations_preserved=True,shared_history_preserved=True,
            retired_snapshot_erased=self.exiting))
        sync()
        if {terminate!r} and self.pid==1 and blocks and blocks[0]["operation"]=="read":
            assert word(table+0x2060)==2 and word(table+0x2078)!=0 and pending
            gdb.execute("set $rax=231");gdb.execute("set $rdi=0x23");gdb.execute("set $rip=$rcx-2")
            terminated=True
        return False
class Local(Checked):
    def __init__(self,address,saved,condition):
        super().__init__("*"+hex(address),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        self.saved,self.expected_condition=saved,condition
    def inspect(self):
        global local_yields
        self.enabled=False;assert reg("rax")==0;verify_private(self.saved)
        assert (word(table+0x2060),word(table+0x2078))==self.expected_condition
        local_yields+=1;sync();return False
class Fatal(Checked):
    def inspect(self):
        assert injected and memory(reg("rip"),4)==bytes.fromhex("faf4ebfd")
        assert word(table+56)==1 and word(table+0x2060)==2
        assert word(table+0x2078)==injected["wrong_generation"]
        fd=word(table+0x2100+312)&0xffffffff
        identity=word(table+0x3000+64+16*fd)
        assert word(descriptor(identity)+32)==injected["actual_generation"]
        Path({str(result)!r}).write_text(json.dumps(dict(result="expected-fifo-wait-identity-rejection",
            exact_invariant_failure_entry=True,inactive_endpoint_generation_verified=True,
            **injected,execution="stopped before completion after native identity rejection")))
        return True
class Incoming(Checked):
    def inspect(self):
        global context,table,shared,injected,entropy_at_fork,entropy_advanced
        context=reg("k_gs_base");table=word(context+928)
        if not table or word(table)!=1:return False
        if not {external!r} and not 0x500000<=reg("rcx")<0x508000:return False
        sync()
        number,pid,state=reg("rax")&0xffffffff,word(table+56),word(table+0x2000)
        if number in (22,293):
            Created(reg("rcx"),reg("rdi"));return False
        if number in (56,57) and state==0:
            if number==56 and ((reg("rdi")&0xffffffff)&~0x1200000!=17 or reg("rsi")!=0):return False
            assert not pending and pid==1
            child=word(table+0x2008) or 2
            pending[1]=capture();pending[1].update(value=child,kind="fork")
            entropy_at_fork=memory(context+192,32)
            Handoff(reg("rcx"),child,0,initial=True);return False
        if number in (0,1):
            item=endpoint(reg("rdi"))
            if not item or item[1]!=(5 if number==0 else 6):return False
            address,buffer=objects()[item[2]];queue=streams[item[2]];count=reg("rdx");pointer=reg("rsi")
            valid_range=pointer+count<=1<<47
            waiting=bool(valid_range and count and not word(item[0]+16)&0x800 and (
                (number==0 and not queue and word(address+24)) or
                (number==1 and word(address+16) and (len(queue)==4096 or (count<=4096 and 4096-len(queue)<count)))))
            if waiting:
                pending[pid]=capture();pending[pid]["gp"]["rcx"]-=2
                pending[pid].update(value=number,kind="pipe-read-retry" if number==0 else "pipe-write-retry")
                blocks.append(dict(pid=pid,operation="read" if number==0 else "write",generation=item[2],
                    native_candidate=pointer,requested=count,buffered=len(queue)))
                other=word(table+0x2010) if pid==1 else 1
                saved=pending[other];shared=history()
                Handoff(saved["gp"]["rcx"],other,saved["value"]);return False
            unrelated=None
            if number==1 and pid==1 and state==1 and word(table+0x2060)==2 and word(table+0x2078)!=item[2]:
                unrelated=(word(table+0x2060),word(table+0x2078))
                if {corrupt!r}:
                    injected=dict(actual_generation=unrelated[1],wrong_generation=item[2],unrelated_write_injected=True)
                    gdb.selected_inferior().write_memory(table+0x2078,item[2].to_bytes(8,"little"))
            source=None
            if number==1:
                try:source=memory(pointer,min(count,4096))
                except gdb.MemoryError:pass
            DataReturned(reg("rcx"),number,item[2],pointer,count,source,memory(address,128),unrelated)
        elif number==3:
            if endpoint(reg("rdi")):Closed("*"+hex(reg("rcx")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
        elif number==24 and state==1:
            other_kind=word(table+(0x2060 if pid==1 else 0x2050))
            if other_kind:
                Local(reg("rcx"),capture(),(word(table+0x2060),word(table+0x2078)));return False
            pending[pid]=capture();pending[pid].update(value=0,kind="yield")
            other=word(table+0x2010) if pid==1 else 1;saved=pending[other];shared=history()
            Handoff(saved["gp"]["rcx"],other,saved["value"])
        elif number==61 and state==1 and pid==1 and reg("rdx")==0:
            pending[1]=capture();pending[1]["gp"]["rcx"]-=2;pending[1].update(value=61,kind="wait-retry")
            child=word(table+0x2010);saved=pending[child];shared=history()
            Handoff(saved["gp"]["rcx"],child,saved["value"])
        elif number in (60,231) and state==1 and pid!=1:
            entropy_advanced |= memory(context+192,32)!=entropy_at_fork
            saved=pending[1];shared=history()
            Handoff(saved["gp"]["rcx"],1,saved["value"],exiting=True)
        return False
class Released(Checked):
    def inspect(self):
        sync();assert created and not streams and memory(table,0x43000)==bytes(0x43000)
        assert word(context+152)==0 and word(context+160)==0
        assert all(word(context+i)==0 for i in (952,960,968))
        if {terminate!r}:assert terminated and pending
        else:assert not pending and any(b["operation"]=="read" for b in blocks) and any(b["operation"]=="write" for b in blocks)
        if not {external!r} and not {terminate!r}:
            assert unrelated_preserved==1 and local_yields==2
            assert next(b for b in blocks if b["operation"]=="read")["native_candidate"]==0x619000
            assert next(b for b in blocks if b["operation"]=="write")["native_candidate"]==0x619000
        if {external!r}:assert entropy_advanced
        Path({str(result)!r}).write_text(json.dumps(dict(result="pass",created_generations=created,
            reclaimed_generations=reclaimed,transfers=transfers,blocks=blocks,resumptions=resumptions,
            unrelated_condition_preserved=unrelated_preserved,local_yields_with_ineligible_counterpart=local_yields,
            explicit_blocked_child_parent_exit_injected=terminated,entropy_history_advanced_across_exec=entropy_advanced,
            complete_ring_contents_and_unused_zeros_verified=True,all_description_reference_counts_verified=True,
            terminal_storage_erased=True,terminal_thread_registrations_erased=True,
            terminal_library_bindings_erased=True),indent=2,sort_keys=True))
        return True
Incoming("*{entry}",type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
Released("*"+hex(symbol("topal_x86_systems_static_image_process_restore_entry")),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
if {corrupt!r}:Fatal("*{fatal}",type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
'''
    return f'set pagination off\nset confirm off\nfile {elf}\ntarget remote {debug}\npython\n{setup}\nend\ncontinue\ndetach\n'

def verify(external_image,timeout=90,write_evidence=False):
    original_script,original_verify=CHILDREN.script,GATE.verify
    def observed(root,values,external=None,corrupt=False,terminate=False):
        CHILDREN.script=lambda *args:script(*args,terminate=terminate)
        if terminate:
            def terminated_runtime(*_args,**_kwargs):
                serial=GATE.observe_gate(values[0],values[1],root/'terminal-serial.log',root/'terminal-qemu.log',timeout,GATE.PROCESS_COMMAND)
                assert b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=23' in serial and b'runtime-clean\n' not in serial
                return GATE.evidence_record(GATE.verify_host(),*values,serial)
            GATE.verify=terminated_runtime
        try:return CHILDREN.observe(root,values,external,corrupt,timeout,
            workload='pipe-substitution-large' if external else 'true',
            rejection='independent FIFO wait identity oracle rejected the injected condition')
        finally:GATE.verify=original_verify
    try:
        with tempfile.TemporaryDirectory(prefix='topal-pipes-',dir='/tmp') as temporary:
            root=Path(temporary);clean=GATE.build_image(root/'clean-artifact')
            external=GATE.build_image(root/'external-artifact',external_image)
            proof=dict(schema='topal-static-process-pipes-qemu/1',harness_sha256=GATE.digest(Path(__file__)),
                fixture_sha256=GATE.digest(LAB/'static_pipe_checks.py'),
                clean=observed(root/'clean',clean),external=observed(root/'external',external,external_image),
                termination=observed(root/'termination',clean,terminate=True),
                counterexample=observed(root/'corrupt',clean,corrupt=True))
            assert len({proof[k]['linked_kernel_sha256'] for k in ('clean','external','termination','counterexample')})==1
        if write_evidence:(LAB/'results/topal-static-process-pipes-gate.json').write_text(json.dumps(proof,indent=2,sort_keys=True)+'\n')
        print('PASS independent FIFO bytes, exact readiness, private/native state, blocked-owner teardown and wrong-generation rejection',flush=True)
        return proof
    finally:CHILDREN.script,GATE.verify=original_script,original_verify

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external-static-user-image',type=Path,required=True)
    parser.add_argument('--timeout',type=float,default=90)
    parser.add_argument('--write-evidence',action='store_true')
    args=parser.parse_args();verify(args.external_static_user_image,args.timeout,args.write_evidence)
