# Initial-thread clear-child-TID word

This record resolves `TK-DEC-076` as a systems-profile refinement. It provides
the first Linux thread-lifecycle registration used by libc without defining
multiple threads, cloning, or a general process table.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall number 218 to `set_tid_address`. The call
registers the current thread's clear-on-exit user word and returns its thread
ID. During thread exit Linux clears the word and wakes a futex waiter.

## Decision

The sealed initial-thread personality accepts a null candidate or one complete
writable four-byte extent in an already-owned user mapping. It requires a live
user context and a return continuation in the finalized executable mapping,
retains the candidate privately without dereference, and returns TID one.

On terminal `exit_group(0)`, a nonnull retained candidate is written with zero
under the existing fault-contained user-output mechanism. The private
candidate is cleared before the user context reaches terminal state and before
any user mapping is revoked. A clear fault fails closed. With no possible
waiter in the one-thread profile, no futex wake is emitted.

The qualification application initializes one data-mapping word to one,
registers it, and verifies the returned TID before terminal exit. Structural
evidence proves the terminal zero and private-state clear occur before the
terminal lifecycle transition.

## Alternatives considered

Returning TID one without retaining and clearing the word would satisfy only
the immediate syscall result and violate the exit contract. Introducing a
futex queue or clone-capable task table now would select synchronization,
ownership, scheduler, and lifetime policy before another thread can exist.

## Cross-architecture pressure

The portable contract is a checked optional user exit word tied to one live
thread, not the x86 syscall number or calling convention. Providers map their
native Linux ABI to the same lifecycle rule. No source-visible address,
register, instruction, task-structure field, or futex representation is
introduced.

This increment does not admit clone, multiple threads, futex waiters or wakes,
robust lists, process-shared synchronization, signals, exec, or general task
lifecycle management.
