# Remaining design-pattern language work

- Status: residual implementation and deferred-design tracker
- Language revision: `v0.2`
- Pattern scope: [`docs/design-patterns`](docs/design-patterns/README.md)
- Detailed status: [`se/design-pattern-language-support.md`](se/design-pattern-language-support.md)

## Purpose

The accepted portable language design, requirements, and formal rules now live
in `docs/`, `se/`, and `spec/`. This non-authoritative root document merges the
former core-language proposal with the unversioned design-pattern comparison
report and retains only work which is deferred or does not yet have terminal
implementation evidence.

The research references and per-pattern descriptions remain in
[`docs/design-patterns`](docs/design-patterns/README.md). Completed design text
is deliberately not duplicated here.

## Fixed boundaries

The remaining work must preserve these approved decisions:

1. Topal exposes no source atomics, locks, fences, memory orders, reclamation
   primitives, or user-authored lock-free algorithms. A compiler may synthesize
   and certify such mechanisms from safe semantic code and constraints.
2. Function-wide `requires`, `effects`, and `guarantees` occur after the
   parameters and before `->`. A named result and `ensures` occur after `->`.
   Parameter properties remain on the parameter; invariants remain on the type
   or task whose state they govern.
3. The retry property is `RetrySafe`. Capitalized standard property
   constructors are unqualified language-context objects, not keywords and not
   `lang` members.
4. Implementation evidence can select only among semantically equivalent
   implementations. It cannot change values, errors, effects, ordering,
   cleanup, transaction outcomes, or protocol traces.
5. Architecture facts are inferred from code plus a future architecture model
   wherever possible. Ordinary source cannot construct or inspect them.
6. Foreign ABI and concrete architecture design remain separate later
   proposals.

## Foundations already present

The current `v0.2` change establishes enough structure for the remaining work
to proceed without another fundamental syntax redesign:

- authoritative portable designs and stable requirement/rule families for
  contracts, evidence, resources, generated concurrency, transactions, time,
  static-rate flow, handlers, specialization, layouts, and information flow;
- a shared ordinary/interface function-header tree, revision separation,
  executable interpreter pre/post checks, root property-name protection, and
  fail-closed handling of unavailable hard guarantees and parameter evidence;
- shared reference models for evidence authority, resource composition,
  exclusivity, regions, channels, snapshots, transactions, time, dataflow,
  resumptions, layouts, sparse values, and information-flow policy; and
- generic artifact revision 2 with contract and complete evidence-context
  retention while revision 1 remains the `v0.1` compatibility format.

These foundations are not terminal conformance for the work below. All six new
specification domains remain `planned` in
[`se/core-language-coverage.md`](se/core-language-coverage.md).

## Remaining non-deferred implementation

### 1. Contracts and evidence

- Parse and represent owned `invariant binding predicate` members after stored
  fields and before operations in nominal types and tasks. Prove establishment
  at construction and preservation at every public transition, including
  suspension boundaries.
- Parse clauses recursively in nested and returned function classifiers, not
  only ordinary and interface declarations.
- Replace diagnostic evaluation of accepted contracts with a static
  purity/totality checker, obligation generation, verified proof consumption,
  and safe check erasure. Diagnostic reevaluation may remain as an assertion on
  the toolchain, not as proof.
- Resolve every standard property constructor consistently in the checker,
  interpreter, LSP, debugger, documentation, and generated-source paths. Test
  that qualified library names have no language-defined optimizer meaning.
- Match hard and preferred guarantees structurally against typed evidence.
  Diagnostics must name the exact property, subject, known or unknown fact,
  assumptions, and considered candidates; whole-expression textual `Prefer`
  recognition is only an interim interpreter acceptance boundary.
- Retain owned invariants, effect/guarantee expressions, proof identities, and
  read-only diagnostic projections through generic substitution, separate
  compilation, import, and linking. Demonstrate that editable metadata cannot
  mint or promote evidence.

Terminal test: a separately compiled smart constructor exports a verified
postcondition which a caller consumes without a runtime recheck, while false,
missing, forged, refuted, or impermissibly trusted evidence fails closed.

### 2. Resource, exclusivity, and allocation evidence

- Implement liveness, last-use, escape, alias, disjoint-span, lifetime-overlap,
  stack, and symbolic-bound analyses in the compiler/checker.
- Bind `ResourceBound`, portable dimensions, `OExec`, `OAlloc`, `NoAlloc`,
  `Progress`, and `Prefer` to the overload/implementation selector. Unknown
  never satisfies a hard bound; preferences retain a correct fallback.
- Add source and interpreter operations for scoped `AllocationRegion`,
  `Allocate region`, checked promotion/copy, moved regions, deterministic
  cleanup, capacity failure, and escape rejection.
- Add a successful interpreter path for `Consumes` and any provable
  `Exclusive` call; until then the interpreter correctly rejects written hard
  classifiers. A compiler must demonstrate representation reuse without making
  mutation or allocator identity observable.

Terminal test: an opaque public immutable update proves in-place reuse, a
region-scoped workload proves peak storage, and the interpreter either enforces
or explicitly rejects the same hard contract.

### 3. Synthesized concurrency and transactions

- Bind `InteractionPolicy`, ordering, admission, `PublishedSnapshot`, `observe`,
  `publish`, transaction domains/views/decisions/outcomes, `transact`, and
  `or-else` to ordinary Topal source, effects, task protocols, and debugger
  traces using the shared reference models.
- Derive endpoint topology from linear ownership and the closed application;
  source and translators must not assert producer/consumer counts or a queue
  mechanism.
- Cover capacity, wraparound, retention pressure, close, cancellation,
  conflicting commits, same-domain nesting, distinct-domain composition,
  cleanup, staged outbox state, compensation failure, and explicit bounded
  retry.
- Add certified compiled SPSC, snapshot, and transaction implementations.
  `LockFree` or `WaitFree` evidence is published only by a concrete mechanism
  whose proof and stress-test obligation is installed.

Terminal test: serial interpretation and each generated mechanism have equal
observable traces; a hard progress requirement selects only a certified
implementation and never exposes a source atomic operation.

### 4. Time and static-rate dataflow

- Bind clocks, instants, durations, deadlines, timeout races, periodic sources,
  ticks, late policies, flows, delays, modes, and actors to source and semantic
  traces.
- Use one absolute propagated deadline through nested requests and structured
  cancellation. Exercise every completion/timeout/cancellation winner and
  monotonic-clock failure.
- Finish connected-clock checks, mode-transition balance, deterministic actor
  ordering, finite minimal capacities, and sequential interpreter execution.
- Add scalar compiled execution and differential tests for fusion,
  vectorization, and parallel mapping while preserving tick and value traces.

Physical WCET, priority, affinity, interrupt binding, and device scheduling are
not part of this slice; they remain architecture-dependent below.

### 5. Effect handlers and specialization

- Implement structural `EffectProtocol`, operation, resumption-mode, handler,
  and `handle body with handler` syntax in the shared tree and every source
  tool.
- Integrate lexical handler selection, effect subtraction, affine resume or
  abandon, deterministic continuation cleanup, explicit errors, and verified
  multi-shot capture into the interpreter.
- Implement direct, selective-CPS, and state-machine compiler lowerings and
  compare their values, effects, errors, and cleanup traces.
- Connect `Specialized` and the compiler-owned implementation plan to a real
  backend. Prove from emitted IR/machine metadata that named static inputs leave
  no residual tag, dictionary, closure, AST node, or dispatch. Include
  code-size-bound failure.

### 6. Composite layouts and information flow

- Complete recursive component-path coverage for `Interleaved`, `Separated`,
  and `Blocked`, then bind multidimensional, blocked, sparse, view, and explicit
  conversion operations to source and artifacts.
- Add positive and negative round trips for row/column order, AoS, SoA, AoSoA,
  boundary blocks, sparse duplicate/zero policy, alignment, overflow, zero-copy
  compatibility, and explicit allocation/transfer evidence.
- Bind policy and label declarations, direct and program-counter flow,
  boundary checks, declassification, endorsement, provenance, and audit effects
  to source, task messages, and ordinary effects.
- Prove label erasure and internal representation selection only after the
  static checks succeed; retain dynamic checks where a policy remains dynamic.

## Compiled performance and binary-code parity

The original comparison asked whether the language specification permits code
as efficient as the strongest comparison language, not whether today's
interpreter is fast. The portable design removes the identified expressiveness
barriers, but binary parity is still unverified because this repository has no
scalar production compiler/backend consuming the new evidence.

| Area | Comparison concern retained from the report | Required closure evidence |
| --- | --- | --- |
| Generic functions, strategies, combinators, static DSLs | avoid residual dispatch, dictionaries, closures, and interpreted ASTs as in Rust/C++/Futhark specializations | typed plan plus emitted-code inspection for inlining, monomorphization, fusion, and `Specialized` |
| Immutable updates and regions | avoid mandatory copies, reference counts, heap traffic, and peak-space growth compared with Rust/C/Futhark | escape/uniqueness proof, allocation-region lowering, allocation/peak/stack accounting, and generated load/store inspection |
| Channels, snapshots, transactions | match bounded queue/storage cost and tail behavior without exposing unsafe source mechanisms | certified mechanism selection, stress/model checking, bounded capacity/resource evidence, and semantic differential traces |
| Layouts, sparse data, quantization | match address arithmetic, packed size, vector access, and conversion count of C/CUDA/accelerator DSLs | canonical layout proof, zero-copy decision, explicit conversion/transfer counts, and backend IR inspection |
| Real-time and safety patterns | avoid hidden allocation, retry, blocking, or check jitter while retaining proof obligations | portable work/space proof now; physical WCET/response/jitter only under a later architecture/scheduler profile |
| Security and robustness | erase verified labels/contracts without trusting forged metadata or weakening failure behavior | proof/evidence authentication, negative authority tests, erasure inspection, and unchanged explicit error/effect traces |

Benchmarks may characterize a backend, but they do not prove portable
conformance. Closure requires structural generated-code evidence, resource
certificates, and differential semantic tests; architecture profiles may then
add measured timing, bandwidth, energy, or leakage validation.

## Explicitly deferred design

### Architecture model

A separate approved model must define execution units, clocks, preemption,
scheduler services, interrupts, memory domains, accessibility/coherence,
capacity, transfers and DMA completion, barriers, scalar/vector/tensor/DSP
operations, code/data placement, device launch, compartments, physical
observation models, and fault/power domains.

The existing opaque evidence seam may accept an optional model identity, but no
present component may verify physical WCET, response time, release jitter,
priority ceiling, placement, zero-copy device access, transfer overlap,
instruction selection, compartment strength, side-channel leakage, or physical
fault independence. The architecture proposal should prefer inference from the
closed application and model; source states only requirements or preferences.

Opaque `Place`, `OwnedAt`, and physical `transfer` remain unnamed future design,
not reserved source vocabulary. If adopted, ownership transfer must have an
explicit completion/cancellation winner and return ownership in every failure
state.

### Foreign ABI and compartment entry

A separate proposal must define foreign symbols, calling conventions, primitive
mapping, layouts at the call boundary, copied/owned/borrowed lifetimes,
destruction, effects, error/exception translation, callbacks into tasks,
threading, sandbox/compartment identity, capability preservation, and target
availability. Existing layouts and evidence do not grant pointer, symbol,
linkage, callback, or ABI authority. Header/module translators retain ordinary
source authority until that checked boundary exists.

## Residual pattern impact

Patterns not listed here need no additional fundamental core semantics, though
their libraries and optimized backends may still be absent.

| Remaining slice | Catalog patterns materially waiting on it |
| --- | --- |
| Contract proof, invariants, evidence import | AP-06, RT-06, SR-01, SR-03, SR-05 |
| Resource inference, uniqueness, regions, specialization | AP-01, FD-01, FD-02, FD-04–FD-06, MM-01–MM-04, MM-06, CC-01, RT-04, HA-01, HA-04, HA-06 |
| Generated concurrency and transactions | CC-03–CC-06, RT-03, SR-03, ST-02, ST-03, DS-04–DS-06 |
| Time and static flow | RT-01, RT-02, RT-05, SR-01, SR-02, SR-05, DS-01, DS-02, HA-10, HA-11 |
| Effect handlers | FD-03 |
| Composite/sparse layouts and information flow | MM-05, MM-06, ST-04, HA-02, HA-03, HA-07, HA-09 |
| Architecture model | MM-04, RT-01–RT-05, SR-01, SR-02, SR-04, ST-05, ST-06, DS-03, HA-01, HA-02, HA-05–HA-07, HA-10, HA-12 |
| Foreign ABI/compartment entry | AP-02, ST-01, ST-06 |
| Standard libraries/backend quality | SR-06, HA-08, HA-12 and optimized realizations of all rows above |

## Closure gates

The design-pattern program is complete only when all of these gates pass:

1. Every non-deferred rule has positive, negative, authority, compatibility,
   and source-tool evidence; each coverage-ledger row is `complete`.
2. Identical semantic examples run in the interpreter and a scalar compiled
   backend with equal values, errors, effects, cleanup, protocols,
   transactions, and time/dataflow traces.
3. The compiler demonstrates specialization, reuse, bounded channels,
   snapshots, transactions, layouts, and resource-bound matching through
   generated-code/artifact evidence.
4. Hard requirements fail closed and report complete obligation context;
   `Prefer` uses a semantically correct fallback.
5. Proof and assumption status survives generic IR, separate compilation,
   linking, diagnostics, documentation, and read-only tool projections.
6. Generated source and foreign translators cannot mint evidence, topology,
   implementation plans, or information-flow authority. Checked providers can
   supply only records within their typed authority.
7. A synthetic architecture-provider test exercises the opaque seam without
   exposing target facts to ordinary code. Physical claims remain unavailable
   until the architecture design is approved.
8. Deferred ABI and architecture entities remain explicitly unavailable and
   no documentation, test, or tool reports them as implemented.
9. Full workspace tests, formatting, linting, traceability checks, malformed
   artifact tests, and `git diff --check` pass after each closure increment.
