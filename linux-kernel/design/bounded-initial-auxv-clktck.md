# Bounded initial `AT_CLKTCK` auxiliary-vector fact

This record resolves `TK-DEC-087` as a systems-profile refinement. It exposes
Linux's userspace accounting tick rate to libc without exposing a hardware
clock, scheduler timer, or target counter.

## Decision

The provider places one native `AT_CLKTCK` entry with value 100 after
`AT_SECURE` and before `AT_NULL` on the bounded initial user stack. The value is
the Linux native userspace `USER_HZ` contract, not the Q35 HPET period or a
kernel scheduling frequency.

Constructed code validates the exact ordered tag and value, validates the moved
terminator, consumes the enlarged bounded block, and continues with the existing
application proof. The executable-name string remains at its fixed address and
retains exact `argv[0]`/`AT_EXECFN` alias identity.

Omitting the fact was rejected because libc exposes the value through
`sysconf(_SC_CLK_TCK)`. Deriving it from HPET or a scheduler timer was rejected
because those frequencies have different semantics. A source-selected value was
rejected because process-entry ABI construction remains kernel policy.

The portable kernel meaning is a baseline-qualified userspace accounting rate;
each architecture provider lowers the native auxiliary-vector layout. This
increment does not add process accounting, CPU-time clocks, interval timers,
scheduler ticks, `times`, or a general auxiliary-vector builder.
