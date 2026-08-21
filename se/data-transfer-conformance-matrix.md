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
| 6. Adapters | `TOPAL-TRANSFER-SEQUENCE-001` | bounded length framing and message queues | every split point, size, and backpressure tests | implemented |
| 7. Virtual host | `TOPAL-HOST-ABI-001`, `HOST-REPLAY-001` | `host::{VirtualHost, ReplayHost}` | denial, deterministic observation, and effect-free replay tests | implemented |
| 8. Native host | `TOPAL-HOST-NATIVE-001` | private native file/socket owners and target support manifest | manifest and native capability tests | implemented |
| 9. IP | `TOPAL-NETWORK-IP-001` | typed family identities, prefixes, IPv4/IPv6 header validators | family-boundary, golden, and malformed tests | implemented |
| 10. Transports | `TOPAL-NETWORK-TRANSPORT-001` | transport-independent binding trait and bounded virtual sequence | partial progress and half-close tests | implemented |
| 11. Stores | `TOPAL-STORE-FOUNDATION-001` | identified memory store, model-specific query trait, bounded changes | identity and subscription-backpressure tests | implemented |
| 12. Transactions | `TOPAL-STORE-TRANSACTION-001` | guarantee comparison and deterministic commit-fault model | strength and uncertain-outcome tests | implemented |
| 13. Files | `TOPAL-STORE-FILE-001` | identity-preserving memory file store and native injected file capability | traversal rejection and rename identity tests | implemented |
| 14. Databases | `TOPAL-STORE-DATABASE-001` | prepared operation and schema-checked row boundary | parameter and row mismatch tests | implemented |
| 15. Devices | `TOPAL-DEVICE-CONTROLLER-001` | bounded virtual controller and explicit DMA requirements | alignment, removal, and ownership tests | implemented |
| 16. I2C | `TOPAL-DEVICE-I2C-001` | deterministic virtual bus and Linux `I2C_RDWR` adapter | combined sensor read, NACK, address, and transfer-limit tests | implemented |
| 17. Firewall | `TOPAL-DATA-OFFLOAD-001` | Ethernet/IPv4 firewall with checksum substitution and copy trace | software/offload differential and zero-payload-copy tests | implemented |
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
