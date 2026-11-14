"""Fail-closed effects for the selected stack-local random extraction leaf.

TOPAL-SYSTEMS-USER-EXTENT-001. These expressions grant no memory owners.
"""


def effects(raw):
    def result(writes=(),reads=(),stores=(),**extra):
        return dict(register_writes=set(writes),memory_reads=list(reads),memory_writes=list(stores),**extra)
    if raw==b'\xc3': return result(control='return',stack_delta=8)
    if len(raw)==6 and raw[0]==15 and 0x80<=raw[1]<=0x8f:
        return result(control='conditional')
    if raw in [bytes.fromhex(s) for s in ['4885ff','4885f6','4839f7','4885c0']]:
        return result()
    if raw==bytes.fromhex('4989fa'): return result([10],argument_alias=(10,7))
    if raw==bytes.fromhex('4989f3'): return result([11],argument_alias=(11,6))
    if raw in [bytes.fromhex('4881ec80000000'),bytes.fromhex('4881c480000000')]:
        return result([4],scratch_delta=-128 if raw[2]==0xec else 128)
    if raw==bytes.fromhex('b801000000'): return result([0],constant=1)
    at,rex=0,0
    if raw and 0x40<=raw[0]<=0x4f: rex,at=raw[0],1
    assert len(raw)>at+1 and not rex&2, 'unreviewed extraction prefix or index'
    opcode,modrm=raw[at:at+2]
    at+=2
    mode,group,rm=modrm>>6,(modrm>>3)&7,modrm&7
    register=group+(8 if rex&4 else 0)
    base=rm+(8 if rex&1 else 0)
    width=8 if rex&8 else 4
    data_registers={0,1,2,8}
    if mode==3:
        assert width==4 and base in data_registers
        if opcode in [0x01,0x31]:
            assert register in data_registers and len(raw)==at
            return result([base],zero_register=base if opcode==0x31 and register==base else None)
        assert opcode==0xc1 and not rex&4 and group==0 and len(raw)==at+1
        assert raw[-1] in [7,8,12,16]
        return result([base])
    assert mode in [0,1], 'unreviewed extraction displacement'
    if rm==4:
        assert base==4 and raw[at]==0x24, 'unreviewed extraction SIB'
        at+=1
    assert base in [4,10,11], 'unreviewed extraction base'
    displacement=0
    if mode==1:
        displacement=int.from_bytes(raw[at:at+1],'little',signed=True)
        at+=1
    memory=(base,displacement,width)
    if opcode==0xc7:
        assert group==0 and not rex and len(raw)==at+4
        return result(stores=[memory],immediate_store=int.from_bytes(raw[at:],'little'))
    assert len(raw)==at and register in data_registers
    if opcode==0x89: return result(stores=[memory],stored_register=register)
    if opcode in [0x8b,0x03,0x0b]: return result([register],[memory])
    assert opcode==0x3b
    return result(reads=[memory])
