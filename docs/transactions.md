# Structured transactions

Revision `v0.2` supplies `TransactionDomain State`, a versioned semantic
resource with atomic publication. A transaction callback receives an immutable
`TransactionView State` and returns one closed decision:

- `Commit value state` proposes a complete replacement state;
- `Abort error` performs no commit; or
- `RetryWhenChanged identities` reports which observations would permit a
  later bounded retry.

`transact` produces `Committed value version`, `Conflict observed-version`, or
`Aborted error`. A candidate is invisible until the domain's one commit point.
The core performs no hidden retry, and cancellation before and after that point
cannot relabel the winner.

```topal
transact is fn (
  domain : TransactionDomain State,
  decide : (
    fn ( view : TransactionView State )
      effects ( TransactionRead domain and TransactionWrite domain )
    -> TransactionDecision Value State Codes
  )
)
  effects ( Transact domain )
-> TransactionOutcome Value Codes
```

The callback may perform pure computation and transaction-scoped operations in
that domain. Other observable effects are invalid unless represented as staged
records in the candidate state. Abandoning a candidate cleans all owned
resources.

Transactions nested in one domain flatten into the outer transaction. Distinct
domains compose atomically only with verified `AtomicCommit` evidence for the
complete set. Otherwise code uses an explicit saga or outbox. `left or-else
right` evaluates `right` only when `left` returns `RetryWhenChanged`, against
the same initial snapshot and before any irrevocable effect.

An in-process serialized reference implementation defines the portable
behavior. A compiler may refine it to STM, a lock-free commit, or another
mechanism. A checked store adapter may provide `AtomicCommit`, `Durable`, and
transaction effects with exact domain, failure-model, provider, and assumption
identities. A translated storage claim is not verified merely because it came
from a foreign module.
