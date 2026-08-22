# Data-transfer implementation conformance matrix

This matrix assigns every approved validation scenario and implementation
phase to evidence. A row is terminal only when each required artifact contains
direct stable-ID evidence or records an approved platform-specific or deferred
disposition. File presence, text search, aggregate test counts, and successful
cross-compilation are not completion evidence by themselves.

The separately versioned public namespaces are `std data`, `std transfer`,
`std store`, `std network`, and `std device`. Their dependency direction and
lack of ambient host authority are normative in
[`TOPAL-TRANSFER-PACKAGE-001` through `005`](../spec/data-transfer-packages.md).

| Phase | Contract/specification | Implementation evidence | Required verification | Status |
| --- | --- | --- | --- | --- |
| 1. Boundaries | `TOPAL-TRANSFER-PACKAGE-001`–`005` | corrected `std` namespace ownership and this matrix | link, traceability, and closure-gate tests | implemented |
| 2. Endpoint foundation | `TOPAL-TRANSFER-ENDPOINT-001`, `SERVICE-001`, `PROTOCOL-001`, `MESSAGE-001` | ordinary `std transfer` source and host substitution boundary | Topal law and cross-tool tests | planned |
| 3. Completion | `TOPAL-TRANSFER-OPERATION-001`, `CANCEL-001`, `BACKPRESSURE-001`, `RETRY-001` | `library/std/transfer/queues.t`; scheduler adapters remain | Topal cancellation, ordering, bounds, and retry laws | partial |
| 4. Regions | `TOPAL-DATA-REGION-001`, `SCATTER-001` | `library/std/data/spans.t`; owned regions remain | Topal bounds, alignment, ownership, and no-copy laws | partial |
| 5. Views | `TOPAL-DATA-VIEW-001`, `VIEW-INVALIDATE-001` | ordinary `std data` validated views | Topal malformed-input and evidence-invalidation laws | planned |
| 6. Adapters | `TOPAL-TRANSFER-SEQUENCE-001` | ordinary `std transfer` framing adapters | Topal chunking, size, and backpressure laws | planned |
| 7. Virtual host | `TOPAL-HOST-ABI-001`, `HOST-REPLAY-001` | Topal virtual protocols over the narrow host boundary | denial, deterministic observation, and effect-free replay tests | planned |
| 8. Native host | `TOPAL-HOST-NATIVE-001` | private native file/socket owners and target support manifest | manifest and native capability tests | implemented |
| 9. IP | `TOPAL-NETWORK-IP-001` | `library/std/network/addresses.t`; packet validators remain | Topal family-boundary, golden, and malformed tests | partial |
| 10. Transports | `TOPAL-NETWORK-TRANSPORT-001` | ordinary transport-independent Topal bindings | Topal partial-progress, half-close, and service-equivalence tests | planned |
| 11. Stores | `TOPAL-STORE-FOUNDATION-001` | `library/std/store/memory.t`; queries and changes remain | Topal shared store laws | partial |
| 12. Transactions | `TOPAL-STORE-TRANSACTION-001` | ordinary Topal guarantee and fault models | Topal consistency and uncertain-outcome laws | planned |
| 13. Files | `TOPAL-STORE-FILE-001` | Topal memory file store plus native injected capability | Topal traversal and rename-identity tests | planned |
| 14. Databases | `TOPAL-STORE-DATABASE-001` | Topal prepared-operation and schema adapters | Topal parameter, row, and transaction laws | planned |
| 15. Devices | `TOPAL-DEVICE-CONTROLLER-001` | Topal virtual controller and explicit DMA contracts | Topal alignment, removal, and ownership laws | planned |
| 16. I2C | `TOPAL-DEVICE-I2C-001` | `library/std/device/i2c.t` plus existing Linux `I2C_RDWR` experiment; capability binding remains | Topal combined read, NACK, address, and limit tests | partial |
| 17. Firewall | `TOPAL-DATA-OFFLOAD-001` | `examples/data-transfer/firewall.t` uses public `std` namespaces | software/offload differential and resource measurements | partial |
| 18. Audit | `TOPAL-TRANSFER-COMPAT-001` | Topal source, tests, examples, documentation, native boundary, and qualification record | closure, compatibility, lint, and complete workspace suites | planned |

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

[`data-transfer-conformance.json`](data-transfer-conformance.json) records each
phase as `implemented`, `platform-specific`, or `deferred` with exact evidence.
The repository test requires phases 1 through 18 exactly once, rejects other
dispositions and empty evidence, and verifies every repository evidence path.
Changing this table alone therefore cannot satisfy closure.
