# Initial-thread empty robust-list registration

This record resolves `TK-DEC-077` as a systems-profile refinement. It admits
the empty robust-list registration performed by libc while deferring futex and
multi-thread recovery policy.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall number 273 to `set_robust_list`. The native
`robust_list_head` is 24 bytes: a list-head link, signed futex offset, and
pending-operation pointer. Registration requires that exact size. Thread exit
walks the list and pending entry to mark owner death and wake waiters.

## Decision

The sealed initial-thread personality accepts a complete readable 24-byte
candidate in an already-owned user mapping and exact native length 24. It
requires a live user context and finalized executable return continuation,
retains the candidate privately without registration-time dereference, and
returns zero.

At terminal `exit_group(0)`, the provider copies the complete head into private
kernel staging under fault recovery. It requires the list link to point to the
head itself and the pending pointer to be null. The futex offset is retained as
opaque data because no entry is visited. Only after validation does it clear
private candidate state and continue terminal disposition. Any copy fault or
nonempty state fails closed.

The qualification application builds an empty head in its RW/NX data mapping,
registers it, and checks the result. Structural evidence proves the complete
terminal copy, empty-state checks, and state clearing.

## Alternatives considered

Returning success without retaining and inspecting the head would miss the
exit contract. Returning `-ENOSYS` would allow some libc fallbacks but would
not advance compatibility. A general robust-list walk would require futex-word
layout, atomic owner-death updates, waiter lookup, wake ordering, and fault
rules that have no implementation foundation yet.

## Cross-architecture pressure

The semantic contract is one native Linux robust-list head tied to a live
thread; each ABI provider owns its syscall number and native layout adapter.
The systems profile exposes neither a user address nor structure offsets to
Topal source. Portable kernel services later consume typed entry and futex
views rather than raw ABI layouts.

This increment does not admit nonempty lists, pending operations, futex owner-
death mutation, wakes, multiple threads, clone, shared synchronization,
signals, exec, or general robust-list lookup.
