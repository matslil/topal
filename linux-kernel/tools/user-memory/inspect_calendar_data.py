"""Explain a saved linear decoder discrepancy without discarding its bytes."""
import json
import re
import struct

from calendar_data import graph_exclusion, guarded_entries, inspect as inspect_tables, require, site
from inspect_call_inventory import Elf, digest
from inspect_callback_bindings import relocation_entries


FUNCTION = 'topal_x86_systems_static_image_process_enter'


def context(text, body, base, address):
    rows = []
    for line in text.decode().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$', line)
        if match is None:
            continue
        at = int(match[1], 16)
        if not address - 24 <= at < address + 24:
            continue
        raw = bytes.fromhex(match[2])
        require(0 < len(raw) <= 15 and body[at - base:at - base + len(raw)] == raw, 'listing context linked bytes')
        rows.append(dict(address=at, bytes=raw.hex(), instruction=match[3].split('#', 1)[0].strip()))
    require(bool(rows), 'missing listing context')
    return rows


def origin_binding(elf, provider_path, origin_path, tables, selected):
    provider = Elf(provider_path)
    raw = origin_path.read_bytes()
    origins = json.loads(raw)
    symbol, original = provider.function(FUNCTION)
    linked_symbol, linked = elf.function(FUNCTION)
    require(origins['function'] == FUNCTION and origins['function_bytes'] == len(original) == len(linked), 'origin function binding')
    require(digest(original) == origins['function_sha256'], 'origin original-function identity')
    patched = bytearray(original)
    relocations = []
    for field, (target, kind, addend) in relocation_entries(provider, symbol['section']).items():
        offset = field - symbol['value']
        if not 0 <= offset < len(original):
            continue
        require(target['name'] in elf.symbols and elf.symbols[target['name']]['section'] != 0, 'origin external target binding')
        value = elf.symbols[target['name']]['value'] + addend
        if kind in [2, 4]:
            payload = struct.pack('<i', value - linked_symbol['value'] - offset)
        elif kind in [1, 10, 11]:
            payload = struct.pack({1: '<Q', 10: '<I', 11: '<i'}[kind], value)
        else:
            raise ValueError('unsupported original-function relocation')
        require(offset + len(payload) <= len(original), 'origin relocation outside function')
        patched[offset:offset + len(payload)] = payload
        relocations.append(dict(offset=offset, kind=kind, target=target['name'], addend=addend))
    require(patched == linked, 'whole original-to-linked reconstruction')
    cursor, selected_spans = 0, []
    for begin, end, index in origins['spans']:
        require(begin == cursor and begin < end <= len(original) and 0 <= index < len(origins['locations']), 'origin span closure')
        cursor = end
        if begin <= selected['address'] - linked_symbol['value'] < end:
            selected_spans.append(dict(begin=begin, end=end, location=origins['locations'][index]))
    require(cursor == len(original) and len(selected_spans) == 1, 'complete selected origin span')
    require(linked_symbol['value'] + origins['labels']['realtime-year-table'] == tables['year_start']
            and linked_symbol['value'] + origins['labels']['realtime-month-table'] == tables['month_start'], 'calendar label/payload agreement')
    return dict(provider_sha256=digest(provider.data), origins_sha256=digest(raw),
                relocations=relocations, selected_spans=selected_spans,
                whole_function_reconstructed=True,
                scope='Emitter metadata describes the writer; it is not execution, immutability or pointer authority')


def inspect(kernel, address, llvm, gnu, graph=None, provider=None, origins=None):
    elf = Elf(kernel)
    symbol, body = elf.function(FUNCTION)
    tables = inspect_tables(body, symbol['value'])
    selected = site(body, symbol['value'], address, tables)
    first, second = [context(value, body, symbol['value'], address) for value in [llvm, gnu]]
    llvm_site = [r for r in first if r['address'] == address]
    gnu_site = [r for r in second if r['address'] == address]
    gnu_containing = [r for r in second if r['address'] <= address < r['address'] + len(bytes.fromhex(r['bytes']))]
    require(len(llvm_site) == 1 and llvm_site[0]['bytes'] == 'cf'
            and llvm_site[0]['instruction'].split()[0] in ['iretd', 'iret'], 'recorded LLVM CF interpretation')
    require(not gnu_site and len(gnu_containing) == 1 and gnu_containing[0]['bytes'] == '00cf', 'recorded GNU containing interpretation')
    result = dict(passed=True, kernel_sha256=digest(elf.data), function=symbol,
                  linked_function_sha256=digest(body), tables=tables, site=selected,
                  llvm_listing_sha256=digest(llvm), gnu_listing_sha256=digest(gnu),
                  llvm_context=first, gnu_context=second,
                  executable_section_flags=elf.sections[symbol['section']][2],
                  hardware_NX_proved=False, runtime_immutability_proved=False,
                  linear_disagreement_retained=True, qualification_complete=False,
                  scope='Exact Gregorian payload and encoded year consumer explain the selected CF byte; no global entry, hardware NX, lifetime, CPU, native coverage or production proof')
    if graph is not None:
        raw = graph.read_bytes()
        value = json.loads(raw)
        require(value['passed'] and value['selected_edge_set_closed'] and not value['frontiers']
                and value['kernel_sha256'] == result['kernel_sha256'], 'conditional graph/kernel identity')
        result['conditional_graph'] = dict(graph_sha256=digest(raw), assumptions=value['assumptions'],
            **graph_exclusion(value['instructions'], elf, tables['year_start'], tables['end']))
        result['conditional_graph']['year_guard'] = guarded_entries(value, tables['year_consumer'])
    require((provider is None) == (origins is None), 'incomplete origin inputs')
    if provider is not None:
        result['origin_binding'] = origin_binding(elf, provider, origins, tables, selected)
    return result
