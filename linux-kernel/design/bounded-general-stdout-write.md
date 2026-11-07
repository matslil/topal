# Bounded general standard-output write

This record resolves `TK-DEC-071` as a systems-profile refinement. It replaces
the exact candidate, return site, length, and payload equality oracle of the
initial stdout qualification with a bounded Linux-compatible adapter for
caller-selected bytes in already-mapped user memory. It changes no core-
language rule and remains narrower than the pinned Linux `write(2)` interface.

## Decision

The initial-process personality accepts `write` when:

- descriptor is one;
- the return instruction lies in the finalized executable mapping;
- length is from one through 256 bytes;
- the complete half-open candidate extent is overflow-free and contained in a
  provider-known readable user mapping; and
- the user context is live in the dispatchable state.

Invalid descriptor or length returns the Linux-compatible `-EBADF` or
`-EINVAL` result selected by the personality. An invalid candidate returns
`-EFAULT` before output. A valid request is copied completely into at least
256 bytes of private kernel staging before the first console effect. The
provider publishes exactly the staged prefix in order and returns the requested
length. It does not compare application bytes with an embedded payload and
does not require one baked call-site address.

The existing initial ELF still writes `TOPAL_KERNEL_USER_WRITE_OK`, so the
physical evidence remains deterministic. Structural evidence must prove that
the provider has no copy of those 26 bytes and that output length comes from
the checked syscall argument.

## Isolation and failure contract

The size cap bounds kernel stack use and console monopolization. Range checks
must reject arithmetic wrap, zero length, cross-hole extents, non-user ranges,
and buffers extending past a readable mapping. Copy completion precedes every
output byte. A mapping change is impossible during this single-context profile;
later concurrent address-space mutation will require pinning or retry semantics.

No user pointer survives return. Unexpected faults after a range was proved
mapped remain fatal because they contradict the closed address-space state;
ordinary invalid candidates are rejected before dereference with `-EFAULT`.

## Cross-architecture pressure

The portable contract is a bounded standard-output write from a proved readable
user extent into private staging. It does not prescribe syscall numbers,
registers, instructions, page-table layout, copy-loop encoding, UART type, or
console polling. Each provider validates the return continuation and candidate
against its own address-space representation.

This increment does not admit descriptor tables, partial writes, blocking,
signals, restart, pipes, files, sockets, vectored I/O, asynchronous I/O, or
unbounded output. The 256-byte cap and closed initial-process mapping set are
explicit compatibility gaps.
