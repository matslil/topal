"""Bind the observed heap publisher's complete current native instruction layout.

The template is deliberately specific to this compiler-owned component. It
qualifies neither another code-generation layout nor arbitrary x86 instructions.
TOPAL-SYSTEMS-USER-EXTENT-001.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from inspect_call_inventory import Elf


TEMPLATE=bytes.fromhex('''
55 48 89 e5 41 57 41 56 41 55 41 54 53 50
49 89 f6 48 89 fb 48 39 fe 48 0f 47 de 49 89 d7
49 bd 07 00 00 00 00 00 00 80 45 31 e4
49 39 dc 73 2c 4c 89 e7 e8 00 00 00 00
4c 89 f9 4c 09 e9 4d 39 f4 ba 00 00 00 00
48 0f 43 ca 48 89 08 49 ff c4 49 81 c7 00 10 00 00
49 39 dc 72 d4 f0 83 0c 24 00 b8 00 70 5f 00
31 c9 48 39 d9 73 14 0f 1f 00
0f 01 38 48 ff c1 48 05 00 10 00 00 48 39 d9 72 ef
f0 83 0c 24 00 48 83 c4 08 5b 41 5c 41 5d 41 5e 41 5f 5d c3
''')


def digest(raw):return hashlib.sha256(raw).hexdigest()


def inspect(build):
    build=Path(build)
    elf=Elf(build/'kernel.elf')
    assert elf.kind==2
    symbol,body=elf.function('m.publish')
    assert len(TEMPLATE)==len(body)==149
    expected=bytearray(TEMPLATE)
    assert expected[51]==0xe8
    struct.pack_into('<i',expected,52,elf.symbols['m.pte']['value']-(symbol['value']+56))
    assert body==expected,'unqualified complete publisher instruction layout'
    source=Path(__file__).resolve().parents[3]/'src/topal-compiler/src/runtime/kernel_frame_memory.ll'
    text=source.read_text()
    start=text.index('define internal void @m.publish(')
    end=text.index('\n}',start)+2
    block=text[start:end]
    assert block in (build/'dispatch.ll').read_text(),'current source publication IR is not the compiled input'
    assert block.count('fence seq_cst')==2
    assert 'store volatile i64 %v,ptr %pte,align 8' in block
    assert 'call void asm sideeffect "invlpg ($0)"' in block
    process_name='topal_x86_systems_static_image_process_enter'
    process_symbol,process_body=elf.function(process_name)
    handoff=bytes.fromhex('65 48 8b 04 25 10 00 00 00 0f 22 d8')
    assert process_body.count(handoff)==1
    handoff_at=process_symbol['value']+process_body.index(handoff)
    return dict(passed=True,kernel_sha256=digest(elf.data),function_sha256=digest(body),
        function=symbol,source_sha256=digest(source.read_bytes()),dispatch_ir_sha256=digest((build/'dispatch.ll').read_bytes()),
        erase_entry=elf.symbols['m.erase']['value'],
        root_switch_sites=dict(before=handoff_at+9,after=handoff_at+12),
        root_switch_instruction_sha256=digest(handoff),
        inspector_sha256=digest(Path(__file__).read_bytes()),
        sites=dict(entry=symbol['value'],stored=symbol['value']+0x61,
            invalidated=symbol['value']+0x73,returning=symbol['value']+0x94),
        sites_meaning=dict(stored='After the first locked publication barrier and before any INVLPG iteration',
            invalidated='Immediately after INVLPG and before index/address increments',
            returning='After the second locked barrier and complete epilogue, before RET'),
        scope='Exact current 149-byte publisher layout, linked PTE-helper call and source/compiled IR binding; not arbitrary-code decoding, continuous CPU state, AP/DMA/async or complete compiler correctness')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=inspect(args.build)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
