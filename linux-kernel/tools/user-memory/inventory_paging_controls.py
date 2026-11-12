"""Inventory selected privileged controls in saved linked x86-64 listings.

TOPAL-SYSTEMS-USER-EXTENT-001. Complete section byte coverage is not a control
flow proof. Alternate instruction starts, indirect/exception entries, runtime
code and page-table stores retain separate obligations. No guest is executed.
"""
import argparse
from bisect import bisect_right
from collections import Counter
import json
from pathlib import Path
import re

from inspect_call_inventory import Elf, digest


PREFIXES={0x26,0x2e,0x36,0x3e,0x64,0x65,0x66,0x67,0xf0,0xf2,0xf3}
CONTROL_NAMES={
    'wrmsr':'msr-write','rdmsr':'msr-read','invlpg':'page-invalidation',
    'invpcid':'context-invalidation','invlpga':'address-space-invalidation',
    'invlpgb':'broadcast-invalidation','tlbsync':'invalidation-synchronization',
    'wrpkru':'protection-key-write','xsetbv':'extended-control-write',
    'lmsw':'machine-status-write','clts':'task-switched-clear',
    'lgdt':'descriptor-table-load','lidt':'interrupt-table-load',
    'lldt':'local-descriptor-load','ltr':'task-register-load',
    'swapgs':'gs-base-swap','stac':'access-check-override-set',
    'clac':'access-check-override-clear','cli':'interrupt-disable',
    'sti':'interrupt-enable','iret':'interrupt-return','iretq':'interrupt-return',
    'iretd':'interrupt-return','sysret':'syscall-return','sysretq':'syscall-return',
    'sysexit':'sysenter-return','sysexitq':'sysenter-return',
    'syscall':'syscall-entry','sysenter':'sysenter-entry',
    'invd':'cache-invalidation','wbinvd':'cache-writeback-invalidation',
    'vmrun':'virtualization-entry','vmlaunch':'virtualization-entry',
    'vmresume':'virtualization-entry','vmxon':'virtualization-control',
    'vmxoff':'virtualization-control','vmload':'virtualization-state-load',
    'vmwrite':'virtualization-state-write',
}


def opcode(raw):
    index=0
    while index<len(raw) and (raw[index] in PREFIXES or 0x40<=raw[index]<=0x4f):index+=1
    assert index<len(raw),'instruction contains prefixes only'
    return index,raw[index:]


def prefix_fragment(raw):
    return bool(raw) and all(byte in PREFIXES or 0x40<=byte<=0x4f for byte in raw)


def classify(raw,instruction):
    # Retain isolated decoder prefix rows, including GNU's isolated REX.WB.
    # They have no independently classified operation at this boundary.
    if prefix_fragment(raw):return None
    words=instruction.lower().replace(',', ' ').split()
    while words and words[0] in ['lock','rep','repz','repnz','repe','repne']:words.pop(0)
    if not words:
        # LLVM may list a LOCK prefix separately. Preserve its byte coverage;
        # this row is not an independently executable instruction or control.
        assert raw and all(byte in PREFIXES for byte in raw),'unrecognized prefix fragment'
        return None
    prefix,body=opcode(raw)
    if body[:2] in [b'\x0f\x20',b'\x0f\x22']:
        assert len(body)==3 and body[2]&0xc0==0xc0,'invalid control-register encoding'
        rex=raw[prefix-1] if prefix and 0x40<=raw[prefix-1]<=0x4f else 0
        control=((body[2]>>3)&7)+(8 if rex&4 else 0)
        write=body[1]==0x22
        assert words[0]=='mov' and len(words)==3
        assert words[1 if write else 2]=='cr'+str(control),'decoder control-register disagreement'
        return 'control-register-write' if write else 'control-register-read'
    assert not any(re.fullmatch(r'cr[0-9]+',word) for word in words),'unclassified control-register encoding'
    kind=CONTROL_NAMES.get(words[0])
    # Independently bind the important direct opcode families; other listed
    # instructions retain the two decoders' interpretation, not a full decoder.
    if body==b'\x0f\x30':assert kind=='msr-write'
    if body==b'\x0f\x32':assert kind=='msr-read'
    if kind in ['msr-write','msr-read']:
        assert body=={'msr-write':b'\x0f\x30','msr-read':b'\x0f\x32'}[kind]
    if len(body)>=3 and body[:2]==b'\x0f\x01' and body[2]&0x38==0x38 and body[2]&0xc0!=0xc0:
        assert kind=='page-invalidation'
    if kind=='page-invalidation':
        assert len(body)>=3 and body[:2]==b'\x0f\x01' and body[2]&0x38==0x38 and body[2]&0xc0!=0xc0
    return kind


def decode(text,start,body):
    rows={}
    for line in text.splitlines():
        match=re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if not match:continue
        address=int(match[1],16)
        assert start<=address<start+len(body),'listing instruction outside selected executable section'
        raw=bytes.fromhex(match[2])
        offset=address-start
        assert 0<len(raw)<=15 and offset+len(raw)<=len(body)
        assert body[offset:offset+len(raw)]==raw,'listing and linked bytes differ'
        assert address not in rows,'duplicate listing instruction'
        instruction=match[3].split('#',1)[0].strip()
        rows[address]=dict(address=address,bytes=raw.hex(),size=len(raw),instruction=instruction,
            kind=classify(raw,instruction))
    cursor=start
    for address,row in sorted(rows.items()):
        # -z may decode zero padding across a symbol boundary before restarting
        # at that symbol. Both exact interpretations remain in the inventory.
        assert address<=cursor,'missing executable section bytes'
        cursor=max(cursor,address+row['size'])
    assert cursor==start+len(body),'incomplete executable section listing'
    return rows


def candidates(start,body,rows):
    """Retain sensitive opcode bytes even inside another listed instruction.

    An embedded byte sequence is not evidence it can execute, nor is its being
    embedded evidence that alternate entry is impossible.
    """
    starts=sorted(rows)
    result=[]
    for offset in range(len(body)-1):
        raw=body[offset:offset+3]
        kind=None
        if raw[:2]==b'\x0f\x30':kind='msr-write'
        elif len(raw)==3 and raw[:2]==b'\x0f\x22' and raw[2]&0xc0==0xc0:kind='control-register-write'
        elif len(raw)==3 and raw[:2]==b'\x0f\x01' and raw[2]&0x38==0x38 and raw[2]&0xc0!=0xc0:kind='page-invalidation'
        if kind is None:continue
        address=start+offset
        end=bisect_right(starts,address)
        parents=[rows[at] for at in starts[max(0,end-15):end] if at+rows[at]['size']>address]
        assert parents
        canonical=False
        for parent in parents:
            if parent['kind']!=kind:continue
            index,_=opcode(bytes.fromhex(parent['bytes']))
            canonical|=address==parent['address']+index
        result.append(dict(address=address,opcode_bytes=raw[:2 if kind=='msr-write' else 3].hex(),kind=kind,
            containing_listing_rows=[r['address'] for r in parents],at_listed_opcode=canonical))
    return result


def join_controls(first,second):
    """Keep the union and explicit disagreements, including unvisited bytes."""
    selected={address:dict(row) for address,row in first.items() if row['kind']}
    for address,row in second.items():
        if row['kind']:
            if address in selected:
                assert (row['bytes'],row['kind'])==(selected[address]['bytes'],selected[address]['kind']),'conflicting selected controls at the same address'
            else:selected[address]=dict(row)
    disagreements=[]
    for address,row in selected.items():
        a,b=first.get(address),second.get(address)
        expected=(row['bytes'],row['kind'])
        match=lambda r:r is not None and (r['bytes'],r['kind'])==expected
        row['decoder_agreement']=match(a) and match(b)
        row['llvm_instruction']=a['instruction'] if a is not None else None
        row['gnu_instruction']=b['instruction'] if b is not None else None
        if not row['decoder_agreement']:disagreements.append(address)
    return selected,sorted(disagreements)


def inventory(kernel,llvm,gnu,graph):
    elf=Elf(kernel)
    assert elf.kind==2
    executable=[(i,s) for i,s in enumerate(elf.sections) if s[2]&4 and s[5]]
    assert len(executable)==1,'qualify executable sections before extending this selected inventory'
    section_index,section=executable[0]
    start,body=section[3],elf.section_bytes(section_index)
    first=decode(llvm.decode(),start,body)
    second=decode(gnu.decode(),start,body)
    selected,disagreements=join_controls(first,second)
    assert graph['passed'] and graph['kernel_sha256']==digest(elf.data)
    process=elf.symbols[graph['function']]
    assert graph['function_address']==process['value'] and graph['function_bytes']==process['size']
    assert graph['linked_function_sha256']==digest(elf.function(graph['function'])[1])
    graph_selected={}
    recovered=[]
    for row in graph['instructions']:
        raw=bytes.fromhex(row['bytes'])
        address=process['value']+row['offset']
        assert body[address-start:address-start+len(raw)]==raw
        kind=classify(raw,row['instruction'])
        if kind:
            graph_selected[address]=dict(bytes=raw.hex(),kind=kind)
            assert address in selected and selected[address]['bytes']==raw.hex() and selected[address]['kind']==kind,'graph has unlisted privileged instruction'
        if address not in first:recovered.append(address)
    for row in selected.values():
        owners=[s for s in elf.symbols.values() if s['section']==section_index and s['info']&15==2
            and s['value']<=row['address']<s['value']+s['size']]
        assert owners,'selected privileged instruction has no function symbol'
        row['functions']=sorted(set(s['name'] for s in owners))
        row['in_selected_process_graph']=row['address'] in graph_selected
    raw_candidates=candidates(start,body,first)
    gnu_candidates=candidates(start,body,second)
    return dict(passed=True,kernel_sha256=digest(elf.data),llvm_listing_sha256=digest(llvm),gnu_listing_sha256=digest(gnu),
        executable_section=dict(index=section_index,address=start,bytes=len(body),sha256=digest(body)),
        llvm_instruction_rows=len(first),gnu_instruction_rows=len(second),
        llvm_overlap_bytes=sum(r['size'] for r in first.values())-len(body),
        gnu_overlap_bytes=sum(r['size'] for r in second.values())-len(body),
        llvm_prefix_fragment_rows=sum(prefix_fragment(bytes.fromhex(r['bytes'])) for r in first.values()),
        gnu_prefix_fragment_rows=sum(prefix_fragment(bytes.fromhex(r['bytes'])) for r in second.values()),
        llvm_unknown_rows=sum('<unknown>' in r['instruction'] for r in first.values()),
        gnu_bad_rows=sum('(bad)' in r['instruction'] for r in second.values()),
        selected_controls=len(selected),kinds=dict(sorted(Counter(r['kind'] for r in selected.values()).items())),
        full_decoder_control_agreement=not disagreements,unmatched_control_addresses=disagreements,
        process_graph_controls=len(graph_selected),graph_instruction_starts_absent_from_linear_listing=recovered,
        controls=[selected[a] for a in sorted(selected)],raw_opcode_candidates=raw_candidates,
        gnu_raw_opcode_candidates=gnu_candidates,
        embedded_or_alternate_candidates=sum(not r['at_listed_opcode'] for r in raw_candidates),
        scope='Byte-complete overlapping linear listings of the selected linked executable section, union of the declared privileged-control families with explicit disagreements, binding to the preceding process graph and retained raw opcode candidates. Passing means the inventory is consistent, not that all interpretations agree or all code is qualified. Not an all-entry/control-flow proof, complete page-table-store inventory, runtime/boot/AP code inventory, native instruction coverage, continuous mode/TLB history or production qualification')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kernel',type=Path)
    parser.add_argument('llvm',type=Path)
    parser.add_argument('gnu',type=Path)
    parser.add_argument('graph',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=inventory(args.kernel,args.llvm.read_bytes(),args.gnu.read_bytes(),json.loads(args.graph.read_bytes()))
    report['inventory_sha256']=digest(Path(__file__).read_bytes())
    report['process_graph_sha256']=digest(args.graph.read_bytes())
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['controls','raw_opcode_candidates','gnu_raw_opcode_candidates']}))
