"""Additional explicit effects for the actual RTC leaf used before file input.

TOPAL-SYSTEMS-USER-EXTENT-001. Numeric memory expressions grant no ownership.
"""
from stat_instruction_effects import effects as stat_effects


def effects(raw):
    def result(writes=(),reads=(),stores=(),**extra):
        return dict(register_writes=set(writes),memory_reads=list(reads),memory_writes=list(stores),**extra)
    if raw==bytes.fromhex('654c8b2425a0030000'):
        return result([12],[("captured-gs",928,8)])
    if raw in [b'\xf8',b'\xf9']:
        return result(carry=bool(raw[0]&1))
    if len(raw)==2 and raw[0] in [0xb0,0xe4,0xe6]:
        if raw[0]==0xb0: return result([0],partial_register_write=0)
        return result([0] if raw[0]==0xe4 else (),io=('in' if raw[0]==0xe4 else 'out',raw[1]))
    if raw==bytes.fromhex('0fb6c0'):
        return result([0],zero_extend_byte=True)
    if len(raw)==7 and raw[:3]==bytes.fromhex('4c8d2d'):
        return result([13],rip_address=int.from_bytes(raw[3:],'little',signed=True))
    if len(raw)==7 and 0x48<=raw[0]<=0x4f and raw[1]==0x69:
        rex,_,modrm=raw[:3]
        assert modrm>>6==3 and not rex&2, 'unreviewed multiply addressing'
        return result([((modrm>>3)&7)+(8 if rex&4 else 0)],stack_pointer_operand=(modrm&7)==4 and not rex&1)
    row=stat_effects(raw)
    uses_stack=any(base==4 for base,_,_ in row['memory_reads']+row['memory_writes'])
    if len(raw)>=3 and 0x48<=raw[0]<=0x4f:
        rex,opcode,modrm=raw[:3]
        if opcode in [0x01,0x29,0x89,0x8b,0x39,0x3b]:
            uses_stack |= ((modrm>>3)&7)==4 and not rex&4
            uses_stack |= modrm>>6==3 and (modrm&7)==4 and not rex&1
    row['stack_pointer_operand']=uses_stack
    return row
