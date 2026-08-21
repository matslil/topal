# Data-transfer and external-resource architecture

This architecture defines approved system intent for the extended standard
library. It realizes `TOPAL-GOAL-COMPOSE-001`, `TOPAL-GOAL-SAFE-001`,
`TOPAL-GOAL-EXPLICIT-001`, and `TOPAL-GOAL-ZEROCOST-001` through
`TOPAL-REQ-TRANSFER-001`, `TOPAL-REQ-DATA-VIEW-001`, `TOPAL-REQ-STORE-001`,
and `TOPAL-REQ-TRANSPORT-BINDING-001`.

It does not add behavior to the accepted `design-0` language revision. Exact
types, source syntax, protocol rules, and standard-library interfaces require
subsequent design and specification work in the normal authority order.

## Architectural boundary

External interaction shall be modeled as capability-authorized operations on
protocol-governed endpoints. Files, databases, network connections, local
messages, and devices shall share this foundation without being reduced to one
weak `read` and `write` interface.

The common model consists of:

- an **endpoint**, which carries identity, authority, lifetime, and a protocol;
- an **operation**, with typed input, output, failure, effects, and resource
  obligations;
- a **completion**, which reports the outcome of a submitted operation;
- a **protocol**, which defines legal operation sequences and state changes;
- a **data shape**, which preserves the relevant boundaries and addressing
  model; and
- a **binding**, which realizes a service protocol using a local or external
  transport.

Synchronous execution is a derived submission followed by waiting for its
completion. Implementations may complete an operation immediately, but shall
preserve the same ownership, cancellation, failure, and trace semantics.

## Authority, identity, and naming

Possession of an endpoint shall grant only its represented authority. Resource
names, paths, addresses, and service-discovery results locate candidates; they
do not themselves grant ambient access. Resolution produces capabilities under
an explicit authority and policy context.

Service identity shall remain distinct from transport address. One service may
be reachable through several local endpoints, IPv4 or IPv6 addresses, replicas,
or transports without changing its application protocol. Location transparency
shall not hide failure, latency, security, consistency, or retry properties that
can affect correctness.

## Data shapes and recursive encapsulation

The extended library shall distinguish these data shapes:

- a **message** is an application-visible logical value transferred as one
  unit;
- a **packet** is a protocol data unit below the application boundary;
- a **frame** is a link-, device-, or representation-level transfer unit;
- a **sequence** is ordered data whose producer-operation boundaries need not
  be retained; and
- a **region** supports explicitly addressed access by position or key.

These terms identify semantics, not fixed positions in a universal stack. A
frame may contain another frame, a packet may tunnel another packet, and a
message may contain another message. Semantic containment of values shall be
distinguished from representational encapsulation through an encoding.

Untrusted data shall not acquire a message, packet, frame, or stored-object type
merely by assertion. A parser or decoder produces a validated view and evidence
for the properties it established. Partial, malformed, unsupported, or
resource-exhausting input produces explicit typed failure.

## Shared data and validated views

Several protocol layers shall be able to inspect one owned data region without
copying or repeatedly decoding it. A validated view associates a format or
protocol interpretation with exact spans of the underlying data and records
the evidence on which that interpretation depends.

Immutable overlapping views may coexist. Mutation requires unique authority
over the affected spans. A mutation shall invalidate validation evidence whose
dependencies overlap the modified data while preserving independent evidence.
Implementations may retain decoded fields, indexes, or checksums when their
dependencies remain valid.

The abstraction shall permit scatter/gather transfer, slicing, batching,
buffer-pool ownership, and safe handoff to or from DMA-capable devices. Device
requirements such as alignment, pinning, addressability, lifetime, and
coherency shall remain explicit resource obligations rather than unsafe
assumptions hidden by a byte-sequence interface.

Declarative transformations such as checksum calculation, segmentation,
encryption, or framing may be discharged by hardware, an operating-system
service, or a software adapter. Substitution is valid only when observable
results, failure, ordering, and effects satisfy the same contract.

## Protocols, submission, and completion

A protocol shall identify its legal states, operations, transitions, message
directions, ordering, and termination behavior. Protocol typing alone shall not
be treated as proof of progress: dependency, backpressure, cancellation, and
deadlock obligations remain subject to Topal's concurrency requirements.

Submission produces a stable operation identity. Completion correlates that
identity with success, typed failure, cancellation, or loss of the endpoint.
An endpoint may expose an ordered completion sequence or another declared
completion relation. Completion order shall not be inferred from submission
order unless the protocol guarantees it.

Retries require an explicit safety contract. An adapter shall not retry an
operation that can duplicate an effect unless idempotence, deduplication, or a
transaction identity establishes equivalent behavior. Timeouts do not by
themselves prove that a remote operation did not occur.

## Stores

A file system shall be treated as one specialization of a store, alongside
relational, graph, document, key-value, object, and other database models. The
shared store foundation covers:

- stable object identities and typed schemas;
- names and relations used to resolve objects;
- lookup, query, insertion, replacement, and removal;
- addressed content regions and structured metadata;
- transactions, snapshots, and change messages where supported;
- declared consistency, durability, replication, and conflict behavior; and
- capability-controlled query and update authority.

A file path is a name-resolution query, not object identity. A file is an
identified object with content and metadata; a directory is a relation from
names to object identities; links are additional or indirect relations; and a
mount federates another store into a namespace. Open object capabilities shall
remain valid according to their protocol even when namespace relations change.

Store specializations shall retain their meaningful operations. Relational and
graph databases need not pretend to be files, and files need not adopt one
universal query language. Distribution shall be expressed through consistency,
placement, durability, caching, replication, and failure contracts rather than
through a separate notion of a remote file.

## IPv4 and IPv6 bindings

IPv4 and IPv6 shall be distinct packet formats which may satisfy shared network
capabilities. Their address types, scopes, headers, fragmentation, checksums,
multicast or broadcast behavior, extension mechanisms, and path-MTU contracts
shall remain distinguishable.

An application service may be bound to TCP over IPv4, TCP over IPv6, both, or a
different compatible transport. Dual-stack behavior is explicit composition of
candidate endpoints; an operating-system adapter may optimize it to a combined
facility only when platform behavior preserves the declared semantics.

Resolution and endpoint selection shall make policy inputs such as address
scope, security, reachability, latency, and required protocol capabilities
available without exposing transport addresses as service identities.

## Device and I2C bindings

Devices may expose message, sequence, region, or specialized transaction
protocols. A typed device service should normally hide raw bus operations while
retaining their effects and failure behavior in its implementation contract.

I2C shall be modeled as a shared-bus transaction protocol rather than a byte
stream. Controller authority, target addressing, direction changes, start,
repeated-start and stop boundaries, acknowledgements, arbitration, clocking,
transfer limits, bus recovery, and target lifetime are relevant parts of its
contract. A combined register-address write and data read must be representable
as one indivisible bus transaction when the device requires that behavior.

Register-oriented devices may expose an addressed region only when the region
contract preserves access width, byte order, volatility, read or write side
effects, legal sequencing, and retry safety. Devices whose operations do not
have region semantics shall expose their own message or transaction protocols.

## Performance and observability

The semantic model shall permit, without requiring, zero-copy inspection,
in-place header updates, scatter/gather operations, batching, pipelining,
kernel bypass, and hardware offload. Programs express required guarantees and
may express resource or latency constraints; they shall not depend on a
particular operating-system buffering strategy.

Traces shall retain endpoint, operation, protocol-transition, data-view, and
completion identities without exposing unstable addresses. Observation shall
not force data copies or change scheduling and completion semantics. Sensitive
payloads and capabilities shall not become trace output without explicit
authorization.

## Layering and extensibility

Higher-level services shall be implementable as protocol-preserving adapters:

```text
application service
  -> message or store protocol
  -> serialization and security
  -> sequence, packet, or bus transaction
  -> operating-system or device endpoint
```

Adapters shall declare which ordering, reliability, security, consistency,
idempotence, and performance properties they preserve, strengthen, or cannot
provide. The foundational layer protects resources and validates contracts;
library and application layers remain free to select buffering, caching,
framing, scheduling, indexing, and batching policies.

## Validation scenarios

Subsequent formalization and implementation shall demonstrate at least:

1. one typed service bound independently to local messages, IPv4, and IPv6;
2. nested Ethernet, IP, transport, and application views over one owned region,
   with a header update invalidating only dependent evidence;
3. zero-copy firewall inspection and scatter/gather forwarding with a bounded
   copy count and explicit checksum obligations;
4. a local and distributed file store using the same object and namespace
   contracts while declaring different consistency and durability;
5. relational and graph stores sharing transaction and capability foundations
   without losing their distinct query models;
6. an I2C combined write/read transaction and a typed sensor service built over
   it, including negative acknowledgement and unsafe-retry cases;
7. cancellation racing with completion without duplicated effects or leaked
   resource authority; and
8. software and hardware-offloaded implementations producing equivalent
   semantic results and traces.

Each scenario requires positive, negative, boundary, resource-exhaustion, and
cross-layer interaction cases. Performance claims require measured copy counts,
allocations, queue depth, throughput, and latency distributions on declared
platform configurations; functional tests alone do not establish zero-copy or
real-time behavior.

## Research influences

The architecture adopts capability-protected endpoints from systems such as
[seL4](https://sel4.systems/Info/Docs/seL4-manual-15.0.0.pdf), typed protocol
contracts explored by
[Singularity](https://www.microsoft.com/en-us/research/publication/singularity-rethinking-the-software-stack/),
explicit distributed communication from
[Barrelfish](https://barrelfish.org/publications/barrelfish_sosp09.pdf), and
policy/mechanism separation from the
[Exokernel](https://pdos.csail.mit.edu/6.828/2019/readings/engler95exokernel.pdf).
It takes uniform namespace and service composition from
[Plan 9](https://9p.io/sys/doc/9.html), while retaining semantic shapes beyond
files. Queue-oriented submission and completion are informed by
[Windows I/O completion ports](https://learn.microsoft.com/en-us/windows/win32/fileio/i-o-completion-ports)
and the
[Demikernel](https://www.microsoft.com/en-us/research/publication/the-demikernel-datapath-os-architecture-for-microsecond-scale-datacenter-systems/).
[Zircon's distinct kernel objects](https://fuchsia.dev/fuchsia-src/reference/kernel_objects/objects)
reinforce that common authority and lifetime do not require erasing transfer
semantics.

These influences are design evidence, not normative dependencies. Topal's
formal contracts and conformance tests remain authoritative for Topal behavior.
