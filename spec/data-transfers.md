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

