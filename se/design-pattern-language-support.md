# Design-pattern language-support architecture

This system view allocates portable `v0.2` design-pattern support across shared
toolchain components. Human-readable semantics are defined in `docs/`; formal
obligations are in `spec/`.

## Component allocation

| Concern | Shared owner | Interpreter responsibility | Compiler/artifact responsibility |
| --- | --- | --- | --- |
| Contracts and evidence | `topal-semantics` | Enforce executable accepted contracts and reject unavailable hard guarantees | Preserve proofs, assumptions, clauses, and subjects |
| Header and handler syntax | `topal-syntax` | Consume the shared tree | Consume the same tree; no private clause grammar |
| Resources and progress | `topal-semantics` | Publish only facts of its reference implementation | Infer bounds and certify selected mechanisms |
| Channels, snapshots, transactions | `topal-semantics`, `topal-language` | Serialized bounded reference behavior | Semantics-equivalent generated mechanisms |
| Time and dataflow | `topal-semantics`, clock adapters | Recorded/logical clock and sequential schedule | Preserve logical trace; physical proof requires architecture evidence |
| Effect handlers | `topal-semantics`, `topal-language` | Explicit affine resumption | Direct/CPS/state-machine lowering with differential evidence |
| Specialization plans | `topal-geir` and future compiler | Reject hard compiled-code-shape claims | Sole producer of plan and code-shape evidence |
| Layout and information flow | `topal-semantics` | Canonical scalar/reference representation | Checked representation selection; erase labels only after proof |

## Trust and authority boundary

Ordinary source, generated source, and translated foreign declarations all
enter through the same syntax and checker. None can set evidence status,
producer identity, topology counts, implementation-plan nodes, or
declassification/endorsement authority. Checked runtime and external providers
submit typed records to a verifier outside ordinary source execution. Provider
identity and assumptions remain in every accepted record.

Semantic proof may affect program admission. Implementation evidence can select
only among implementations already proven semantically equivalent. Missing
hard implementation evidence fails closed; missing preferred evidence uses a
correct fallback.

## Revision and compatibility

`v0.1` remains accepted with its post-result effect/resource classification.
`v0.2` uses pre-arrow `requires`, `effects`, and `guarantees` and a post-arrow
named result plus `ensures`. Artifacts record their revision. Cross-revision use
requires a defined compatibility projection; no tool silently reinterprets a
header.

## Implementation status

The accepted design and formal rules intentionally lead implementation. The
current change establishes the shared foundations below; a row is not terminal
until the core-language coverage ledger changes it from `planned` to
`complete`.

| Slice | Implemented in this change | Remaining terminal evidence |
| --- | --- | --- |
| Contracts and evidence | `v0.2` ordinary/interface header tree, executable interpreter pre/post checks, fail-closed hard guarantees, typed evidence policy, revision-2 GEIR retention | static proof/erasure, owned invariant syntax and preservation, nested function-type clauses, proof import/export through a compiled caller |
| Resources and exclusivity | dimension/progress composition, exclusivity and region reference checkers, parameter syntax, interpreter fail-closed boundary | compiler liveness/escape/span analyses, source region operations, concrete bound matching, successful interpreter consumption support |
| Concurrency and transactions | deterministic channel-selection, snapshot, and transaction reference models | source operation binding, protocol/effect integration, cancellation race corpus, certified generated SPSC/snapshot/transaction mechanisms |
| Time and dataflow | clock, deadline, periodic policy, balance, causality, capacity, and sequential schedule models | source operation binding, trace integration, mode transitions, compiled differential execution |
| Handlers and specialization | affine/multi-shot resumption model and compiler-only typed plan authority | effect-protocol/handler source grammar and execution, cleanup integration, direct/CPS/state-machine lowerings, emitted-code evidence |
| Layout and information flow | multidimensional/sparse validation, zero-copy predicate, finite lattice, program-counter propagation, scoped authority | complete component-path model, source checking/operations, message/effect labels, compiler representation selection and erasure proof |
| Architecture seam | opaque optional model identity and fail-closed evidence boundary | synthetic provider integration test and the separately approved architecture model |
| Compiled parity | artifact schema can carry required evidence | a scalar CPU backend and generated-code/differential tests; no current repository component can establish binary-code parity |

These gaps are also retained in the repository-root remaining-work document.
Reference-model tests are implementation evidence for their algorithms, not a
claim that the corresponding Topal source forms or compiled lowerings already
exist.

## Deferred provider boundary

The core retains an opaque implementation-evidence schema with optional
architecture-model identity. It does not define target profiles, processors,
device launches, physical clocks, schedulers, memory domains, transfers, fault
domains, foreign symbols, or ABIs. Later architecture and ABI proposals must
use this verifier boundary and cannot make target facts semantic values.

## Risk and verification

This is high risk because it affects proof admission, ownership, concurrency,
transactions, information flow, and compatibility. Verification requires
positive, negative, authority-boundary, and cross-revision cases; deterministic
model tests; parser and source-tool corpora; canonical artifact round trips;
and full workspace tests. Generated synchronization may claim progress only
after a mechanism-specific proof/test obligation exists. Physical measurements
validate a later architecture profile and are not portable conformance.
