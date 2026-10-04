# Linux 7.2.9 source-inventory coverage

This report is the reviewed interpretation of the generated x86-64 inventory.
Counts are coverage anchors, not implementation or semantic-conformance claims.

| Set | Count | What the count establishes | Important remainder |
| --- | ---: | --- | --- |
| native syscall-table rows | 385 | every `common` or `64` row is indexed; x32 and i386 are excluded | arguments, subordinate protocols, state, failure, and configuration |
| installed UAPI headers | 1,025 | every file emitted by x86-64 `headers_install` is hashed | macro expansion, layouts, validity rules, and runtime meaning |
| documented ABI entries | 6,289 | every `What:` record under `Documentation/ABI` is indexed with stability | prose contract review and configured/runtime presence |
| direct `_IO*` definitions | 2,333 | direct ioctl macro definitions in installed headers are located | aliases, computed/nonstandard numbers, owners, structures, and lifecycle |
| native vDSO exports | 15 | version-script exports and preprocessor conditions are indexed | implementation availability, ELF mapping, and function semantics |
| Device Tree bindings | 6,276 | all YAML and legacy text binding files are hashed | schema interpretation, cross-schema constraints, and driver behavior |

The exact aggregate hashes are in [summary.json](7.2.9/x86_64/summary.json).
The archive identity comes from `LK-BASELINE-001`; generated records store
source-relative locations and do not copy Linux implementation bodies.

## Reconciliation model

No single upstream representation enumerates the Linux userspace ABI. Later
coverage work joins these source sets with runtime probes and subsystem
ledgers:

- syscall rows are reconciled with installed syscall-number headers, syscall
  implementations, libc behavior, seccomp/audit names, and differential tests;
- installed headers are classified into constants, data layouts, flag spaces,
  commands, protocol messages, and feature-discovery mechanisms;
- ioctl entries are grouped by owning descriptor type and protocol rather than
  treating the encoded number as the contract;
- `Documentation/ABI` records are reconciled with pseudo-filesystem producers,
  configuration, permissions, namespaces, polling, and mutations;
- Netlink, sockets, BPF, perf, `io_uring`, KVM, and other multiplexed APIs get
  command/attribute/state-machine ledgers; and
- a pinned Linux reference VM records configuration-dependent presence and
  absence for the selected machine.

An inventory item stays in scope until it has an explicit disposition from
`LK-SCOPE-004`. A file digest change during a baseline update invalidates its
prior review even when an extractor reports the same names.

## Known extractor limits

The extractor is intentionally lexical where semantic preprocessing would
silently choose a configuration. It does not claim to evaluate C, Kconfig,
preprocessor alternatives, YAML schemas, or ABI prose. It preserves enough
identity and provenance to force a reviewed disposition. The checked-in
results can be reproduced with the procedure in [the inventory README](README.md).

