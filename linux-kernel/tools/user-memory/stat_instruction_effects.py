"""Fail-closed effects for the reviewed 64-bit stat preparation subset.

TOPAL-SYSTEMS-USER-EXTENT-001. Address expressions below identify effects;
they do not grant ownership, bounds, nonaliasing or stable mapping lifetime.
"""


def effects(raw):
    def result(writes=(),reads=(),stores=(),**extra):
        return dict(register_writes=set(writes),memory_reads=list(reads),memory_writes=list(stores),**extra)
    if raw == b'\xfc': return result(clear_direction=True)
    if raw == b'\xc3': return result(control='return',stack_delta=8)
    if raw[:1] in [b'\xe8',b'\xe9']:
        assert len(raw) == 5
        return result(control='call' if raw[0] == 0xe8 else 'jump')
    if raw[:1] == b'\x0f':
        assert len(raw) == 6 and 0x80 <= raw[1] <= 0x8f
        return result(control='conditional')
    if raw in [bytes.fromhex(s) for s in ['f3a4','f348ab','f348af','f348a7']]:
        if raw == bytes.fromhex('f3a4'):
            return result([1,6,7],[(6,0,1)],[(7,0,1)],repeated='copy')
        if raw == bytes.fromhex('f348ab'):
            return result([1,7],(),[(7,0,8)],repeated='fill')
        if raw == bytes.fromhex('f348af'):
            return result([1,7],[(7,0,8)],repeated='scan')
        return result([1,6,7],[(6,0,8),(7,0,8)],repeated='compare')
    if len(raw) in [1,2]:
        high = len(raw) == 2
        opcode = raw[-1]
        assert not high or raw[0] == 0x41
        if 0x50 <= opcode <= 0x57:
            return result(stack_delta=-8,push=opcode-0x50+(8 if high else 0))
        if 0x58 <= opcode <= 0x5f:
            destination = opcode-0x58+(8 if high else 0)
            assert destination != 4, 'POP RSP does not have a fixed stack delta'
            return result([destination],stack_delta=8,pop=destination)
        raise AssertionError(('unsupported short stat instruction',raw.hex()))
    assert len(raw) >= 3 and 0x48 <= raw[0] <= 0x4f, 'stat instructions require REX.W'
    rex,opcode = raw[:2]
    if 0xb8 <= opcode <= 0xbf:
        assert len(raw) == 10 and not rex&6
        return result([opcode-0xb8+(8 if rex&1 else 0)])
    assert opcode in [0x01,0x29,0x89,0x8b,0x39,0x3b,0x81,0xc7,0xc1], ('unsupported stat opcode',raw.hex())
    modrm = raw[2]
    mode,group,rm = modrm>>6,(modrm>>3)&7,modrm&7
    register = group+(8 if rex&4 else 0)
    base,at = rm+(8 if rex&1 else 0),3
    if mode != 3:
        assert not rex&2, 'indexed stat addressing is unreviewed'
        if rm == 4:
            assert raw[at] == 0x24, 'only unindexed RSP/R12 SIB is reviewed'
            at += 1
        assert not (mode == 0 and rm == 5), 'RIP-relative stat addressing is unreviewed'
    displacement = 0
    if mode in [1,2]:
        width = 1 if mode == 1 else 4
        displacement = int.from_bytes(raw[at:at+width],'little',signed=True)
        at += width
    immediate = 4 if opcode in [0x81,0xc7] else 1 if opcode == 0xc1 else 0
    assert len(raw) == at+immediate
    memory = (base,displacement,8)
    if opcode == 0x89:
        return result([base]) if mode == 3 else result(stores=[memory])
    if opcode == 0x8b:
        return result([register],() if mode == 3 else [memory])
    if opcode in [0x39,0x3b]:
        return result(reads=() if mode == 3 else [memory])
    if opcode in [0x01,0x29]:
        assert mode == 3, 'memory arithmetic is outside the stat contract'
        return result([base])
    assert not rex&4, 'extended opcode groups are unreviewed'
    if opcode == 0xc7:
        assert group == 0 and mode != 3
        return result(stores=[memory])
    if opcode == 0xc1:
        assert group in [4,5] and mode == 3
        return result([base])
    assert opcode == 0x81 and group in [0,1,4,7]
    if group == 7: return result(reads=() if mode == 3 else [memory])
    assert mode == 3, 'memory arithmetic is outside the stat contract'
    return result([base])
