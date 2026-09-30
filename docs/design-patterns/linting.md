# Design-pattern linting and demonstrations

This document turns the design-pattern research catalog into an incremental
tooling plan. It does not change the semantics of a pattern or claim that a
source-level resemblance establishes a real-time, safety, security, hardware,
or distributed-systems guarantee.

## Disposition terms

- **Demonstrable** means current Topal source can implement the pattern's core
  structure in an executable example. A later family PR must add a documented
  positive example, a negative lint fixture, and resource-baseline coverage.
- **Partial** means an executable model can demonstrate some structure, but
  current Topal cannot establish at least one material condition of the
  pattern. The example and lint output must name that condition; they must not
  present the model as a complete implementation.
- **Deferred** means a material language, runtime, operating-system, device,
  distributed-system, or qualified-target facility is absent. The family PR
  records a feature-gap conformance fixture rather than a misleading imitation
  or a resource benchmark.

## Lint rule contract

Every design-pattern rule has a stable identity of the form
`design-pattern <ID> <rule>`, a catalog version, a rule-view revision, and one
of these checkability labels:

- `structural-advisory`: a bounded syntactic shape suggests a pattern could
  help. It is a warning, explains the evidence and likely false-positive
  boundary, and may be suppressed.
- `limitation-warning`: a recognized pattern-shaped construction lacks an
  observable condition required by the catalog entry. It is a warning and
  names the missing condition; it does not assert a runtime failure.
- `not-checkable`: the relevant condition is not available to source analysis.
  No source finding is emitted. The catalog/test fixture records the gap
  instead.

Rules use the versioned, read-only `design-pattern-syntax/1` lint view. The
linter derives bounded facts from parsed declarations, expressions, calls,
decisions, and lexical scope, then supplies each attachment only the facts in
its contract. The view contains no application values, filesystem, process,
network, debugger, architecture, timing, or ambient authority. The linter
remains responsible for parsing, source spans, diagnostics, policy, and
suppression.

## Example and measurement contract

For a demonstrable pattern, the family PR supplies:

1. a commented executable Topal example whose use case cannot reasonably be
   reduced to a trivial one-off expression;
2. a focused positive lint assertion for that pattern's stable rule IDs;
3. a commented negative fixture using an inadequate simpler or over-complex
   construction, with its expected warning IDs; and
4. an approved per-test resource baseline using average child CPU time and
   peak child resident memory. Wall-clock time is diagnostic only, not a
   conformance metric.

A partial pattern follows the same structure but labels the unproved condition
in the example and expected limitation warning. A deferred pattern instead
gets a comment-only feature-gap fixture and traceability entry; it has no
invented executable or resource claim. All examples preserve their ordinary
language conformance tests. Lint fixtures run through `topal-lint` and assert
the relevant finding rather than asserting a globally empty diagnostic set.

## Current catalog disposition

| Pattern | Disposition | Required missing condition or first lint focus |
| --- | --- | --- |
| AP-01 | Partial | Strategy shape is expressible; semantic capability/authority evidence is not yet source-complete. |
| AP-02 | Partial | Adapter shape and checked C boundary exist; general validated external boundaries need declared authority and failure evidence. |
| AP-03 | Demonstrable | `L-DESIGN-PATTERN-AP-03` advises on state-named Boolean fields and warns when a state-named `Union` reaches eight alternatives. |
| AP-04 | Partial | `L-DESIGN-PATTERN-AP-04` advises on a protocol-named Boolean and warns that an explicit protocol `Union` remains runtime-checked. |
| AP-05 | Partial | Lexical scoping exists; deterministic cleanup/destruction is absent. |
| AP-06 | Partial | Smart construction can be modeled; opaque construction, invariant propagation, and proof import remain incomplete. |
| FD-01 | Partial | Bulk pipelines are expressible; association/parallelism and resource/optimization evidence are incomplete. |
| FD-02 | Partial | Compositional parser structure is expressible; complete parser error, recursion, and performance evidence remains qualified. |
| FD-03 | Deferred | Effect-handler source grammar and execution are planned but absent. |
| FD-04 | Partial | A closed vocabulary and interpreter are expressible; specialization and multi-interpreter optimization evidence are incomplete. |
| FD-05 | Deferred | Staging annotations and specialization-plan evidence are absent from ordinary source. |
| FD-06 | Partial | Immutable update is expressible; persistence sharing and allocation evidence are incomplete. |
| MM-01 | Deferred | Source-level uniqueness transfer and compiler liveness/escape proof are absent. |
| MM-02 | Deferred | Region operations and compiler region/lifetime lowering are absent. |
| MM-03 | Deferred | Static allocation bounds and no-hot-path-allocation proof are absent. |
| MM-04 | Deferred | Source-level owned regions, validated views, and zero-copy proof are absent. |
| MM-05 | Deferred | Qualified array/layout declarations and representation selection are absent. |
| MM-06 | Deferred | Cache/target evidence and qualified tiling lowering are absent. |
| CC-01 | Deferred | Fork/join source operations and certified work-stealing runtime are absent. |
| CC-02 | Deferred | Actor/mailbox source operations and isolation runtime are absent. |
| CC-03 | Deferred | Channel operations and protocol/session checking are absent. |
| CC-04 | Deferred | Bounded ring storage, atomic publication, and target memory-order evidence are absent. |
| CC-05 | Deferred | Snapshot/RCU source operations and safe reclamation mechanism are absent. |
| CC-06 | Deferred | Transaction source operations and conflict/rollback execution are absent. |
| RT-01 | Deferred | Clock binding, physical schedule evidence, and qualified runtime are absent. |
| RT-02 | Deferred | Task, priority, WCET, and scheduler evidence are absent. |
| RT-03 | Deferred | Priority-ceiling lock semantics and target scheduler evidence are absent. |
| RT-04 | Deferred | Dedicated-core/device polling authority and latency evidence are absent. |
| RT-05 | Deferred | Interrupt/deferred-work binding and device authority are absent. |
| RT-06 | Partial | Bounded source structure can be modeled; whole-program WCET proof is absent. |
| SR-01 | Partial | Fallback/monitor shape is expressible; independent assurance and safe switching evidence are absent. |
| SR-02 | Partial | Heartbeat state can be modeled; clocks, failure detection, and independent watchdog authority are absent. |
| SR-03 | Partial | Alternatives and acceptance conditions are expressible; checkpoint/recovery and diversity evidence are incomplete. |
| SR-04 | Partial | A voter is expressible; fault independence and physical-fault evidence are external. |
| SR-05 | Partial | Safe-state transitions are expressible; invariant proof and physical fail-safe authority are incomplete. |
| SR-06 | Deferred | Supervision, restart, mailbox, and isolation runtime are absent. |
| ST-01 | Deferred | Object-capability authority cannot yet be created, transferred, and checked in ordinary source. |
| ST-02 | Partial | Fail-closed validation shape is expressible; complete mediation needs enforceable authority boundaries. |
| ST-03 | Deferred | Independent authority values and joint authorization semantics are absent. |
| ST-04 | Deferred | Information-flow labels, declassification, and source checking are absent. |
| ST-05 | Deferred | Constant-time source/target proof and secret-data annotations are absent. |
| ST-06 | Deferred | Hardware compartment authority and qualified deployment evidence are absent. |
| DS-01 | Partial | Bounded retry control flow is expressible; clock, jitter, failure classification, and remote-operation authority are absent. |
| DS-02 | Partial | Closed breaker state is expressible; elapsed-time and dependency-health evidence are absent. |
| DS-03 | Partial | Logical cell boundaries can be modeled; capacity and execution isolation are absent. |
| DS-04 | Deferred | Durable deduplication storage and at-least-once transport are absent. |
| DS-05 | Deferred | Durable transactional participants and compensating-operation authority are absent. |
| DS-06 | Deferred | Durable atomic outbox/store/transport integration is absent. |
| HA-01 | Deferred | Qualified SIMT kernel, launch, and target execution are absent. |
| HA-02 | Deferred | Address-space/shared-memory operations and target lowering are absent. |
| HA-03 | Deferred | Qualified tensor layout and accelerator memory-transaction evidence are absent. |
| HA-04 | Deferred | Kernel graph, fusion legality, and target lowering are absent. |
| HA-05 | Deferred | DMA/async transfer, events, and overlap evidence are absent. |
| HA-06 | Deferred | Algorithm/schedule source separation and target schedule provider are absent. |
| HA-07 | Deferred | Quantized type, calibration, and accelerator integer-lowering evidence are absent. |
| HA-08 | Deferred | Mixed-precision type, accumulation, and numerical-error evidence are absent. |
| HA-09 | Deferred | Sparse format, validity, and target sparse-operation evidence are absent. |
| HA-10 | Deferred | Static systolic topology and accelerator mapping are absent. |
| HA-11 | Deferred | Source dataflow operations and static schedule execution are absent. |
| HA-12 | Deferred | Fixed-point saturating type and DSP arithmetic evidence are absent. |

## Delivery order

The next PRs deliver demonstrable patterns by family in this order:
abstraction/lifecycle, functional/DSL, safety/robustness, then the partial
patterns whose structural warnings are sound under `design-pattern-syntax/1`.
Deferred entries advance only with the language/runtime/target work that
provides their missing condition. Each family PR includes documentation,
requirements, lint rules, positive and negative fixtures, resource traceability
where executable, and a review of false-positive risk.
