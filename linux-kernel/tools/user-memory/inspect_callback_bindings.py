"""Bind selected process callback slots to actual root arguments and objects.

TOPAL-SYSTEMS-USER-EXTENT-001, TOPAL-SYSTEMS-POLICY-LINKED-INPUTS-001.
This reads compiler-owned artifacts. Numeric identities are evidence to inspect,
not a source/kernel API for constructing access or execution authority.
"""
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from inspect_call_inventory import Elf, digest
from inventory_process_accesses import decoded_rows

SLOTS = ((952, 13, 'kernel_library'), (960, 14, 'time_library'),
         (976, 16, 'image_library'), (984, 17, 'dispatch_library'),
         (992, 18, 'process_identity_library'))


def relocation_entries(elf, owner):
    """Keep the symbol-table index, including anonymous section symbols."""
    result = {}
    for section in elf.sections:
        if section[1] != 4 or section[7] != owner:
            continue
        assert section[9] == 24 and section[5] % 24 == 0
        for at in range(section[4],section[4]+section[5],24):
            offset, info, addend = struct.unpack_from('<QQq',elf.data,at)
            assert offset not in result
            result[offset] = (elf.tables[section[6]][info>>32],info&0xffffffff,addend)
    return result


def inspect(build, tool):
    kernel = Elf(build/'kernel.elf')
    root = Elf(build/'root.o')
    graph = json.loads((build/'process-access-inventory.json').read_text())
    assert graph['passed'] and graph['kernel_sha256'] == digest(kernel.data)
    process, process_body = kernel.function(graph['function'])
    provenance = json.loads((build/'baseline/provenance.json').read_text())
    root_symbol, root_body = root.function('_topal_kernel_entry')
    linked_root, linked_body = kernel.function('_topal_kernel_entry')
    assert root_symbol['size'] == linked_root['size']
    root_relocations = root.relocations(root_symbol['section'])
    entry_calls = [at for at, r in root_relocations.items() if r == (graph['function'], 4, -4)]
    assert len(entry_calls) == 1
    call = entry_calls[0]-root_symbol['value']-1
    assert linked_body[call] == 0xe8
    assert linked_root['value']+call+5+struct.unpack_from('<i',linked_body,call+1)[0] == process['value']
    records = []
    for slot, argument, key in SLOTS:
        record = provenance[key]
        stem = key.replace('_','-')
        obj = Elf(build/'baseline'/(stem+'.o'))
        assert digest(obj.data) == record['object_sha256']
        assert digest((build/'baseline'/(stem+'.t')).read_bytes()) == record['source_sha256']
        assert digest((build/'baseline'/(stem+'.modules.json')).read_bytes()) == record['module_snapshot_sha256']
        export = record['native_export']
        target, _ = kernel.function(export)
        fields = [at for at, r in root_relocations.items() if r == (export, 2, -4)]
        assert len(fields) == 1, (key,fields)
        field = fields[0]-root_symbol['value']
        assert linked_body[field-3:field] == bytes.fromhex('48 8d 05')
        assert linked_body[field+4] == 0x50
        assert linked_root['value']+field+4+struct.unpack_from('<i',linked_body,field)[0] == target['value']
        section_bases, sections = {}, []
        for section_record in record['sections']:
            marker = section_record['marker']
            begin, end = obj.symbols[marker+'.begin'], obj.symbols[marker+'.end']
            section = obj.sections[begin['section']]
            payload = obj.section_bytes(begin['section'])
            assert begin['section'] == end['section'] and begin['value'] == 0
            assert end['value'] == section[5] == section_record['bytes']
            assert digest(payload) == section_record['sha256']
            assert section[2] & (7|0x400) == (6 if section_record['executable'] else 2)
            actual_begin, actual_end = kernel.symbols[marker+'.begin'], kernel.symbols[marker+'.end']
            assert actual_begin['section'] == actual_end['section']
            assert actual_end['value']-actual_begin['value'] == len(payload)
            owner = kernel.sections[actual_begin['section']]
            assert owner[2] & (7|0x400) == (6 if section_record['executable'] else 2)
            section_bases[begin['section']] = actual_begin['value']
            sections.append(dict(marker=marker,object_section=begin['section'],address=actual_begin['value'],
                bytes=len(payload),executable=section_record['executable'],object_sha256=digest(payload)))
        allocated = {i for i,s in enumerate(obj.sections) if s[2]&2 and s[5]}
        assert allocated == set(section_bases)
        relocation_count = 0
        for section_record in sections:
            index = section_record['object_section']
            payload = bytearray(obj.section_bytes(index))
            for offset, (value, kind, addend) in relocation_entries(obj,index).items():
                name = value['name']
                if value['section']:
                    assert value['section'] in section_bases, (stem,name)
                    address = section_bases[value['section']]+value['value']
                else:
                    address = kernel.symbols[name]['value']
                    assert kernel.symbols[name]['section']
                if kind in [2,4]:
                    raw = struct.pack('<i',address+addend-section_bases[index]-offset)
                elif kind == 1:
                    raw = struct.pack('<Q',address+addend)
                elif kind == 10:
                    raw = struct.pack('<I',address+addend)
                elif kind == 11:
                    raw = struct.pack('<i',address+addend)
                else:
                    raise AssertionError(('unsupported allocated relocation',stem,kind))
                assert 0 <= offset <= len(payload)-len(raw)
                payload[offset:offset+len(raw)] = raw
                relocation_count += 1
            begin = kernel.symbols[section_record['marker']+'.begin']
            owner = kernel.sections[begin['section']]
            offset = begin['value']-owner[3]
            actual = kernel.section_bytes(begin['section'])[offset:offset+len(payload)]
            assert actual == payload, ('linked allocated payload differs',stem,section_record['marker'])
            section_record['linked_sha256'] = digest(actual)
        functions = []
        for function in record['functions']:
            name = function['symbol']
            original, original_body = obj.function(name)
            actual, actual_body = kernel.function(name)
            assert len(actual_body) == len(original_body)
            assert actual['value'] == section_bases[original['section']]+original['value']
            functions.append(dict(symbol=name,address=actual['value'],bytes=len(actual_body),
                sha256=digest(actual_body),recorded_callees=function['callees'],
                scope='Recorded call/resource certificate, not a new independent instruction/callee proof'))
        assert target['section'] in {kernel.symbols[s['marker']+'.begin']['section'] for s in sections if s['executable']}
        store = bytes.fromhex('49 89 82')+struct.pack('<I',slot)
        stores = [at for at in range(graph['roots'][1]['setup_offset']) if process_body[at:at+7] == store]
        assert len(stores) == 1
        store_at = stores[0]
        load = bytes.fromhex('48 8b 84 24')+struct.pack('<I',8*(argument-6)+40)
        loads = [r['offset'] for r in graph['instructions'] if r['offset'] < store_at
                 and process_body[r['offset']:r['offset']+8] == load]
        assert loads
        load_at = max(loads)
        intermediate = [r for r in graph['instructions'] if load_at+8 <= r['offset'] < store_at]
        assert intermediate and all(r['mnemonic']=='cmp' or r['mnemonic'].startswith('j') for r in intermediate)
        records.append(dict(gs_slot=slot,argument=argument,entry_stack_offset=8*(argument-6),
            role=key,native_export=export,target=target['value'],target_bytes=target['size'],
            root_lea_offset=field-3,root_push_offset=field+4,
            registration_load_offset=load_at,registration_store_offset=store_at,
            object_sha256=digest(obj.data),source=record['source'],source_sha256=record['source_sha256'],
            allocation_relocations_verified=relocation_count,sections=sections,functions=functions,
            recorded_maximum_stack_bytes=record['maximum_stack_bytes'],stack_limit_bytes=record['stack_limit_bytes']))
    # The selected nineteen-argument invocation has thirteen stack pushes. Start
    # at the retained-store LEA immediately preceding the earliest callback LEA.
    start = min(r['root_lea_offset'] for r in records)-8
    command = [str(tool),'-d','--x86-asm-syntax=intel',
        '--start-address='+str(linked_root['value']+start),
        '--stop-address='+str(linked_root['value']+call+5),str(build/'kernel.elf')]
    result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (build/'callback-root-disassembly.stdout.log').write_bytes(result.stdout)
    (build/'callback-root-disassembly.stderr.log').write_bytes(result.stderr)
    (build/'callback-root-disassembly.command.json').write_text(json.dumps(dict(argv=command,exit_code=result.returncode),indent=2)+'\n')
    assert result.returncode == 0
    decoded = decoded_rows(result.stdout.decode(),linked_root,linked_body)
    pushes = [r['offset'] for r in decoded.values() if r['mnemonic']=='push']
    assert len(pushes) == 13
    assert sum(r['mnemonic']=='call' for r in decoded.values()) == 1
    assert not any('rsp' in r['instruction'] and r['mnemonic'] not in ['push','call'] for r in decoded.values())
    for record in records:
        assert pushes.index(record['root_push_offset']) == 19-record['argument']
    by_slot = {r['gs_slot']:r for r in records}
    sites = []
    for row in graph['indirect_transfers']:
        at = row['offset']
        lo = at-47
        assert process_body[lo:lo+5] == bytes.fromhex('65 48 8b 04 25')
        slot = struct.unpack_from('<I',process_body,lo+5)[0]
        assert slot in by_slot
        assert process_body[lo+9:lo+18] == bytes.fromhex('48 81 f8 00 00 00 00 0f 84')
        assert process_body[lo+22:at+2] == bytes.fromhex(
            '49 89 e3 48 81 e4 f0 ff ff ff 48 81 c4 f0 ff ff ff 4c 89 9c 24 00 00 00 00 ff d0')
        assert not any(e['kind']=='direct-transfer' and lo < e['target'] <= at for e in graph['edges'])
        sites.append(dict(offset=at,address=process['value']+at,return_address=process['value']+at+2,
                          gs_slot=slot,native_export=by_slot[slot]['native_export']))
    report = dict(passed=True,kernel_sha256=digest(kernel.data),root_sha256=digest(root.data),
        process_graph_sha256=digest((build/'process-access-inventory.json').read_bytes()),
        baseline_provenance_sha256=digest((build/'baseline/provenance.json').read_bytes()),
        root_call_offset=call,root_stack_pushes=13,callbacks=records,sites=sites,
        scope='Exact root relocation/argument/registration-slice and allocated object-to-linked byte binding. ELF flags are not hardware permissions; record call certificates are not independently re-proved here. Native target/permission/state checks, callee/user-pointer effects and complete owner/lifetime/entry qualification remain separate')
    (build/'callback-bindings.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,callbacks=len(records),functions=sum(len(r['functions']) for r in records),
        sections=sum(len(r['sections']) for r in records),
        relocation_count=sum(r['allocation_relocations_verified'] for r in records),scope=report['scope'])))


if __name__ == '__main__':
    inspect(Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve())
