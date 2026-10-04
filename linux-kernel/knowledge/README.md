# Linux design knowledge library

This library records the Linux 7.2.9 design that constrains or informs the
Topal kernel. It separates externally required behavior from Linux-internal
mechanisms so that the Topal design can preserve the former without copying the
latter unnecessarily.

## Record form

Every substantive record has a stable `LK-` identity and contains:

- **subject:** one interface, subsystem, mechanism, or relationship;
- **baseline:** upstream version and applicable architecture/configuration;
- **contract:** observable behavior or internal design purpose;
- **sources:** primary documentation and exact source locations;
- **Topal relevance:** required semantic facility or mapping question;
- **status:** `mapped` when boundaries and source families are known,
  `source-indexed` when a reproducible extractor covers the stated source
  boundary, `reviewed` when the described contract has human-readable semantic
  review, or later `implemented`, `qualified`, or `blocked`; and
- **relationships:** parent, dependency, consumer, test, and replacement IDs.

Generated inventories may use a machine-readable companion format later, but a
generated entry never becomes `reviewed` merely because extraction succeeded.
Conversely, a handwritten summary never establishes exhaustive interface
coverage without a generated reconciliation.

## Authority distinctions

The library uses four source classes:

1. Linux UAPI headers, syscall tables, ABI documents, and observed behavior for
   the external compatibility contract;
2. architecture and platform specifications for hardware meaning;
3. Linux internal documentation/source for research into one implementation;
4. QEMU and firmware specifications for the qualified virtual board.

An internal Linux function or structure is not automatically an external
compatibility requirement. An exported constant or structure is not complete
without its runtime validity, lifetime, ordering, failure, privilege, and
version behavior.

## Baseline update procedure

1. Pin and verify the new stable archive.
2. Reproduce both generated inventories from clean source trees.
3. Diff syscall tables, exported UAPI, documented ABI, vDSO, configuration,
   device schemas, and runtime probes.
4. Classify every addition, removal, layout change, and semantic change.
5. Update reviewed mappings and tests.
6. Run the prior and new differential suites.
7. Change `LK-BASELINE-001` only in the pull request that contains the complete
   disposition report.

## Current contents

- [Catalog and stable identities](catalog.md)
- [Linux userspace interface map](common/linux-uapi.md)
- [Kernel design map](common/kernel-design.md)
- [Driver-framework map](common/driver-frameworks.md)
- [Device Tree map](common/device-tree.md)
- [x86-64 ABI and entry map](x86_64/abi-and-entry.md)
- [x86-64 platform and board map](x86_64/platform-and-board.md)
- [Generated inventory coverage](../inventory/coverage.md)
