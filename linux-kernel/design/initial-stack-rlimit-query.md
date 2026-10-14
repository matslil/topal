# Initial stack resource-limit query

This record resolves `TK-DEC-078` as a systems-profile refinement. It exposes
the initial Linux stack-limit fact needed by libc and command-line programs
without defining mutable resource accounting.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall 97 to `getrlimit` and syscall 302 to
`prlimit64`. Resource selector 3 is `RLIMIT_STACK`; the initial soft limit is
`_STK_LIM` (8 MiB) and the hard limit is `RLIM_INFINITY`. A native rlimit pair
contains two unsigned 64-bit values.

## Decision

The sealed initial-process personality supports:

- `getrlimit(RLIMIT_STACK, old-limit)`; and
- `prlimit64(0-or-self, RLIMIT_STACK, null, old-limit)`.

Both require a live context, finalized executable return continuation, and a
complete writable 16-byte destination. They copy the same private constant
pair under fault recovery and return zero. Invalid destinations return
`-EFAULT`; other resource selectors, a nonnull new-limit candidate, or another
PID return `-EINVAL` or `-ESRCH` as applicable without user access.

The qualification application invokes both forms into its writable stack and
checks the complete pair after each return.

## Alternatives considered

Returning a scalar would not match the Linux layout. Adding every resource at
once would require file-table, memory-accounting, scheduler, IPC, and security
policies that are not present. Accepting a new-limit pointer without enforcing
the result would provide false compatibility.

## Cross-architecture pressure

The portable contract is one immutable resource fact and fault-contained
layout publication. Syscall numbers, argument registers, native selector
values, and structure encoding remain ABI-provider details. Future providers
can map the same fact to their native Linux ABI without exposing target layout
to Topal source.

This increment does not admit other resources, mutation, enforcement,
inheritance, multiple processes, accounting, cgroups, capabilities, or a
general resource-limit service.
