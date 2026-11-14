# Bounded initial `sched_yield`

This record resolves `TK-DEC-089` as a systems-profile refinement. It gives the
sole runnable initial thread a Linux-compatible scheduling point without
claiming multi-thread scheduling or moving policy into the machine provider.

## Decision

While the initial user context is live and its return continuation is
executable, Linux `sched_yield()` returns zero. The kernel observes that the
bounded profile has no other runnable user thread, preserves the same context
and all private process state, and returns through the ordinary syscall path.

The qualification program invokes the syscall, checks zero, and immediately
continues through another implemented call. The syscall receives its own
dispatch arm; it must not be satisfied by the unsupported-call fallback.

Returning `-ENOSYS` was rejected because the scheduling point is implementable
within the current profile. Fabricating a context switch or provider callback
was rejected because no peer exists and runnable selection is kernel policy.

The portable meaning is a successful yield with no other eligible context;
x86-64, AArch64, and RISC-V differ only in syscall ABI lowering. This increment
does not add a user runnable queue, priorities, affinity, multiple threads,
preemption, migration, SMP scheduling, or a general scheduler syscall family.
