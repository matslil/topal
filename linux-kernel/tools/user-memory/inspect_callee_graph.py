"""Inspect linked direct/callback/table edges beyond the selected process body.

TOPAL-SYSTEMS-USER-EXTENT-001. This is a conditional saved-code graph, with
explicit return, entry, mutation, ownership and asynchronous obligations. A
closed selected edge set is not complete kernel control-flow qualification.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct

from callee_control import Unresolved,TERMINALS,category,guarded_table,matched,merge_window,no_bypass,require,transfer,window_rows
from inspect_call_inventory import Elf,digest
from inventory_paging_controls import decode


def payload(elf,address,size,readonly=False):
    require(type(address) is int and type(size) is int and 0<=address<address+size<=1<<64,
            'callee payload range')
    sections=[(i,s) for i,s in enumerate(elf.sections) if s[2]&2 and s[5]
        and s[3]<=address<address+size<=s[3]+s[5]]
    require(len(sections)==1,'callee payload allocated section')
    index,section=sections[0]
    if readonly:require(not section[2]&1 and not section[2]&4,'jump table section is writable/executable')
    raw=elf.section_bytes(index)[address-section[3]:address-section[3]+size]
    require(len(raw)==size,'callee payload bytes')
    return raw


def entries(elf,graph,process,body):
    require(graph['passed'] and graph['kernel_sha256']==digest(elf.data)
        and graph['function_address']==process['value'] and graph['function_bytes']==len(body)
        and graph['linked_function_sha256']==digest(body),'callee process graph identity')
    setup=[]
    for at in range(len(body)-23):
        if body[at:at+3]==bytes.fromhex('488d05') and body[at+7:at+22]==bytes.fromhex('b9820000c04889c248c1ea200f30b9'):
            setup.append((at,at+7+struct.unpack_from('<i',body,at+3)[0]))
    require(len(setup)==1,'callee programmed syscall entry')
    at,syscall=setup[0];require(body[syscall:syscall+3]==bytes.fromhex('0f01f8'),'callee syscall entry bytes')
    roots=[dict(offset=0,source='selected function entry'),
        dict(offset=syscall,source='actual RIP-relative LSTAR setup',setup_offset=at)]
    for value in sorted(elf.defined_symbols,key=lambda s:(s['name'],s['section'],s['value'])):
        if value['section']==process['section'] and process['value']<value['value']<process['value']+len(body):
            require(value['info']&15==2,'unqualified interior entry symbol')
            roots.append(dict(offset=value['value']-process['value'],source='exported symbol',symbol=value['name']))
    require(roots==graph['roots'],'callee complete declared process roots')
    return [dict(row,address=process['value']+row['offset']) for row in roots]


def callbacks(elf,bindings,graph,process,body,llvm,gnu):
    require(bindings['passed'] and bindings['kernel_sha256']==digest(elf.data),'callee callback identity')
    by_slot={row['gs_slot']:row for row in bindings['callbacks']}
    require(len(by_slot)==len(bindings['callbacks'])==5,'callee callback slot closure')
    root,root_body=elf.function('_topal_kernel_entry');call=bindings['root_call_offset']
    require(root_body[call:call+1]==b'\xe8' and root['value']+call+5
        +struct.unpack_from('<i',root_body,call+1)[0]==process['value'],'callee actual root invocation')
    first=min(row['root_lea_offset'] for row in by_slot.values())-8
    cursor=root['value']+first;limit=root['value']+call+5;pushes=[]
    while cursor<limit:
        row=matched(cursor,llvm,gnu);name=row['mnemonic']
        require(not (category(row)!='ordinary' and cursor!=root['value']+call),'callee root argument transfer')
        if name=='push':pushes.append(cursor-root['value'])
        require('rsp' not in row['instruction'].lower() or name in ['push','call'],'callee root argument stack mutation')
        cursor+=row['size']
    require(cursor==limit and len(pushes)==bindings['root_stack_pushes']==13,'callee root argument push closure')
    process_rows={process['value']+row['offset']:row for row in graph['instructions']}
    for record in by_slot.values():
        target=elf.symbols[record['native_export']];lea=record['root_lea_offset']
        require(root_body[lea:lea+3]==bytes.fromhex('488d05') and root_body[lea+7:lea+8]==b'\x50'
            and root['value']+lea+7+struct.unpack_from('<i',root_body,lea+3)[0]==target['value'],
            'callee root callback LEA/target')
        require(record['root_push_offset']==lea+7 and pushes.index(lea+7)==19-record['argument'],
            'callee root callback argument position')
        load,store=record['registration_load_offset'],record['registration_store_offset']
        require(body[:7]==bytes.fromhex('41514150575651') and body[load:load+8]
            ==bytes.fromhex('488b8424')+struct.pack('<I',8*(record['argument']-6)+40),
            'callee registration argument load')
        require(body[store:store+7]==bytes.fromhex('498982')+struct.pack('<I',record['gs_slot']),
            'callee registration slot store')
        cursor=process['value']+load+8
        while cursor<process['value']+store:
            witness=matched(cursor,llvm,gnu)
            require(witness['mnemonic']=='cmp' or category(witness)=='conditional',
                'callee registration value overwritten')
            require(cursor in process_rows,'callee registration outside retained process graph')
            cursor+=witness['size']
        require(cursor==process['value']+store,'callee contiguous registration slice')
    result={}
    require(len(bindings['sites'])==len(graph['indirect_transfers']),'callee process indirect closure')
    for row,original in zip(bindings['sites'],graph['indirect_transfers']):
        at=row['offset'];require(at==original['offset'],'callee callback site order')
        require(body[at:at+2]==bytes.fromhex('ffd0'),'callee callback native instruction')
        load=at-47
        require(body[load:load+5]==bytes.fromhex('65488b0425'),'callee callback slot load')
        slot=struct.unpack_from('<I',body,load+5)[0]
        require(slot==row['gs_slot'] and slot in by_slot,'callee callback slot binding')
        record=by_slot[slot];require(row['native_export']==record['native_export'],'callee callback export')
        target=elf.symbols[record['native_export']]
        require(target['section'] and target['info']&15==2 and target['value']==record['target']
            and target['size']==record['target_bytes'],'callee actual callback target')
        require(body[load+9:load+18]==bytes.fromhex('4881f8000000000f84')
            and body[load+22:at+2]==bytes.fromhex('4989e34881e4f0ffffff4881c4f0ffffff4c899c2400000000ffd0'),
            'callee callback contiguous nonzero/stack adapter')
        require(not any(edge['kind']=='direct-transfer' and load<edge['target']<=at for edge in graph['edges']),
            'callee callback direct-entry bypass')
        result[process['value']+at]=dict(target=target['value'],slot=slot,export=record['native_export'])
    return result


def inspect(build,llvm_text,gnu_text,recoveries=(),boot_image=None):
    elf=Elf(build/'kernel.elf');require(elf.kind==2,'callee linked executable')
    executable=[(i,s) for i,s in enumerate(elf.sections) if s[2]&4 and s[5]]
    require(len(executable)==1,'qualify multiple executable sections before extending this reader')
    section_index,section=executable[0];start=section[3];code=elf.section_bytes(section_index);end=start+len(code)
    llvm=decode(llvm_text.decode(),start,code);gnu=decode(gnu_text.decode(),start,code)
    recovery_records=[];recovery_starts=set()
    for address,first,second in recoveries:
        require(address not in recovery_starts,'duplicate recovery window')
        recovery_starts.add(address);stop=min(address+128,end)
        first_rows=window_rows(first.decode(),start,code,address,stop)
        second_rows=window_rows(second.decode(),start,code,address,stop)
        applied_end=merge_window(llvm,first_rows,address,stop,end)
        require(applied_end==merge_window(gnu,second_rows,address,stop,end),'decoder recovery applied range differs')
        recovery_records.append(dict(address=address,stop=stop,applied_end=applied_end,
            llvm_sha256=digest(first),gnu_sha256=digest(second)))
    graph_raw=(build/'process-access-inventory.json').read_bytes();graph=json.loads(graph_raw)
    binding_raw=(build/'callback-bindings.json').read_bytes();binding=json.loads(binding_raw)
    require(binding['process_graph_sha256']==digest(graph_raw),'callee callback/graph identity')
    process,body=elf.function(graph['function'])
    roots=entries(elf,graph,process,body);bound=callbacks(elf,binding,graph,process,body,llvm,gnu)
    boot=None
    if boot_image is not None:
        from inspect_boot_entries import inspect as inspect_boot
        boot=inspect_boot(build/'kernel.elf',boot_image)
        require(boot['kernel_sha256']==digest(elf.data),'callee boot/ELF identity')
        roots+=boot['roots']
    pending=[row['address'] for row in roots];seen={};attempted=set();occupied={};edges=[];frontiers=[];tables={};terminals=[]
    while pending:
        address=pending.pop()
        if address in attempted:continue
        attempted.add(address)
        if not start<=address<end:
            frontiers.append(dict(address=address,reason='outside selected executable section'));continue
        try:row=matched(address,llvm,gnu)
        except Unresolved as error:
            frontiers.append(dict(address=address,reason=str(error)));continue
        require(address+row['size']<=end,'callee reached instruction extent')
        for byte in range(address,address+row['size']):
            require(byte not in occupied,'overlapping reached callee instructions')
            occupied[byte]=address
        seen[address]=row;kind=category(row)
        if kind in TERMINALS:
            terminals.append(dict(address=address,kind=kind));continue
        if kind in ['call','jmp','conditional']:
            try:destination=transfer(row)
            except Unresolved as error:
                frontiers.append(dict(address=address,reason=str(error)));continue
            if destination is not None:
                edges.append(dict(source=address,target=destination,kind='direct-transfer'));pending.append(destination)
            elif address in bound:
                destination=bound[address]['target'];pending.append(destination)
                edges.append(dict(source=address,target=destination,kind='callback-registration-assumption'))
            elif kind=='jmp':
                try:
                    table=guarded_table(gnu,address,lambda at,n:payload(elf,at,n,True))
                    owners=[symbol for symbol in elf.defined_symbols if symbol['info']&15==2
                        and symbol['section']==section_index and symbol['value']<=address<symbol['value']+symbol['size']]
                    require(bool(owners),'callee table jump function owner')
                    require(any(symbol['value']<=table['guard_begin']
                        and all(symbol['value']<=at<symbol['value']+symbol['size'] for at in table['targets'])
                        for symbol in owners),'callee table guard/targets leave function')
                    # Every guard/load instruction receives the same two-decoder
                    # control check as the graph, even if its path is unvisited.
                    cursor=table['guard_begin']
                    while cursor<=address:
                        witness=matched(cursor,llvm,gnu)
                        if cursor==table['guard']:require(witness['llvm_mnemonic']=='ja','LLVM table guard condition')
                        cursor+=witness['size']
                    require(cursor==address+row['size'],'callee complete guard instruction span')
                    tables[address]=table
                    for index,target in enumerate(table['targets']):
                        pending.append(target);edges.append(dict(source=address,target=target,kind='linked-table-assumption',index=index))
                except Unresolved as error:frontiers.append(dict(address=address,reason=str(error)))
            else:frontiers.append(dict(address=address,reason='unbound nested indirect call'))
            if kind=='jmp':continue
        following=address+row['size'];pending.append(following)
        edges.append(dict(source=address,target=following,kind='call-return-assumption' if kind=='call' else 'fallthrough'))
    declared=[row['address'] for row in roots]+[row['target'] for row in bound.values()]
    for table in tables.values():no_bypass(table,declared,edges)
    reached=sorted(seen);functions=[]
    from bisect import bisect_left
    for symbol in elf.defined_symbols:
        if symbol['section']!=section_index or symbol['info']&15!=2 or not symbol['size']:continue
        lo,hi=symbol['value'],symbol['value']+symbol['size']
        count=bisect_left(reached,hi)-bisect_left(reached,lo)
        if count:functions.append(dict(symbol=symbol,address=lo,bytes_sha256=digest(payload(elf,lo,symbol['size'])),reached_instructions=count))
    return dict(passed=True,schema='topal.conditional-callee-graph/1',kernel_sha256=digest(elf.data),
        process_graph_sha256=digest(graph_raw),callback_binding_sha256=digest(binding_raw),
        llvm_listing_sha256=digest(llvm_text),gnu_listing_sha256=digest(gnu_text),
        recovery_windows=recovery_records,
        boot_entries=boot,
        executable_section=dict(index=section_index,address=start,bytes=len(code),sha256=digest(code)),
        roots=roots,callbacks=bound,guarded_tables=[tables[at] for at in sorted(tables)],
        reached_instructions=len(seen),reached_bytes=len(occupied),functions=functions,
        coalesced_prefix_instructions=sum(len(row['llvm_fragments'])>1 for row in seen.values()),
        ordinary_decoder_aliases=sum(row['mnemonic']!=row['llvm_mnemonic'] for row in seen.values()),
        frontiers=frontiers,selected_edge_set_closed=not frontiers,terminals=terminals,
        instructions=[seen[at] for at in reached],edges=edges,
        process_helpers={str(offset):process['value']+offset in seen for offset in [88215,94687]},
        assumptions=['Listed selected process entries'+(' plus saved protected boot handoff/four initial IDT targets' if boot else '')+' only; other loader, runtime, exception, AP and arbitrary interior entries remain separate.',
            'Previously bound callback slots remain the registered root targets; complete writer/lifecycle stability is not established here.',
            'Executable code, linked tables and local saved-index slots stay unchanged during use; ELF flags alone do not establish hardware permissions.',
            'Calls may return at their encoded continuations; return-stack ownership, exception behavior and whole callee effects remain separate.'],
        qualification_complete=False,
        scope='Consistent two-decoder saved-code traversal of declared roots, direct callees, registered callbacks and selected guarded jump tables under explicit assumptions. Optional boot roots are rederived from saved image/ELF bytes, not accepted from a supplied target list. Passing or selected_edge_set_closed does not establish complete kernel entries/CFI, runtime lifetime/immutability, memory authority, native instruction coverage, production activation or global unreachability.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['build','llvm','gnu','output']:parser.add_argument(name,type=Path)
    parser.add_argument('--recovery',nargs=3,action='append',default=[],metavar=('ADDRESS','LLVM','GNU'))
    parser.add_argument('--boot-image',type=Path,help='Rebind the saved protected handoff and initial IDT roots from this image and the actual ELF.')
    args=parser.parse_args();recoveries=[(int(at,0),Path(first).read_bytes(),Path(second).read_bytes())
        for at,first,second in args.recovery]
    result=inspect(args.build.resolve(),args.llvm.read_bytes(),args.gnu.read_bytes(),recoveries,
        args.boot_image.read_bytes() if args.boot_image else None)
    args.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({key:result[key] for key in ['passed','reached_instructions','reached_bytes',
        'coalesced_prefix_instructions','ordinary_decoder_aliases','selected_edge_set_closed','frontiers','process_helpers','qualification_complete']}))
