# Bounded static ELF post-random `brk` page alignment

## Decision

After the cross-boundary growth to `0x00618d80`, the pinned x86-64 static
executable requests one further program-break growth to `0x00619000`. The
bounded static-process profile shall admit this exact monotonic request, update
the byte-granular current break, return the requested value, and permit a write
and read at the final newly admitted byte `0x00618fff`.

The existing opaque 34-frame anonymous extent already backs the entire newly
admitted interval. The provider shall therefore allocate no storage and change
no translation entry. Before updating the break, it shall validate the exact
live post-growth state, saved executable continuation, old current break,
complete heap-leaf set, disjoint stack leaf, anonymous-extent identity, and
reusable-random service state. Failure shall preserve every process resource
and the prior break.

Terminal return shall require the aligned-break state, reset the current break,
remove all 34 anonymous leaves, clear and verify the complete extent, and
return it through the existing affine inverse. No heap, random-service, or
translation state survives qualification.

## Portable boundary

Source names one bounded byte-granular growth within an already-backed
anonymous reserve. It does not expose the x86-64 page size, alignment value,
virtual address, syscall number, register convention, leaf topology, or the
fact that this target needs no additional translation edit. AArch64, RISC-V,
and capability-system providers may use different granules or mapping objects
while preserving the exact logical transition, usable zero-filled interval,
exclusive ownership, and complete rollback.

## Rejected alternatives

- Returning the requested value without updating current-break state was
  rejected because later Linux-visible behavior must observe the new break.
- Treating the request as a complete no-op was rejected because the bytes from
  the old break through the new break become newly admitted process memory even
  though their physical pages are already present.
- Allocating another page or rebuilding the high leaf table was rejected
  because the existing anonymous extent already provides all required backing.
- Introducing arbitrary growth, shrink, or a general VMA service was rejected
  as broader than the observed startup boundary.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact later growth within the existing anonymous reserve and
preserves complete terminal rollback. It does not support another request,
shrink, lazy allocation, `mmap`, `munmap`, `mprotect`, faults, signals,
concurrency, fork/exec inheritance, BusyBox redistribution, containers, or a
prompt.
