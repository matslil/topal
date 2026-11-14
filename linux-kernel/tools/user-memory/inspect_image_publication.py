"""Bind exact linked RELRO and exec image-leaf publication instructions.

TOPAL-SYSTEMS-USER-EXTENT-001. Only the two selected compiler-owned layouts
are recognized. This is read-only artifact inspection, not an x86 decoder.
"""
import argparse
import json
from pathlib import Path
import struct

from inspect_call_inventory import Elf,digest


def inspect(build):
    build=Path(build)
    elf=Elf(build/'kernel.elf')
    symbol,body=elf.function('topal_x86_systems_static_image_process_enter')
    origins=json.loads((build/'process-byte-origins.json').read_bytes())
    graph=json.loads((build/'process-access-inventory.json').read_bytes())
    assert origins['function']==graph['function']==symbol['name']
    assert origins['function_bytes']==graph['function_bytes']==len(body)
    assert graph['kernel_sha256']==digest(elf.data) and graph['linked_function_sha256']==digest(body)
    fatal=origins['labels']['fatal']
    assert body[fatal:fatal+2]==b'\xfa\xf4','expected selected invariant halt'
    profiles={}
    def qualify(kind,entry,expected,sites,count,has_final_barrier):
        assert body[entry:entry+len(expected)]==expected,'unqualified '+kind+' publication layout'
        assert all(site in {r['offset'] for r in graph['instructions']} for site in sites.values())
        profiles[kind]=dict(entry=symbol['value']+entry,bytes=len(expected),sha256=digest(expected),
            count=count,final_barrier=has_final_barrier,
            sites={name:symbol['value']+site for name,site in sites.items()})
    def loop(count,final):
        result=bytearray(bytes.fromhex('0f ae f0 48 b8')+struct.pack('<Q',0x5e5000)+b'\xb9'+struct.pack('<I',count))
        invalidated=len(result)+3
        result+=bytes.fromhex('0f 01 38 48 05 00 10 00 00 ff c9 0f 85 ef ff ff ff')
        if final:result+=bytes.fromhex('0f ae f0')
        return result,invalidated
    entry=origins['labels']['static-process-relro-mprotect-leaves']
    relro=bytearray(bytes.fromhex('49 8b 00 48 83 e0 9f 4c 89 d2 48 83 ca 07 48 0f ba ea 3f 48 39 d0 0f 85'))
    relro+=struct.pack('<i',fatal-(entry+len(relro)+4))
    relro+=bytes.fromhex('4c 89 d0 48 83 c8 05 48 0f ba e8 3f 49 89 00 49 81 c2 00 10 00 00 49 83 c0 08 ff c9 0f 85')
    relro+=struct.pack('<i',-(len(relro)+4))
    loop_at=entry+len(relro)
    suffix,post=loop(8,True)
    relro+=suffix
    qualify('relro',entry,relro,dict(entry=entry,stored=loop_at+3,
        invalidated=loop_at+post,finished=entry+len(relro)),8,True)
    reset=bytearray(bytes.fromhex('65 48 8b 14 25 70 00 00 00'))
    for offset in range(0xf28,0xfb8,8):
        reset+=bytes.fromhex('48 8b 82')+struct.pack('<I',offset)
        reset+=bytes.fromhex('48 25 00 f0 ff ff 49 b9 07 00 00 00 00 00 00 80 4c 09 c8 48 89 82')+struct.pack('<I',offset)
    assert body.count(bytes(reset))==1,'unique exec reset required'
    entry=body.index(bytes(reset))
    loop_at=entry+len(reset)
    suffix,post=loop(18,False)
    reset+=suffix
    qualify('exec',entry,reset,dict(entry=entry,stored=loop_at+3,
        invalidated=loop_at+post,finished=entry+len(reset)),18,False)
    selected={r['offset'] for r in graph['instructions'] if r['mnemonic']=='invlpg'}
    assert selected=={row['sites']['invalidated']-symbol['value']-3 for row in profiles.values()},'other reached process invalidations require qualification'
    return dict(passed=True,kernel_sha256=digest(elf.data),function_sha256=digest(body),
        origins_sha256=digest((build/'process-byte-origins.json').read_bytes()),
        graph_sha256=digest((build/'process-access-inventory.json').read_bytes()),
        inspector_sha256=digest(Path(__file__).read_bytes()),profiles=profiles,
        scope='Exact selected RELRO store/check/invalidation/barrier loop and exec unrolled image-leaf reset/invalidation loop. No complete source producer, alternate entry, other callee, AP/DMA/asynchronous or continuous-mode qualification')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=inspect(args.build)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
