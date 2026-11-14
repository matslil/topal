"""Read back the selected owned-extent call census from ELF and disassembly.

TOPAL-SYSTEMS-USER-EXTENT-001. This checks calls to two helpers, not every user
load/store, dominance, admission results or complete production qualification.
All operations inspect host artifacts; this module never controls guest state.
"""
import hashlib
import json
from pathlib import Path
import re
import struct
import sys


ERASE_RETURN = bytes.fromhex('48 c7 44 24 f8 00 00 00 00')
EXTERNAL = {
    'runtime-readable-extent': 'topal_x86_user_extent_readable_v1',
    'runtime-writable-extent': 'topal_x86_user_extent_writable_v1',
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


class Elf:
    """Small independent ELF64 reader for these compiler-owned artifacts."""
    def __init__(self, path):
        self.data = Path(path).read_bytes()
        assert self.data[:7] == b'\x7fELF\x02\x01\x01'
        assert struct.unpack_from('<H', self.data, 18)[0] == 62
        self.kind = struct.unpack_from('<H', self.data, 16)[0]
        offset = struct.unpack_from('<Q', self.data, 40)[0]
        width, count, _ = struct.unpack_from('<3H', self.data, 58)
        assert width == 64 and count and offset + count * width <= len(self.data)
        self.sections = [struct.unpack_from('<II4QII2Q', self.data, offset + i * width)
                         for i in range(count)]
        self.tables = {}
        self.symbols = {}
        for index, section in enumerate(self.sections):
            if section[1] != 2:
                continue
            assert section[9] == 24 and section[5] % 24 == 0
            strings = self.section_bytes(section[6])
            symbols = []
            for at in range(section[4], section[4] + section[5], 24):
                name, info, _, owner, value, size = struct.unpack_from('<IBBHQQ', self.data, at)
                name = strings[name:strings.index(0, name)].decode()
                symbol = dict(name=name, info=info, section=owner, value=value, size=size)
                symbols.append(symbol)
                if name:
                    assert name not in self.symbols, name
                    self.symbols[name] = symbol
            self.tables[index] = symbols

    def section_bytes(self, index):
        section = self.sections[index]
        start, size = section[4:6]
        assert start + size <= len(self.data)
        return self.data[start:start + size]

    def function(self, name):
        symbol = self.symbols[name]
        assert symbol['info'] & 15 == 2 and symbol['size'] and symbol['section']
        section = self.sections[symbol['section']]
        offset = symbol['value'] - section[3]
        assert 0 <= offset and offset + symbol['size'] <= section[5]
        return symbol, self.section_bytes(symbol['section'])[offset:offset + symbol['size']]

    def relocations(self, owner):
        result = {}
        for section in self.sections:
            if section[1] != 4 or section[7] != owner:
                continue
            assert section[9] == 24 and section[5] % 24 == 0
            for at in range(section[4], section[4] + section[5], 24):
                offset, info, addend = struct.unpack_from('<QQq', self.data, at)
                assert offset not in result
                result[offset] = (self.tables[section[6]][info >> 32]['name'],
                                  info & 0xffffffff, addend)
        return result


def verify_calls(elf, inventory):
    """Check every recorded relative call, helper binding and return erasure."""
    assert inventory['schema'] == 'topal.owned-extent-call-inventory/1'
    assert set(inventory['helpers']) == set(EXTERNAL)
    symbol, body = elf.function(inventory['function'])
    assert len(body) == inventory['function_bytes']
    assert elf.kind in [1, 2]
    if elf.kind == 1:
        assert digest(body) == inventory['function_sha256']
        relocations = elf.relocations(symbol['section'])
    for helper, offset in inventory['helpers'].items():
        assert 0 <= offset <= len(body) - 5 and body[offset] == 0xe9
        if elf.kind == 1:
            name, kind, addend = relocations[symbol['value'] + offset + 1]
            assert (name, kind, addend) == (EXTERNAL[helper], 4, -4)
        else:
            target = symbol['value'] + offset + 5 + struct.unpack_from('<i', body, offset + 1)[0]
            assert target == elf.symbols[EXTERNAL[helper]]['value']
    offsets = [row['offset'] for row in inventory['calls']]
    assert offsets and offsets == sorted(set(offsets))
    for row in inventory['calls']:
        offset = row['offset']
        assert 0 <= offset <= len(body) - 5 - len(ERASE_RETURN)
        assert body[offset] == 0xe8
        target = offset + 5 + struct.unpack_from('<i', body, offset + 1)[0]
        assert target == inventory['helpers'][row['target']]
        assert body[offset + 5:offset + 5 + len(ERASE_RETURN)] == ERASE_RETURN
        assert row['nearest_labels'] and row['source_line'] > 0 and row['source_column'] > 0
    return symbol


def inspect(build):
    inventory = json.loads((build / 'extent-call-inventory.json').read_text())
    elf = Elf(build / 'provider.o')
    assert elf.kind == 1
    symbol = verify_calls(elf, inventory)
    helper_addresses = {symbol['value'] + offset: name
                        for name, offset in inventory['helpers'].items()}
    decoded = {}
    for line in (build / 'extent-call-disassembly.stdout.log').read_text().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+callq?\s+0x([0-9a-f]+)(?:\s|$)', line)
        if not match:
            continue
        address, target = (int(value, 16) for value in match.groups())
        if symbol['value'] <= address < symbol['value'] + symbol['size'] and target in helper_addresses:
            offset = address - symbol['value']
            assert offset not in decoded
            decoded[offset] = helper_addresses[target]
    assert decoded == {row['offset']: row['target'] for row in inventory['calls']}, (
        'decoded helper calls disagree with producer census', len(decoded), len(inventory['calls']))
    root = Path(__file__).resolve().parents[3]
    source_hashes = {}
    for row in inventory['calls']:
        source = root / row['source_file']
        assert source.resolve().is_relative_to(root / 'src')
        lines = source.read_text().splitlines()
        assert row['source_line'] <= len(lines)
        assert '.call(' in lines[row['source_line'] - 1]
        source_hashes[row['source_file']] = digest(source.read_bytes())
    report = dict(passed=True, call_count=len(decoded),
                  directions={name: sum(row['target'] == name for row in inventory['calls'])
                              for name in EXTERNAL},
                  source_sha256=source_hashes, provider_sha256=digest(elf.data),
                  inventory_sha256=digest((build / 'extent-call-inventory.json').read_bytes()),
                  disassembly_sha256=digest((build / 'extent-call-disassembly.stdout.log').read_bytes()),
                  scope=inventory['scope'])
    (build / 'extent-call-inspection.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    inspect(Path(sys.argv[1]).resolve())
