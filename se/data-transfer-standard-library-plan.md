# Data-transfer standard-library implementation plan

This plan turns the approved
[data-transfer and external-resource architecture](data-transfers.md) into a
dependency-ordered pull-request series. It realizes `TOPAL-REQ-TRANSFER-001`,
`TOPAL-REQ-DATA-VIEW-001`, `TOPAL-REQ-STORE-001`, and
`TOPAL-REQ-TRANSPORT-BINDING-001` without enlarging the completed fundamental
`std` namespace or granting ordinary library code ambient platform authority.

## Scope and completion boundary

The series delivers an extended standard-library foundation for:

- typed endpoint protocols, operation submission, completion, cancellation,
  and backpressure;
- messages, packets, frames, sequences, regions, and validated layered views;
- local messaging and replaceable IPv4, IPv6, file, database, and device
  bindings;
- common store identity, namespace, transaction, consistency, durability, and
  change contracts;
- I2C controller, target, and combined-transaction support; and
- bounded-copy processing, scatter/gather, batching, DMA obligations, and
  equivalent software or hardware offload.

Completion means that every validation scenario in `se/data-transfers.md` has
normative rules, ordinary Topal interfaces where expressible, a deterministic
reference implementation or platform boundary, cross-tool evidence, and an
explicit terminal disposition. It does not require every database engine,
network protocol, filesystem, bus, or operating system to ship in the initial
library.

Algorithms such as sorting and searching remain a separate extended-library
effort. This series may use already admitted fundamental algorithms but shall
not mix unrelated algorithm expansion into its PRs.

## Package and authority architecture

The public surface belongs to separately versioned ordinary Topal packages,
not the flat fundamental `std` module. The intended public namespaces are:

```text
transfer   endpoint, operation, completion, message, sequence, and adapters
data       owned regions, spans, validated views, packets, frames, and codecs
store      identity, namespace, transaction, snapshot, change, and guarantees
network    addresses, resolution, IPv4, IPv6, transport, and service bindings
device     controller, target, register, DMA, and bus protocols
```

Physical package layout and exact source names are admitted in the first phase;
namespace names are architectural ownership boundaries, not approval of an API
spelling in advance.

Ordinary Topal code defines protocols, adapters, validation, store logic, and
reference behavior. Irreducible platform access enters through a small shared
host boundary that:

- receives explicit capabilities from the embedding application;
- exposes no filesystem, network, clock, process, or device authority by
  default;
- returns versioned typed observations rather than host handles or error
  numbers as language semantics;
- supports deterministic mock and replay backends;
- separates operating-system mechanism from library policy; and
- is consumable by the interpreter and future compiled runtime without
  defining two meanings.

The language server and linter analyze contracts and source without opening
external endpoints. The source debugger records external completions and
replays their observations without repeating external effects.

## Admission rules

Every public interface and adapter shall satisfy these rules:

1. Use the narrowest capability and data shape that preserves the operation's
   semantics; do not replace a message, transaction, or addressed region with a
   generic byte stream.
2. Keep service identity, resource identity, name, path, and transport address
   distinct.
3. State ordering, atomicity, delivery, retry, cancellation, timeout,
   consistency, durability, and resource-limit behavior explicitly where it
   can affect correctness.
4. Treat untrusted representations as data until validated; views retain their
   source-span and evidence dependencies.
5. Preserve immutable source semantics while permitting proven unique mutation,
   buffer reuse, batching, scatter/gather, and offload.
6. Make partial transfer, partial parsing, endpoint loss, unsupported
   capability, and resource exhaustion typed outcomes.
7. Do not promise zero-copy, bounded latency, durability, or delivery without
   measurable and enforceable preconditions.
8. Keep platform-specific properties available through typed refinements
   rather than strings, optional fields, or undocumented adapter behavior.

## Pull-request series

### 1. Roadmap, package boundaries, and conformance matrix

- Admit exact public package/module ownership and versioning.
- Inventory reusable `design-0` task, protocol, resource, layout,
  serialization, and trace rules and identify missing formal semantics.
- Add a matrix assigning every architecture obligation to specification,
  library package, host boundary, tools, examples, tests, and terminal phase.
- Record deliberate initial omissions by protocol, platform, and store kind.
- Add a closure gate so no row becomes complete through an evidence substring
  or aggregate test count alone.

Acceptance: the matrix covers every statement in the architecture validation
scenarios and distinguishes direct, shared, platform-boundary, and deferred
evidence.

### 2. Endpoint and protocol foundation

- Formalize endpoint identity, capability authority, legal protocol state,
  typed operations, typed failures, and endpoint closure.
- Define service identity separately from endpoint and address identity.
- Add protocol compatibility and adapter-preservation rules.
- Implement an in-memory endpoint pair and a typed request/reply service as the
  first complete vertical slice.
- Trace endpoint creation, submission, transition, completion, and closure with
  stable semantic identities.

Acceptance: one service executes unchanged through direct function application
and local task-message binding, with equivalent values and applicable semantic
traces.

### 3. Submission, completion, cancellation, and backpressure

- Formalize stable operation identities and the completion relation.
- Define immediate completion as the same semantic operation as queued
  completion, not a separate synchronous API meaning.
- Specify cancellation races, endpoint loss, timeout uncertainty, queue bounds,
  and completion ordering.
- Implement deterministic submission and completion queues over the shared task
  scheduler.
- Add retry admission based on idempotence, deduplication, or transaction
  identity.

Acceptance: exhaustive state-machine tests cover completion before cancellation,
cancellation before completion, simultaneous observation, endpoint loss, queue
exhaustion, and non-idempotent retry rejection without duplicated effects.

### 4. Owned data regions and spans

- Admit byte and generic element regions independently of text and List
  semantics.
- Formalize owned, shared immutable, uniquely mutable, pinned, and externally
  backed region capabilities.
- Add checked spans, slicing, concatenated/scatter-gather descriptions, and
  explicit alignment and addressability obligations.
- Provide ordinary immutable reference operations and a host substitution
  boundary for efficient storage and transfer.
- Define copy-count and allocation instrumentation used by later performance
  gates.

Acceptance: ownership, overlap, boundary, lifetime, alignment, resource-limit,
and scatter/gather tests agree across reference and substituted implementations.

### 5. Validated views and recursive encapsulation

- Formalize message, packet, frame, sequence, and region distinctions.
- Add validated views tied to exact source spans and validation evidence.
- Define semantic containment separately from encoded encapsulation.
- Track evidence dependencies so unique mutation invalidates overlapping views
  and preserves independent views.
- Add incremental parsing with explicit incomplete, malformed, unsupported, and
  exhausted outcomes.

Acceptance: nested Ethernet/IPv4/TCP/application and
Ethernet/IPv6/TCP/application examples share one region; targeted header changes
invalidate only dependent evidence and require no payload copy.

### 6. Sequences, messages, and framing adapters

- Define ordered sequence source/sink protocols, partial transfer, end,
  half-close where applicable, and flow control.
- Define application message boundaries and message-to-sequence framing codecs.
- Add length-delimited and validated native-serialization message adapters.
- Preserve packet/frame boundaries through datagram-style endpoints.
- Add batching and pipelining without changing individual operation identity.

Acceptance: arbitrary chunking cannot change decoded messages, malformed or
oversized frames fail deterministically, and batching preserves declared order
and backpressure.

### 7. Platform boundary and deterministic virtual backend

- Specify the versioned host-operation ABI independently of POSIX or Windows
  numeric handles and error codes.
- Implement a deterministic virtual backend for endpoints, clocks, names,
  stores, networks, and devices.
- Add capability injection and denial at interpreter/runtime embedding
  boundaries.
- Add debugger recording and replay of external completion observations.
- Prove that LSP and linter analysis performs no live host access.

Acceptance: all subsequent functional suites can run without ambient machine
state, and denied authority fails before a host operation is submitted.

### 8. Network identity, resolution, IPv4, and IPv6

- Add distinct IPv4 and IPv6 address, prefix, scope, endpoint-address, and
  packet-view types with shared capabilities only where semantics agree.
- Specify resolution from service identity to ordered candidate endpoints under
  explicit policy.
- Preserve IPv4 header/checksum/fragmentation and IPv6 extension/scope/
  fragmentation distinctions.
- Add deterministic virtual routing, MTU, loss, duplication, reordering, and
  reachability scenarios.

Acceptance: packet golden vectors and malformed cases pass independently for
IPv4 and IPv6; address strings never substitute for typed identity or scope.

### 9. UDP, TCP, and transport-independent services

- Bind message endpoints to UDP without losing datagram boundaries or delivery
  limitations.
- Bind sequence endpoints to TCP with connection, ordered partial transfer,
  half-close, reset, and backpressure semantics.
- Bind one typed application service independently to local messages, TCP/IPv4,
  and TCP/IPv6.
- Make dual-stack listening explicit candidate composition; allow a host
  adapter to combine it only when platform behavior is equivalent.
- Add authentication/security adapter slots without designing a private TLS
  substitute in this phase.

Acceptance: the service produces equivalent semantic results through all three
bindings while traces retain distinct transport failures and addresses.

### 10. Store identity, schema, namespace, and change foundation

- Formalize store, object, schema, relation, query, and change identities.
- Define explicit lookup, insertion, replacement, removal, snapshot, and
  subscription capabilities.
- Separate object identity from names and name-resolution queries.
- Implement deterministic in-memory key-value, relational, and graph reference
  stores sufficient to validate their shared foundation and distinct query
  models.
- Define stable change messages and bounded subscription backpressure.

Acceptance: shared store laws hold without forcing relational or graph queries
through paths, byte streams, or one universal query representation.

### 11. Transactions, consistency, durability, and replication contracts

- Formalize transaction identity, commit, abort, conflict, isolation,
  consistency, durability, placement, and replication requirements.
- Distinguish acknowledged submission from durable commit.
- Define retry and uncertain-outcome behavior for lost completions.
- Add a deterministic fault-injection model for partitions, replica loss,
  stale reads, conflicts, and recovery.
- Admit only guarantees that an adapter can demonstrate or conservatively
  reject.

Acceptance: litmus tests distinguish serializable, snapshot, causal, and
eventual observations; no weaker adapter silently satisfies a stronger request.

### 12. File-store specialization

- Define files as identified objects with addressed content and structured
  metadata; directories as name-to-object relations; links as direct or
  indirect relations; and paths as resolution queries.
- Specify open-capability behavior across rename, unlink, mount, and namespace
  changes.
- Add atomic rename/update requirements, snapshots and change messages where
  supported, and explicit behavior where they are unavailable.
- Implement an in-memory reference file store followed by one capability-rooted
  host-filesystem adapter.
- Exercise local and virtual distributed file stores through the same object
  and namespace contracts with different declared guarantees.

Acceptance: traversal cannot escape granted roots, path races do not replace
object identity, and distributed failures remain distinguishable from local
not-found outcomes.

### 13. Database adapter boundary

- Define prepared operation, parameter, row/message, cursor/sequence, and
  transaction adapter contracts without embedding one vendor protocol.
- Map relational and graph results into typed application values through
  explicit schemas and validation.
- Preserve database error vocabularies and uncertain commit outcomes.
- Add one deterministic reference adapter and one real relational adapter
  behind opt-in integration tests.
- Record graph and document adapter obligations even if their first real
  backends remain deferred.

Acceptance: query values are not constructed through string interpolation,
row/schema mismatches fail explicitly, and reference and real adapters satisfy
the same transaction scenarios.

### 14. Device-controller and DMA foundation

- Formalize controller, target, command, status, interrupt/event, register, and
  transfer-queue protocols.
- Add explicit alignment, pinning, coherency, addressability, cache, and device
  lifetime obligations.
- Distinguish declarative command submission from volatile register access.
- Implement virtual controller and device backends before exposing real device
  authority.
- Define safe scatter/gather and buffer handoff through unique ownership.

Acceptance: simulated DMA completion, device removal, reset, timeout, queue
exhaustion, and stale-buffer access cannot leak or duplicate ownership.

### 15. I2C vertical slice

- Define controller and target capabilities, typed address modes, speed and
  transfer limits, and shared-bus scheduling.
- Represent start, repeated-start, direction changes, acknowledgement,
  arbitration, clocking, stop, recovery, and combined transaction atomicity.
- Add register-region views only for devices whose access contracts justify
  them.
- Implement a deterministic virtual bus and one capability-rooted host adapter.
- Build a typed sensor service over a combined register-address write/read.

Acceptance: negative acknowledgement, arbitration loss, device disappearance,
partial transfer, unsafe retry, and bus recovery are independently testable;
the sensor service does not expose raw bus authority.

### 16. Zero-copy firewall and offload equivalence

- Build the architecture's firewall scenario over nested frame and packet views.
- Inspect different layers without payload copying, update selected headers with
  evidence invalidation, and forward through scatter/gather.
- Add declarative checksum, segmentation, encryption, and framing substitutions
  where admitted.
- Compare pure software, operating-system-assisted, and simulated hardware
  offload results and traces.
- Establish platform-qualified copy-count, allocation, queue-depth, throughput,
  and latency baselines.

Acceptance: functional equivalence is exact; performance claims state the
platform, workload, measurement method, variance, and allowed regression rather
than relying on elapsed-time anecdotes.

### 17. Cross-platform adapters and final audit

- Add adapter conformance kits for POSIX-style readiness, Linux completion
  queues, Windows completion ports, capability kernels, and other supported
  hosts without making any one model normative.
- Audit every public capability for ambient-authority leaks and every external
  operation for explicit effects and cleanup.
- Run malformed-input fuzzing, state-machine/model tests, fault injection,
  differential adapter tests, debugger replay, and resource/performance suites.
- Close the conformance matrix with implemented, platform-specific, or
  deliberately deferred dispositions.
- Publish compatibility and versioning rules for endpoints, protocols, encoded
  data, stores, and host-operation ABIs.

Acceptance: all eight architecture validation scenarios are terminal and every
supported adapter passes the same applicable conformance kit.

## Evidence required in every phase

Each phase updates, in authority order, every applicable design clarification,
system trace, formal rule, public API, tool requirement, example, test, and
implementation. Every public operation records:

- exact input, output, typed failure, effect, capability, and lifetime
  contracts;
- protocol state and ordering consequences;
- cancellation, timeout, retry, partial-progress, and cleanup behavior;
- resource and backpressure limits;
- interoperability and adapter-preservation obligations;
- positive, negative, boundary, malformed, exhaustion, and interaction cases;
  and
- complexity, allocation, copy-count, or latency guarantees only when those
  properties are part of the contract.

Default tests use deterministic virtual resources. Real filesystem, database,
network, and device tests are opt-in, capability-rooted, hermetic where
possible, and identify environmental assumptions. Tests shall not require
public Internet access or an unowned physical device.

## Review and risk sequencing

Phases 2 through 7 establish the safety-critical substrate and require
independent review of type soundness, capability confinement, cancellation,
and resource lifetime. Phases 8 through 13 add protocol and consistency risk;
their reviewers assess malformed input, distributed failure, and transaction
semantics. Phases 14 through 16 add memory, device, concurrency, and performance
risk and require platform-specialist review plus differential reference
implementations.

No phase may mark a sampled execution as proof of race freedom, deadlock
freedom, durability, protocol progress, or performance bounds. Formal or
exhaustive models are required where the contract makes a universal claim.

## Stack and release discipline

Each phase is a cohesive PR based on the exact accepted predecessor. A later
phase may be prepared in a stack only while every predecessor remains
independently reviewable and mergeable. When a prerequisite merges, successors
are rebased onto updated `main` and retargeted without accumulating unrelated
changes.

Every phase passes formatting, warning-free workspace linting, the complete
workspace test suite under the repository memory limit, focused conformance
tests, traceability gates, and diff validation. Platform integration and
performance jobs record their environment and remain separate from deterministic
semantic conformance. A phase is complete only after human review and merge;
the final audit, not the existence of public names, determines release
readiness.
