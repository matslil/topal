# Bounded runtime anonymous-page protection

This record resolves `TK-DEC-081` as a systems-profile refinement. It extends
the existing opaque anonymous-page reservation with Linux-compatible runtime
protection changes without exposing a target page-table format or protection
encoding to Topal source.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall 10 to `mprotect`. A successful call changes
access permissions for every page intersecting its page-aligned range and
requires the start address to be page aligned. Unsupported protection bits,
unmapped ranges, and invalid addresses fail without changing permissions.

## Decision

The bounded initial-process personality admits
`mprotect(selected-address, 1..4096, protection)` only while the existing
anonymous reservation is active. The supported protection set is `PROT_NONE`,
`PROT_READ`, or `PROT_READ | PROT_WRITE`; executable and other bits are
rejected. The selected address must exactly match the provider-owned anonymous
page and the nonzero length must fit within that page.

On success the provider atomically replaces the active translation with the
requested non-executable state, performs ordered target-qualified completion,
records the new private protection state, and returns zero. `PROT_NONE` leaves
the backing reservation active but makes its translation non-present;
subsequent `mprotect` may restore read or read-write access without clearing
the page. `munmap` revokes any active protection state, including
`PROT_NONE`, and a later `mmap` begins a fresh zero-filled generation.

The qualification application maps the page read-write, mutates it, changes
it to read-only and observes the retained byte, changes it to `PROT_NONE`,
restores read-write access and observes the same byte, then unmaps it. Its
second mapping still proves generation zero-fill and final unmap.

## Alternatives considered

Treating `PROT_NONE` as `munmap` would lose the live mapping generation and
would incorrectly permit a second `mmap`. Leaving a present supervisor-only
translation would conflate Linux user inaccessibility with provider kernel
access policy. Adding a portable page-table protection operation would expose
an architecture mechanism where the source lifecycle needs only an opaque
reservation.

## Cross-architecture pressure

The portable contract expresses only active-generation identity, the abstract
none/read/read-write permission states, non-executability, and ordered
completion. Syscall numbers and native bits, selected address, page-table
entries, break-before-make rules, barriers, and translation invalidation remain
provider details. AArch64 and RISC-V providers can implement the same states
using their own translation regimes.

This increment does not admit executable permission, growth flags, protection
keys, multiple pages or mappings, partial-range splitting, file backing,
shared memory, page-fault recovery, concurrent edits, or a general VMA service.
