# Synthesized concurrency and transaction semantics

## Formal text

### TOPAL-CONC-TOPOLOGY-001 — Inferred endpoint topology

The checker SHALL derive producer and consumer counts from linear endpoint
ownership and the closed application graph. Source MAY choose bounded capacity,
ordering, and admission through `InteractionPolicy` but SHALL NOT assert counts
or a mechanism. Blocking admission SHALL add a dependency edge; rejection and
drop SHALL be explicit protocol outcomes.

### TOPAL-CONC-IMPL-001 — Semantics-preserving mechanism selection

A compiler or runtime MAY select a serial queue, direct call, SPSC ring, or
another implementation only when capacity, order, close, cancellation,
completion, and failure traces equal the declared interaction semantics. Safe
source SHALL expose no atomic, lock, fence, memory-order, reclamation, or raw
shared-reference operation.

### TOPAL-CONC-SNAPSHOT-001 — Immutable version publication

`observe` on `PublishedSnapshot T` SHALL return one immutable version and keep
it live until its `SnapshotView T` ends. `publish` SHALL make one complete new
version visible at one protocol point. Under retention pressure it SHALL block
or explicitly reject according to policy; it SHALL never reclaim an observed
version or expose a partial update.

### TOPAL-CONC-PROTECTED-001 — Restricted immediate handler

`ImmediateHandler` SHALL require verified non-suspension, no allocation,
allowed effects, and bounded work for the complete handler segment. It SHALL
not introduce a critical-section syntax. Physical interrupt binding, priority,
and priority-ceiling evidence remain unavailable without the architecture and
scheduler model.

### TOPAL-TXN-DOMAIN-001 — Versioned transaction view

`TransactionDomain S` SHALL own one versioned state. A callback SHALL observe
one immutable `TransactionView S` and perform only pure computation plus its
domain-scoped transaction effects. Other observable effects SHALL be rejected
unless staged in candidate domain state.

### TOPAL-TXN-COMMIT-001 — Atomic candidate publication

`Commit V S` SHALL propose a complete candidate. `transact` SHALL publish it at
one domain commit point and return `Committed V Version`, or publish none and
return `Conflict ObservedVersion` or `Aborted Error`. No observer SHALL see a
partial candidate.

### TOPAL-TXN-CONFLICT-001 — Explicit retry decision

`RetryWhenChanged I` SHALL name observed identities and SHALL NOT cause a hidden
retry. `left or-else right` SHALL evaluate the right callback only after the
left produces that decision without irrevocable effects, using the same initial
view. Retrying execution SHALL require an explicit bound, deadline, or
cancellation rule and applicable `RetrySafe` evidence.

### TOPAL-TXN-CANCEL-001 — Commit and cancellation winner

Cancellation before the commit point SHALL discard the candidate and clean its
resources. Once commit wins, later cancellation SHALL NOT relabel it aborted.
The trace SHALL identify the unique winning protocol point.

### TOPAL-TXN-COMPOSE-001 — Domain composition

Nested transactions in one domain SHALL flatten into the outer transaction.
Transactions in distinct domains SHALL compose atomically only with verified
`AtomicCommit` evidence naming the complete set. Otherwise an explicit saga or
outbox SHALL expose intermediate and compensation outcomes.
