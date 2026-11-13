"""Identify the selected RTC constants and their encoded read consumer.

Bytes in an executable section can also be data. This reader establishes the
selected payload/consumer shape, not hardware NX, CFI or global unreachability.
"""
from datetime import date
import hashlib
import struct


YEAR_FIRST, YEAR_END = 1970, 2262
YEAR_BYTES = (YEAR_END - YEAR_FIRST) * 16
MONTH_BYTES = 12 * 16


def require(value, reason):
    if not value:
        raise ValueError(reason)


def calendar_payload():
    """Use the standard Gregorian date oracle, independent of Rust leap()."""
    epoch = date(1970, 1, 1)
    years = b''.join(struct.pack('<QQ', (date(year, 1, 1) - epoch).days,
                                 int((date(year + 1, 1, 1) - date(year, 1, 1)).days == 366))
                     for year in range(YEAR_FIRST, YEAR_END))
    months = b''.join(struct.pack('<QQ', (date(1970, month, 1) - epoch).days,
                                  (date(1970 + (month == 12), month % 12 + 1, 1) - date(1970, month, 1)).days)
                      for month in range(1, 13))
    return years + months


def rip_references(body, base, target):
    result = []
    for at in range(len(body) - 6):
        if body[at:at + 3] == bytes.fromhex('4c8d2d'):
            if base + at + 7 + struct.unpack_from('<i', body, at + 3)[0] == target:
                result.append(at)
    return result


def year_guard(body, base, lea, table, end):
    begin = lea - 37
    require(begin >= 0 and lea + 24 <= len(body), 'complete year consumer')
    raw = body[begin:lea]
    for offset, expected in [
        (0, bytes.fromhex('4881f8') + struct.pack('<I', YEAR_FIRST)),
        (7, bytes.fromhex('0f82')),
        (13, bytes.fromhex('4881f8') + struct.pack('<I', YEAR_END - 1)),
        (20, bytes.fromhex('0f87')),
        (26, bytes.fromhex('4881c0') + struct.pack('<i', -YEAR_FIRST)),
        (33, bytes.fromhex('48c1e004')),
    ]:
        require(raw[offset:offset + len(expected)] == expected, 'year consumer bound/index encoding')
    targets = [base + begin + stop + struct.unpack_from('<i', raw, displacement)[0]
               for stop, displacement in [(13, 9), (26, 22)]]
    require(targets[0] == targets[1] and base <= targets[0] < base + len(body), 'common year rejection target')
    require(not table <= targets[0] < end, 'year rejection targets data')
    require(body[lea + 7:lea + 24] == bytes.fromhex('4901c54d8bbd000000004d8bb508000000'),
            'year consumer reads both qwords')
    return dict(address=base + begin, lea=base + lea, rejection=targets[0],
                first_year=YEAR_FIRST, last_year=YEAR_END - 1, stride=16,
                maximum_row_offset=(YEAR_END - YEAR_FIRST - 1) * 16,
                bytes=body[begin:lea + 24].hex())


def inspect(body, base):
    payload = calendar_payload()
    at = body.find(payload)
    require(at >= 0, 'complete Gregorian calendar payload absent')
    require(body.find(payload, at + 1) == -1, 'ambiguous Gregorian calendar payload')
    year, month, end = base + at, base + at + YEAR_BYTES, base + at + len(payload)
    years = rip_references(body, base, year)
    months = rip_references(body, base, month)
    require(len(years) == len(months) == 1, 'unique encoded year/month references')
    consumer = year_guard(body, base, years[0], year, end)
    require(not year <= base + months[0] < end, 'month reference lies in data')
    return dict(year_start=year, year_end=month, month_start=month, end=end,
                year_rows=YEAR_END - YEAR_FIRST, month_rows=12, bytes=len(payload),
                payload_sha256=hashlib.sha256(payload).hexdigest(),
                year_consumer=consumer, month_reference=base + months[0])


def site(body, base, address, tables):
    require(tables['year_start'] <= address < tables['year_end'], 'site outside calendar year data')
    offset = address - tables['year_start']
    row, field = divmod(offset, 16)
    require(field == 0 and body[address - base] == 0xcf, 'selected CF byte is not day-field start')
    days, leap = struct.unpack_from('<QQ', body, tables['year_start'] - base + row * 16)
    return dict(address=address, year=YEAR_FIRST + row, days=days, leap=leap,
                row_index=row, row_bytes=body[address - base:address - base + 16].hex())


def graph_exclusion(rows, elf, start, end):
    """Check declared graph rows only, without promoting its entry assumptions."""
    sections = [(s[3], elf.section_bytes(i)) for i, s in enumerate(elf.sections) if s[2] & 4 and s[5]]
    count = 0
    for row in rows:
        address, raw = row['address'], bytes.fromhex(row['bytes'])
        require(row['size'] == len(raw) and len(raw) > 0, 'graph row size')
        containing = [(a, b) for a, b in sections if a <= address and address + len(raw) <= a + len(b)]
        require(len(containing) == 1, 'graph row outside one executable section')
        origin, data = containing[0]
        require(data[address - origin:address - origin + len(raw)] == raw, 'graph and linked bytes differ')
        require(not (address < end and start < address + len(raw)), 'declared graph reaches calendar data')
        count += 1
    require(count > 0, 'empty declared graph')
    return dict(checked_instructions=count, overlaps=0, global_unreachability_proved=False)


def guarded_entries(graph, consumer):
    begin, end = consumer['address'], consumer['lea'] + 24
    sizes = dict(zip([begin + i for i in [0, 7, 13, 20, 26, 33, 37, 44, 47, 54]],
                     [7, 6, 7, 6, 7, 4, 7, 3, 7, 7], strict=True))
    rows = {row['address']: row for row in graph['instructions']}
    require(all(address in rows and rows[address]['size'] == size for address, size in sizes.items()),
            'conditional graph lacks complete year guard/reads')
    declared = [r['address'] for r in graph['roots']] + [r['target'] for r in graph['callbacks'].values()]
    require(not any(begin < address < end for address in declared), 'declared entry bypasses year guard')
    checked = 0
    for edge in graph['edges']:
        if begin < edge['target'] < end:
            require(edge['kind'] == 'fallthrough' and edge['source'] in sizes
                    and edge['source'] + sizes[edge['source']] == edge['target'],
                    'declared edge bypasses year guard')
            checked += 1
    require(checked == len(sizes) - 1, 'missing or duplicate year fallthrough edge')
    return dict(instructions=len(sizes), checked_inner_edges=checked,
                declared_bypass_absent=True, complete_entry_set_proved=False)
