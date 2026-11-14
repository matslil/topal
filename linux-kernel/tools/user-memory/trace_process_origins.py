"""Join selected linked potential accesses to exact test-only emission spans.

TOPAL-SYSTEMS-USER-EXTENT-001. Writer locations do not prove pointer provenance,
admission dominance, source-to-native conformance or a stable access lifetime.
"""
from bisect import bisect_right
from collections import Counter
import json
from pathlib import Path
import struct
import sys

from inspect_call_inventory import Elf, digest
from inspect_callback_bindings import relocation_entries


def inspect(build, source_root):
    origin_bytes = (build/'process-byte-origins.json').read_bytes()
    origins = json.loads(origin_bytes)
    assert origins['schema'] == 'topal.owned-process-byte-origins/1'
    graph_bytes = (build/'process-access-inventory.json').read_bytes()
    graph = json.loads(graph_bytes)
    assert graph['passed'] and graph['function'] == origins['function']
    provider, kernel = Elf(build/'provider.o'), Elf(build/'kernel.elf')
    original_symbol, original = provider.function(origins['function'])
    linked_symbol, linked = kernel.function(origins['function'])
    assert len(original) == len(linked) == origins['function_bytes']
    assert digest(original) == origins['function_sha256']
    assert digest(kernel.data) == graph['kernel_sha256']
    assert digest(linked) == graph['linked_function_sha256']
    patched = bytearray(original)
    relocations = []
    for field, (symbol, kind, addend) in relocation_entries(provider,original_symbol['section']).items():
        at = field-original_symbol['value']
        if not 0 <= at < len(original):
            continue
        assert symbol['name'] and kernel.symbols[symbol['name']]['section']
        target = kernel.symbols[symbol['name']]['value']
        if kind in [2,4]:
            raw = struct.pack('<i',target+addend-linked_symbol['value']-at)
        elif kind == 1:
            raw = struct.pack('<Q',target+addend)
        elif kind == 10:
            raw = struct.pack('<I',target+addend)
        elif kind == 11:
            raw = struct.pack('<i',target+addend)
        else:
            raise AssertionError(('unsupported process relocation',kind))
        assert at+len(raw) <= len(original)
        patched[at:at+len(raw)] = raw
        relocations.append(dict(offset=at,symbol=symbol['name'],kind=kind,addend=addend))
    assert patched == linked, 'complete provider process bytes must reconstruct the linked function'
    locations = origins['locations']
    spans = origins['spans']
    source_hashes, source_lines = {}, {}
    for row in locations:
        path = (source_root/row['file']).resolve()
        assert path.is_relative_to(source_root/'src/topal-compiler/src') and path.suffix == '.rs'
        if row['file'] not in source_lines:
            data = path.read_bytes()
            source_hashes[row['file']] = digest(data)
            source_lines[row['file']] = data.decode().splitlines()
        assert 1 <= row['line'] <= len(source_lines[row['file']]) and row['column'] > 0
    next_byte = 0
    for start,end,index in spans:
        assert start == next_byte and start < end <= len(original)
        assert 0 <= index < len(locations)
        next_byte = end
    assert next_byte == len(original)
    starts = [row[0] for row in spans]
    labels = {}
    for label,at in origins['labels'].items():
        assert 0 <= at <= len(original)
        labels.setdefault(at,[]).append(label)
    label_offsets = sorted(labels)
    accesses = []
    counts = Counter()
    kinds = {}
    for node in graph['instructions']:
        assert bytes.fromhex(node['bytes']) == linked[node['offset']:node['offset']+node['size']]
        if not node['access_kinds']:
            continue
        first = bisect_right(starts,node['offset'])-1
        assert first >= 0
        last_byte = node['offset']+node['size']
        included = []
        cursor = node['offset']
        index = first
        while cursor < last_byte:
            start,end,location = spans[index]
            assert start <= cursor < end
            included.append(dict(start=cursor,end=min(end,last_byte),location=location))
            cursor = min(end,last_byte)
            index += 1
        primary = locations[included[0]['location']]
        counts[primary['file']] += 1
        kinds.setdefault(primary['file'],Counter()).update(node['access_kinds'])
        label_index = bisect_right(label_offsets,node['offset'])-1
        nearest = sorted(labels[label_offsets[label_index]]) if label_index >= 0 else []
        accesses.append(dict(offset=node['offset'],size=node['size'],bytes=node['bytes'],
            instruction=node['instruction'],access_kinds=node['access_kinds'],
            byte_origins=included,nearest_labels=nearest))
    assert len(accesses) == graph['access_instruction_count']
    excerpts = [dict(row,source_line=source_lines[row['file']][row['line']-1]) for row in locations]
    report = dict(passed=True,function=origins['function'],function_bytes=len(original),
        provider_sha256=digest(provider.data),kernel_sha256=digest(kernel.data),
        linked_function_sha256=digest(linked),origin_sha256=digest(origin_bytes),
        process_graph_sha256=digest(graph_bytes),spans=len(spans),locations=excerpts,
        source_hashes=source_hashes,process_relocations=relocations,
        access_instructions=len(accesses),accesses=accesses,
        groups=[dict(file=path,primary_access_instructions=counts[path],
                     access_kind_counts=dict(sorted(kinds[path].items()))) for path in sorted(counts)],
        scope='Complete emission-span/source-location coverage for the selected process function and listed graph potential accesses, with whole-function object-to-linked byte reconstruction. Source locations identify writers, not pointer authority, transfer semantics, admission dominance or stable mapping lifetime. Other roots/callees/unvisited instructions and production qualification remain separate')
    (build/'process-access-origins.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(passed=True,function_bytes=len(original),spans=len(spans),
        source_locations=len(locations),source_files=len(source_hashes),
        access_instructions=len(accesses),access_origin_groups=len(counts),
        process_relocations=len(relocations),scope=report['scope'])))


if __name__ == '__main__':
    inspect(Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve())
