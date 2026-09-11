# Time and static-rate dataflow semantics

## Formal text

### TOPAL-TIME-CLOCK-001 — Clock identity and observations

`Clock` SHALL be an opaque provider-created resource. `Instant C`, `Deadline C`,
and `Tick C S` SHALL retain the exact clock identity. Instants from different
clocks SHALL NOT compare or subtract. `now C` SHALL perform `Read C`; a
monotonic provider SHALL reject or diagnose a decreasing observation.

### TOPAL-TIME-DEADLINE-001 — Absolute deadline propagation

A relative timeout SHALL construct one absolute deadline at entry. Nested
requests, joins, and structured cancellation SHALL propagate that same value
without restarting its duration. Completion, timeout, and cancellation SHALL
race at explicit protocol points with exactly one winner; timeout SHALL NOT
roll back a completed effect.

### TOPAL-TIME-PERIODIC-001 — Periodic release policy

`Periodic` SHALL produce ticks containing scheduled and observed instants and a
sequence identity. `CatchUp`, `SkipLate`, and `Coalesce` SHALL determine the
complete late-release sequence. The next scheduled instant SHALL derive from
phase plus an integral period count, not from the previous observed release.

### TOPAL-TIME-TRACE-001 — External temporal trace

Clock reads, tick releases, deadline observations, and timeout winners SHALL be
external observations in the semantic trace. Recorded or logical clocks MAY
replay them deterministically. Physical WCET, response, release-jitter, and
deadline evidence SHALL remain unknown until an approved provider verifies it.

### TOPAL-FLOW-RATE-001 — Exact static rates

Every `Flow` actor port SHALL name a logical clock and exact nonnegative static
consume/produce rate. Each closed mode SHALL have its own rates and mode changes
SHALL occur only at tick boundaries. A dynamic stream SHALL NOT acquire static
rates unless the complete closed graph and values are proved.

### TOPAL-FLOW-BALANCE-001 — Repetition balance

For every edge from actor `u` to `v`, a derived positive integral repetition
vector `q` SHALL satisfy `q[u] * produces = q[v] * consumes`. A graph with no
such finite vector, a disconnected undeclared clock relation, or arithmetic
overflow SHALL be rejected.

### TOPAL-FLOW-CAUSAL-001 — Delayed feedback

Every directed causal cycle SHALL contain a positive explicit initial delay.
A zero-delay cycle SHALL be rejected. `delay` is the sole language-provided
state introduction on a feedback edge and SHALL expose its initial value.

### TOPAL-FLOW-SCHEDULE-001 — Deterministic bounded schedule

The checker SHALL derive a deterministic periodic sequential firing order and
finite minimum edge capacities, or reject the graph. The interpreter SHALL
execute that order per logical period. Fusion, vectorization, or parallel
mapping SHALL preserve produced value and tick traces; physical placement and
pipeline timing remain architecture-dependent.
