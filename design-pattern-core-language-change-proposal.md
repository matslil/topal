# Proposed core-language changes for design-pattern coverage

- Status: proposal; not part of the accepted Topal language design
- Baseline: `design-0` after design-pattern library PR #494
- Pattern scope: [`docs/design-patterns`](docs/design-patterns/README.md)

## Purpose

This document proposes the smallest coherent extension of Topal's core model
needed to support the useful semantics and performance envelopes represented by
all 60 cataloged design patterns. It is an implementation plan, not an edit to
the authoritative language design. Each semantic change below still requires a
human decision before the affected files in `docs/`, `se/`, or `spec/` are
changed.

“Support a pattern” does not always mean exposing the pattern's mechanism in
source. Topal code should state values, protocols, effects, ownership, resource
bounds, temporal behavior, and other observable requirements. The compiler may
then select a lock-free queue, RCU, a hazard-pointer scheme, a priority-ceiling
implementation, a GPU schedule, or another mechanism when it can prove that the
mechanism implements those requirements. This distinction is essential to
keeping the source safe, readable, portable, and executable by an interpreter.

## Decisions incorporated into this proposal

The following are treated as settled inputs rather than open questions:

1. Topal will not expose source atomics, memory-order operations, mutexes, or a
   facility for users to implement lock-free data structures directly.
2. Safe restricted Topal constructs may carry requirements from which the
   compiler synthesizes and certifies a lock-free implementation. A program may
   require or prefer a progress class, but may not prescribe CAS loops,
   reclamation epochs, fences, or another algorithm.
3. Foreign ABIs, symbols, calling conventions, callback conventions, and
   architecture descriptions are deferred. This proposal does not specify any
   of them.
4. A future architecture model will describe execution units, clocks, memory
   domains, device transfers, interrupts, scheduling facilities, instructions,
   compartments, and physical fault domains. The core language must be able to
   consume certified evidence from that model without making target facts part
   of the program's semantic result.
5. Low-level facts are inferred from code, constraints, topology, layouts, and
   the future architecture model whenever possible. Source annotations state
   hard requirements or selection preferences only where inference cannot
   recover programmer intent.
6. Hardware-dependent patterns may remain conditionally supported until the
   architecture model exists. The core change made now is the typed seam needed
   to express their portable semantics and accept later implementation evidence.
7. Function-level contracts, effects, and guarantees appear after the argument
   list and before `->`. A named returned value and clauses which directly
   relate that value appear after `->`.
8. The retry property is named `RetrySafe`. Capitalized property names are
   language-context classifier constructors, not keywords and not ordinarily
   qualified through `lang`.

These decisions resolve the apparent conflict between supporting CC-04/CC-05
and forbidding user-authored lock-free algorithms. Topal supports the bounded
channel and read-mostly snapshot use cases and permits the same generated
mechanisms; it deliberately does not make Disruptor, RCU, or hazard-pointer
instructions part of portable source semantics.

## Design invariants

Every change in this proposal shall preserve these rules:

- **One semantic program.** Interpretation and compilation have the same value,
  error, effect, ordering, transaction, and protocol meaning.
- **Requirements, not pragmas.** A hard implementation requirement is checked;
  failure is a build or execution diagnostic. A `Prefer` goal may fall back to
  any semantically valid implementation.
- **Inference first.** Evidence is derived from typed code and closed application
  construction. Explicit declarations are required only at opaque, public,
  higher-order, or externally assumed boundaries.
- **No target-dependent value semantics.** Architecture evidence may select
  among semantically equivalent implementations but ordinary code cannot branch
  on processor model, cache size, or compiler choice.
- **No unsafe escape hatch.** Unproved memory safety, data-race freedom,
  totality, ownership, or protocol fidelity cannot be accepted as trusted
  evidence. Trust may be explicit for environmental assumptions and
  semantics-preserving optimization laws, with its status retained.
- **No hidden failure.** Contract checks, capacity exhaustion, deadline
  observations, transaction conflicts, and unsupported implementation
  requirements use explicit results or diagnostics; they do not introduce
  exceptions or undefined behavior.
- **Mechanism remains replaceable.** Generated synchronization, allocation,
  scheduling, placement, and transfer choices are not observable unless the
  source explicitly requires an observation or bound.

## Syntax and authority conventions

### Function-header placement

Every proposed ordinary function, task handler, callback, and nested function
type uses this order in the new language revision:

```text
fn ( ParameterDeclarations )
  requires InputPredicate
  effects EffectExpression
  guarantees FunctionGuaranteeExpression
-> result : ResultClassifier
  ensures ResultRelation
```

Each clause is optional and occurs at most once. Parameter-specific classifiers
remain in `ParameterDeclarations`, for example
`input : Buffer : Exclusive`; they do not move into a post-arrow position.
`requires`, `effects`, and `guarantees` describe invocation or implementation
of the function and appear before `->`. The return classifier and a named
return binding appear after `->`; only `ensures` may use that binding. A nested
callback or returned function carries its own clauses around its own arrow.

For a returned function, the inner clauses remain inside the outer result
classifier:

```topal
make-reader is fn (
  store : Store
)
-> reader : (
  fn ( key : Key )
    effects ( Read store )
  -> Optional Value
)
  ensures ( reader-uses-store store reader )
```

`Read store` describes invoking `reader`, not invoking `make-reader`, while the
outer `ensures` may use the returned-function binding `reader`.

The ordinary `:` classification operator remains available in expressions,
parameter declarations, and call-site implementation selection. For example,
`operation : NoAlloc` classifies an operation object at a call site. It is not
the old post-return function classifier and is unaffected by this placement
rule.

The clause names above and declaration-level `invariant` are contextual
structural words. Capitalized names such as `ResourceBound`, `RetrySafe`, and
`Flow` are statically resolved objects supplied by the selected language
context. Lowercase operations such as `transact`, `observe`, and `declassify`
are ordinary functions unless a proposal explicitly says otherwise.

### Notation in this proposal

A `topal` code block is intended source syntax. A `text` code block is a type,
metadata, or construction schema and is not copied literally into source.
Names such as `State`, `BoundExpression`, and `FailureModel` in a schema are
metavariables unless the surrounding text explicitly introduces them as
language objects. Constructor options use Topal's ordinary parenthesized
`name is value` form. No capitalized proposal name is a keyword merely because
the compiler defines its meaning.

### Actors and permissions

This proposal uses the following actor names consistently:

| Actor | May do | May not do |
| --- | --- | --- |
| **Source author** | Write application or library declarations, state contracts, configure semantic resources, request hard guarantees, and state `Prefer` goals. | Mint compiler-verified safety, progress, specialization, layout, timing, or architecture evidence. |
| **Source generator or foreign translator** | Emit the same checked Topal source that a human author could write, including types, layouts, effects, contracts, and claims whose status the checker determines. | Gain special language authority, bypass validation, create a foreign call before the ABI proposal, inject provider assumptions outside an approved adapter interface, or label a claim `verified` merely because a header or module asserted it. |
| **Language checker/compiler** | Define language-context objects, infer and verify semantic evidence, lower code, produce certified implementation evidence, and reject unavailable hard guarantees. | Change semantic results in response to optimization or silently assume external facts. |
| **Interpreter/runtime provider** | Implement language-defined resources and operations and produce evidence for that concrete implementation when its verifier accepts the provider. | Claim compiled-target properties that its own implementation does not meet or ignore a hard guarantee. |
| **External adapter/provider** | Implement a checked semantic boundary and supply provider-identified facts about a store, clock, service, or device through an approved evidence interface. | Forge compiler-owned evidence or convert an environmental assumption into an unconditional proof. ABI and architecture providers remain deferred. |
| **Analysis, debugger, or qualification tool** | Read typed evidence, certificates, assumptions, and diagnostic projections and independently validate them. | Mutate program semantics or upgrade evidence status merely by editing metadata. |

A source generator has exactly the authority of generated source, not that of
the compiler. An external provider obtains additional authority only through a
typed provider interface defined and checked by the relevant language,
runtime, or future architecture specification. Until such an interface is
defined, imported claims remain `externally-assumed` or
`trusted-unverified`, as appropriate, and cannot discharge protected safety or
implementation obligations.

## Proposed extension set

| Proposal ID | Extension | Primary gaps closed |
| --- | --- | --- |
| TOPAL-PROP-CONTRACT-001 | Relational function and state contracts | AP-06, RT-06, SR-01, SR-03, SR-05 |
| TOPAL-PROP-EVIDENCE-001 | Versioned semantic and implementation evidence | distributed correctness, fault assumptions, backend selection |
| TOPAL-PROP-RESOURCE-001 | Composable work, space, capacity, and progress guarantees | MM-02/03, CC-01/04/05, RT, DS-03, HA |
| TOPAL-PROP-UNIQUE-001 | Inferred exclusivity, consumption, and allocation regions | MM-01..MM-04, FD-06 |
| TOPAL-PROP-CONC-SYNTH-001 | Compiler-synthesized channels and published snapshots | CC-04, CC-05, RT-03/04/05 |
| TOPAL-PROP-TRANSACTION-001 | Structured atomic transactions and compensation evidence | CC-06, SR-03, ST-02, DS-04..DS-06 |
| TOPAL-PROP-TIME-001 | Clock, instant, deadline, periodic-release, and temporal contracts | RT-01..RT-06, SR-01/02/05, DS-01/02 |
| TOPAL-PROP-FLOW-001 | Clocked, static-rate dataflow | RT-01, HA-10, HA-11 |
| TOPAL-PROP-HANDLER-001 | Lexically resolved typed effect handlers | FD-03 |
| TOPAL-PROP-SPECIALIZE-001 | Guaranteed specialization and implementation-plan evidence | FD-01/02/04/05, MM-06, HA-01/04/06 |
| TOPAL-PROP-LAYOUT-001 | Multidimensional, blocked, separated, and sparse layouts | MM-05/06, HA-02/03/07/09 |
| TOPAL-PROP-INFOFLOW-001 | General confidentiality/integrity labels | ST-04 and policy support for ST-01..ST-03 |
| TOPAL-PROP-ARCH-SEAM-001 | Opaque architecture-evidence seam | hardware-dependent portions of RT, SR, ST, and HA |

The proposals extend existing Topal concepts. They do not add a second trait
system, macro language, exception model, mutable-reference system, concurrency
model, or target-specific source dialect.

## Entity permission index

This table is the authority cross-check for the detailed sections. “Use” means
construct, call, configure, or require; it does not imply permission to create
evidence.

| Entity family | Intended source user and form | Evidence, value, or implementation producer |
| --- | --- | --- |
| `requires`, `effects`, `guarantees`, result binding, `ensures` | Function/interface/handler authors and checked source generators use the fixed header positions. | The checker verifies clauses and the compiler/interpreter enforces them. |
| `invariant` | Nominal-type and task-definition owners declare it on owned state. | The checker proves preservation; runtime diagnostic checks do not create proof. |
| `RetrySafe`, `AtomicCommit`, `Durable`, `Compensates` | Authors require them pre-arrow and may define source-owned relations where permitted. | Checker proofs or checked external adapters with retained status and assumptions. |
| `Progress`, `Specialized` | Authors require or prefer them pre-arrow or at ordinary implementation-selection sites. | Compiler, concrete interpreter/runtime, checked artifact producer, or later architecture provider as restricted by the property. |
| Evidence status and `ImplementationEvidence` records | No writable source form; tools may inspect their qualified/read-only projection. | Checker/compiler and approved provider/verifier interfaces only. |
| `ResourceBound`, portable dimensions, `OExec`, `OAlloc`, `NoAlloc` | Authors state or prefer bounds in `guarantees`; libraries may compose the fixed dimensions. | Generic checker for symbolic facts; concrete compiler/interpreter/runtime for implementation facts. |
| `Exclusive`, `Consumes` | Authors place them on the parameter they classify. | Checker derives exclusivity and enforces consumption at each call; no trusted source production. |
| `AllocationRegion`, `Allocate region` | Authors construct/pass scoped regions and call region-parameterized operations. | Compiler/runtime implements storage and cleanup; checked adapters may provide opaque region resources. |
| `InteractionPolicy`, ordering, admission, capacity | Task/application authors configure semantic interaction policy. | Checker retains policy and derives topology; compiler/runtime chooses the queue and produces progress evidence. |
| Producer/consumer counts and synthesized queue/snapshot mechanisms | No direct source declarations; authors use endpoints and `PublishedSnapshot` operations. | Checker derives counts; compiler/runtime implements and certifies mechanisms. |
| Transaction domains, views, decisions, outcomes, and `or-else` | Authors construct in-process domains, write callbacks/decisions, call `transact`, and inspect outcomes. | Transaction runtime produces commit outcomes; checked store adapters may implement domains and provide evidence. |
| Clocks, instants, deadlines, ticks, and `Periodic` | Authors pass clocks, build deadlines/periodic sources, and consume observations. | Runtime/clock adapter creates clocks and observations; future architecture/timing providers prove physical bounds. |
| Temporal and `ImmediateHandler` classifiers | Authors require or prefer them before a function/handler arrow. | Checker proves portable shape; future approved timing/architecture provider proves physical properties. |
| `Flow` and `delay` | Authors and checked graph translators declare rates, connect flows, and add stateful delays. | Checker derives causality, schedule, and capacity; compiler/interpreter implements the logical graph. |
| `EffectProtocol`, operations, `Handler`, `Resumption`, `handle ... with ...`, `MultiShot` | Protocol owners declare operations; authors implement handlers and lexical handling. | Checker selects handlers and proves affine/multi-shot safety; compiler/interpreter implements resumptions and cleanup. |
| Implementation-plan IR | No source form; authors and tools request a read-only diagnostic projection. | Compiler/backend only; imported candidates cannot prescribe plan nodes. |
| `Shape`, extended `Layout`, sparse schemas, views/conversions | Authors and foreign-layout translators construct checked descriptions and request conversions. | Checker validates; compiler selects internal representation and proves implementation properties. |
| Information policies, labels, `Labeled`, `declassify`, `endorse` | Policy owners define policy; code possessing exact authority may call the authority-changing operations. | Checker verifies flow; application-approved context/provider provisions unforgeable authorities. |
| `Place`, `OwnedAt`, physical `transfer`, fault independence, and target-specific properties | No present source use; names and syntax are deferred. | Future architecture/device/provider proposal only. |

## TOPAL-PROP-CONTRACT-001 — relational contracts

### Model

Add explicit function-header clauses and one declaration-level invariant
clause. The complete function-header order is:

```text
fn ( Parameters )
  requires Predicate
  effects EffectExpression
  guarantees GuaranteeExpression
-> result : ResultClassifier
  ensures Relation
```

`requires P` is a caller obligation over the complete input and therefore
appears immediately after the parameter list. `ensures R` is an implementation
obligation relating the complete input, returned result, and declared effect
trace and therefore appears after the result declaration. `effects E` gives the
function's allowed effect-row upper bound. `guarantees G` states hard evidence
requirements or explicit selection preferences for the function itself. Both
therefore appear after the arguments and before `->`; neither classifies the
result type or binds the returned value.

The pre-arrow clauses may refer to parameters, captured identities, static
inputs, and other names visible at the declaration. They cannot refer to the
result binding, which has not yet been introduced. A property which directly
relates the returned value to the inputs or effect trace belongs in `ensures`.
A cross-invocation property such as `RetrySafe`, however, classifies the whole
operation and uses its own explicitly named relation over completed outcomes;
it does not capture the single-invocation `result` binding.

The name between `->` and `:` binds the returned value for the `ensures` clause.
It is a contract binding, not a local binding available in the function body.
A function without an `ensures` clause may retain the existing unnamed
`-> ResultClassifier` form.

The proposed declaration is:

```topal
withdraw is fn (
  account : Account,
  amount : Positive Money
)
  requires ( amount <= account available )
-> result : Result ( Account, WithdrawalErrorCode )
  ensures ( withdrawal-relation account amount result )
```

`requires`, `effects`, `guarantees`, and `ensures` are contextual structural
words in these exact header positions, not ordinary operations looked up by
name. Each is optional and may occur at most once. Multiple predicates,
effects, or guarantees compose inside their clause with the ordinary
composition operators appropriate to that object kind. When `ensures` is
present, the result binding is explicit—there is no implicitly reserved
`result` identifier.

This clause form replaces the current post-return `: Classifier` form only in
the proposed language revision. The `design-0` grammar and meaning remain
unchanged. Adopting the proposal therefore requires coordinated revisions to
the function, effect, performance, and module design and specification rather
than interpreting old source differently. In a higher-order result classifier,
clauses belonging to the returned function remain inside that function
classifier; the outer function's pre-arrow clauses describe only the outer
invocation.

`invariant` does not occur in the function header. It attaches to the nominal
type or task whose state it governs. A type invariant binds its complete value;
a task invariant binds the complete task state. The owner must establish the
invariant during construction and preserve it through every visible transition.
Both forms use an explicit contract-only binding in source:

```topal
Account is type
  available : Money

  invariant value (
    value available >= zero-money
  )

AccountTask is Task (
  queue-size is 10,
  identity is account-task
)

account-service is AccountTask
  balance : Money

  invariant state (
    state balance >= zero-money
  )

  start is fn ( initial : Money )
    requires ( initial >= zero-money )
  -> Completed
    @ balance is initial
    Completed
```

Each owner has at most one structural `invariant` clause; independently useful
conditions compose within its predicate. The `value` or `state` name is visible
only to that invariant. In a nominal type it follows the stored components and
precedes operation declarations. In a task definition it follows the persistent
state fields and precedes `start` and the message handlers. It is not placed
around any function arrow.

These are structural proof clauses rather than ordinary function classifiers.
A precondition is not a constraint on every value of a parameter type, and a
postcondition can relate old input, success alternatives, error alternatives,
and effects.

### Static rules

- Contract predicates are pure and total. They may use constraints, dependent
  identities, exact arithmetic, capability laws, and previously verified
  contracts.
- A call with `requires P` is valid only when the compiler has verified
  evidence for `P` at that call. Trusted-unverified evidence cannot authorize
  memory safety, bounds safety, totality, ownership, protocol fidelity, or check
  elimination.
- Dynamic data is validated explicitly through an existing constraint/smart
  constructor returning `Result`. Successful construction produces the evidence
  used at later calls. Topal does not inject a hidden failing precondition.
- Every successful and failing return path must satisfy the applicable
  postcondition. A postcondition can distinguish `Result` alternatives.
- An invariant is checked at construction and at every public transition of its
  owning type or task. A task invariant that spans suspension must use version,
  transaction, or protocol evidence, consistent with the current task model.
- Opaque code publishes its contract and proof status in compiled evidence.
  Visible code may have contracts inferred.

### Execution and lowering

Contracts whose evidence is verified are erased from ordinary execution.
Explicit validation remains an ordinary `Result`-producing computation and may
be eliminated only when its success is proved. The interpreter may run optional
diagnostic contract checks, but those checks cannot change a valid program's
defined result or replace missing proof.

### Declaration and authority

Source authors and source generators may write `requires`, `ensures`, and an
`invariant` on an entity they are permitted to declare. They may also write
pre-arrow `effects` and `guarantees`; the authority to mention a property does
not grant authority to produce its evidence. The checker proves the obligations
or records the permitted non-verified status. An opaque or external adapter may
publish a contract, but its provider identity and proof status cross the
boundary with it. A foreign translator can translate a foreign annotation into
the same contract syntax, but the annotation is not `verified` unless an
approved checker validates a proof or certificate.

The compiler may infer contracts for visible bodies and retain them as typed
evidence. It may not strengthen a caller's obligation beyond the explicitly
published `requires` clause at an opaque or separately compiled boundary. The
interpreter enforces accepted dynamic validations and optional diagnostic
checks; it does not manufacture proof evidence from successful test runs.

## TOPAL-PROP-EVIDENCE-001 — semantic and implementation evidence

### Two evidence classes

Topal shall make the existing distinction explicit in the object taxonomy:

1. **Semantic evidence** proves facts on which accepted program behavior may
   rely: a constraint, function relation, transaction law, protocol transition,
   or ownership fact.
2. **Implementation evidence** proves that one implementation meets a resource,
   progress, placement, timing, or architecture-dependent requirement while
   preserving the semantic contract.

Implementation evidence never changes the set of semantic results. Forgetting
it may lose an optimization or make a hard implementation requirement
unsatisfied, but it cannot make an otherwise safe operation unsafe.

Every evidence object retained across a public or opaque boundary contains:

```text
property identity
classified subject and static parameters
verified | trusted-unverified | externally-assumed status
derivation or provider identity
assumption identities
selected language revision
optional architecture-model identity
```

This block describes retained compiler metadata, not source syntax.
`verified`, `trusted-unverified`, and `externally-assumed` are evidence-status
alternatives in artifacts and introspection; they are not function-header
keywords and do not introduce executable operations.

`externally-assumed` is distinct from `trusted-unverified`: it states a fact
about a clock, service, deployment, device, or other participant outside the
closed Topal proof boundary. The compiler proves conclusions conditionally and
lists the assumptions in artifacts and diagnostics.

### Source vocabulary and namespace

The header words `requires`, `effects`, `guarantees`, `ensures`, and
`invariant` are contextual grammar because their positions determine the
structure and scope of a declaration. The capitalized property names below are
instead statically resolved classifier constructors. They are not reserved
keywords.

The selected language version owns these constructors and their verification
rules. Consistent with Topal's language-module model, it introduces them into
the source file's main scope. Source therefore writes `RetrySafe (...)`, not
`lang RetrySafe (...)`. The qualified `lang` scope remains for introspection,
diagnostic controls, and similarly explicit language boundaries. Evidence
descriptors may be inspected through that qualified introspection API, without
making the property constructor's ordinary source spelling qualified.

A conflicting declaration in the active root scope is diagnosed rather than
silently shadowing the language binding. A library may publish the same text in
one of its own explicitly qualified namespaces, but that declaration has no
compiler-defined property semantics. Selecting another language revision may
change the available vocabulary only as part of that explicit, immutable
language-context change.

### Extensibility boundary

The selected language version continues to own atomic capability meanings and
verification rules. Libraries may define quantified relational properties when
their meaning reduces to the language's contract and effect-trace logic. They
may not attach new optimizer semantics to an arbitrary name.

The next language revision should add this small standard vocabulary. Within a
`guarantees` clause, the classified subject is the function being declared, so
the constructors do not repeat a `Function` or `Operation` argument:

```text
RetrySafe (
  identity is Identity,
  equivalent is CompletionEquivalence
)
AtomicCommit ( domains is Set TransactionDomain )
Durable ( store is Store, failures is FailureModel )
Compensates (
  forward is ForwardOperation,
  relation is CompensationRelation
)
Progress ( class is ProgressClass )
Specialized ( static-inputs is StaticInputs )
```

These signatures are classifier-constructor schemas, not additional grammar
productions. Only the constructor names at the start of each schema are new
source vocabulary. `Identity`, `CompletionEquivalence`, `TransactionDomain`,
`Store`, `FailureModel`, `ForwardOperation`, `CompensationRelation`, and
`StaticInputs` are metasyntactic roles filled by statically typed objects, not
additional keywords. `ProgressClass` is the closed language type defined by
TOPAL-PROP-RESOURCE-001. A completion-equivalence object is a verified pure
equivalence relation over two complete observations of the classified
operation, where each observation contains its returned outcome and observable
effect trace. Their meanings are:

- `RetrySafe` states that re-executing the declared effectful operation with
  the same stable identity after an unobserved completion has an equivalent
  completed outcome and observable trace under `CompletionEquivalence`. It
  does not make the function pure, guarantee that a retry occurs, or authorize
  an unbounded retry loop. `Idempotent` remains the distinct existing
  algebraic function law.
- `AtomicCommit` states that changes in exactly the named transaction domains
  become observable at one commit point or not at all. It makes no claim about
  unnamed domains or effects.
- `Durable` states that a committed change in the named store survives the
  explicitly named failure model. It is never an unqualified promise to
  survive every physical failure.
- `Compensates` states the precondition and observable-trace relation under
  which the declared compensation operation compensates the named forward
  operation. It does not erase history or imply that compensation cannot fail.
- `Progress` classifies the selected complete implementation with the closed
  progress class defined by TOPAL-PROP-RESOURCE-001. `LockFree` and `WaitFree`
  evidence may come only from the compiler or an approved implementation
  provider.
- `Specialized` states that the selected implementation has eliminated the
  runtime structures or dispatch attributable to the named static inputs. Its
  implementation plan and code-shape evidence remain compiler-owned.

`RetrySafe`, `AtomicCommit`, `Durable`, and `Compensates` are semantic evidence;
an external store may make their proof conditional on explicit environmental
assumptions. `Progress` and `Specialized` are implementation evidence and
cannot be asserted by ordinary programmer trust.

`FaultIndependent` is not introduced by this proposal. Physical fault domains
are architecture- and deployment-dependent, so its relation, parameters,
provider ownership, and source spelling are deferred to the architecture-model
proposal. Portable code may retain an abstract need for independent resources,
but it cannot claim physical independence until that later model supplies and
defines the evidence.

For example:

```topal
commit-order is fn (
  change : OrderChange
)
  requires ( valid-change change )
  effects (
    Write order-store
    and Write outbox
  )
  guarantees (
    RetrySafe (
      identity is change id,
      equivalent is same-commit-observation
    )
    and AtomicCommit (
      domains is ( order-store, outbox )
    )
    and Durable (
      store is order-store,
      failures is ProcessRestart
    )
  )
-> result : Result ( CommitVersion, StoreErrorCode )
  ensures ( commit-result-consistent change result )
```

The pre-arrow guarantees classify `commit-order`; they do not classify
`Result ( CommitVersion, StoreErrorCode )`. Only `ensures` receives the
contract-only `result` binding. `same-commit-observation` is an ordinary
source-owned verified relation. `ProcessRestart` illustrates a failure-model
value supplied by a checked runtime/store provider; it is not a keyword or a
fixed core failure model.

### Property authority

| Entity | Source-author use | Who may establish evidence |
| --- | --- | --- |
| `RetrySafe` | Require it in pre-arrow `guarantees`, define the named identity and equivalence objects, or use it in a source-owned capability claim. | The checker may verify a visible relation; an external operation adapter may supply provider-scoped evidence or an explicit external assumption. |
| `AtomicCommit` | Require it for a function or adapter over the named domains. | A verified transaction implementation or checked external store adapter; ordinary source cannot assert it merely because calls are adjacent. |
| `Durable` | Require it for a named store and failure model. | A checked store/runtime provider, normally conditional on external assumptions. The compiler cannot derive physical durability from Topal code alone. |
| `Compensates` | Declare a source-owned compensation relation and require it on the compensation function. | The checker may verify the relation; an unresolved source-owned semantic claim may be `trusted-unverified` where project policy permits, but its status remains visible. |
| `Progress` | Require or prefer a class in pre-arrow `guarantees` or at an ordinary implementation-selection site. | Only the compiler, interpreter/runtime implementation, or a later approved architecture provider may establish it for a concrete implementation. |
| `Specialized` | Require or prefer elimination of the named static inputs. | Only the compiler/backend, or a checked producer of already-specialized typed IR or machine code, may establish it. |

Neither a programmer nor a foreign translator can write an evidence-status
modifier that turns one of these claims into `verified`. They write the
property application; the checker assigns the status from its derivation or
provider path. Qualification tools may reject a status or validate its
certificate, but do not rewrite it.

## TOPAL-PROP-RESOURCE-001 — resource and progress guarantees

### General form

Retain `OExec`, `OAlloc`, `NoAlloc`, and current `Prefer` behavior. Add a common
representation for exact or asymptotic implementation bounds:

```text
ResourceBound (
  dimension is ResourceDimension,
  scope is ResourceScope,
  bound is BoundExpression
)
```

The next language revision supplies this architecture-neutral dimension
vocabulary. A particular interpreter or backend may leave an implementation
dimension such as `Stack` or `CodeSize` unknown when it has no corresponding
resource model:

| Dimension | Meaning and composition |
| --- | --- |
| `Work` | Total abstract work; sequential composition adds and alternatives take a conservative maximum. `OExec` remains its asymptotic shorthand. |
| `Span` | Longest dependency path under unbounded parallel capacity; sequential dependency adds and proven independent branches take a maximum. |
| `AllocationTotal` | Total dynamic allocation; `OAlloc` remains its asymptotic shorthand. |
| `PeakLive` | Maximum simultaneously live storage with a known layout; composition uses lifetime overlap, not total allocation. |
| `Retained` | Storage that may remain live after the classified operation or suspension point. |
| `Stack` | Maximum implementation stack storage after lowering for a concrete implementation. |
| `QueueEntries` | Maximum admitted entries for a named interaction or endpoint. |
| `TransferCount` | Number of semantic region transfers or copies across a named boundary. |
| `CodeSize` | Size of emitted implementation code, used mainly to bound specialization. |

`Scope` is a function invocation, handler segment, task lifetime, endpoint,
named allocation region, or complete application. Dimensions use typed units;
storage quantities use bits or bytes only when the selected layouts make them
well defined. Unknown is not infinity and does not satisfy a hard bound.

Target-specific dimensions such as elapsed time, cycles, energy, bandwidth, or
named-memory capacity are not defined now. The architecture seam specifies how
a later architecture model adds such dimensions without changing the matching,
trust, hard-requirement, or `Prefer` rules.

In source, a bound on the declared function is written in the pre-arrow
`guarantees` clause. `Invocation` denotes that function's one invocation; a
named endpoint, region, handler segment, task, or application value may be used
as the scope when it is visible in the declaration:

```topal
encode is fn (
  input : Packet
)
  guarantees (
    ResourceBound (
      dimension is Work,
      scope is Invocation,
      bound is 4 * ( input size )
    )
    and ResourceBound (
      dimension is PeakLive,
      scope is Invocation,
      bound is 4[KiB]
    )
  )
-> Bytes
```

`ResourceBound`, the portable dimension values in the table, and `Invocation`
are unqualified language-context objects, not keywords. `OExec`, `OAlloc`, and
`NoAlloc` remain shorthand guarantee constructors. `BoundExpression` is a
static, pure, total expression over visible static measures. A declaration
cannot use the post-arrow result binding in a resource bound; if resource use
depends on an outcome, the pre-arrow bound must conservatively cover all
outcomes or use a language-defined bound expression over the operation's closed
alternatives rather than the runtime `result` name.

`HandlerSegment` is used only on the owning handler declaration. A task-lifetime
scope is used on that task's `start`/`terminate` contract, and the complete
application scope is used on the root task's contract. These are still
pre-arrow guarantees of the declaration which owns the scope; the proposal does
not add a free-standing syntax by which unrelated source can assert a bound for
another entity.

### Progress

Add the closed implementation evidence class:

```text
ProgressClass =
  MayBlock
  | ObstructionFree
  | LockFree
  | WaitFree ( maximum-own-steps is BoundExpression )
```

This class describes a complete implementation of an interaction, not source
operations available to the programmer. Only the compiler, a verified runtime
component, or a future architecture provider may produce `LockFree` or
`WaitFree` evidence. Programmer-authored trusted evidence is rejected.

Progress is conditional on listed external assumptions such as scheduler
service and eventual hardware completion. It does not imply a real-time bound.
A hard classifier rejects a backend which cannot certify it; `Prefer` permits a
blocking or serialized fallback.

The writable form also belongs before the arrow:

```topal
send is fn (
  endpoint : OutputEndpoint,
  value : Message
)
  guarantees (
    Progress ( class is LockFree )
  )
-> result : Result ( Completed, SendErrorCode )
  ensures ( send-result-consistent endpoint value result )
```

`Progress` is the classifier constructor. `MayBlock`, `ObstructionFree`,
`LockFree`, and `WaitFree` are alternatives of the language-provided
`ProgressClass`; `WaitFree` is constructed with
`maximum-own-steps is BoundExpression`. They are requirements on a complete
implementation, not synchronization operations a source body can call.

### Derivation and reporting

Resource expressions remain symbolic through generic typed IR and are
specialized after concrete layouts, capacities, and implementations are known.
Every failure to meet a hard requirement reports the dimension, scope,
derived/unknown bound, assumptions, and candidate implementations considered.
The interpreter supplies evidence for its own implementation or rejects a hard
implementation requirement; it never silently ignores the requirement.

### Use and authority

Source authors may write portable `ResourceBound`, `OExec`, `OAlloc`,
`NoAlloc`, and `Progress` requirements or wrap them in `Prefer`. They may define
descriptive static functions which combine these language-defined dimensions,
but cannot define a new atomic dimension or verification rule. The generic
checker derives symbolic `Work`, `Span`, allocation, lifetime, capacity, and
transfer facts when possible. A concrete compiler or interpreter establishes
`Stack`, `CodeSize`, and implementation `Progress` only for the implementation
it actually selects.

A runtime or external adapter may supply checked bounds for an opaque operation
with provider identity and assumptions. A foreign translator may reproduce a
header's complexity annotation as a claim, but it remains unverified or
externally assumed until an approved checker or certificate establishes it.
Analysis and qualification tools may consume the retained expression and proof;
they cannot promote its status. Target-specific dimensions remain unavailable
until the architecture-model proposal defines their providers.

## TOPAL-PROP-UNIQUE-001 — inferred exclusivity and regions

### Exclusive-use evidence

Add the invocation-local parameter classifier `Exclusive`. In source it
classifies the parameter, not the returned value or the complete function:

```topal
rewrite is fn (
  input : Buffer : Exclusive
)
-> Buffer
```

For the classified invocation and lifetime interval, it means that no other
observable value can refer to storage which the implementation intends to
reuse. It is derived from last use, escape, span-disjointness, and ownership
analysis; it is never a permanent property of `Buffer` or another semantic
type.

An overload may require `Exclusive input` to expose an optimized path. Ordinary
source need not be rewritten: when evidence is absent, a semantically equivalent
immutable fallback remains applicable. A call may make exclusivity a hard
requirement, in which case the checker proves that the old value is not used or
observed afterward. Programmer trust cannot fabricate exclusivity.

`Consumes` is the explicit public parameter classifier for the stronger call
contract:

```topal
finish is fn (
  input : Buffer : Consumes
)
-> Encoded
```

It does not expose a mutable reference. It says only that the caller
relinquishes the previous semantic version on successful entry. Because both
names classify an input, they remain inside the argument list under the global
placement rule. `Exclusive input` and `Consumes input` in prose are semantic
shorthand, not the writable declaration form.

Source authors may use either classifier on an overload or public boundary.
The checker alone establishes invocation-local `Exclusive` evidence at each
call and enforces the later-use restriction for `Consumes`; programmer trust,
a source generator, or a foreign annotation cannot fabricate it. A compiler
uses accepted evidence to select in-place reuse, while an interpreter may
execute the immutable fallback. No evidence is required when such a fallback
is applicable and no hard classifier was written.

### Allocation regions

Add a scoped `AllocationRegion` resource. Values constructed inside a region
retain a hidden dependency on it. A dependent value may leave the scope only by:

- moving ownership of the entire region;
- proving that its storage was promoted or copied to an enclosing lifetime; or
- proving that the value has no runtime storage dependency on the region.

Region cleanup is deterministic and composes with ordinary resource cleanup.
The compiler should infer regions from lexical lifetimes and resource bounds.
An explicit named region is used only to state an API boundary, capacity, peak
bound, or bulk-release requirement. Source cannot perform pointer arithmetic or
observe the allocator selected for a region.

`AllocationRegion` is an ordinary language-provided scoped resource, not a
keyword or address type. Explicit allocation into one is requested by passing
the region to an operation whose pre-arrow effect row contains the
language-defined `Allocate region` effect:

```topal
scratch is AllocationRegion (
  capacity is 64[KiB]
)

parse-in is fn (
  region : AllocationRegion,
  input : Bytes
)
  effects ( Allocate region )
  guarantees (
    ResourceBound (
      dimension is PeakLive,
      scope is region,
      bound is 64[KiB]
    )
  )
-> result : Result ( SyntaxTree, ParseErrorCode )
  ensures ( parse-result-describes input result )
```

An application or library author may construct or accept an explicit region
and pass it to such operations. The compiler may introduce unnamed regions when
their identity is not observable and may choose their storage strategy. The
interpreter/runtime provider implements allocation and cleanup for its region
values. An external adapter may provide a region only through a checked resource
adapter; a foreign translator cannot turn an arbitrary pointer or allocator
handle into `AllocationRegion`, and direct foreign ownership remains deferred
with the ABI.

An interpreter may implement every region with its ordinary allocator and bulk
lifetime bookkeeping. A compiler may use stack storage, an arena, scratch
storage, or individual allocations as long as the declared bounds and escape
rules hold.

## TOPAL-PROP-CONC-SYNTH-001 — synthesized concurrency mechanisms

### No public low-level memory concurrency

The existing prohibition on source mutexes and shared-memory atomics remains.
There is no public memory-order enum, compare-and-swap, fence, lock, hazard
pointer, epoch, quiescent-state operation, or unchecked shared reference.

Topal instead extends implementation evidence for the existing task and
protocol model. The compiler derives these facts where possible:

```text
ProducerCount endpoint N
ConsumerCount endpoint N
Capacity endpoint N
Ordering endpoint Ordered | Unordered
Admission endpoint Block | Reject | Drop ( policy is DropPolicy )
Progress endpoint ProgressClass
```

This is compiler metadata, not a list of writable declarations. `ProducerCount`
and `ConsumerCount` are compiler-only derived facts. `Capacity`, `Ordering`,
and `Admission` retain semantic choices made through checked task/interaction
construction; `Progress` is produced only for the selected implementation. The
proposal extends the ordinary `Task` option schema with per-interaction policy,
using normal `name is value` construction rather than new keywords:

```topal
UpdateTask is Task (
  identity is update-task,
  interactions is (
    apply-update is InteractionPolicy (
      capacity is 1_024,
      ordering is Ordered,
      admission is Reject
    )
  )
)
```

`InteractionPolicy`, `Ordered`, `Unordered`, `Block`, `Reject`, and
`Drop ( policy is DropPolicy )` are language-context objects. A task or
application author may choose the semantic policy. The checker derives endpoint
counts from linear ownership and the closed topology; source authors and
generated source cannot assert those counts. A compiler or runtime chooses the
queue mechanism and alone produces its progress evidence. An external transport
remains behind a local adapter task and has no special authority to claim SPSC
topology.

Linear endpoint ownership, the constructed task graph, protocol directions,
queue bounds, effect independence, and static application topology normally
supply the first five facts. Only implementation selection supplies `Progress`.

For example, a bounded endpoint proven to have one producer and one consumer,
fixed layout messages, and no dynamic resizing is eligible for an SPSC ring.
The backend may select a preallocated lock-free ring and publish its progress,
allocation, capacity, and layout evidence. Source code observes only the
endpoint's declared protocol, ordering, admission, completion, and failure.

### Published immutable snapshots

Add a standard semantic `PublishedSnapshot T` resource with two operations:

```topal
observe is fn (
  snapshot : PublishedSnapshot T
)
  effects ( Read snapshot )
-> SnapshotView T

publish is fn (
  snapshot : PublishedSnapshot T,
  value : T
)
  effects ( Write snapshot )
-> Result ( Completed, SnapshotPublishErrorCode )
```

`observe` returns one immutable version and extends that version's lifetime for
the returned view. `publish` installs one complete new version at a protocol
linearization point. Readers observe either the previous or new complete
version according to the resource's declared operation order, never a partial
update. Reclamation is not observable except through declared resource bounds.

The compiler may implement this with direct immutable sharing, generation
copies, reference counting, RCU, epochs, hazard pointers, or another verified
scheme. No choice grants source access to its mechanism. A hard `Progress` or
`PeakLive` requirement can eliminate unsuitable implementations. A stalled
view which prevents a required retention bound is a compile-time protocol error
when statically provable; otherwise `publish` follows its declared `Block` or
`Reject` retention-pressure policy. Rejection returns an explicit capacity
error, and blocking contributes a dependency edge subject to deadlock analysis.
An unobserved old version may always be reclaimed; an observed version is never
silently replaced.

This abstraction covers the useful read-mostly publication behavior of CC-05.
It intentionally does not allow a Topal library to reproduce a particular RCU
API or observe grace periods.

`PublishedSnapshot`, `SnapshotView`, `observe`, and `publish` are unqualified
language-provided resource and operation names. Application and library authors
may construct a snapshot with ordinary named options:

```topal
current is PublishedSnapshot (
  initial is initial-state,
  retained-versions is 2,
  pressure is Reject
)
```

They may call the operations, retain views, choose `Block` or `Reject`, and
request bounds or progress. They cannot implement those names with RCU or
hazard-pointer primitives. The interpreter/runtime or compiler supplies the
concrete implementation and its evidence; an external adapter may wrap an
external snapshot service only as an ordinary effectful resource with explicit
assumptions, not as unchecked shared memory.

### Protected operations

Shared devices or other resources that require mutual exclusion remain owned by
one task or endpoint. A protected operation is a non-suspending bounded handler
on that owner, not a user-visible critical section. A future architecture and
scheduler model may lower such a handler to a priority-ceiling protected object
when required; it may also use a direct call, queue, or other mechanism.

There is no `protected` declaration or critical-section syntax. A source author
writes an ordinary task handler and may require the derived `ImmediateHandler`
classifier in its pre-arrow `guarantees` clause as shown under
TOPAL-PROP-TIME-001. The checker verifies non-suspension, effects, and bounds.
The compiler/runtime chooses a serial queue, direct call, or other safe
implementation. Only the future scheduler/architecture provider may establish
that this operation is implemented by a physical priority-ceiling mechanism;
a programmer or translated foreign annotation cannot do so.

The checker shall infer and publish:

- the complete effect/resource set of the handler;
- whether the handler can suspend, allocate, or call an unbounded operation;
- its work and stack bounds when known; and
- every caller and protocol ordering edge in the closed application.

These facts prepare RT-03 without defining priorities or ceilings before the
architecture model exists.

## TOPAL-PROP-TRANSACTION-001 — structured transactions

### Transaction domains

Add a language-defined `TransactionDomain State` resource. A domain owns a
versioned semantic state and provides a `transact` operation whose callback
computes a candidate update from an immutable snapshot:

```topal
transact is fn (
  domain : TransactionDomain State,
  decide : (
    fn ( view : TransactionView State )
      effects (
        TransactionRead domain
        and TransactionWrite domain
      )
    -> TransactionDecision Value State Codes
  )
)
  effects ( Transact domain )
-> TransactionOutcome Value Codes
```

The decision and outcome types are closed language-provided unions. Their
conceptual schemas are:

```text
TransactionDecision Value State Codes
  Commit Value State
  Abort Error Codes
  RetryWhenChanged (Set ObservedIdentity)

TransactionOutcome Value Codes
  Committed Value Version
  Conflict ObservedVersion
  Aborted Error Codes
```

`TransactionDomain`, `TransactionView`, `TransactionDecision`, and
`TransactionOutcome` are unqualified language-provided types or constructors.
`transact` is an ordinary language-provided function rather than a keyword.
`TransactionRead`, `TransactionWrite`, and `Transact` are parameterized effect
constructors. Notice that both the callback's and `transact`'s effects precede
their respective arrows; neither effect expression classifies an outcome type.
An in-process domain uses ordinary construction syntax:

```topal
orders is TransactionDomain (
  initial-state is empty-order-state,
  serialization is order-transaction-order
)
```

The `State` parameter is inferred from `initial-state`. An external domain is
instead supplied by its checked adapter and cannot be constructed from an
unvalidated foreign handle. `serialization` is a source-owned deterministic
ordering operation; the constructor rejects it unless the checker has verified
the required totality and ordering laws.

A candidate update is invisible until commit. Commit publishes the new state
atomically at one domain-defined protocol point. `Conflict` is explicit; the
core operation performs no hidden unbounded retry. A library retry driver may
retry under an explicit bound, deadline, cancellation rule, and `RetrySafe`
evidence.

The callback may perform pure computation plus transaction-scoped reads and
writes in the same declared domain. Other observable effects are rejected
unless they are represented as staged durable records within that domain.
Cancellation or error before commit discards the candidate and cleans its
resources. Once commit wins, cancellation cannot relabel it as aborted.

### Composition

Nested transactions in the same domain flatten into the outer transaction.
Transactions from different domains compose atomically only when an adapter
provides one `AtomicCommit` capability for the complete domain set. Otherwise
the program must use an explicit saga or outbox.

Add `or-else` for transaction decisions. The left alternative runs first. The
right runs only when the left constructs the distinguished `RetryWhenChanged`
decision and has performed no irrevocable effect. Both alternatives use the
same initial snapshot. This deterministic order avoids scheduler-dependent
result choice.

`or-else` is an ordinary language-provided infix function over compatible
decision callbacks, written `left or-else right`; it is not control-flow syntax.
Source authors may construct `Commit`, `Abort`, and `RetryWhenChanged` decisions
inside the callback. Only the transaction implementation constructs
`Committed`, `Conflict`, or `Aborted` after attempting the commit protocol.

A transaction domain defines its observable serialization/order contract.
Internal conflicts, CAS, STM logs, a serial task, or a database transaction are
implementation choices. The compiler may synthesize a lock-free implementation
without exposing those choices to source.

### External stores and compensation

An external adapter may supply `AtomicCommit`, `Durable`, and transaction
effects for a store. Their evidence names the exact domain, operation, failure
model, and external assumptions. This defines the semantic boundary needed by
an idempotent consumer and transactional outbox while leaving the ABI and
storage engine unspecified.

The compensation property uses the agreed pre-arrow placement:

```topal
undo-reservation is fn (
  reservation : Reservation
)
  effects ( Write reservation-store )
  guarantees (
    Compensates (
      forward is reserve,
      relation is reservation-compensation
    )
  )
-> result : Result ( Completed, CompensationErrorCode )
  ensures ( compensation-result-consistent reservation result )
```

The `Compensates` guarantee relates the declared compensation function to the
named forward function. Its relation states the precondition and observable
trace relation under which compensation holds; it does not claim that history
is erased. A saga records completed forward steps and applies compensations in
reverse declared order.

### Use and authority

Application and library authors may construct an in-process
`TransactionDomain` from an initial state, call `transact`, write decision
callbacks, combine them with `or-else`, inspect outcomes, and require
`RetrySafe`, `AtomicCommit`, `Durable`, or `Compensates` evidence. The checker
enforces callback effects and composition. The compiler/interpreter/runtime
implements the domain's atomic commit and may prove evidence for that concrete
implementation.

An external store adapter may construct an opaque domain implementation and
provide `AtomicCommit` or `Durable` evidence only through its checked provider
boundary, with exact domains, failure model, assumptions, and status. A source
generator or foreign translator may generate adapter source and declarations,
but cannot obtain a callable foreign ABI or mint transaction evidence under
this proposal. Qualification tools may validate transaction traces and provider
certificates without changing their status.

## TOPAL-PROP-TIME-001 — time and temporal contracts

### Values and effects

Add these parameterized semantic types:

```text
Clock
Instant Clock
Duration Unit
Deadline Clock
Tick Clock Sequence
```

These are type schemas, not declaration syntax. `Clock` is an opaque resource
interface. `Instant clock`, `Deadline clock`, and `Tick clock sequence` retain
the identity of one concrete clock value; `Duration unit` is an ordinary typed
quantity. The standard observation operation uses the agreed effect placement:

```topal
now is fn (
  clock : Clock
)
  effects ( Read clock )
-> Instant clock
```

`Clock`, `Instant`, `Duration`, `Deadline`, and `Tick` are unqualified
language-context type constructors, not keywords. A runtime or checked clock
adapter supplies a `Clock`; source code cannot construct an arbitrary instant
or forge a clock identity. Source authors may pass clock values, calculate with
compatible instants and durations, construct a deadline from an instant and
policy, and inspect ticks. The clock provider produces observations and states
whether monotonicity is verified or externally assumed.

An `Instant C` is opaque and comparable only with another instant from the same
clock identity. Subtracting two instants yields a duration; adding a duration to
an instant yields another instant or an explicit range error. A monotonic clock
guarantees that successive observed instants do not decrease. Reading a clock
remains the resource-parameterized effect `Read clock`.

A `Deadline C` is an absolute instant plus expiration policy. Relative timeout
syntax constructs one hidden deadline at entry and propagates that same value
through nested calls rather than restarting a duration. The existing
`with-timeout` structured operation accepts either a relative `Duration` or an
explicit `Deadline`; it is not a new keyword. For example,
`deadline with-timeout ( service request value )` uses the already-constructed
absolute deadline. An explicit deadline uses ordinary construction, for example
`deadline is Deadline ( instant is cutoff, policy is timeout-policy )`; the
clock identity is inferred from `cutoff`. Code may pass a deadline without
reading it. Inspecting time remaining performs `Read C`.

Time is an external observation and may differ between executions. Determinism
is preserved by including the clock interaction and the resulting timeout/tick
alternative in the declared protocol trace.

### Periodic release

Add a standard periodic generator parameterized by clock, period, phase, and a
late-release policy:

```topal
ticks is Periodic (
  clock is control-clock,
  period is 10[ms],
  phase is control-phase,
  late-policy is Coalesce
)
```

Each yielded tick carries its scheduled instant, observed release instant, and
sequence identity. The policy defines whether late ticks are delivered,
skipped, or coalesced. There is no implicit drift from repeatedly sleeping a
relative duration.

`Periodic` is a language-provided generator constructor. `CatchUp`, `SkipLate`,
and `Coalesce` are alternatives of its language-provided late-policy type. A
source author chooses the policy and consumes `Tick` values; the interpreter or
runtime provider observes the clock and yields them. A compiler may derive a
logical schedule, but only a clock/runtime or future architecture provider can
establish physical release observations or jitter evidence.

### Temporal implementation evidence

The core recognizes the shape, but not the physical verification, of:

```text
WorstCaseExecution ( scope is ResourceScope, bound is Duration )
ResponseWithin ( interaction is Interaction, bound is Duration )
ReleaseJitter ( source is PeriodicSource, bound is Duration )
DeadlineMet ( interaction is Interaction, deadline is Deadline )
```

These are implementation guarantees conditional on a clock and architecture
model. Until such a model is selected, they remain unknown and cannot satisfy a
hard requirement. They may be externally assumed for analysis, but the artifact
must expose that status and must not call the result unconditionally verified.

Priority, preemption, affinity, clock frequency, interrupt masking, and
scheduler admission are deliberately absent from this proposal.

All four names are unqualified language-provided implementation-guarantee
constructors. A function-level requirement is written before the arrow:

```topal
control-step is fn (
  input : SensorFrame
)
  guarantees (
    WorstCaseExecution (
      scope is Invocation,
      bound is 1[ms]
    )
  )
-> ControlCommand
```

A source author may require or prefer these guarantees but cannot produce their
evidence. Before the architecture model exists, a compiler or interpreter can
only reject a hard physical bound or carry provider-identified external
assumptions; it cannot label the bound verified. An approved future
architecture/scheduler provider may prove one for a selected implementation.
An external timing analyzer or qualification tool may return a certificate
through that future checked provider interface, but editing generated metadata
or translating a vendor claim does not establish evidence.

### Restricted immediate handlers

Define a derived classifier for a handler segment:

```text
ImmediateHandler (
  maximum-work is BoundExpression,
  allowed-effects is EffectExpression
) =
  NoAlloc
  and NonSuspending
  and ResourceBound (
    dimension is Work,
    scope is HandlerSegment,
    bound is maximum-work
  )
  and EffectsWithin allowed-effects
```

It is useful for polling and split-phase device handling. A future architecture
model may bind an external event or interrupt to such a handler and prove its
latency. The handler immediately acknowledges or captures bounded data and
transfers ownership to a normal task. The core does not define an interrupt
entry point or hardware priority.

`ImmediateHandler`, `NonSuspending`, `HandlerSegment`, and `EffectsWithin` are
unqualified language-provided classifier objects, not executable operations.
The writable use is a pre-arrow guarantee on an ordinary task handler:

```topal
capture-sample is fn (
  _ : MessageContext,
  device : SampleDevice
)
  effects ( HardwareAccess device )
  guarantees (
    ImmediateHandler (
      maximum-work is 32,
      allowed-effects is HardwareAccess device
    )
  )
-> result : Result ( Sample, DeviceErrorCode )
  ensures ( sample-result-valid device result )
```

Source authors may request this classifier but cannot assert its proof. The
checker derives non-suspension, effects, allocation, and work evidence from the
handler body; an opaque runtime handler must provide checked implementation
evidence. Only the future architecture provider may bind the handler to an
interrupt/event source or claim a physical response time.

## TOPAL-PROP-FLOW-001 — clocked static-rate dataflow

Add a static-rate refinement of typed streams:

```topal
DecimatedSamples is Flow (
  value is Sample,
  clock is audio-clock,
  consumes is 2,
  produces is 1,
  initial-delay is 0
)
```

Each rate option accepts an exact static `Nat` expression. `Flow` is an
unqualified language-provided stream classifier constructed with ordinary
named options, not a keyword. The clock option is a logical clock identity and
does not itself grant access to a physical clock.

Rates are exact nonnegative static values per logical tick. A mode-dependent
flow is a closed sum of individually static-rate modes; mode changes occur only
at declared tick boundaries. The language-provided ordinary construction
`delay ( initial is value, input is flow )` is the only way to introduce state
across a feedback edge; `delay` is not control-flow syntax.

The checker builds the flow graph and shall:

1. solve producer/consumer balance equations;
2. reject inconsistent rates;
3. reject a zero-delay causal cycle;
4. derive a periodic sequential schedule when one exists;
5. derive finite minimum channel capacities for that schedule; and
6. retain the schedule, capacities, and assumptions as implementation evidence.

The interpreter executes the derived sequential schedule one logical tick at a
time. Compilers may fuse actors, allocate static buffers, vectorize periods, or
map the graph to execution units. Physical placement, pipeline timing, DSP/FPGA
resources, and systolic processing-element layout remain architecture-model
work.

Ordinary task streams remain dynamic and do not acquire static-rate semantics
by inference unless the complete closed graph and exact rates are proved.

### Use and authority

Application, library, and generated source may declare `Flow` classifiers,
connect compatible flows, choose closed modes, and insert explicit `delay`
values. They may not assert a derived repetition schedule, minimum capacity,
causality proof, fusion plan, or hardware placement. The checker constructs the
closed graph, verifies rates and causal delays, and produces schedule/capacity
evidence. The interpreter executes the resulting logical schedule. A compiler
may use that evidence for buffers, fusion, and vectorization; only a future
architecture provider may establish physical placement, timing, DSP/FPGA
resource, or systolic evidence. A foreign graph translator has the same rights
as generated source and its imported rate claims are checked rather than
trusted by origin.

## TOPAL-PROP-HANDLER-001 — typed effect handlers

### Scope and resolution

Add lexically scoped handling for explicitly declared effect protocols. A
handler is selected statically; there is no runtime search through a dynamic
handler stack. Each effect operation has a typed input, result, resource
identity, and declared resumption mode.

An effect protocol uses one new language construction, `EffectProtocol`.
`operation` and `resumption` are contextual structural words only inside that
construction. Operation arguments precede `->`, and the value supplied when
the operation resumes follows it:

```topal
ConsoleEffects is EffectProtocol
  write is operation (
    console : Console,
    text : String
  )
    resumption is OneShot
  -> Unit
```

`EffectProtocol`, `OneShot`, and `Multiple` are unqualified language-context
objects. `ConsoleEffects write console text` applies the declared operation and
adds its resource-parameterized abstract effect; it does not invoke an ordinary
overload with the same terminal name. `Multiple` permits more than one resume
only when the handle site has verified `MultiShot` evidence.

A handler implementation is constructed with `Handler Protocol`. Each member
receives the operation arguments plus an affine resumption. The operation's
post-arrow type becomes the first parameter of `Resumption`; the second is the
complete handled-body result:

```topal
console-handler is Handler ConsoleEffects ( HandledResult : Type )
  write is fn (
    _ : Console,
    _ : String,
    continuation : Resumption Unit HandledResult
  )
    effects ( Effects () )
  -> HandledResult
    continuation resume Unit
```

`Handler`, `Resumption`, and `resume` are language-provided objects and
operations, not keywords. `( HandledResult : Type )` is a static classifier
binder supplied when the handler is applied, not a runtime argument. The
handler member is an ordinary function-shaped declaration, so its `effects`
clause is before its arrow. It returns the complete handled-body result either
by applying `resume` once or by constructing that final result without
resuming. If the handled-body result is a `Result`, it may return an error
admitted by that classifier; handlers do not acquire a separate exception
channel.

The handle expression itself introduces the contextual structural words
`handle` and `with`:

```topal
handled-result is handle body with console-handler
```

The handler must implement every operation in the handled closed effect set.
Its output type matches the body result, and its implementation effects replace
the handled abstract effects in the enclosing function's inferred row.
Unmatched effects propagate normally.

### Resumptions and resources

Resumptions are affine and one-shot by default. A handler must choose exactly
one of:

- resume once with an operation result;
- return a final handled result without resuming; or
- return an explicit error permitted by the handler contract.

Abandoning a resumption performs deterministic cleanup of resources owned by
the continuation. Resuming after the handler scope ends, resuming twice, or
retaining an affine resource across duplicated execution is invalid.

Multi-shot resumption requires language-defined `MultiShot` evidence proving
that the captured continuation and every retained value are duplicable, that
no affine resource is duplicated, and that repeated effects are permitted by
the handler contract. This evidence must be verified; programmer trust is not
sufficient for ownership safety.

`MultiShot` is an implementation/safety classifier inferred for a particular
handle site, not a keyword and not a claim a programmer may mark trusted. A
protocol author may choose `Multiple` as an operation's semantic resumption
mode, and a handler author may implement it, but the checker accepts the handle
site only after deriving `MultiShot` evidence for the captured continuation.

### Lowering

The interpreter represents a resumption explicitly. A compiler may inline a
lexically known handler, use direct-style exception-free control, transform
only effectful regions to selective CPS, or emit a state machine. No heap
continuation is semantically required for a statically eliminated or one-shot
handler. The selected lowering publishes allocation and specialization
evidence when a caller requires it.

Errors remain `Result` values. Effect handlers do not catch errors implicitly
and do not replace tasks, generators, or protocol handlers.

### Use and authority

The owner of an effect protocol may declare its operations and resumption modes.
Application and library authors may apply those operations, define complete
handlers, and write lexical `handle ... with ...` expressions. The checker owns
static handler selection, completeness, effect subtraction, affine resumption
checking, and `MultiShot` verification. The interpreter and compiler implement
resumptions and cleanup and may publish allocation/specialization evidence for
their selected lowering.

An external adapter may implement a handler whose implementation effects and
assumptions are explicit, but it receives no hidden authority beyond its
resource inputs. A foreign translator may emit an `EffectProtocol` and handler
adapter as checked Topal source; this does not create dynamic foreign callback
or unwinding semantics, which remain ABI work. Debuggers and qualification
tools may inspect operation, resumption, and cleanup traces but cannot resume a
program continuation or change its evidence.

## TOPAL-PROP-SPECIALIZE-001 — guaranteed specialization and planning

### Guaranteed specialization

Topal's current static functions, typed construction, generic intermediate
code, ordered overloads, and retained evidence already provide the machinery
for multi-stage specialization. Add one hard implementation guarantee:

```text
Specialized ( static-inputs is StaticObjectIdentities )
```

Inside a function's pre-arrow `guarantees` clause the classified function is
implicit. The guarantee means that the named static inputs and retained
implementation alternatives are substituted before final code generation, all
branches and interpreter structures depending only on them are eliminated, and
no runtime dictionary, tag, closure, or residual dispatch remains solely
because of those inputs. It does not require inlining unrelated dynamic work.

For example, `T` is bound statically by the input classifier and may be named by
the guarantee:

```topal
encode is fn (
  value : ( T : Serializable )
)
  guarantees (
    Specialized ( static-inputs is T )
  )
-> Bytes
```

The guarantee classifies `encode`, not `Bytes`. If a function returns another
function, a `Specialized` requirement on the returned function is written
inside that returned function's classifier around its own arrow.

The compiler normally derives and applies specialization without an annotation.
An opaque producer may publish a specialized machine implementation or typed IR
from which the consumer can produce one. A hard requirement fails if neither is
available. `Prefer (Specialized ...)` permits a generic fallback.

Static evaluation remains pure, total, hygienic typed construction. This
proposal does not add source-text quotation, untyped macros, runtime code
generation, or target inspection.

### Compiler-owned implementation plan

Every optimizing backend should use a typed implementation-plan IR separate
from semantic IR. It records proposed:

- fusion and materialization boundaries;
- iteration order, tiling, and vector/parallel partitioning;
- allocation region and buffer reuse;
- channel implementation and capacity;
- data layout and conversion points;
- transfer and completion dependencies; and
- the evidence and cost model supporting each choice.

This plan is compiler material, not a new program value and not visible through
ordinary `lang` introspection. Tools may emit a stable diagnostic projection so
humans and qualification systems can inspect why requirements were or were not
met. A reproducible build records the architecture-model identity, cost-model
identity, decisions, and verified bounds.

Only the compiler/backend creates or mutates the implementation-plan IR. A
runtime, external adapter, or producer of precompiled code may offer candidate
implementations and evidence through a checked input interface, but cannot
prescribe internal plan nodes. Source authors, source generators, debuggers,
and qualification tools may request a stable read-only projection. That
projection is not accepted as an edited plan or proof on a later build.

Algorithm/schedule separation is therefore preserved without making source
spell CUDA block sizes or Halide schedules. Portable code states semantic and
resource requirements; the compiler infers the plan. A future architecture
model may add implementation-only plan choices for memory domains, execution
units, DMA, systolic placement, or instructions.

### Hard implementation constraints

The initial portable plan constraints are expressed through existing or
proposed evidence rather than new pragmas:

- `NoAlloc` or a region/peak bound prevents an unwanted materialization;
- `ResourceBound ( dimension is TransferCount, scope is boundary, bound is 0 )`
  requires a zero-copy path;
- `Span`, `Progress`, or temporal bounds constrain scheduling;
- `Exclusive` enables reuse without mandating it;
- layout classification constrains representations at declared boundaries; and
- `Specialized` removes a binding-time abstraction.

If several plans satisfy all hard constraints, `Prefer` and the backend cost
model select among them. Plan choice never changes semantic overload
applicability.

### Use and authority

Source authors may require or prefer `Specialized` and name only static objects
visible in the declaration. They cannot assert that elimination occurred.
The compiler normally infers specialization and is the ordinary evidence
producer. A checked producer of typed IR or precompiled machine code may supply
`Specialized` evidence only when its artifact records the static inputs,
residual-code proof, provider identity, and compatible language revision. A
foreign translator can request specialization of generated Topal adapters but
cannot certify the resulting code shape. Analysis and qualification tools may
inspect and independently verify the read-only plan/code evidence.

## TOPAL-PROP-LAYOUT-001 — compositional data layouts

### Semantic shapes

Add a standard static `Shape` value: an ordered product of named or positional
dimensions with exact `Nat` extents. Arrays and tensor-like libraries retain
their shape as dependent static evidence. Shape is semantic; layout remains a
separate physical representation.

`Shape` and the extended `Layout` are unqualified language-context
constructors. They use existing named attribute construction rather than new
grammar. For example:

```topal
FrameShape is Shape (
  height is 1_080,
  width is 1_920
)

FrameTiles is Layout (
  dimensions is FrameShape,
  dimension-order is (
    FrameShape height,
    FrameShape width
  ),
  blocking is (
    ( FrameShape height, 8 ),
    ( FrameShape width, 16 )
  ),
  component-organization is Separated
)

StoredFrame is FrameTiles Frame
```

Extend `Layout` with a recursive sequence-layout vocabulary:

```text
dimensions           complete semantic dimension identities
dimension-order      permutation of those identities
strides               optional complete stride map
blocking              optional dimension -> positive tile extent map
component-organization Interleaved | Separated |
                       Blocked ( extent is PositiveNat )
```

`dimension-order` and `blocking` generalize today's one-dimensional `stride`.
Named `Shape` fields construct static dimension identities; ordinary field
selection such as `FrameShape height` supplies those identities to layout
options. The blocking value is an ordered product of
`( dimension-identity, positive-extent )` entries rather than special map
syntax.

Every field is checked for complete coverage, non-overlap, alignment, and total
storage size. Boundary tiles retain explicit extents; padding is reserved and
unobservable under the existing rules.

`component-organization` permits an array of semantic records to use AoS, SoA,
or AoSoA without rewriting the semantic type into a record of arrays. The
layout tree identifies each semantic field path and array dimension, so a
backend can calculate every component address and a debugger can reconstruct
the semantic value.

### Sparse layouts

Add library semantic sparse arrays parameterized by shape and zero value, plus
standard layout schemas for coordinate, compressed-dimension, and blocked
forms. A sparse layout explicitly records:

```text
value layout
index layouts
compressed dimensions and ordering
block shape
canonical ordering and duplicate policy
```

Reading validates bounds, ordering, duplicates, block shape, and zero-policy
invariants. Writing emits the canonical representation. Sparse iteration is an
ordinary capability whose operation identities remain available to backend
selection; a format name alone never promises speed.

`SparseArray` is a standard-library semantic type, while coordinate,
compressed-dimension, and blocked schemas are checked `Layout` constructions.
Their fields use the same `name is value` syntax as `FrameTiles`; the schema
block above lists accepted fields and is not source. A library author may define
new named combinations of the language-defined layout fields, but cannot add an
unchecked address-calculation rule or make a format name imply implementation
evidence.

### Views and conversions

A function may quantify over `L : Layout T` and retain `L` through typed generic
IR. A zero-copy view between layouts is valid only when the compiler proves that
the same storage satisfies both layouts and that lifetimes/access rights agree.
Otherwise conversion is an explicit fallible operation with allocation,
transfer, and peak-space evidence.

Internal layouts remain compiler-selectable. An explicit layout is required at
a stored, device, packet, or other representation boundary, or when a hard
implementation requirement depends on it. Layout selection does not identify a
memory address space, cache, device, ABI, or instruction.

When zero-copy behavior is required for an operation, it is a pre-arrow
implementation guarantee rather than a property placed after the view's result
type:

```topal
frame-view is fn (
  frame : StoredFrame
)
  guarantees (
    NoAlloc
    and ResourceBound (
      dimension is TransferCount,
      scope is Invocation,
      bound is 0
    )
  )
-> FrameView
```

### Use and authority

Application and library authors may construct shapes, request explicit layouts
at representation boundaries, quantify over layouts, and request conversion or
zero-copy guarantees. The checker validates all layout arithmetic, coverage,
alignment, lifetimes, access rights, sparse invariants, and conversions. The
compiler selects internal layouts and produces zero-copy, transfer, allocation,
and code-generation evidence for its selected implementation. The interpreter
may use a canonical representation while preserving the same semantic shape
and explicit conversion behavior.

A foreign header/module translator may generate `Shape` and `Layout`
constructions describing foreign records or arrays; those constructions are
checked exactly like handwritten source and do not create an ABI, foreign
symbol, borrowed pointer, or trusted offset. An external storage/device adapter
may supply validated bytes and provider assumptions through ordinary resources.
Debuggers may use retained layout evidence to reconstruct semantic values but
cannot alter layout evidence in a running or compiled program.

## TOPAL-PROP-INFOFLOW-001 — confidentiality and integrity labels

### Policy labels

Retain `Sensitive T` and quantitative `Leakage` as the simple provenance and
observation model. Add an optional general policy construction:

```text
InformationPolicy (
  confidentiality-label is LabelType,
  integrity-label is LabelType,
  can-flow-to is FlowRelation,
  join is JoinOperation,
  meet is MeetOperation
)

InformationLabel (
  policy is Policy,
  confidentiality is PolicyConfidentialityValue,
  integrity is PolicyIntegrityValue
)

Labeled (
  policy is Policy,
  label is PolicyLabel,
  value is T
)
```

This block gives constructor schemas; the field names and `is` tokens show the
ordinary source construction. `InformationPolicy`, `InformationLabel`, and
`Labeled` are unqualified language-context constructors, not keywords. A
concrete source declaration supplies named, statically known label types and
operations, then constructs labels and labeled classifiers from that policy.
The checker verifies the lattice laws rather than trusting the names of `join`,
`meet`, or `can-flow-to`.

For example:

```topal
MedicalPolicy is InformationPolicy (
  confidentiality-label is MedicalConfidentiality,
  integrity-label is MedicalIntegrity,
  can-flow-to is permitted-medical-flow,
  join is join-medical-labels,
  meet is meet-medical-labels
)

RestrictedValidated is InformationLabel (
  policy is MedicalPolicy,
  confidentiality is Restricted,
  integrity is Validated
)

ProtectedPatient is Labeled (
  policy is MedicalPolicy,
  label is RestrictedValidated,
  value is PatientRecord
)
```

The selected policy's labels form verified finite or symbolically decidable
lattices. Explicit data flow joins labels. A result whose control path depends
on a labeled value receives the appropriate implicit-flow label, including
effects and messages selected by that branch.

Confidentiality flows only upward according to `can-flow-to`; integrity flows
in the policy-defined dual direction. Application boundaries declare accepted
labels in addition to current sensitive-provenance rules.

### Authority-changing operations

`declassify` requires an unforgeable authority naming the source label, target
label, purpose, and scope. `endorse` analogously changes integrity and requires
its own authority. Both are observable auditable effects and retain provenance;
neither can be introduced by a conversion or trusted law claim.

Both are ordinary language-provided functions. Their source-level shape is:

```topal
declassify is fn (
  authority : DeclassificationAuthority (
    policy is Policy,
    source is Source,
    target is Target,
    purpose is Purpose,
    scope is Scope
  ),
  value : Labeled (
    policy is Policy,
    label is Source,
    value is T
  )
)
  effects ( Declassification authority )
-> Labeled (
  policy is Policy,
  label is Target,
  value is T
)

endorse is fn (
  authority : EndorsementAuthority (
    policy is Policy,
    source is Source,
    target is Target,
    purpose is Purpose,
    scope is Scope
  ),
  value : Labeled (
    policy is Policy,
    label is Source,
    value is T
  )
)
  effects ( Endorsement authority )
-> Labeled (
  policy is Policy,
  label is Target,
  value is T
)
```

The capitalized authority and effect names are language-provided constructors;
the lowercase function names are ordinary operations. Their effects precede
their arrows. Neither operation binds or refers to a returned value unless an
individual declaration adds a post-arrow `ensures` clause.

Static labels erase after verification. Dynamic principals or labels may need
runtime values and policy checks, which remain explicit. `Leakage` continues to
model quantitative channels such as timing and may be required in addition to
lattice flow. Target-specific cache, speculation, power, and physical
observation models remain architecture/security-model inputs.

### Use and authority

A policy owner may declare a policy and its label vocabulary and may arrange
for declassification or endorsement authorities to enter an application
through a restricted constructed context. Ordinary source may label data,
propagate it, declare accepted boundary labels, and call `declassify` or
`endorse` only when it possesses the exact authority. Source authors and
generators cannot construct, copy, widen, or derive those authorities through
ordinary values or trusted claims.

The checker verifies lattice laws, explicit and implicit flows, authority
identity/scope, and effect propagation. The compiler may erase static labels
only after verification. An external adapter may introduce labeled data and
provider-identified provenance but receives no declassification or endorsement
authority merely because the data is foreign. A policy-administration adapter
may supply an authority only through an application-approved capability
boundary. Analysis and qualification tools may audit the retained flow and
authority-use trace but cannot rewrite it. Architecture-specific leakage
evidence remains deferred.

## TOPAL-PROP-ARCH-SEAM-001 — future architecture evidence

### Core boundary defined now

The core shall define an `ImplementationEvidence` kind and matching rules, but
no architecture schema. Evidence from a future architecture model is attached
to a concrete implementation and has at least:

```text
model identity and version
implementation subject identity
property or resource-dimension identity
value or bound
assumptions and environmental scope
derivation/certificate identity
verification status
```

This is a compiler-artifact schema. `ImplementationEvidence` has no ordinary
source constructor, literal, declaration, or keyword. A source author requests
its property through the property's normal classifier—for example a pre-arrow
`guarantees ( NoAlloc )` clause or an ordinary call-site classification—and
never writes an `ImplementationEvidence (...)` value. Consequently it has no
pre-arrow or post-arrow placement of its own.

Architecture evidence participates in hard classifications and `Prefer` exactly
like other implementation evidence. It is available to overload/implementation
selection and the compiler's plan, but not to ordinary runtime functions or
static `lang context`. This prevents a source program from changing its value
semantics merely because it is compiled for a different target.

Architecture-aware packages may contain several semantically equivalent
implementations whose source headers state portable requirements or whose
compiled artifacts carry provider evidence. Merely placing evidence-shaped data
in a package does not classify an implementation. The compiler selects one only
after the closed application and model are known. When no implementation
satisfies a hard requirement, it reports the unsatisfied property rather than
silently weakening it.

The compiler and interpreter may create evidence for their own checked
implementations. A runtime or external adapter may submit provider-scoped
evidence through a typed verifier interface. A future architecture provider
may do the same only after its model and certificate rules are approved. Source
authors and source generators, including foreign header/module translators,
cannot invoke that provider interface from ordinary Topal code or label a
metadata record verified. Analysis and qualification tools may validate a
certificate and return that validation through an approved provider; otherwise
their output is a diagnostic, not evidence.

### What the future model must eventually describe

This proposal reserves no surface names, but the model will eventually need to
provide evidence about:

- execution units, concurrency, preemption, affinity, and scheduler service;
- clocks, timers, interrupt/event sources, and latency assumptions;
- memory domains, accessibility, coherence, cache, alignment, and capacity;
- transfers, DMA, barriers, completion, and ownership transitions;
- scalar, vector, matrix/tensor, DSP, and accelerator operations;
- code/data placement, spatial pipelines, and device launch;
- compartments, protection transitions, and capability preservation; and
- power, clock, memory, device, and other physical fault domains.

This list is a requirement inventory, not an architecture design.

### Portable ownership seam

The portable core may parameterize a resource by an opaque `Place` identity
supplied later and may express an ownership-transfer protocol between places:

```text
OwnedAt Place Region
transfer : OwnedAt A Region, Destination B -> Completion (OwnedAt B Region)
```

This block is deliberately a future type/signature schema, not source syntax.
`Place`, `OwnedAt`, `Destination`, and this form of `transfer` are not reserved
or introduced by the present proposal. If the architecture proposal adopts
them, `Place` values will be opaque provider-created identities, `OwnedAt` will
be a language-provided ownership classifier, and `transfer` will be an ordinary
effectful operation whose effects and guarantees precede its arrow. Source code
may then receive places, pass them, and participate in the typed protocol, but
will not construct, inspect, or forge them.

The sender cannot use the region after transfer submission except through the
completion protocol. Success returns ownership at the destination; failure
returns ownership in a declared state. Cancellation has an explicit winner and
cleanup rule. This is enough to type a double-buffered pipeline without naming
host memory, GPU global memory, NPU SRAM, DMA descriptors, cache flushes, or
fences. Those meanings and any zero-copy/overlap evidence come from the future
model.

If even opaque `Place` proves premature during detailed design, the transfer
protocol may remain in the standard library until the architecture model is
approved. No other proposal depends on a predefined list of places.

Only the future architecture/device provider could implement a physical
transfer and establish accessibility, completion, zero-copy, or overlap
evidence. The compiler would enforce ownership before selecting it; the
interpreter could use a semantic copy implementation when no hard physical
guarantee is present. A foreign translator could eventually emit calls through
the separately approved ABI and adapter model, but has no such permission in
this proposal.

## Deliberately deferred work

The following are outside this proposal and must not be inferred from its
examples:

1. Any foreign symbol, ABI, calling convention, primitive ABI mapping, linkage,
   callback-thread rule, exception convention, or direct borrowed foreign
   reference.
2. Concrete processor, GPU, NPU, DSP, MCU, FPGA, operating-system, cache,
   interrupt-controller, bus, or device descriptions.
3. Concrete memory spaces, DMA operations, barrier/fence instructions, launch
   grids, cooperative groups, tensor instructions, systolic placement, or
   accelerator schedules.
4. Scheduler algorithms, priority numbers, affinity syntax, priority ceilings,
   preemption rules, WCET calculation, or admission formulas tied to hardware.
5. Hardware compartment entry ABIs, sealed capability representations, process
   isolation mechanisms, or physical fault-domain topology.
6. Target observation models for cache, speculation, power, electromagnetic,
   or other side channels.

The core changes above make assumptions, requirements, ownership transitions,
and certified results representable. They do not claim the deferred evidence
exists.

## Explicit non-goals

- Users cannot implement or observe a CAS loop, lock, fence, RCU grace period,
  hazard pointer, epoch, or memory-order operation in portable Topal.
- Users cannot require the compiler to select one named lock-free, allocator,
  scheduler, tiling, fusion, or reclamation algorithm. They require semantic and
  resource properties.
- Target facts do not enter ordinary values, pattern matching, static language
  introspection, or serialized semantic results.
- An interpreter is not required to meet a compiled target's timing, progress,
  code-size, or placement guarantee. It must reject a hard requirement it
  cannot satisfy while remaining able to interpret the underlying semantic
  program when that requirement is a preference.
- A compiler-generated implementation plan is not a stable machine ABI and
  cannot be edited as unsafe code.
- `trusted-unverified` evidence never authorizes a violation of Topal's memory,
  type, bounds, ownership, totality, race, deadlock, or protocol safety.

## Change and implementation plan

The work should be delivered in independently reviewable semantic increments.
Each increment follows the repository authority order: approved human-readable
design, system goals/requirements, formal specification, conformance tests, and
shared implementation.

Every phase must carry forward two cross-cutting deliverables: an exact grammar
or explicit “no source form” statement for each introduced entity, and an actor
permission table covering declaration, use, evidence production, import, and
inspection. Source generators and foreign translators are tested as ordinary
source producers. Provider interfaces are separate typed tool/runtime inputs
and must not be reachable as an authority-escalation API from ordinary source.

### Phase 1 — contracts and evidence foundation

Purpose: establish the proof and implementation-evidence model used by every
later phase.

1. Extend `docs/abstractions.md` with the structural `requires`, `ensures`, and
   declaration-level `invariant` clauses, the named result contract binding,
   the proof-only call rule, and evidence status/provenance. Extend
   `docs/effects.md`, `docs/performance.md`, and `docs/modules.md` with the
   pre-arrow `effects` and `guarantees` clauses. The matching specifications,
   including `spec/functions.md`, and the corresponding entry in `decisions.md`
   must replace the current post-return colon rule for the new language revision
   while leaving `design-0` unchanged.
2. Extend `docs/capabilities.md` with the standard relational vocabulary in
   TOPAL-PROP-EVIDENCE-001 and `docs/modules.md` with its unqualified
   language-context name-resolution rules. Do not allow arbitrary names to
   acquire compiler meaning.
3. Extend `docs/interfaces.md`, `docs/modules.md`, and `docs/introspection.md`
   so public and generic artifacts retain contracts, evidence status,
   assumptions, and subject identities. Architecture evidence remains excluded
   from value-level introspection.
4. Add system requirements `TOPAL-REQ-CONTRACT-001` and
   `TOPAL-REQ-EVIDENCE-001`.
5. Specify contract satisfaction, call obligations, postcondition production,
   invariant preservation, evidence forgetting, and trust restrictions with
   stable rule IDs.
6. Implement proof objects and diagnostics before allowing optimizations to
   consume new evidence.

Exit condition: a separately compiled smart constructor can publish a verified
postcondition; a caller can consume it without rechecking; a false, unresolved,
or unsafe trusted claim is rejected with a minimal obligation trace.

### Phase 2 — resource, exclusivity, and region evidence

Purpose: make memory, jitter-related, and specialization requirements reliable
across opaque boundaries without exposing representation mutation.

1. Extend `docs/performance.md` with `ResourceBound`, architecture-neutral
   dimensions, progress classes, hard/preferred matching, composition, and
   unknown bounds.
2. Extend `docs/resources.md` and `docs/execution.md` with invocation-local
   `Exclusive`, optional `Consumes`, and scoped `AllocationRegion` lifetimes.
3. Extend generic typed IR in `docs/modules.md` to retain symbolic bounds,
   lifetimes, exclusivity requirements, and region identities.
4. Add system requirements `TOPAL-REQ-RESOURCE-BOUND-001` and
   `TOPAL-REQ-EXCLUSIVE-001`.
5. Specify conservative composition separately for work, span, total
   allocation, peak live storage, retained storage, stack, queues, transfers,
   and code size.
6. Implement inference first: liveness/escape, disjoint spans, lifetime overlap,
   stack analysis, and symbolic bound simplification. Explicit contracts are
   then thin checks over the same evidence.

Exit condition: a compiled public immutable update can prove in-place reuse, a
region-scoped parser can prove bounded peak storage, and an interpreter either
satisfies or explicitly rejects the same hard resource classifier.

### Phase 3 — synthesized concurrency and transactions

Purpose: support low-latency and atomic workflows without source atomics or
locks.

1. Extend `docs/tasks.md` and add `docs/concurrency-implementations.md` for
   inferred endpoint topology/admission evidence, synthesized implementation
   progress, published snapshots, and protected non-suspending handlers.
2. Add `docs/transactions.md` defining transaction domains, decisions,
   conflict, commit, cancellation, `or-else`, nested composition, external
   adapters, and compensation.
3. Extend `docs/effects.md` with transaction-scoped effects and trace relations.
4. Add system requirements `TOPAL-REQ-CONC-SYNTH-001` and
   `TOPAL-REQ-TRANSACTION-001`, tracing to the existing race, deadlock, transfer,
   and store requirements.
5. Specify linearization only for the high-level operations. The formal memory
   model continues to expose no source atomic event.
6. Implement a correct serialized/reference lowering first, then certified
   SPSC/snapshot/transaction alternatives. A backend publishes progress evidence
   only after its proof/test obligation is installed.

Exit condition: the same source endpoint can use a serial interpreter queue and
a compiled lock-free SPSC ring; a hard lock-free requirement selects only the
certified latter; and transaction failure never exposes a partial update.

### Phase 4 — time and static-rate flow

Purpose: express portable temporal intent and deterministic synchronous
behavior while leaving scheduler and hardware mapping for the architecture
model.

1. Add `docs/time.md` for clocks, instants, durations, deadlines, periodic
   release, lateness policy, and external time observations.
2. Add `docs/dataflow.md` for static rates, logical clocks, delays, balance,
   causality, derived schedules, and bounded buffers.
3. Extend `docs/tasks.md` so deadlines propagate through requests and structured
   cancellation without resetting.
4. Add system requirements `TOPAL-REQ-TIME-001` and
   `TOPAL-REQ-STATIC-FLOW-001`.
5. Specify clock identity, comparison, timeout races, tick traces, rate
   equations, causal cycles, and interpreter tick order.
6. Implement a deterministic logical-clock flow interpreter and an external
   monotonic-clock adapter. Physical deadline verification remains unavailable
   until an architecture model provides evidence.

Exit condition: a periodic control/dataflow graph has one defined interpreted
trace, statically derived finite buffers, explicit late-tick behavior, and
conditional deadline obligations with no implied scheduling claim.

### Phase 5 — handlers and guaranteed specialization

Purpose: close the general algebraic-effect and staging gaps without introducing
dynamic handler search or textual code generation.

1. Extend `docs/effects.md` and `docs/execution.md` with closed effect protocols,
   lexical handlers, affine resumptions, cleanup, and verified multi-shot use.
2. Extend `docs/performance.md`, `docs/introspection.md`, and `docs/modules.md`
   with `Specialized` and compiler-owned implementation-plan evidence.
3. Add system requirements `TOPAL-REQ-EFFECT-HANDLER-001` and
   `TOPAL-REQ-SPECIALIZE-001`.
4. Specify handler typing/effect subtraction, resumption state, abort cleanup,
   and guarantee matching. Keep `Result` and task/generator rules unchanged.
5. Implement the interpreter's explicit continuation form, then direct,
   selective-CPS, and state-machine compiler lowerings with differential tests.

Exit condition: a one-shot handled program has identical interpreted/direct/CPS
results and cleanup traces, and a static DSL interpreter can require that its
binding-time structures are absent from emitted runtime code.

### Phase 6 — layouts and information flow

Purpose: finish architecture-neutral representation and policy semantics before
introducing hardware models.

1. Extend `docs/layouts.md` with semantic shapes, multidimensional order,
   blocking, component organization, sparse schemas, and layout-polymorphic
   views. Leave its foreign-boundary section deferred.
2. Extend `docs/sensitive.md` with optional confidentiality/integrity policies,
   implicit-flow propagation, declassification, and endorsement.
3. Add system requirements `TOPAL-REQ-LAYOUT-COMPOSE-001` and
   `TOPAL-REQ-INFOFLOW-001`.
4. Specify address calculation, validation, canonical sparse representation,
   view compatibility, policy flow, authority use, and effect propagation.
5. Implement scalar reference paths before vector/sparse specialization. Labels
   may erase only after verified policy checking.

Exit condition: one semantic tensor can use AoS, SoA, AoSoA, blocked, or sparse
storage through checked layouts, and labeled control/data cannot cross an
unauthorized application boundary in either interpreter or compiler.

### Phase 7 — architecture model, explicitly later

After phases 1–6 have stable semantics, separately propose the architecture
model. That proposal must fill the evidence inventory in
TOPAL-PROP-ARCH-SEAM-001 and demonstrate CPU, GPU, NPU, and DSP/MCU profiles.
Foreign ABI design remains a separate later proposal even if it consumes the
same layouts, ownership protocols, effects, and implementation evidence.

## Proposed requirements and formal rule inventory

The following IDs are suggested so implementation work has explicit targets.
They do not become stable repository IDs until their design is approved.

| System requirement | Required formal rule families |
| --- | --- |
| `TOPAL-REQ-CONTRACT-001` | `TOPAL-FUNCTION-CLAUSE-PLACEMENT-001`, `TOPAL-CONTRACT-REQUIRES-001`, `TOPAL-CONTRACT-ENSURES-001`, `TOPAL-CONTRACT-INVARIANT-001` |
| `TOPAL-REQ-EVIDENCE-001` | `TOPAL-EVIDENCE-KIND-001`, `TOPAL-EVIDENCE-STATUS-001`, `TOPAL-EVIDENCE-NAME-001`, `TOPAL-EVIDENCE-PRODUCER-001`, `TOPAL-EVIDENCE-BOUNDARY-001`, `TOPAL-EVIDENCE-ASSUMPTION-001` |
| `TOPAL-REQ-RESOURCE-BOUND-001` | `TOPAL-PERF-DIMENSION-001`, `TOPAL-PERF-COMPOSE-001`, `TOPAL-PERF-PREFER-001`, `TOPAL-PERF-PROGRESS-001` |
| `TOPAL-REQ-EXCLUSIVE-001` | `TOPAL-VALUE-EXCLUSIVE-001`, `TOPAL-VALUE-CONSUMES-001`, `TOPAL-RESOURCE-REGION-001`, `TOPAL-RESOURCE-ESCAPE-001` |
| `TOPAL-REQ-CONC-SYNTH-001` | `TOPAL-CONC-TOPOLOGY-001`, `TOPAL-CONC-IMPL-001`, `TOPAL-CONC-SNAPSHOT-001`, `TOPAL-CONC-PROTECTED-001` |
| `TOPAL-REQ-TRANSACTION-001` | `TOPAL-TXN-DOMAIN-001`, `TOPAL-TXN-COMMIT-001`, `TOPAL-TXN-CONFLICT-001`, `TOPAL-TXN-CANCEL-001`, `TOPAL-TXN-COMPOSE-001` |
| `TOPAL-REQ-TIME-001` | `TOPAL-TIME-CLOCK-001`, `TOPAL-TIME-DEADLINE-001`, `TOPAL-TIME-PERIODIC-001`, `TOPAL-TIME-TRACE-001` |
| `TOPAL-REQ-STATIC-FLOW-001` | `TOPAL-FLOW-RATE-001`, `TOPAL-FLOW-BALANCE-001`, `TOPAL-FLOW-CAUSAL-001`, `TOPAL-FLOW-SCHEDULE-001` |
| `TOPAL-REQ-EFFECT-HANDLER-001` | `TOPAL-EFFECT-HANDLE-001`, `TOPAL-EFFECT-RESUME-001`, `TOPAL-EFFECT-CLEANUP-001`, `TOPAL-EFFECT-MULTISHOT-001` |
| `TOPAL-REQ-SPECIALIZE-001` | `TOPAL-FUNCTION-SPECIALIZED-001`, `TOPAL-IMPL-PLAN-001`, `TOPAL-IMPL-REPRODUCIBLE-001` |
| `TOPAL-REQ-LAYOUT-COMPOSE-001` | `TOPAL-LAYOUT-SHAPE-001`, `TOPAL-LAYOUT-BLOCK-001`, `TOPAL-LAYOUT-COMPONENT-001`, `TOPAL-LAYOUT-SPARSE-001`, `TOPAL-LAYOUT-VIEW-001` |
| `TOPAL-REQ-INFOFLOW-001` | `TOPAL-INFO-LATTICE-001`, `TOPAL-INFO-IMPLICIT-001`, `TOPAL-INFO-DECLASSIFY-001`, `TOPAL-INFO-ENDORSE-001` |
| `TOPAL-REQ-ARCH-EVIDENCE-001` | `TOPAL-IMPL-EVIDENCE-001`, `TOPAL-IMPL-SELECTION-001`, `TOPAL-IMPL-UNAVAILABLE-001` |

## Conformance and test strategy

Each phase requires positive, negative, boundary, interoperability, and
generated-code evidence.

### Common conformance cases

- Parse and type-check the proposed objects under a new selected language
  revision; `design-0` acceptance and meaning remain unchanged.
- Accept `requires`, `effects`, and `guarantees` only before the applicable
  function arrow; accept a named result and `ensures` only after that arrow.
  Apply the same rule independently to nested callbacks and returned functions,
  and reject a clause attached visually or semantically to the wrong arrow.
- Keep parameter-specific `Exclusive` and `Consumes` classifiers in the
  argument declaration and retain ordinary `:` classification at call sites;
  reject either as a post-return function classifier in the new revision.
- Round-trip every new contract/evidence object through generic typed IR and
  static introspection where permitted.
- Reject missing, refuted, incorrectly scoped, forged, or impermissibly trusted
  evidence.
- Run identical authority-negative cases for handwritten source, generated
  source, and translated foreign declarations. None may mint evidence status,
  topology counts, implementation plans, unforgeable information-flow
  authority, or deferred architecture/ABI objects.
- Accept provider evidence only through the typed provider interface, retain
  its provider, assumptions, language/model identity, and certificate, and
  reject copied or edited metadata as an evidence input.
- Verify that forgetting implementation evidence preserves semantic typing but
  may make a hard selection inapplicable.
- Run the same semantic examples in the interpreter and compiled scalar backend
  and compare values, errors, effects, cleanup, protocol transitions, and
  transaction/time traces.
- Confirm that a hard implementation requirement is rejected when the selected
  execution environment supplies no proof; confirm that `Prefer` selects a
  correct fallback.

### Specialized cases

- **Concurrency:** prove SPSC topology from linear endpoints; select a certified
  ring; reject a second producer; verify wraparound, capacity, cancellation,
  close, and progress metadata. Conformance and name-resolution tests must show
  that the selected language supplies no atomic, lock, fence, memory-order, or
  reclamation primitive; ordinary user identifiers with such names gain no
  special semantics.
- **Snapshots:** retain old views across publication, release in every order,
  enforce peak/retention bounds, and compare serialized/RCU/reference-counted
  lowerings for identical observations.
- **Transactions:** exercise success, explicit conflict, bounded retry,
  `or-else`, nested same-domain composition, forbidden effects, cancellation
  races, cleanup, outbox commit, and compensation failure.
- **Time:** exercise same-clock and cross-clock typing, one absolute propagated
  deadline, each late-tick policy, clock failure, and timeout/reply races.
- **Dataflow:** exercise balanced and unbalanced rates, initial delays, causal
  cycles, mode transitions, derived capacity, interpreter schedule, and fused
  compiled execution.
- **Handlers:** exercise resume, abort, error, cleanup, forbidden double resume,
  invalid multi-shot capture, effect translation, and each lowering strategy.
- **Specialization:** inspect emitted IR/machine metadata to prove that required
  static dispatch/AST structures are absent; include code-size-bound failure.
- **Layouts:** round-trip and negatively validate multidimensional, SoA/AoSoA,
  blocked-edge, and sparse layouts; prove when views are zero-copy versus
  explicit conversions.
- **Information flow:** exercise explicit and implicit flow, effect/message
  labels, declassification scope, endorsement scope, boundary rejection, and
  `Leakage` composition.

Generated-code tests establish evidence shape and required absence/presence of
operations; measured performance remains architecture-profile validation, not
portable conformance.

## Complete pattern disposition

The following table accounts for every catalog entry. “Current” means no new
fundamental semantics are needed. “Extend” names a proposal in this document.
“Synthesized” means the source expresses a higher-level contract and the
compiler may generate the named mechanism. “Architecture-later” identifies the
physical evidence that remains conditional.

| Pattern | Disposition | Support and remaining boundary |
| --- | --- | --- |
| AP-01 | Current + TOPAL-PROP-SPECIALIZE-001 | Existing functions/interfaces/capabilities express Strategy; guaranteed specialization makes zero-cost use enforceable across opaque boundaries. |
| AP-02 | Current layouts + TOPAL-PROP-EVIDENCE-001 | Validated adapters and authority are expressible; foreign ABI/linkage is deferred. |
| AP-03 | Current | Closed algebraic state, exhaustive patterns, and layout freedom are sufficient. |
| AP-04 | Current | Linear protocol endpoints, dependent state, and explicit close/cancel cover typestate/session use. |
| AP-05 | Current | Affine resource ownership and deterministic cleanup cover RAII. |
| AP-06 | TOPAL-PROP-CONTRACT-001 | Relational contracts close the pre/post/invariant and proof-erasure gap. |
| FD-01 | Current + TOPAL-PROP-RESOURCE-001 | Bulk combinators and laws permit fusion/parallelism; span/resource evidence makes selection enforceable. |
| FD-02 | Current + TOPAL-PROP-SPECIALIZE-001 | Parser combinators are expressible; specialization can guarantee removal of combinator overhead. |
| FD-03 | TOPAL-PROP-HANDLER-001 | Closed lexical handlers provide general typed operations and resumptions. |
| FD-04 | Current + TOPAL-PROP-SPECIALIZE-001 | Typed construction and interpreter functions are sufficient; specialization removes a static deep representation. |
| FD-05 | TOPAL-PROP-SPECIALIZE-001 | Existing static typed construction becomes a guaranteed staging boundary without quotations. |
| FD-06 | Current + TOPAL-PROP-UNIQUE-001 | Persistent sharing remains semantic; dead versions may be reused predictably. |
| MM-01 | TOPAL-PROP-UNIQUE-001 | Compiler-derived exclusivity or hard consumption evidence enables reliable in-place lowering. |
| MM-02 | TOPAL-PROP-UNIQUE-001 + TOPAL-PROP-RESOURCE-001 | Scoped regions and peak/retention bounds support arena behavior. |
| MM-03 | TOPAL-PROP-RESOURCE-001 | Peak, stack, queue, phase, and exact no-allocation evidence complete preallocation analysis. |
| MM-04 | Current + TOPAL-PROP-UNIQUE-001 | Owned regions and views cover CPU memory; device ownership uses TOPAL-PROP-ARCH-SEAM-001 later. |
| MM-05 | TOPAL-PROP-LAYOUT-001 | Recursive component organization adds SoA/AoSoA without changing semantic records. |
| MM-06 | TOPAL-PROP-SPECIALIZE-001 + TOPAL-PROP-LAYOUT-001 | Compiler-owned schedules can tile affine/bulk code; cache/scratch placement is architecture-later. |
| CC-01 | TOPAL-PROP-RESOURCE-001 | Existing scopes permit work stealing; work/span/grain/progress evidence controls applicability and jitter claims. |
| CC-02 | Current | Isolated tasks and implementation-transparent interactions cover actor/event-loop structure. |
| CC-03 | Current + TOPAL-PROP-CONC-SYNTH-001 | Existing protocols cover CSP; inferred topology/capacity/progress selects the channel implementation. |
| CC-04 | Synthesized by TOPAL-PROP-CONC-SYNTH-001 | A bounded SPSC endpoint may lower to a Disruptor-style ring; source has no atomics, cache lines, or wait-loop mechanism. |
| CC-05 | Synthesized by TOPAL-PROP-CONC-SYNTH-001 | `PublishedSnapshot` may lower to RCU/hazards/epochs; source cannot implement or observe those algorithms. |
| CC-06 | TOPAL-PROP-TRANSACTION-001 | Structured domains provide STM-like atomic composition with explicit conflicts and bounded retry. |
| RT-01 | TOPAL-PROP-TIME-001 + TOPAL-PROP-FLOW-001 | Periodic/logical execution is semantic; physical cyclic schedule/WCET is architecture-later. |
| RT-02 | TOPAL-PROP-TIME-001 + TOPAL-PROP-RESOURCE-001 | Period/deadline/bounds are expressible; priority, CPU assignment, preemption, and admission are architecture-later. |
| RT-03 | Synthesized protected handler | A serial non-suspending resource handler may later lower to priority ceiling; no source lock or priority exists now. |
| RT-04 | Current + TOPAL-PROP-RESOURCE-001 | Productive bounded polling and `NoAlloc` are expressible; core isolation/NUMA/frequency are architecture-later. |
| RT-05 | TOPAL-PROP-TIME-001 immediate handler | Restricted split-phase logic is checkable; interrupt binding, masking, and priority are architecture-later. |
| RT-06 | TOPAL-PROP-CONTRACT-001 + TOPAL-PROP-RESOURCE-001 | Termination plus exact work/space obligations prepare target WCET verification. |
| SR-01 | TOPAL-PROP-CONTRACT-001 + TOPAL-PROP-TIME-001 | Controller/switching semantics are portable; physical independence and verified deadline are architecture-later. |
| SR-02 | TOPAL-PROP-TIME-001 + TOPAL-PROP-EVIDENCE-001 | Heartbeat protocols are current; independent watchdog and schedule assumptions remain external/architecture evidence. |
| SR-03 | TOPAL-PROP-CONTRACT-001 + TOPAL-PROP-TRANSACTION-001 | Pure candidates, atomic commit, acceptance, and compensation cover recovery blocks. |
| SR-04 | TOPAL-PROP-EVIDENCE-001 | Voting is current; diverse implementations and fault domains are explicit external/architecture evidence. |
| SR-05 | TOPAL-PROP-CONTRACT-001 + TOPAL-PROP-TIME-001 | Typestate is current; guarded invariants and bounded response close the portable gap, with actuator binding later. |
| SR-06 | Current/library | Task ownership, typed termination, cleanup, and protocols support supervision; policy is a standard library. |
| ST-01 | Current + abstract boundary evidence | Capabilities/effects provide least authority; host/foreign binding is deferred. |
| ST-02 | Current + TOPAL-PROP-TRANSACTION-001 | Mediation is current; a transaction domain supplies atomic policy updates. |
| ST-03 | Current + TOPAL-PROP-TRANSACTION-001 | Linear authorities/protocols cover two-authority actions; transactions provide atomic consumption when required. |
| ST-04 | TOPAL-PROP-INFOFLOW-001 | Label lattices, implicit flow, declassification, and integrity endorsement provide general IFC. |
| ST-05 | Current `Sensitive`/`Leakage` + architecture-later | Core already carries quantitative modeled leakage; microarchitectural/physical observation models are deferred. |
| ST-06 | Current capabilities/layouts + architecture-later | Compartment authority can be modeled abstractly; entry ABI and capability-preserving hardware are deferred. |
| DS-01 | TOPAL-PROP-TIME-001 + TOPAL-PROP-EVIDENCE-001 | Existing results/bounds plus deadlines, randomness as an explicit service, and `RetrySafe` evidence support safe retry. |
| DS-02 | Current + TOPAL-PROP-TIME-001 | A task state machine implements the breaker; monotonic windows use explicit clocks. |
| DS-03 | TOPAL-PROP-RESOURCE-001 + TOPAL-PROP-EVIDENCE-001 | Logical isolation is current; budgets and fault-domain assumptions become evidence. |
| DS-04 | TOPAL-PROP-TRANSACTION-001 + TOPAL-PROP-EVIDENCE-001 | Stable IDs plus atomic/durable/retry contracts support deduplicating consumers. |
| DS-05 | Current + TOPAL-PROP-TRANSACTION-001 | Protocol state machines express sagas; compensation relations and durable adapters strengthen them. |
| DS-06 | TOPAL-PROP-TRANSACTION-001 | One domain commits business state and outbox record; publication remains a separately retryable effect. |
| HA-01 | Current + TOPAL-PROP-SPECIALIZE-001 | Pure bulk operations permit SIMT lowering; device eligibility/launch is architecture-later. |
| HA-02 | TOPAL-PROP-LAYOUT-001 + architecture-later | Shapes/blocks are portable; shared/local placement, groups, banks, and barriers are deferred. |
| HA-03 | TOPAL-PROP-LAYOUT-001 | Dimension order, strides, alignment, and retained layout evidence support coalesced representations. |
| HA-04 | Current + TOPAL-PROP-SPECIALIZE-001 | Pure graph semantics permit fusion; compiler plan/resource evidence makes materialization decisions inspectable. |
| HA-05 | TOPAL-PROP-ARCH-SEAM-001 | Portable ownership/completion supports double buffering; memory domains, DMA, fences, and overlap are deferred. |
| HA-06 | TOPAL-PROP-SPECIALIZE-001 | Algorithm remains semantic; compiler-owned plan performs schedule selection, with physical choices architecture-later. |
| HA-07 | Current numbers + TOPAL-PROP-LAYOUT-001 | Exact quantized semantics/layouts are portable; NPU instruction/operator evidence is architecture-later. |
| HA-08 | Current/library | `Approx`, explicit conversions, widening, and rounding permit mixed precision; standard operations/backends remain implementation work. |
| HA-09 | TOPAL-PROP-LAYOUT-001 | Canonical sparse semantic types/layouts expose invariants while backend evidence selects sparse hardware. |
| HA-10 | TOPAL-PROP-FLOW-001 + architecture-later | Affine/static flow is representable; processing-element placement and synthesis resources are deferred. |
| HA-11 | TOPAL-PROP-FLOW-001 | Clocked rates, delays, balance, and derived buffers provide synchronous dataflow semantics. |
| HA-12 | Current/library | Fixed point, explicit saturation/rounding, bits, and layouts already permit exact DSP instruction lowering. |

No catalog entry requires a public atomic, lock, ABI, or architecture syntax
under these dispositions. Hardware-dependent entries remain conditional rather
than being falsely labeled portable.

## Acceptance criteria for the completed program of work

The core-language program is complete when all of the following hold:

1. Every non-deferred semantic behavior in the disposition table has an
   approved human-readable design, requirement, formal rule, conformance test,
   and interpreter implementation under one selected language revision.
2. At least one compiled CPU backend consumes implementation evidence for
   specialization, exclusive reuse, bounded channels, snapshots, transactions,
   layouts, and resource bounds without changing interpreter-visible meaning.
3. Conformance and name-resolution tests demonstrate that the selected language
   supplies no atomic, lock, fence, memory-order, raw shared-reference, or user
   reclamation mechanism and grants no special semantics to similarly named
   ordinary declarations.
4. Hard progress/resource/specialization requirements fail closed when evidence
   is missing. Preferences execute a semantically equivalent fallback.
5. All proof and external-assumption statuses survive generic IR, separate
   compilation, linking, diagnostics, generated documentation, and traceability.
6. The architecture seam can accept a synthetic test model and select an
   implementation without allowing ordinary source computation to observe the
   model identity or target facts.
7. Deferred ABI and architecture items are each identified as unavailable; no
   test or document implies their guarantees before their later proposals are
   approved.
8. Every introduced entity has one documented source form or an explicit
   statement that it is metadata/deferred, plus conformance tests for which
   actors may declare, use, produce, import, and inspect it.

## Risk and review plan

This is a high-risk language program even though this file is only a proposal.
Contracts, effect handlers, transactions, resource inference, information flow,
and concurrency lowering affect type/effect soundness, cleanup, determinism,
memory safety, and security claims. Each implementation phase should receive an
independent semantic review and an implementation/conformance review. The
concurrency, transaction, temporal, and information-flow phases additionally
need specialized review for progress, race/deadlock interaction, cancellation,
side effects, and trust boundaries.

The main risks and controls are:

| Risk | Required control |
| --- | --- |
| Capability/property vocabulary grows into an uncheckable trait system | Permit library properties only when reducible to fixed relational logic; keep optimizer meanings version-owned. |
| Architecture choices leak into program semantics | Restrict architecture evidence to implementation selection and reject value-level introspection. |
| Trusted evidence compromises safety | Prohibit trust for memory/type/ownership/totality/protocol safety and compiler-issued progress claims. |
| Transactions introduce schedule nondeterminism | Use explicit conflicts and protocol-defined ordering; no hidden unbounded retry. |
| Effect handlers duplicate resources or cleanup | Affine one-shot default; verified multi-shot evidence; differential cleanup tests. |
| Resource guarantees become unverifiable promises | Unknown never satisfies hard bounds; preserve provider, assumptions, certificate, and model identity. |
| Generated source or editable metadata becomes an evidence-forging path | Give generators only ordinary source authority; accept evidence only through typed verifier/provider interfaces and reject edited projections. |
| Optimization directives make source backend-specific | Express constraints as semantic/resource evidence; keep schedules in compiler-owned plans. |
| Interpreter cannot honor physical guarantees | Reject unsupported hard implementation requirements and allow preference-based semantic fallbacks. |
| “Deferred” silently becomes “supported” | Maintain explicit unavailable diagnostics and negative tests until an architecture or ABI proposal is approved. |

## Resulting design character

The resulting Topal remains a pure, safe, deterministic semantic language. Its
source describes what must be true: invariants, effects, protocols, ownership,
transactions, information policy, time observations, and resource bounds. The
compiler is free to decide how those truths are implemented and may infer a
lock-free, in-place, fused, vectorized, sparse, or accelerator implementation
when the program and future architecture model justify it.

This is intentionally different from exposing the mechanisms used by C++,
CUDA, or an RTOS. It preserves a readable interpreted program while allowing a
compiled artifact to be as efficient as those mechanisms whenever certified
evidence exists. Where ABI or physical architecture evidence is indispensable,
the claim remains explicit and deferred instead of being approximated by an
unsafe or target-specific core feature.
