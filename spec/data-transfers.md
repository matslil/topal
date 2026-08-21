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
