# Bounded static ELF post-random `brk` growth

## Decision

After the post-random current-break query, the pinned x86-64 static executable
requests program-break growth from `0x005f7d80` to `0x00618d80`. The bounded
static-process profile shall admit this one exact monotonic request, provide
zero-filled anonymous backing for every newly admitted byte, update the
byte-granular current break, and return the requested value. The qualification
payload shall write and read the final admitted byte before terminal return.

The process reserve is refined from one data page into one opaque 34-frame
growth extent of zeroed anonymous data pages spanning the initial and
post-random growth. The x86-64 hierarchy's separately owned fifth frame is the
high leaf table because the range crosses the 6 MiB page-table boundary and
the bounded stack must remain disjoint above it. Source does not receive any
physical address or translation-frame identity.

The initial growth maps the first data frame and leaves the remaining data
frames zero and private. The post-random growth validates
the exact live process state, saved executable continuation, old break,
existing first leaf, absent remaining heap leaves, the already-published high
table, and the disjoint high stack leaf before publication. It installs eight
additional leaves in the existing low leaf table and 25 leaves in the high
table, invalidates the admitted virtual range, and only then reports success.

Terminal return removes all 34 anonymous leaves while retaining the high stack
leaf, resets the current break, clears and verifies the complete 34-frame
extent, and returns it to the construction allocator through the existing
affine inverse. No heap or translation state survives qualification.

## Portable boundary

Source names one bounded post-random program-break growth profile. The profile
owns an opaque anonymous growth extent and a provider-selected translation
allowance; it does not expose page counts, page-table levels, leaf flags,
physical addresses, invalidation instructions, syscall numbers, or registers.
Providers for AArch64, RISC-V, and capability architectures may use different
translation granules and hierarchy resources while preserving the same
byte-granular Linux result, zero-fill guarantee, exclusive ownership, and
complete rollback.

## Rejected alternatives

- Returning success without mapping was rejected because BusyBox immediately
  depends on usable heap memory.
- Treating the range as ELF payload was rejected because anonymous heap state
  is not file-backed executable content.
- Keeping the prior single page and relying on faults was rejected because the
  current kernel has no general page-fault allocator and would advertise
  unavailable memory.
- Exposing one x86 page-table frame as a portable language value was rejected
  because translation topology is provider policy.
- Introducing a general VMA tree, overcommit, shrink, or arbitrary `brk` was
  rejected as broader than the observed startup boundary.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact later growth and complete terminal rollback. It does not
support another nonzero request, shrink, lazy allocation, `mmap`, `munmap`,
`mprotect`, faults, signals, concurrency, fork/exec inheritance, BusyBox
redistribution, containers, or a prompt.
