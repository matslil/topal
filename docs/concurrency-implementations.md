# Synthesized concurrency implementations

Topal source describes isolated tasks, protocols, ordering, admission, and
bounded capacity. It does not expose atomics, locks, fences, memory orders,
hazard pointers, epochs, or mutable shared references. Revision `v0.2` adds the
portable information needed for a compiler to synthesize low-latency
implementations without weakening that boundary.

## Interaction policy and inferred topology

A task may associate a policy with each interaction:

```topal
UpdateTask is Task (
  identity is update-task,
  interactions is (
    apply-update is InteractionPolicy (
      capacity is 1024,
      ordering is Ordered,
      admission is Reject
    )
  )
)
```

`Ordered` and `Unordered` define observable delivery order. `Block`, `Reject`,
and `Drop ( policy is P )` define admission. Rejection and dropping are explicit
protocol outcomes; blocking contributes a dependency edge to deadlock
analysis.

The checker derives producer and consumer counts from linear endpoint ownership
and the closed application graph. Source cannot assert these counts. A bounded,
fixed-layout endpoint with one producer and one consumer may be implemented by
a preallocated SPSC ring. The serial interpreter queue and generated ring must
have the same capacity, ordering, close, cancellation, and result behavior.
Only a backend with installed proof and test obligations may publish
`LockFree` or `WaitFree` evidence.

## Published snapshots

`PublishedSnapshot T` publishes complete immutable versions. `observe` returns
a `SnapshotView T` which keeps exactly that version alive; `publish` installs a
new complete version at one protocol linearization point. A reader sees the old
or new version, never a partially updated value.

Construction names an initial value, maximum retained versions, and `Block` or
`Reject` pressure behavior. A view is affine with deterministic release.
Publication which cannot respect its retention bound blocks or returns an
explicit capacity error according to policy. The compiler may choose copying,
reference counting, RCU, epochs, or hazards, but those mechanisms and grace
periods remain unobservable.

## Protected handlers

A resource remains owned by one task. A handler classified by
`ImmediateHandler` must be non-suspending, allocation-free, effect-bounded, and
work-bounded. It is still an ordinary task handler; there is no `protected`
block or critical-section syntax. A compiler may use a direct call or serial
queue. Priority-ceiling lowering and physical response-time proof remain part
of the deferred scheduler/architecture model.

The checker owns topology, suspension, effect, allocation, and bound proofs.
Programmers and translators choose semantic policies and request guarantees;
they cannot choose a synchronization algorithm or mint progress evidence.
