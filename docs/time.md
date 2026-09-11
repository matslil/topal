# Time and deadlines

Revision `v0.2` makes time an explicit external observation. `Clock` is an
opaque provider-created resource. `Instant C`, `Deadline C`, and `Tick C S`
retain the identity of one clock; values from different clocks cannot be
compared or subtracted. `Duration U` is an exact typed quantity.

`now clock` performs `Read clock`. A monotonic provider guarantees that
successive observations do not decrease. Adding a duration to an instant is
fallible on range overflow, and subtracting same-clock instants produces a
duration.

A deadline stores one absolute instant and expiration policy. Passing a
relative duration to `with-timeout` constructs one deadline at entry, then
propagates that same value through nested requests and structured cancellation.
It never restarts the duration at each call. Timeout and completion race at
explicit protocol points; the winning observation is retained in the trace and
does not roll back an already completed effect.

`Periodic` produces ticks with a scheduled instant, observed release instant,
and sequence identity. Its `CatchUp`, `SkipLate`, or `Coalesce` policy defines
late delivery, avoiding drift from repeated relative sleeps. An interpreter may
use a deterministic logical clock or a recorded observation sequence. A
runtime clock adapter may supply monotonic external observations.

`WorstCaseExecution`, `ResponseWithin`, `ReleaseJitter`, and `DeadlineMet` are
recognized implementation-property shapes. A program may request or prefer
them before a function arrow, but no current component may verify physical
timing. A hard requirement therefore fails until the deferred architecture and
scheduler model or an approved timing provider establishes it.

Programmers pass clocks, instants, deadlines, and ticks and choose lateness
policy. Clock providers create observations. Compiler analysis can prove only
portable ordering and propagation; priority, affinity, preemption, interrupt
binding, and physical timing evidence remain deferred.
