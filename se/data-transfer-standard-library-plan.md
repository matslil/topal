# Data-transfer standard-library implementation plan

This plan turns the approved
[data-transfer and external-resource architecture](data-transfers.md) into a
dependency-ordered pull-request series. It realizes `TOPAL-REQ-TRANSFER-001`,
`TOPAL-REQ-DATA-VIEW-001`, `TOPAL-REQ-STORE-001`, and
`TOPAL-REQ-TRANSPORT-BINDING-001` without enlarging the completed flat
fundamental surface of `std` or granting ordinary library code ambient platform
authority.

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

The public surface belongs to separately versioned ordinary Topal namespaces
nested below the stable `std` root. The intended public namespaces are:

```text
std transfer   endpoint, operation, completion, message, sequence, and adapters
std data       owned regions, spans, validated views, packets, frames, and codecs
std store      identity, namespace, transaction, snapshot, change, and guarantees
std network    addresses, resolution, IPv4, IPv6, transport, and service bindings
std device     controller, target, register, DMA, and bus protocols
```

The implementation resides in ordinary `.t` source below `library/std/`.
Portable contracts, adapters, policies, reference models, and codecs are Topal
code. Rust is restricted to irreducible host/native mechanisms and their narrow
runtime bridge.

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

## Native backend architecture

The portable library shall not call a lowest-common-denominator `read` or
`write` shim. Native access is divided into four layers:

1. the ordinary Topal protocol and data-shape contracts;
2. a versioned semantic host-operation ABI using Topal identities, capabilities,
   regions, operations, completions, and typed failures;
3. a memory-safe native backend which maps that ABI to documented operating-
   system objects, calls, completion facilities, and cancellation; and
4. an application/platform broker which obtains user consent, entitlements,
   sandbox extensions, or privileged device authority and injects only the
   resulting capabilities.

The backend implementation may use Rust plus small reviewed C, Objective-C,
Swift, Java, or Kotlin shims where a platform facility has no stable C ABI.
Unsafe code is confined to native handle, buffer, and callback adapters. It
shall never construct a Topal capability from a pathname, integer, pointer, or
foreign handle supplied by untrusted Topal code.

Backends use documented libc, SDK, NDK, Win32, and framework entry points. They
shall not hard-code private system-call numbers or private framework symbols.
On Linux, a maintained binding may invoke a documented syscall whose libc
wrapper is unavailable, but the backend must probe support at runtime and retain
a conforming fallback. Operating-system name or version alone is not evidence
that an operation is permitted by the running kernel, sandbox, security policy,
filesystem, device, or transport provider.

### Native resources and completions

Foreign descriptors, handles, ports, callbacks, and request structures are
backend-private resources associated with stable Topal endpoint and operation
identities. Their numeric values and addresses never enter language semantics,
serialization, or deterministic traces.

Every submitted native operation owns its request storage and borrowed or
pinned buffers until exactly one terminal completion has been consumed.
Immediate native success and pending native success both pass through the same
completion path. Cancellation is a request, not proof of cancellation: the
backend retains resources until it observes the platform's terminal result and
maps the race to the portable completion contract.

Native callbacks and readiness notifications enqueue completion records into
the Topal scheduler. They do not execute arbitrary Topal application code on an
operating-system callback, signal, dispatch, Binder, or completion-port thread.
Readiness is not reported as transfer completion; the adapter retries the
nonblocking operation and records its actual progress or typed failure.

Common failures have portable classifications such as denied authority,
unavailable endpoint, malformed input, exhausted resource, interrupted,
cancelled, timed out, partial progress, and uncertain outcome. Platform error
domains and codes remain structured provenance for diagnostics and specialized
handling; they are not collapsed into misleading universal equivalence.

### Platform mechanisms

| Platform | Resource and transfer mechanisms | Completion integration | Authority and unavailable facilities |
| --- | --- | --- | --- |
| Linux | file descriptors; `openat2`/`openat`, `statx`, `preadv`/`pwritev`, `renameat2`, `fsync`; sockets with `sendmsg`/`recvmsg`; `mmap`, `ioctl`, and documented device ABIs; `I2C_RDWR` through `i2c-dev` | `io_uring` when each required opcode and semantic is runtime-probed; nonblocking descriptors with `epoll` fallback; bounded worker execution only for operations with no safe asynchronous interface | inject pre-opened descriptors or capability-rooted directory handles; apply sandboxing such as namespaces, seccomp, or Landlock where the host selects it; reject rather than emulate guarantees unavailable from the running kernel/filesystem/device |
| Windows | owned `HANDLE` and Winsock `SOCKET` resources; `CreateFileW`, `ReadFile`, `WriteFile`, scatter/gather APIs where supported; Winsock `AcceptEx`/`ConnectEx`/`WSASend`/`WSARecv`; `DeviceIoControl` for admitted device contracts | overlapped operations associated with I/O completion ports; `CancelIoEx` races are completed and reaped through IOCP; registered I/O is an optional measured substitution | capabilities originate from the embedding process and its access token; file sharing, delete, reparse-point, and path-resolution semantics remain explicit; reject a capability-rooted store guarantee if the selected API cannot enforce it without a race |
| macOS | owned POSIX file descriptors and documented `openat`, positioned/vectored I/O, sockets, `mmap`, and device APIs; Dispatch I/O for stream or random-access descriptor operations; Network.framework for TCP, UDP, TLS, path changes, listeners, and modern network policy | `kqueue` for readiness-based descriptor facilities, Dispatch queues for Dispatch I/O, and Network.framework completions; all are marshalled into the Topal scheduler | app-container descriptors, user-selected security-scoped URLs, entitlements, and brokered IOKit/DriverKit services become explicit capabilities; no general device or I2C capability is advertised where public platform APIs do not provide one |
| Android | Bionic/POSIX descriptors, sockets, positioned/vectored I/O, `mmap`, `ioctl`, and `epoll` where allowed; NDK facilities; Java/Kotlin framework bridges for Storage Access Framework, `ContentResolver`, Binder-backed services, and descriptors received as `ParcelFileDescriptor` | nonblocking descriptors with `epoll`; framework/Binder callbacks marshalled into the scheduler; `io_uring` only after runtime kernel, opcode, and seccomp-policy probing | app-private storage descriptors and framework/user-granted content capabilities replace ambient paths under scoped storage; arbitrary device or I2C access is unsupported for ordinary apps and requires an explicitly privileged/system host adapter |
| iOS | app-container and user/document-provider file capabilities through Foundation with Dispatch I/O where appropriate; Network.framework for direct TCP/UDP/TLS services; `URLSession` for HTTP and background transfers whose lifetime can outlive the process | Dispatch and Network.framework completions enter the scheduler; background `URLSession` is a distinct persistent transfer protocol, not disguised as an in-process pending operation | sandbox extensions, document-picker results, security-scoped resources, entitlements, and approved frameworks are the only authority sources; raw devices, arbitrary I2C, unrestricted filesystem paths, and restricted network-extension behavior are explicitly unavailable unless a supported entitled host supplies them |

Darwin similarities do not make macOS and iOS one backend contract: their
sandbox, lifecycle, background execution, entitlement, device, and networking
surfaces differ. Android likewise shares a Linux kernel without promising the
same syscall availability, filesystem namespace, device access, or security
policy as a general Linux host.

### Build, selection, and conformance

The portable packages contain no conditional semantic behavior. A runtime is
built with one or more native backend modules selected by target and SDK, then
constructs a capability inventory from facilities actually supplied by the
embedding application. Optional fast paths are selected per endpoint and
operation after runtime probing; they do not change the public protocol.

Each native backend publishes a machine-readable support manifest containing
backend ABI revision, target, minimum supported platform/SDK, available
operation families, limits, cancellation behavior, and known semantic
restrictions. Unsupported facilities fail capability construction rather than
appearing and failing after unrelated application work.

The same adapter conformance kit runs against the deterministic virtual backend
and every native backend. Native tests use temporary capability roots,
loopback/private networks, disposable databases, virtual devices, and explicit
mobile test-host grants. Cross-compilation proves only build compatibility;
behavioral conformance requires execution on the platform. CI records OS build,
kernel, SDK, filesystem, security/sandbox mode, device/backend versions, and
feature probes with each result.

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

### 8. Native backend ABI and platform scaffolding

- Implement the versioned host-operation ABI and backend-private resource table.
- Add common request ownership, pinned-buffer lifetime, completion marshalling,
  cancellation-race, error-provenance, and runtime feature-probe machinery.
- Build minimal Linux, Windows, macOS, Android, and iOS adapters that accept an
  embedding-supplied capability and perform one region read plus one local or
  loopback message transfer where the platform permits it.
- Generate support manifests and reject unavailable capability construction.
- Establish native CI runners or recorded manual qualification for every
  claimed target; cross-compilation remains a separate gate.

Acceptance: every native adapter passes the common immediate/pending,
partial-progress, cancellation, endpoint-loss, denied-authority, and cleanup
tests without exposing a native descriptor, handle, pointer, callback, or error
number as portable semantics.

### 9. Network identity, resolution, IPv4, and IPv6

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

### 10. UDP, TCP, and transport-independent services

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

### 11. Store identity, schema, namespace, and change foundation

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

### 12. Transactions, consistency, durability, and replication contracts

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

### 13. File-store specialization

- Define files as identified objects with addressed content and structured
  metadata; directories as name-to-object relations; links as direct or
  indirect relations; and paths as resolution queries.
- Specify open-capability behavior across rename, unlink, mount, and namespace
  changes.
- Add atomic rename/update requirements, snapshots and change messages where
  supported, and explicit behavior where they are unavailable.
- Implement an in-memory reference file store followed by one capability-rooted
  adapter for each claimed desktop/server platform and framework-granted mobile
  file capabilities.
- Exercise local and virtual distributed file stores through the same object
  and namespace contracts with different declared guarantees.

Acceptance: traversal cannot escape granted roots, path races do not replace
object identity, and distributed failures remain distinguishable from local
not-found outcomes.

### 14. Database adapter boundary

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

### 15. Device-controller and DMA foundation

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

### 16. I2C vertical slice

- Define controller and target capabilities, typed address modes, speed and
  transfer limits, and shared-bus scheduling.
- Represent start, repeated-start, direction changes, acknowledgement,
  arbitration, clocking, stop, recovery, and combined transaction atomicity.
- Add register-region views only for devices whose access contracts justify
  them.
- Implement a deterministic virtual bus and a Linux `i2c-dev` adapter; add only
  platform adapters backed by documented public device facilities and record
  ordinary Android and iOS applications as unsupported rather than simulating
  raw bus authority.
- Build a typed sensor service over a combined register-address write/read.

Acceptance: negative acknowledgement, arbitration loss, device disappearance,
partial transfer, unsafe retry, and bus recovery are independently testable;
the sensor service does not expose raw bus authority.

### 17. Zero-copy firewall and offload equivalence

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

### 18. Cross-platform adapters and final audit

- Complete the adapter conformance kits for Linux `io_uring`/`epoll`, Windows
  IOCP/overlapped I/O, macOS Dispatch I/O/Network.framework/`kqueue`, Android
  NDK/framework-brokered descriptors, and iOS Dispatch/Network.framework/
  `URLSession` without making any one model normative.
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
