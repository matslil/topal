# Bounded static ELF initial stack-limit query

## Decision

The bounded static process may extend its optional-`rseq` startup profile with
one exact immutable stack-limit query. After receiving `-ENOSYS` for `rseq`,
the process requests Linux `prlimit64(0, RLIMIT_STACK, null, old-limit)` into a
temporary complete 16-byte destination in its writable initial stack.

The provider validates the live startup state, exact query form, sealed
executable continuation, and complete writable destination. It publishes the
established initial-process pair—8 MiB soft limit and `RLIM_INFINITY` hard
limit—returns zero, and retains no pointer or mutable resource state. The
payload validates both native words, clears its temporary stack allocation,
and continues to terminal dispatch.

Source names an immutable initial stack-limit fact, not a syscall number,
selector, PID encoding, address, field layout, or numeric limits. The portable
element remains the checked query contract in `TOPAL-SYSTEMS-USER-RLIMIT-STACK-001`;
the architecture provider owns native lowering.

This systems-only refinement does not add limit mutation or enforcement,
another resource, accounting, inheritance, multiple processes, cgroups,
BusyBox redistribution, or a prompt.
