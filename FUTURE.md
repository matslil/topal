# Future work

This document collects promising language and compiler work which fits Topal's
design but is not part of the current language commitment. Entries should state
what could be guaranteed, which assumptions the guarantee needs, and how failure
or uncertainty would be reported.

The first topic is a unified verification system built from structural test
tables, symbolic proof obligations, capability laws, and typed task protocols.

## Verification outcomes

Tests and proofs should use a common set of outcomes without presenting sampled
execution as proof:

- **proven** — every feasible symbolic obligation was discharged;
- **exhaustively verified** — every value in a proven finite domain was checked;
- **disproved** — a counterexample or invalid trace was found;
- **tested** — selected concrete values ran successfully, without universal
  proof;
- **infeasible** — the row's assumptions cannot hold; and
- **unresolved** — the compiler can prove neither the property nor its
  negation.

Only `proven` and `exhaustively verified` produce verified law evidence.
`Disproved` produces a diagnostic witness when possible. `Tested` records useful
confidence and coverage but cannot authorize a semantics-changing optimization.
`Unresolved` prevents certification while leaving an otherwise valid function
available without the requested verified capability.

An explicitly trusted claim remains possible at a narrow foreign or proof
boundary. Trust must be visible in source, static introspection, compiled
metadata, and diagnostics.

## Compiler-generated structural test tables

Topal's [unit-test tables and structural path
coverage](testing.md) could generate table rows automatically from a function:

- one row for every feasible decision path;
- zero and one-or-more cases for structural repetition;
- base and recursive cases for structural recursion;
- constraint boundaries and union alternatives;
- effect and dependency results supplied by mocks; and
- expected results derived where the compiler can evaluate them.

Concrete generated rows would provide a small regression suite and useful
examples. One concrete value per execution path does not prove a general law:
different values can follow the same path and produce different algebraic
results.

Generated concrete rows should therefore be described as structural coverage,
not universal verification.

## Symbolic proof tables

A stronger generated table can contain symbolic rows rather than concrete
inputs. Each row records:

```text
symbolic inputs
path assumptions
required capability and constraint evidence
the result or relation to prove
```

For every feasible path or relational path combination, the compiler attempts
to prove the obligation for all values satisfying the assumptions. A row is
then `proven`, `infeasible`, `disproved`, or `unresolved`.

Concrete values are unnecessary for a successful symbolic table. When a row is
disproved, a solver may generate a concrete counterexample solely for the
diagnostic.

The compiler should emit proof certificates which a smaller independent checker
validates. Solver success by itself should not silently authorize program
transformations.

## Capability-law verification

Function-law capabilities such as `Associative`, `Commutative`, `Identity`,
and `Idempotent` define relational proof templates. For example,
`Associative operation` generates:

```topal
( a operation b ) operation c = a operation ( b operation c )
```

The compiler expands both sides, combines the feasible paths through every
invocation, and generates one symbolic row per feasible relational combination.
The operation receives `Associative` evidence only when every row is proven or
infeasible.

This detects laws which path sampling would miss. Subtraction has one ordinary
execution path, and a sample such as `(0, 0, 0)` satisfies associativity, but
symbolic evaluation disproves:

```topal
( a - b ) - c = a - ( b - c )
```

Verified capability evidence becomes a reusable proof boundary. A generic
reduction can rely on `Associative operation` without reopening the operation's
implementation at every call.

## Exhaustive finite verification

When every quantified type has a proven finite enumeration, the compiler may
check every input combination. This is proof even when implemented through the
testing runtime:

```text
Finite T
  every-value : List T
  every value of T occurs exactly once
```

The compiler must prove the completeness and uniqueness of the enumeration.
The number of combinations may grow rapidly: associativity over `T` requires
`|T|³` cases before symmetry or other verified reductions.

Symbolic reasoning, equivalence partitions, and symmetry may reduce the work,
but a representative value stands for a partition only when the compiler proves
that the property is invariant throughout that partition.

## Pure function verification

Topal's effect-free, total fragment is a good target for automated proof. An
ordinary value-law proof should initially require:

- an empty observable effect row;
- total termination;
- deterministic value semantics;
- visible implementation or already verified contracts;
- equality evidence for compared results; and
- no cleanup behavior which introduces an observable effect at the proof
  boundary.

Static functions already satisfy stronger versions of these requirements, but
ordinary runtime functions can also be proved by treating their inputs as
universally quantified values.

The strongest initial domain includes:

- tuples, records, variants, and unions;
- exact integers, rationals, decimals, modular numbers, and bits;
- constraints and dependent fields;
- immutable lists and arrays;
- finite recursive algebraic data;
- exhaustive pattern matching;
- structural recursion with decreasing measures; and
- finite folds with suitable invariants.

Useful proof techniques include symbolic evaluation, algebraic normalization,
SMT solving for supported theories, structural induction, constraint evidence,
and previously verified capability laws.

Purity is necessary but does not make every true property decidable.
Higher-order arguments may lack sufficient contracts, opaque implementations
may expose too little evidence, and a solver may not support the required
theory. Such obligations remain `unresolved`.

Approximate arithmetic can be pure and deterministic while familiar algebraic
laws are false. Verification must use its declared rounding and exceptional
value semantics rather than assume exact-number laws.

## Recursion and induction

Zero-versus-one-or-more structural coverage is useful for tests but insufficient
for proof. A symbolic recursive table needs an induction hypothesis or invariant:

```text
base row
  prove the property for the base constructor

recursive row
  assume the property for structurally smaller components
  prove it for the current constructor
```

The compiler can reuse the decreasing measure already required by Topal's
termination checker. A finite fold uses an analogous fold invariant. If the
compiler cannot synthesize an adequate invariant, it should expose the symbolic
row as unresolved and eventually permit the programmer to supply one.

Productive generators require coinductive or trace reasoning rather than an
ordinary termination proof and should be a later extension.

## Task and protocol verification

Typed message passing can support proofs about communication traces and task
states even though a task is not a pure mathematical function.

### Protocol fidelity

An endpoint protocol can be treated as a state machine. The compiler can prove
that:

- only messages allowed in the current state are sent or received;
- each interaction transitions to an allowed next state;
- each request reply has the declared type;
- caller-controlled and receiver-controlled choices are not confused;
- a linear continuation is resumed at most once; and
- a terminated endpoint is not reused.

Endpoints carry their protocol state semantically even if source syntax does not
visibly rebind them after every transition.

### Communication completeness

Within a closed structured task scope, verification can establish that:

- every request receives one reply or declared cancellation outcome;
- every completion obligation is consumed;
- every stream continuation is resumed, returned, or cancelled;
- no reply escapes its request scope;
- no child task survives its owning scope; and
- endpoints terminate in allowed protocol states.

### Data-race freedom

Task isolation already supplies a strong basis:

- task state cannot escape;
- only one handler segment has state authority;
- communicated values are immutable; and
- suspension releases state authority before another handler runs.

The compiler can turn these rules into a formal race-freedom guarantee.

### Internal deadlock freedom

The compiler can derive wait edges from value requests, completion waits,
bounded backpressure, generator resumes, joins, and cancellation cleanup.
It then symbolically explores reachable combinations of handler and protocol
states.

A reachable closed state is a deadlock when it has unfinished internal
obligations but no runnable transition, deliverable message, completion,
cancellation transition, or declared external wait.

The analysis must be path-sensitive. A graph cycle is not itself a proof of
deadlock when its edges cannot be active together. Conversely, blocking queue
admission creates a wait edge even for an interaction described informally as
an event.

Diagnostics should present a minimal communication trace:

```text
Task A handles Refresh
Task A requests Task B Read
Task B requests Task A CurrentVersion
Task A cannot handle CurrentVersion while awaiting Read
```

### Ordering and state invariants

Protocol proofs can establish that initialization precedes ordinary messages,
authentication precedes protected requests, replies correspond to their
requests, and close acknowledgement orders all earlier operations.

Each handler can additionally be checked as a state transition:

```text
task invariant before handler
protocol transition and handler execution
task invariant after completion or suspension
```

An invariant which must survive suspension needs explicit version, transaction,
or protocol evidence because the task may process another message before the
handler resumes.

### Cancellation safety

Verification can check that cancellation follows a declared protocol path,
cannot skip resource cleanup, cannot leave a child using scope-owned
capabilities, and converges with normal completion on compatible terminal
states. Reply-versus-cancellation races must be explicit alternatives rather
than scheduler-dependent hidden behavior.

## External boundaries and assumptions

Filesystem, network, device, clock, user, and foreign-service interactions cross
the closed application proof boundary even when their adapter executes in the
same process. The compiler cannot prove that an arbitrary external participant
eventually responds or follows an undeclared behavior.

It can still prove:

- local protocol use and message types;
- validation before external values enter constrained types;
- resource lifetime and effect ordering;
- local invariants conditional on declared response constraints; and
- absence of internal circular waiting.

An external endpoint may publish a trusted progress or protocol contract.
Application liveness is then conditional on that contract.

Timeouts turn an external wait into an application-controlled protocol choice,
but rely on the clock or timer adapter as an external progress source.

## Scheduler fairness

Safety properties such as protocol fidelity and race freedom do not require a
fair scheduler. Liveness properties do. A proof that a runnable task eventually
executes depends on a runtime guarantee such as:

```text
every continuously runnable task is eventually scheduled
```

The precise fairness guarantee must become part of the Topal runtime contract
before the compiler claims eventual completion.

## Scaling verification

Complete composition of task and protocol state machines can grow
exponentially. Promising techniques include:

- verified summaries for child task scopes;
- hierarchical proof by structured scope;
- partial-order reduction for independent messages;
- symbolic payload constraints rather than concrete payload enumeration;
- linear endpoint ownership;
- static request-order ranks for simple acyclic designs;
- user-supplied invariants checked by the compiler; and
- reuse of previously verified protocol capabilities.

The checker may conservatively reject or leave unusual safe designs unresolved.
Diagnostics should identify the missing invariant, order, protocol state, or
external assumption.

## Possible staged implementation

1. Generate concrete structural test rows and minimal counterexamples.
2. Add symbolic rows for pure, nonrecursive exact-value functions.
3. Verify fundamental capability laws and finite enumerated domains.
4. Add structural induction and fold invariants.
5. Emit and independently check proof certificates.
6. Formalize endpoint protocol state and linear communication obligations.
7. Prove task-scope race freedom and communication completeness.
8. Add path-sensitive internal deadlock analysis.
9. Add cancellation, fairness-conditional liveness, and protocol summaries.
10. Explore coinductive generator and effect-trace verification.

Each stage should preserve the distinction between proof, exhaustive checking,
sampled testing, explicit trust, and unresolved obligations.
