# Bounded runtime anonymous page

This record resolves `TK-DEC-080` as a systems-profile refinement. It adds the
first Linux runtime mapping without exposing virtual addresses, page-table
formats, invalidation instructions, or allocation policy to Topal source.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall 9 to `mmap` and syscall 11 to `munmap`.
Anonymous private mappings do not use a file, are initially zero-filled, and
round a nonzero length to page boundaries. A successful `munmap` removes every
covered mapping and succeeds even when part or all of the selected valid range
is already unmapped.

## Decision

Before initial user execution, source code allocates one frame and consumes it
with `user address space reserve anonymous`. The operation returns an opaque
affine reservation tied to the exact address-space and frame identities. It
does not publish a user translation, grant user access, or reveal a numeric
address. Only one reservation exists.

The initial-process personality admits
`mmap(null, 1..4096, PROT_READ|PROT_WRITE,
MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)`. When the reservation is non-present, the
provider zeroes the complete frame through kernel authority, publishes one
RW/NX user mapping with ordered target completion, records it active, and
returns its provider-selected page-aligned address. A second activation while
active returns `-ENOMEM`; unsupported arguments return `-EINVAL` without
changing the reservation.

`munmap(selected-address, 1..4096)` revokes the translation with ordered target
completion and returns zero. The same valid call when already non-present is a
successful no-op. A later admitted `mmap` creates a fresh generation and
zeroes the complete page again. Terminal process completion revokes an active
generation before returning to source.

After user completion, `user address space release anonymous` requires the
reservation to be non-present, consumes it, and returns the exact original
frame for ordinary release. The address space remains separately owned and
continues through its existing image teardown and reclaim lifecycle.

The qualification application maps the page, proves initial zero, mutates and
observes a byte, unmaps it, maps it again, proves zero was restored, and unmaps
it before status-zero exit.

## Alternatives considered

Pre-mapping a page and merely returning its address would make it accessible
before `mmap`. Allocating from inside the architecture provider would bypass
affine source ownership. Extending `brk` would not qualify the separate Linux
mapping ABI. A general VMA tree would prematurely decide overlap, splitting,
concurrency, accounting, overcommit, file backing, fault, and rollback policy.

## Cross-architecture pressure

The portable elements express reservation, non-publication, exclusive runtime
activation, complete revocation, generation freshness, and exact backing
return. Syscall numbers and registers, native flags, selected virtual address,
page-table representation, publication barriers, and invalidation mechanisms
remain provider details. AArch64 and RISC-V providers can implement the same
lifecycle with their own translation regimes.

This increment does not admit multiple pages or mappings, caller-selected
addresses, executable or read-only protections, file backing, `mremap`,
`madvise`, page faults, overcommit, sharing, fork, concurrent edits, arbitrary
user-copy use of the new mapping, or a general VMA service.
