# Data transfers

Topal treats message passing, stored data, network communication, and device
access as related operations without pretending that they have one identical
shape. Applications work with messages and structural data views. Protocol
implementations may divide those messages into packets, while link and device
adapters may divide packets into frames. Each level can contain another value
of the same kind when tunnelling or encapsulation requires it.

The extended standard library is ordinary Topal source beneath the stable
`std` root:

- `std data` describes spans, views, regions, and encoding-independent shapes;
- `std transfer` describes bounded queues, operations, completion, and retry;
- `std store` describes identity-based storage independently of paths or query
  languages;
- `std network` describes service candidates and protocol addresses; and
- `std device` describes controller-level transfers such as I²C transactions.

These scopes are separate so an application imports only the vocabulary it
uses. A file system is one kind of store beside relational, graph, object, and
other databases. Local files, remote file services, and replicated stores can
therefore share storage laws without erasing their different consistency and
durability guarantees.

## Source-first interfaces

Portable algorithms, validation, policies, and adapters belong in `.t` files
under `library/std`. Rust is reserved for the host boundary that cannot be
expressed portably: operating-system calls, completion facilities, protected
memory mappings, and device-controller access. Linux adapters use the
appropriate syscalls; Windows, macOS, Android, and iOS adapters use their native
counterparts. Those adapters substitute capabilities beneath the same Topal
contracts rather than becoming a second public standard library.

The initial executable slice provides span validation and overlap tests,
bounded queue policy, retry evidence, an identity-oriented reference store,
IPv4/IPv6 prefix validation, service candidates, and I²C transaction
descriptions. It is intentionally small while the language gains richer named
data constructors and host-capability bindings.

## Fast paths

A data region owns bytes; spans identify bounded portions; validated views add
protocol-specific evidence without copying the region. A firewall can inspect
an Ethernet header, an IP header, and a transport header through separate
views, then forward the original region. Scatter/gather operations preserve
the same ownership model when output consists of several spans.

Native adapters may replace a portable operation with zero-copy or hardware
offload only when observable output, ordering, cancellation, and failure
semantics remain equivalent. The source-level
[`firewall.t`](../examples/data-transfer/firewall.t) example demonstrates the
portable policy side of this boundary.

The more advanced [`packet-filter.t`](../examples/data-transfer/packet-filter.t)
keeps that boundary while showing a modern datapath organization. Structural
validation fails closed; an immutable policy snapshot applies a denied-source
rule before accepting rules; known flows take an exact-match fast path;
fragments enter an explicit bounded slow path; unmatched traffic is dropped;
and verdicts can be processed as a batch without shared per-packet mutation.
IPv4 and IPv6 remain distinct validated families but feed the same service
policy. A native adapter may realize this organization with an early kernel
hook, user-space packet rings, an operating-system filter, or hardware offload.
The example proves decisions, not zero-copy or throughput: those claims still
require platform measurements and software/offload differential testing.

## Web controllers

[`rest-controller.t`](../examples/data-transfer/rest-controller.t) shows a
small REST API with controller functions embedded directly in a Topal
`Interface` implementation. The controller receives semantic request values
and returns status, media type, representation, and version. The HTTP adapter
owns parsing limits, protocol negotiation, deadlines, connection management,
and serialization, so the controller can be tested through ordinary function
calls without a socket.

The example distinguishes safe and idempotent methods, rejects unsupported
methods before controller dispatch, validates replacement input, uses an exact
representation-version precondition for mutation, and returns public
RFC 9457-style problem values without leaking internal diagnostics. Endpoint
authority and storage effects remain inputs to a future host-backed controller;
the example deliberately introduces no ambient network or database authority.

Address families and transports remain orthogonal. One service can expose IPv4
and IPv6 candidates without duplicating the service definition. I²C uses the
same operation, bounded-transfer, and completion ideas, but does not acquire
network-only concepts such as IP prefixes or routing.
