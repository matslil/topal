"""Independent inactive x86 byte oracle; numeric inputs grant no authority.

TOPAL-SYSTEMS-KERNEL-IMAGE-TABLES-001. Non-image RW/NX/MMIO bytes describe
the draft provider template, not effective hardware permissions or attributes.
"""
from kernel_image_plan import Geometry, Region, partition, require

PAGE, LARGE, LIMIT, NX = 4096, 2 << 20, 1 << 30, 1 << 63


def geometry(layout):
    require((layout['minimum_granule'], layout['large_granule'], layout['address_limit']) ==
            (PAGE, LARGE, LIMIT), 'unsupported geometry')
    regions, end = [], 0
    for item in layout['spans']:
        a, b, role = item['start'], item['end'], item['role']
        require(type(a) is int and type(b) is int and end <= a < b <= LIMIT and a >= 0x200000,
                'invalid ordered image')
        require(a % PAGE == b % PAGE == 0 and not (a < 0x800000 and b > 0x400000),
                'unaligned or user-window image')
        if end < a:
            regions.append(Region(end, a, 'mutable'))
        regions.append(Region(a, b, role))
        end = b
    require(regions, 'empty image')
    if end < LIMIT:
        regions.append(Region(end, LIMIT, 'mutable'))
    return partition(regions, Geometry(PAGE, LARGE, LIMIT))


def page_count(layout):
    return 5 + len(geometry(layout).split_blocks)


def encode(layout, base, low=None):
    plan = geometry(layout)
    pages = 5 + len(plan.split_blocks)

    def outside(address, length):
        return (type(address) is int and address % PAGE == 0 and 0x1000000 <= address
                and address+length <= LIMIT and all(address >= p['end'] or address+length <= p['start']
                                                    for p in layout['spans']))

    require(outside(base, pages*PAGE), 'invalid bank extent')
    if low is not None:
        require(outside(low, PAGE) and (low+PAGE <= base or base+pages*PAGE <= low),
                'low leaf needs separate disjoint extent')
    words = [0] * (pages*512)
    words[0], words[512], words[515] = base+PAGE+7, base+2*PAGE+7, base+3*PAGE+3

    def leaf(address, large):
        rights = next(p.rights for p in plan.parts if p.start <= address < p.end)
        return address | 0x61 | (2 if rights.write else 0) | (0 if rights.execute else NX) | (0x80 if large else 0)

    splits = {block: i+5 for i, block in enumerate(plan.split_blocks)}
    for block in range(0, LIMIT, LARGE):
        slot = 1024 + block//LARGE
        if block in splits:
            table = splits[block]
            words[slot] = base+table*PAGE+3
            for index in range(512):
                words[table*512+index] = leaf(block+index*PAGE, False)
        else:
            words[slot] = leaf(block, True)
    words[3*512+502], words[3*512+503] = NX | 0xfec0009b, NX | 0xfee0009b
    if low is not None:
        words[1026], words[1027] = low | 7, base+4*PAGE+7
    return b''.join(word.to_bytes(8, 'little') for word in words)


def verify(layout, base, data, low=None):
    require(data == encode(layout, base, low), 'inactive bank bytes differ from independent template')
