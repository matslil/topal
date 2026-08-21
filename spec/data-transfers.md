# Data-transfer semantics

## Formal text

### TOPAL-TRANSFER-ENDPOINT-001 — Endpoint identity and authority

Every endpoint shall have a stable semantic identity, a finite lifetime, an
explicit protocol state, and an unforgeable capability authorizing its use.
Possession of a service identity, resource name, path, address, integer, or
serialized representation shall not grant endpoint authority.

### TOPAL-TRANSFER-SERVICE-001 — Service identity

A service identity shall be distinct from every endpoint identity and
transport address through which the service is reachable. Binding a service to
a different compatible endpoint shall preserve the service's request, reply,
failure, and effect contract while retaining binding-specific observations.

### TOPAL-TRANSFER-PROTOCOL-001 — Legal operations and closure

An endpoint protocol shall define its legal states, operations, message
directions, state transitions, ordering, and terminal behavior. An operation
submitted in a state where it is not legal shall produce a typed failure and
shall not perform the operation's effect.

### TOPAL-TRANSFER-MESSAGE-001 — Application message boundary

A message endpoint shall transfer one application-visible logical value as one
unit. Queueing, local binding, or another compatible adapter shall not merge or
split that boundary. A bounded endpoint which cannot accept a message shall
report resource exhaustion without consuming the message.

### TOPAL-TRANSFER-OPERATION-001 — Submission and completion identity

Submission shall produce a stable operation identity. Exactly one terminal
completion shall correlate that identity with success, typed failure,
cancellation, endpoint loss, or explicitly uncertain timeout. Immediate and
queued completion shall have the same semantic relation.

### TOPAL-TRANSFER-CANCEL-001 — Cancellation race

Cancellation shall be a request and shall not by itself establish the terminal
outcome. Completion which races with cancellation shall retain exactly the
single outcome observed by the endpoint and shall neither duplicate an effect
nor release operation resources before that observation.

### TOPAL-TRANSFER-BACKPRESSURE-001 — Bounded submission

Every submission queue shall have an explicit bound. Reaching it shall produce
typed exhaustion without submitting an operation or consuming its input.
Completion ordering shall be the relation declared by the endpoint protocol
and shall not be inferred from submission order.

### TOPAL-TRANSFER-RETRY-001 — Retry admission

Automatic retry of an operation which may have performed an effect shall be
rejected unless idempotence, deduplication, or transaction identity establishes
equivalent behavior. A timeout alone shall not provide that evidence.

### TOPAL-DATA-REGION-001 — Owned regions and checked spans

A region shall own or explicitly retain its element storage. Every span shall
be checked against overflow and the exact region bound before access. Shared
immutable spans may overlap; mutation shall require unique authority over the
affected span.

### TOPAL-DATA-SCATTER-001 — Scatter/gather descriptions

A scatter/gather value shall describe an ordered collection of checked spans
without copying their payload. Alignment, pinning, addressability, and external
lifetime requirements shall be validated before a host substitution uses it.

### TOPAL-DATA-VIEW-001 — Validation evidence

Untrusted representation data shall acquire a message, packet, frame, or stored
object interpretation only through validation. The resulting view shall retain
its exact source span and evidence dependencies. Incomplete, malformed,
unsupported, and exhausted input shall remain distinct outcomes.

### TOPAL-DATA-VIEW-INVALIDATE-001 — Mutation invalidation

Mutation shall invalidate every validation dependency overlapping the changed
span and shall preserve independent evidence. Access through invalidated
evidence shall fail before interpreted data is observed.

### TOPAL-TRANSFER-SEQUENCE-001 — Sequence and framing

A sequence shall preserve byte order but not producer-operation boundaries.
A message-to-sequence codec shall recover exactly the encoded message
boundaries under arbitrary chunking and shall report incomplete, malformed,
oversized, and queue-exhausted input without emitting a partial message.

### TOPAL-HOST-ABI-001 — Semantic host operations

The host boundary shall be versioned and shall exchange semantic capability,
operation, and observation values rather than native descriptors, pointers, or
error numbers. No authority shall exist until the embedding application
injects a capability.

### TOPAL-HOST-REPLAY-001 — Effect-free replay

A debugger replay backend shall return recorded completion observations in
their semantic order without submitting the recorded external operations.
Static source tools shall neither inject capabilities nor submit operations.

### TOPAL-HOST-NATIVE-001 — Native resource confinement

A native backend shall privately own every descriptor, handle, socket, request,
and callback supplied by the embedding host. It shall publish a versioned
support manifest, reject unavailable capability construction, and retain
request resources until exactly one terminal completion is consumed.

### TOPAL-NETWORK-IP-001 — Distinct Internet Protocol families

IPv4 and IPv6 addresses, prefixes, scopes, and validated packet headers shall
retain their distinct family semantics. A shared network capability shall not
erase IPv4 checksum/fragmentation or IPv6 extension/scope distinctions, and a
text address shall not substitute for typed service or endpoint identity.

### TOPAL-NETWORK-TRANSPORT-001 — Transport bindings

UDP bindings shall preserve datagram boundaries and their delivery limits. TCP
bindings shall expose ordered partial transfer, backpressure, half-close, and
reset. Binding the same typed service to local messages, IPv4, or IPv6 shall
preserve service values while retaining transport-specific observations.

### TOPAL-STORE-FOUNDATION-001 — Store identity and change

A store shall keep object identity distinct from names and query expressions.
Lookup, insertion, replacement, removal, schema-specific query, and change
subscription shall require explicit capabilities. Change delivery shall be
ordered according to its declared relation and bounded by explicit
backpressure.

### TOPAL-STORE-TRANSACTION-001 — Guarantees and uncertain commit

Transactions shall declare their isolation, consistency, durability, placement,
and replication requirements. An adapter shall conservatively reject a
requirement it cannot demonstrate. A lost completion after possible commit
shall report uncertain outcome rather than abort or success, and retry shall
require transaction or deduplication identity.

### TOPAL-STORE-FILE-001 — File store specialization

A file shall be an identified object with addressed content and metadata; a
directory shall relate names to identities; and a path shall be a resolution
query. Namespace rename or unlink shall not replace an already-open object
identity, and resolution shall not escape the granted capability root.

### TOPAL-STORE-DATABASE-001 — Database adapters

Database adapters shall use prepared operations, typed parameters, explicit
row schemas, cursors, and transaction identities. They shall preserve vendor
failure provenance and uncertain commit rather than construct queries through
string interpolation or coerce mismatched rows.

### TOPAL-DEVICE-CONTROLLER-001 — Controller and DMA ownership

Device controllers shall expose typed targets, commands, status, events, and
queues. DMA submission shall validate alignment, pinning, addressability,
coherency, size, and device lifetime and shall transfer unique buffer ownership
until completion, reset, or removal returns a terminal outcome.

### TOPAL-DEVICE-I2C-001 — Combined bus transactions

I2C controller and target capabilities shall preserve address mode, transfer
limits, starts, repeated starts, direction, acknowledgements, arbitration,
stop, and retry safety. A required register-address write/read shall be one
indivisible combined transaction, not two independently retryable operations.

### TOPAL-DATA-OFFLOAD-001 — Bounded-copy substitution

Nested frame, packet, transport, and application views may inspect one owned
region and forward it through scatter/gather. Software and admitted operating-
system or hardware checksum, segmentation, encryption, and framing
substitutions shall produce equivalent values, failures, ordering, effects,
and semantic traces. Copy and allocation claims shall be instrumented.

### TOPAL-TRANSFER-COMPAT-001 — Version negotiation

Public package, encoded protocol, and host-operation ABI revisions shall be
negotiated before use. An implementation may satisfy an older compatible minor
revision within the same major revision; it shall reject a different major or
an implementation older than the required minor revision.
