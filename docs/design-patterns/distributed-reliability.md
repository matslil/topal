# Distributed reliability patterns

## DS-01 — Bounded retry with backoff and jitter

**Good fit.** A remote or external operation has classified transient failures
and is safe to repeat within a caller deadline and load budget.

**References.** Azure recommends controlled retry for transient faults and
warns about idempotency, total latency, and shared-resource amplification.[^ds1]

**Hardware assumptions.** Distributed nodes or fallible external devices with
clocks and independently failing communication.

**Problem.** A one-shot call turns brief faults into failures, while immediate
unbounded retries synchronize clients and worsen overload.

**Structure.** Classify errors, cap attempts and elapsed time, increase delay,
add randomness to desynchronize callers, and preserve one operation identity.

**Limitations.** Adds latency and load, cannot fix permanent faults, and can
duplicate non-idempotent effects when a reply is lost. Random jitter conflicts
with bit-for-bit deterministic replay unless modeled explicitly.

**Core-language support required.** Typed error classes, bounded repetition,
monotonic time, explicit randomness effect, cancellation/deadline propagation,
idempotency or deduplication evidence, and outstanding-effect semantics.

## DS-02 — Circuit breaker

**Good fit.** Calls to a dependency repeatedly time out or fail and fast local
failure is preferable while the dependency recovers.

**References.** AWS describes closed/open/probe behavior to prevent repeated
calls from amplifying a failing service.[^ds2]

**Hardware assumptions.** None; commonly distributed, but applies to devices
and other external dependencies.

**Problem.** Repeated slow calls exhaust threads, queues, connections, and
deadlines and can cascade failure through the system.

**Structure.** Track results in a state machine; after a threshold open the
circuit and fail fast; after a delay admit bounded probes and close on recovery.

**Limitations.** Thresholds can oscillate or hide recovery; shared breakers
need consistent state; a breaker preserves capacity but does not repair data.

**Core-language support required.** Typed state machine, monotonic time,
concurrency-safe counters/windows, fast typed fallback, effect/resource
identity, bounded probes, and observable metrics without leaking authority.

## DS-03 — Bulkhead / cell isolation

**Good fit.** Failure or overload in one tenant, dependency, or function must
not consume all capacity needed by independent work.

**References.** Azure recommends complete segmentation to contain a
malfunction's blast radius.[^ds3]

**Hardware assumptions.** Useful from one process to multiple nodes; strong
isolation may require distinct CPU, memory, queue, network, power, or fault
domains.

**Problem.** Shared pools and global queues let one slow dependency or workload
cause system-wide resource exhaustion.

**Structure.** Partition resource budgets and queues into cells, route work to
one cell, and apply independent admission, timeout, and failure policy.

**Limitations.** Stranded capacity and skew reduce utilization; shared lower
layers can defeat isolation; cell sizing and failover are operational concerns.

**Core-language support required.** Named resource identities and budgets,
bounded task scopes/queues, capability-limited access, isolated cancellation
and failure propagation, routing types, and platform evidence for physical
separation.

## DS-04 — Idempotent consumer with deduplication key

**Good fit.** At-least-once delivery or retries can present the same logical
message more than once.

**References.** Azure defines the Idempotent Consumer pattern so duplicate
processing has the same effect as one delivery.[^ds3]

**Hardware assumptions.** Durable or replicated storage is normally required
when deduplication must survive process/node restart.

**Problem.** A lost acknowledgement causes redelivery after the original side
effect committed, corrupting counters, inventory, payments, or device actions.

**Structure.** Carry a stable operation ID; atomically record processed IDs
with the business effect or make the operation algebraically idempotent; return
the recorded result on duplicates.

**Limitations.** Deduplication state grows and needs retention policy; key reuse
is dangerous; irrevocable physical effects may not be queryable or repeatable.

**Core-language support required.** Stable typed identities, atomic/transactional
storage effects, idempotency capability evidence, replay-safe result storage,
explicit retention bounds, and protocol-level acknowledgement semantics.

## DS-05 — Saga with compensating transactions

**Good fit.** A long business workflow spans independently committed services
or devices and one global ACID transaction is unavailable or undesirable.

**References.** AWS describes sagas as local transactions connected by
continuation and compensation.[^ds4]

**Hardware assumptions.** Distributed services/stores with partial failure;
also applies to long-lived physical workflows where reversal is possible.

**Problem.** Partial completion leaves inconsistent state when a later step
fails, while holding distributed locks is not viable.

**Structure.** Encode forward steps and durable progress; associate each
committed step with an idempotent compensating action; orchestrate or
choreograph completion and rollback.

**Limitations.** Compensation is not true rollback, isolation is weak, and
external/physical actions may be irreversible. State machines and testing grow
complex.

**Core-language support required.** Durable typed workflow state, protocols,
ordered effects, idempotency keys, explicit compensation types, cancellation
and timeout, exhaustive recovery, and transaction boundaries visible to the
type/effect system.

## DS-06 — Transactional outbox

**Good fit.** One service must atomically change local durable state and
eventually publish a related message, but the store and broker cannot share a
transaction.

**References.** AWS states that the transactional outbox resolves the
distributed dual-write problem by persisting the state change and event
together.[^ds5]

**Hardware assumptions.** Transactional durable store plus an independently
failing message transport and relay.

**Problem.** A crash between database commit and message publish loses the
event; publishing first can advertise a state change that later aborts.

**Structure.** Commit the business row and outbox record in one local
transaction; a relay publishes records and marks/removes them; consumers
deduplicate repeated delivery.

**Limitations.** Adds storage, relay latency, ordering/cleanup complexity, and
at-least-once duplicates. It does not produce global exactly-once effects.

**Core-language support required.** Atomic local transaction effects, durable
algebraic event values, stable IDs, ordered relay streams, idempotent consumers,
resource-specific failure types, and lifecycle/retry supervision.

## Sources

[^ds1]: Microsoft Azure, [Recommendations for Handling Transient Faults](https://learn.microsoft.com/en-us/azure/well-architected/design-guides/handle-transient-faults).
[^ds2]: AWS, [Circuit Breaker Pattern](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/circuit-breaker.html).
[^ds3]: Microsoft Azure Well-Architected Framework, [Architecture Design Patterns That Support Reliability](https://learn.microsoft.com/en-us/azure/well-architected/reliability/design-patterns).
[^ds4]: AWS, [Saga Patterns](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/saga-patterns.html).
[^ds5]: AWS, [Cloud Design Patterns](https://docs.aws.amazon.com/pdfs/prescriptive-guidance/latest/cloud-design-patterns/cloud-design-patterns.pdf), “Transactional outbox pattern.”
