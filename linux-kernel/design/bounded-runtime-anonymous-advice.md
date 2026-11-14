# Bounded runtime anonymous-page advice

This record resolves `TK-DEC-082` as a systems-profile refinement. It adds one
Linux-compatible reclaim hint to the existing opaque anonymous-page generation
without exposing backing addresses, zeroing mechanisms, or page-table formats
to Topal source.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall 28 to `madvise`. `MADV_DONTNEED` discards
private anonymous contents in the selected range; later access observes
zero-filled memory. The mapping and its protection remain in place.

## Decision

The bounded initial-process personality admits
`madvise(selected-address, 1..4096, MADV_DONTNEED)` only while the existing
anonymous mapping generation is active. The address must exactly match the
provider-selected page and the nonzero length must fit within that page.

On success the provider zeroes the complete backing frame through kernel
authority, preserves the active generation and its current none/read/read-write
non-executable protection, and returns zero. The operation is valid while the
user translation is inaccessible because provider authority, rather than user
access, performs the discard. Unsupported advice or ranges fail without
changing state or contents.

The qualification application mutates the page, makes it read-only and proves
the retained byte, makes it inaccessible, applies `MADV_DONTNEED`, restores
read-write access, and proves complete zeroing before another mutation.

## Alternatives considered

Treating advice as a silent no-op would falsely report reclaim semantics.
Unmapping the page would lose the active generation and permit overlapping
`mmap`. Deferring zeroing until a page fault would require a fault-driven VMA
service that this bounded resident-page increment does not yet possess.

## Cross-architecture pressure

The portable contract expresses discard, complete future zero observation,
generation and protection preservation, and provider authority. Syscall and
advice numbers, backing access, cache maintenance, translation representation,
and reclaim strategy remain provider details. AArch64 and RISC-V providers can
implement the same resident-page transition without adopting x86 mechanisms.

This increment does not admit other advice, partial ranges, lazy fault-backed
discard, reclaim pressure, multiple mappings, file backing, sharing, or a
general VMA service.
