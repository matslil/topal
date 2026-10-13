# Bounded external data-segment and BSS contract

This record resolves `TK-DEC-062` as a systems-profile refinement. It extends
the single external executable object with one distinct non-executable data
load segment and an explicit memory-only zero tail. It reuses the existing
separate code/data mapping roles and changes no core-language rule.

## Decision

The sealed initial image may contain exactly two load segments: immutable
executable code and immutable non-executable data. Each segment must name a
nonoverlapping page-aligned user range and a bounded nonoverlapping file
extent. The data segment may have a memory size larger than its file size; the
difference is BSS and must be explicitly zeroed before write authority is
revoked.

The loader must copy code and initialized data from their respective extents
in the same validated external ELF resource. It must never derive data bytes
from provider immediates or assume allocator contents are zero. It must verify
initialized bytes and every BSS byte after read-only publication. Code remains
RO/X, data becomes RO/NX, and no writable-executable state is admitted.

```text
validate RX code segment + R data segment with file-size <= memory-size
  -> copy code extent -> revoke write -> verify -> publish execute
  -> copy initialized data -> explicitly zero BSS -> revoke write -> verify
  -> enter -> observe initialized data and zero tail -> exact return
  -> BSS witness while both mappings remain resident
```

After the external-copy witness and before the existing data witness, source
publishes `TOPAL_KERNEL_USER_IMAGE_BSS_OK`. Portable source does not observe
segment numbers, addresses, file offsets, sizes, zeroing instructions, page
tables, or resource bytes.

## Initial bounded x86-64 qualification

The first ELF64 file has three program headers. Its first `PT_LOAD` describes
the existing 848-byte `PF_R|PF_X` code image at `0x604000`. Its second
`PT_LOAD` describes one 8-byte `PF_R` initialized extent at file offset 4096
and user address `0x605000`, with a 16-byte memory size. Its third header
retains the non-executable GNU-stack policy. `AT_PHNUM` changes coherently from
two to three.

The initialized data begins with the existing `m` sentinel. The loader copies
the complete 8-byte file extent into the existing data-role mapping, writes
zero to the following 8-byte BSS extent, revokes write authority, and verifies
both qwords. Constructed user code observes the sentinel and all BSS bytes
before continuing. Any overflow, overlap, wrong permission, nonzero BSS,
partial copy, post-publication write, or metadata/auxv mismatch fails closed.

## Cross-architecture pressure

The portable contract is two role-distinct load extents plus explicit
zero-initialized memory beyond the data file extent. It does not prescribe ELF
class, program-header encoding, page size, qword stores, cache maintenance, or
target permissions. Providers must implement complete initialization and
publication using their target's memory and instruction-coherence rules.

This increment does not admit writable process data, more than two load
segments, arbitrary sizes or placements, relocations, interpreters, dynamic
linking, TLS, PIE, ASLR, demand paging, or a general allocator-backed process
image.
