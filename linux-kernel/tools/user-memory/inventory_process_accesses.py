"""Inventory a selected linked x86 process's direct-control instruction graph.

TOPAL-SYSTEMS-USER-EXTENT-001. This lists address-bearing and implicit-stack
instructions, not proven user accesses. External/indirect callees, exception
entry, pointer provenance, admission dominance and lifetime remain separate.
"""
from collections import Counter
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from inspect_call_inventory import Elf, digest, verify_calls


def decoded_rows(text, symbol, body):
    rows = {}
    for line in text.splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$', line)
        if not match:
            continue
        offset = int(match[1], 16) - symbol['value']
        if not 0 <= offset < len(body):
            continue
        raw = bytes.fromhex(match[2])
        assert 0 < len(raw) <= 15 and offset + len(raw) <= len(body)
        assert body[offset:offset + len(raw)] == raw, ('decoder bytes', offset)
        assert offset not in rows
        instruction = match[3].split('#', 1)[0].strip()
        words = instruction.split()
        while words[0] in ['rep', 'repne', 'repe', 'repnz', 'repz', 'lock']:
            words.pop(0)
        rows[offset] = dict(offset=offset, size=len(raw), bytes=raw.hex(),
                            instruction=instruction, mnemonic=words[0])
    assert rows
    return rows


def disassemble(build, tool, symbol, body, start, end, label):
    command = [str(tool), '-d', '--x86-asm-syntax=intel',
               '--start-address=' + str(symbol['value'] + start),
               '--stop-address=' + str(symbol['value'] + end), str(build/'kernel.elf')]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    (build/(label+'.stdout.log')).write_bytes(result.stdout)
    (build/(label+'.stderr.log')).write_bytes(result.stderr)
    (build/(label+'.command.json')).write_text(json.dumps(
        dict(argv=command, exit_code=result.returncode), indent=2)+'\n')
    assert result.returncode == 0, result.stderr.decode(errors='replace')
    return decoded_rows(result.stdout.decode(), symbol, body)


def branch(row):
    """Cross-check each direct branch operand against its actual displacement."""
    at = row['offset']
    raw = bytes.fromhex(row['bytes'])
    match = re.match(r'^\w+\s+0x([0-9a-f]+)', row['instruction'])
    if not match:
        assert row['mnemonic'] in ['call', 'jmp'] and raw[0] == 0xff
        return None
    if raw[0] in [0xe8, 0xe9]:
        assert len(raw) == 5
        delta = struct.unpack_from('<i', raw, 1)[0]
    elif raw[:1] == b'\x0f' and 0x80 <= raw[1] <= 0x8f:
        assert len(raw) == 6
        delta = struct.unpack_from('<i', raw, 2)[0]
    elif raw[0] in [0xeb, 0xe0, 0xe1, 0xe2, 0xe3] or 0x70 <= raw[0] <= 0x7f:
        assert len(raw) == 2
        delta = struct.unpack_from('<b', raw, 1)[0]
    else:
        raise AssertionError(('unclassified direct branch', row))
    return at + len(raw) + delta, int(match[1], 16)


def access_kinds(row):
    name, assembly = row['mnemonic'], row['instruction']
    kinds = []
    operands = re.findall(r'\[[^]]+\]', assembly)
    if operands and name != 'lea':
        kinds.append('explicit-addressed-operand')
        if 'gs:[' in assembly:
            kinds.append('gs-based')
        elif 'fs:[' in assembly:
            kinds.append('fs-based')
        elif all(re.search(r'\b(?:rsp|rbp)\b', operand) for operand in operands):
            kinds.append('stack-base-operand')
        elif all('rip' in operand for operand in operands):
            kinds.append('rip-relative-operand')
        else:
            kinds.append('general-address-operand')
    if name.startswith(('push', 'pop')) or name in ['call', 'ret', 'retf', 'iret', 'iretq', 'enter', 'leave']:
        kinds.append('implicit-stack')
    if name in ['movsb','movsw','movsd','movsq','stosb','stosw','stosd','stosq',
                'lodsb','lodsw','lodsd','lodsq','scasb','scasw','scasd','scasq',
                'cmpsb','cmpsw','cmpsd','cmpsq','insb','insw','insd','outsb','outsw','outsd']:
        kinds.append('string-operation')
    return kinds


def inventory(build, tool):
    census = json.loads((build/'extent-call-inventory.json').read_text())
    provider = Elf(build/'provider.o')
    verify_calls(provider, census)
    elf = Elf(build/'kernel.elf')
    assert elf.kind == 2
    symbol = verify_calls(elf, census)
    _, body = elf.function(census['function'])
    rows = disassemble(build, tool, symbol, body, 0, len(body), 'process-access-disassembly')
    linear_offsets = set(rows)

    # Derive the actual programmed syscall entry from this instruction sequence,
    # rather than guessing an address or granting authority from a label map.
    sites = []
    for at in range(len(body)-23):
        if body[at:at+3] == bytes.fromhex('48 8d 05') and body[at+7:at+22] == bytes.fromhex(
                'b9 82 00 00 c0 48 89 c2 48 c1 ea 20 0f 30 b9'):
            target = at+7+struct.unpack_from('<i', body, at+3)[0]
            sites.append((at, target))
    assert len(sites) == 1
    lstar_site, syscall = sites[0]
    assert body[syscall:syscall+3] == bytes.fromhex('0f 01 f8')
    roots = [dict(offset=0, source='selected function entry'),
             dict(offset=syscall, source='actual RIP-relative LSTAR setup', setup_offset=lstar_site)]
    for name, value in sorted(elf.symbols.items()):
        if (value['section'] == symbol['section'] and value['value'] != symbol['value']
                and symbol['value'] <= value['value'] < symbol['value'] + len(body)):
            assert value['info'] & 15 == 2
            roots.append(dict(offset=value['value']-symbol['value'], source='exported symbol', symbol=name))
    pending = [r['offset'] for r in roots]
    seen, occupied, edges, external, indirect, terminals, recovered = {}, {}, [], [], [], [], []
    while pending:
        at = pending.pop()
        assert 0 <= at < len(body), ('fallthrough outside selected body', at)
        if at in seen:
            continue
        if at not in rows:
            label = 'process-access-recovery-'+str(len(recovered))
            assert len(recovered) < 32, 'unexpected decoder recovery growth'
            recovered_rows = disassemble(build, tool, symbol, body, at, min(at+128, len(body)), label)
            assert at in recovered_rows
            rows.update(recovered_rows)
            recovered.append(dict(offset=at, label=label, decoded_offsets=sorted(recovered_rows)))
        row = rows[at]
        assert '<unknown>' not in row['instruction'] and row['mnemonic'] not in ['.byte', '.word']
        for byte in range(at, at+row['size']):
            assert byte not in occupied, ('overlapping reached instructions', at, occupied.get(byte))
            occupied[byte] = at
        seen[at] = row
        name = row['mnemonic']
        row['access_kinds'] = access_kinds(row)
        if name in ['ret','retf','iret','iretq','sysret','sysretq','hlt','ud2','int','int3','syscall','sysenter','sysexit']:
            terminals.append(dict(offset=at, instruction=row['instruction']))
            continue
        if name.startswith('j') or name.startswith('loop') or name == 'call':
            target = branch(row)
            if target is None:
                indirect.append(dict(offset=at, instruction=row['instruction'],
                    obligation='Callee provenance/body/effects are not discharged by this graph'))
            else:
                destination, address = target
                assert symbol['value'] + destination == address
                if 0 <= destination < len(body):
                    pending.append(destination)
                    edges.append(dict(source=at, target=destination, kind='direct-transfer'))
                else:
                    names = sorted(n for n, s in elf.symbols.items()
                                   if s['section'] and s['value'] == address)
                    assert names, ('external transfer has no exact symbol', row)
                    external.append(dict(offset=at, address=address, symbols=names,
                        instruction=row['instruction']))
            if name == 'jmp':
                continue
        following = at+row['size']
        edges.append(dict(source=at, target=following, kind='call-return-assumption' if name == 'call' else 'fallthrough'))
        pending.append(following)
    assert lstar_site in seen
    calls_in_graph = [r for r in census['calls'] if r['offset'] in seen]
    calls_outside_graph = [r for r in census['calls'] if r['offset'] not in seen]
    assert all(edge['target'] in seen for edge in edges)
    counts = Counter(kind for row in seen.values() for kind in row['access_kinds'])
    report = dict(passed=True, function=census['function'], function_address=symbol['value'],
        function_bytes=len(body), linked_function_sha256=digest(body),
        kernel_sha256=digest(elf.data), provider_sha256=digest(provider.data),
        extent_inventory_sha256=digest((build/'extent-call-inventory.json').read_bytes()),
        roots=roots, reached_instructions=len(seen), reached_bytes=len(occupied),
        unvisited_bytes=len(body)-len(occupied), linear_instruction_rows=len(linear_offsets),
        unknown_reached_instructions=0, overlapping_reached_instructions=0,
        recovered_windows=recovered, access_kind_counts=dict(sorted(counts.items())),
        access_instruction_count=sum(bool(r['access_kinds']) for r in seen.values()),
        extent_calls_in_graph=len(calls_in_graph), extent_calls_outside_graph=calls_outside_graph,
        external_transfers=sorted(external,key=lambda r:r['offset']),
        indirect_transfers=sorted(indirect,key=lambda r:r['offset']), terminals=sorted(terminals,key=lambda r:r['offset']),
        instructions=[seen[at] for at in sorted(seen)], edges=edges,
        scope='Selected linked process function direct-control overapproximation from listed roots, with call-return assumptions and explicit external/indirect boundaries. Address-bearing instructions are not classified as proven user accesses. No full-kernel/exception/callee graph, path feasibility, pointer authority, admission dominance, mapping lifetime or native coverage qualification')
    (build/'process-access-inventory.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['instructions','edges','terminals','external_transfers','indirect_transfers']}))


def verify_coverage(build):
    graph = json.loads((build/'process-access-inventory.json').read_text())
    paths = [build/name/'observations.json' for name in ['first-image-exit', 'source-user-extents']]
    paths = [path for path in paths if path.is_file()]
    assert len(paths) == 1
    observed = json.loads(paths[0].read_text())
    coverage = observed['extent_call_coverage']
    assert graph['passed'] and observed['passed'] and coverage['observation_passed']
    assert coverage['kernel_sha256'] == graph['kernel_sha256']
    assert coverage['inventory_sha256'] == graph['extent_inventory_sha256']
    offsets = {row['offset'] for row in graph['instructions']}
    assert {row['offset'] for row in coverage['reached']} <= offsets
    report = dict(passed=True, native_reached_calls=len(coverage['reached']),
        kernel_sha256=graph['kernel_sha256'], observation_sha256=digest(paths[0].read_bytes()),
        graph_sha256=digest((build/'process-access-inventory.json').read_bytes()),
        scope='Selected observed helper calls must belong to the listed direct-control graph; not native instruction/access coverage or proof of complete roots/callees')
    (build/'process-access-coverage.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    if sys.argv[1] == '--verify-coverage':
        verify_coverage(Path(sys.argv[2]).resolve())
    else:
        inventory(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())
