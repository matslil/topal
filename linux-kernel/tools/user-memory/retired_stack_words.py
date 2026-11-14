"""Exact private x86 stores to already-retired stack words.

TOPAL-SYSTEMS-USER-EXTENT-001. Numeric offsets and these store encodings grant
no stack ownership, lifetime, nonalias or permission authority.
"""
import struct


def encodings(words):
    assert type(words) is int and 1<=words<=255
    return [bytes.fromhex('48c78424')+struct.pack('<i',-8*index)+bytes(4)
        for index in range(1,words+1)]


def take(s,words):
    start=s.at
    rows=[]
    for raw in encodings(words):
        row=s.nodes[s.at]
        assert row['size']==len(raw) and row['bytes']==raw.hex(), 'retired stack word store'
        rows.append(row)
        s.take(raw.hex())
    return dict(start=start,end=s.at,words=words,bytes=words*8,
        relative_start=-words*8,relative_end=0,instructions=rows,
        register_writes=[],flags_unchanged=True,
        scope='Exact immediate-zero stores to the preceding private stack words; conditional on their established retired layout, owner/nonalias and stable writable lifetime')
