"""Encode the declared ABI cases independently of the Rust fixture encoder."""
import json
from pathlib import Path
import struct

ENTRY, PRE_EXEC = 0x4B4000, 0x4C0000
MAX = (1 << 64) - 1


def number(value):
    return int(value, 16) if isinstance(value, str) else value & MAX


def code(stage, witness_type):
    cases = json.loads((Path(__file__).parent / 'namespace-cases.json').read_text())
    assert cases['schema'] == 'topal-kernel-namespace-witness/1'
    w = witness_type()
    for operation in cases[stage]:
        name, args = operation[0], operation[1:]
        if name == 'call':
            w.call(number(args[0]), tuple(number(v) for v in args[1]), number(args[2]))
        elif name == 'store':
            w.store(number(args[0]), number(args[1]))
        elif name == 'check':
            w.memory(number(args[0]), number(args[1]))
        elif name == 'byte':
            w.raw('c60425')
            w.code.extend(struct.pack('<IB',number(args[0]),number(args[1])))
        elif name == 'word32':
            w.raw('c70425')
            w.code.extend(struct.pack('<II',number(args[0]),number(args[1])))
        elif name == 'check32':
            w.imm('a1',number(args[0]))
            w.expect(number(args[1]))
        elif name == 'text':
            text = args[1].encode() + b'\0'
            text += bytes((-len(text)) % 8)
            for offset in range(0,len(text),8):
                w.store(number(args[0])+offset,int.from_bytes(text[offset:offset+8],'little'))
        elif name == 'fill':
            w.imm('48bf',number(args[0]))
            w.imm('48b9',number(args[1]))
            w.imm('48b8',number(args[2]))
            w.raw('fcf3aa')
        elif name == 'time':
            w.imm('48bf',number(args[0]))
            w.imm('48b8',201)
            w.raw('0f054885c078fe4981fcab51000075fe4889c3')
            w.imm('48a1',number(args[0]))
            w.raw('4839d875fe')
            w.imm('48bf',0)
            w.imm('48b8',201)
            w.raw('0f054839d872fe4883c3054839d877fe')
        else:
            raise ValueError(name)
    w.imm('48bf',0x607000)
    w.imm('48b9',0x2000//8)
    w.raw('31c0fcf348ab')
    return w.finish()
