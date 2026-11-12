"""ELF scope controls for complete native-artifact validation, not guest tests.

TOPAL-SYSTEMS-USER-EXTENT-001: a name cannot substitute for symbol identity.
The small ELF64 fixtures encode distinct local definitions and exact indices.
"""
from pathlib import Path
import struct
import tempfile
import unittest

from inspect_call_inventory import Elf


def elf_bytes(symbols, relocation=None):
    strings=bytearray(b'\0');entries=[bytes(24)]
    for name,binding,section,value in symbols:
        offset=len(strings);strings.extend(name.encode()+b'\0')
        entries.append(struct.pack('<IBBHQQ',offset,(binding<<4)|2,0,section,value,int(section!=0)))
    raw=bytearray(64);sections=[(0,)*10]
    def section(kind,flags,address,data,link=0,info=0,alignment=1,width=0):
        while len(raw)%alignment:raw.append(0)
        offset=len(raw);raw.extend(data)
        sections.append((0,kind,flags,address,offset,len(data),link,info,alignment,width))
    section(1,6,0,b'\x90\xc3\x90\xc3',alignment=16)
    section(3,0,0,strings)
    first_external=next((i+1 for i,s in enumerate(symbols) if s[1]),len(entries))
    section(2,0,0,b''.join(entries),link=2,info=first_external,alignment=8,width=24)
    if relocation is not None:
        section(4,0,0,struct.pack('<QQq',0,(relocation<<32)|4,-4),link=3,info=1,alignment=8,width=24)
    while len(raw)%8:raw.append(0)
    shoff=len(raw)
    for row in sections:raw.extend(struct.pack('<II4QII2Q',*row))
    ident=b'\x7fELF\x02\x01\x01'+bytes(9)
    struct.pack_into('<16sHHIQQQIHHHHHH',raw,0,ident,1,62,1,0,0,shoff,0,64,0,0,64,len(sections),0)
    return raw


class ElfSymbolScopes(unittest.TestCase):
    def read(self,symbols,relocation=None):
        temporary=tempfile.TemporaryDirectory();self.addCleanup(temporary.cleanup)
        path=Path(temporary.name)/'fixture.o';path.write_bytes(elf_bytes(symbols,relocation))
        return Elf(path)

    def test_distinct_local_definitions_remain_enumerable_but_not_name_resolvable(self):
        elf=self.read([('p.copy',0,1,0),('p.copy',0,1,1),('p.copy',0,1,0)])
        self.assertEqual([s['value'] for s in elf.defined_symbols],[0,1])
        self.assertNotIn('p.copy',elf.symbols)
        with self.assertRaisesRegex(AssertionError,'ambiguous local'):elf.function('p.copy')
        self.assertEqual([s['value'] for s in elf.tables[3][1:]],[0,1,0])

    def test_global_definition_has_named_scope_without_hiding_local_definitions(self):
        elf=self.read([('same',0,1,0),('same',1,1,2)])
        self.assertEqual(elf.function('same')[0]['value'],2)
        self.assertEqual({s['value'] for s in elf.defined_symbols},{0,2})

    def test_local_definition_cannot_satisfy_an_undefined_global_name(self):
        elf=self.read([('same',0,1,0),('same',1,0,0)])
        self.assertEqual(elf.symbols['same']['section'],0)
        with self.assertRaises(AssertionError):elf.function('same')

    def test_undefined_global_declaration_prefers_its_global_definition(self):
        elf=self.read([('same',1,0,0),('same',1,1,2)])
        self.assertEqual(elf.function('same')[0]['value'],2)

    def test_conflicting_global_definitions_are_still_rejected(self):
        with self.assertRaisesRegex(AssertionError,'conflicting external'):
            self.read([('same',1,1,0),('same',1,1,1)])

    def test_identical_definitions_are_deduplicated_for_enumeration(self):
        elf=self.read([('same',0,1,0),('same',0,1,0)])
        self.assertEqual(len(elf.defined_symbols),1)
        self.assertEqual(elf.function('same')[1],b'\x90')

    def test_named_relocation_cannot_conflate_local_and_global_targets(self):
        elf=self.read([('same',0,1,0),('same',1,1,2)],relocation=1)
        self.assertEqual(elf.tables[3][1]['value'],0)
        with self.assertRaisesRegex(AssertionError,'ambiguous named relocation'):elf.relocations(1)
        elf=self.read([('same',0,1,0),('same',1,1,2)],relocation=2)
        self.assertEqual(elf.relocations(1),{0:('same',4,-4)})


if __name__=='__main__':unittest.main()
