# Advanced application examples plan

This plan turns the approved data-transfer architecture into two executable,
self-checking Topal examples. It traces to `TOPAL-GOAL-SAFE-001`,
`TOPAL-GOAL-COMPOSE-001`, `TOPAL-GOAL-ZEROCOST-001`,
`TOPAL-REQ-TRANSFER-001`, `TOPAL-REQ-DATA-VIEW-001`, and
`TOPAL-REQ-TRANSPORT-BINDING-001`. The examples exercise existing design; they
do not introduce new language semantics or claim that the current host adapter
can receive production network traffic.

## Scope and review split

The work is delivered as a native GitHub stack so each part can be reviewed and
reverted independently:

1. this design, research basis, acceptance criteria, and implementation plan;
2. an advanced packet-filter policy and executable Topal example;
3. a transport-independent REST controller and executable Topal example; and
4. corpus integration, documentation, traceability, and validation evidence.

The packet and HTTP host boundaries remain explicit. Native adapters eventually
provide packet regions, clocks, endpoint capabilities, HTTP messages, and
completion operations. The examples own portable parsing policy, validation,
routing, authorization decisions, and application behavior in Topal.

## Packet-filter design

`TOPAL-EXAMPLE-PACKET-001` requires a packet filter to demonstrate:

- fail-closed bounded validation before any header projection;
- one owned region with non-copying Ethernet, IP, and transport views;
- IPv4 and IPv6 as distinct inputs to one family-independent policy;
- an immutable compiled rule snapshot selected once for a packet batch;
- an exact-match fast path before an ordered general-rule path;
- explicit accept, drop, and slow-path verdicts with a default drop;
- separate policy decisions from counters, logging, connection tracking, and
  forwarding effects;
- per-receive-queue ownership and batching so the hot path needs no shared
  mutable state; and
- semantic equivalence between portable and offloaded decisions.

This follows current kernel practice without binding Topal to Linux. XDP runs
early in the receive path, AF_XDP can redirect frames into user-space UMEM,
and redirect processing uses bulk queues. nftables demonstrates compiled sets,
maps, and verdict maps rather than repeatedly walking only textual rules. The
Topal example therefore expresses a small decision kernel that an adapter may
specialize to XDP, AF_XDP, nftables, another operating-system facility, or
hardware while preserving its observable verdicts.

The example is not a benchmark. Performance claims require platform-qualified
measurements of throughput, latency distribution, copy count, allocation count,
batch size, queue occupancy, and drop causes as required by `se/data-transfers.md`.

Primary design references:

- [Linux AF_XDP documentation](https://www.kernel.org/doc/html/latest/networking/af_xdp.html)
- [Linux XDP redirect documentation](https://www.kernel.org/doc/html/latest/bpf/redirect.html)
- [Linux XDP receive metadata documentation](https://www.kernel.org/doc/html/latest/networking/xdp-rx-metadata.html)
- [nftables sets and maps](https://wiki.nftables.org/wiki-nftables/index.php/Sets)

## REST-controller design

`TOPAL-EXAMPLE-REST-001` requires a small REST API to demonstrate:

- transport-neutral request and response values at the controller boundary;
- controller functions embedded directly in a controller implementation;
- a thin router that selects a controller operation by method and resource;
- safe and idempotent method behavior consistent with HTTP semantics;
- typed request validation and application errors;
- explicit status, media type, and response representation selection;
- conditional mutation with a version precondition to prevent lost updates;
- RFC 9457-style problem responses without exposing internal diagnostics;
- authority passed to controller functions rather than ambient access; and
- controller tests through direct calls, independent of sockets and HTTP
  serialization.

The controller owns application semantics. A future HTTP adapter owns message
syntax, limits, deadlines, connection lifecycle, protocol negotiation, and
serialization. This permits the same controller functions to be called
directly, by a task endpoint, or through HTTP without embedding network effects
in business logic.

Primary design references:

- [RFC 9110: HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html)
- [RFC 9457: Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html)

## Implementation sequence

1. Add reusable Topal policy modules below `std packet filter` and `std web`
   only where more than one example or test needs the operation.
2. Implement self-checking examples under `examples/data-transfer/`; retain the
   existing simple firewall as an introductory example.
3. Cover positive, negative, boundary, IPv4/IPv6, default-deny, rule-order,
   conditional-update, unsupported-method, and malformed-request behavior in
   Topal assertions.
4. Load both examples through the shared standard-library Topal corpus and the
   linter and language-server source corpora. Do not add Rust golden assertions
   for Topal behavior.
5. Update user documentation, the conformance matrix, and traceability. State
   which native and measured-performance aspects remain deferred.
6. Run the complete workspace tests and static checks under the repository's
   memory-limited test policy. Resource-baseline updates, if any, may add only
   newly introduced test identities and must not alter existing entries.

## Acceptance criteria

- Both examples execute with the interpreter and reject a deliberately false
  Topal assertion.
- The packet filter makes the same decision for equivalent IPv4 and IPv6
  service traffic, rejects truncated input, honors rule priority, and defaults
  to drop.
- The REST example directly exercises embedded controller functions for read,
  create/update, delete, validation failure, missing resource, unsupported
  method, and stale precondition cases.
- Linter and LSP corpora accept both sources.
- Documentation distinguishes executable policy evidence from deferred native
  networking and measured performance evidence.
