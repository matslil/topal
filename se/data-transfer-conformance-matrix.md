# Data-transfer implementation conformance matrix

This matrix assigns every approved validation scenario and implementation
phase to evidence. A row is terminal only when each required artifact contains
direct stable-ID evidence or records an approved platform-specific or deferred
disposition. File presence, text search, aggregate test counts, and successful
cross-compilation are not completion evidence by themselves.

The separately versioned public packages are `data`, `transfer`, `store`,
`network`, and `device`. Their dependency direction and lack of ambient host
authority are normative in
[`TOPAL-TRANSFER-PACKAGE-001` through `005`](../spec/data-transfer-packages.md).

| Phase | Contract/specification | Implementation evidence | Required verification | Status |
| --- | --- | --- | --- | --- |
| 1. Boundaries | `TOPAL-TRANSFER-PACKAGE-001`–`005` | package ownership and this matrix | link, traceability, and closure-gate tests | implemented |
| 2. Endpoint foundation | `TOPAL-TRANSFER-ENDPOINT-001`, `SERVICE-001`, `PROTOCOL-001`, `MESSAGE-001` | `topal-transfer` reference endpoint and request/reply | capability confinement, state, boundary, and backpressure tests | implemented |
| 3. Completion | `TOPAL-TRANSFER-OPERATION-001`, `CANCEL-001`, `BACKPRESSURE-001`, `RETRY-001` | deterministic `operation::Scheduler` | cancellation race, completion order, bounds, and retry tests | implemented |
| 4. Regions | `TOPAL-DATA-REGION-001`, `SCATTER-001` | `region::{Region, Span, ScatterGather}` | overflow, bounds, alignment, and no-copy tests | implemented |
| 5. Views | `TOPAL-DATA-VIEW-001`, `VIEW-INVALIDATE-001` | `view::{MutableRegion, ValidatedView}` | malformed and exact mutation-dependency tests | implemented |
| 6. Adapters | sequence, message, framing, datagram | `transfer` codecs and queues | chunking, bounds, backpressure | pending |
| 7. Virtual host | host ABI, capability injection, replay | deterministic host backend and tool boundaries | denial and replay tests | pending |
| 8. Native host | native resource and completion rules | target backend scaffolds and manifests | common native conformance kit | pending |
| 9. IP | IPv4/IPv6 identity, parsing, routing | `network` packet views and virtual routes | golden and malformed vectors | pending |
| 10. Transports | UDP, TCP, service bindings | local and virtual transport adapters | equivalent service results | pending |
| 11. Stores | identity, schema, query, changes | reference key-value, relational, graph stores | shared store laws | pending |
| 12. Transactions | isolation, durability, replication | deterministic fault model | consistency litmus tests | pending |
| 13. Files | object, namespace, path resolution | memory and capability-rooted file stores | traversal and path-race tests | pending |
| 14. Databases | prepared operations and typed rows | reference and opt-in relational adapter | schema and transaction scenarios | pending |
| 15. Devices | controller, target, DMA obligations | virtual controller/device | removal and ownership tests | pending |
| 16. I2C | bus and combined transaction protocol | virtual bus and Linux `i2c-dev` adapter | fault and unsafe-retry tests | pending |
| 17. Firewall | bounded-copy mutation and offload | nested-view firewall scenario | differential and resource baselines | pending |
| 18. Audit | compatibility and terminal dispositions | all supported platform adapters | complete conformance and audit suite | pending |

## Architecture scenario closure

| Scenario from `se/data-transfers.md` | Owning phases | Terminal evidence |
| --- | --- | --- |
| Local, IPv4, and IPv6 service binding | 2, 9, 10 | same typed service values plus transport-specific traces |
| Nested Ethernet/IP/transport/application views | 4, 5, 9 | shared-region and evidence-invalidation tests |
| Zero-copy firewall and scatter/gather | 4, 5, 17 | copy/allocation counters and exact output equivalence |
| Local and distributed file store | 11–13 | shared object/namespace laws and distinct guarantees |
| Relational and graph stores | 11, 12, 14 | shared transaction laws and distinct query tests |
| I2C sensor service | 15, 16 | combined transaction, NACK, and retry-safety tests |
| Cancellation/completion race | 2, 3, 7, 8 | exhaustive model and backend conformance tests |
| Software/offload equivalence | 4, 5, 15, 17 | differential results, traces, and measurements |

## Closure gate

A machine-readable row disposition shall eventually be one of `implemented`,
`platform-specific`, or `deferred`, accompanied by exact evidence identifiers.
Until that gate is implemented in phase 1 follow-through, every `pending` row
above blocks release. Changing a status by editing this table alone does not
satisfy the gate.
