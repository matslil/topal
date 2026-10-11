# Step 4 bounded multi-frame extent increment

## Outcome

The checked kernel root now allocates one contiguous three-frame physical
extent after the active-translation mapping probe, publishes
`TOPAL_KERNEL_FRAME_EXTENT_OK`, and releases the complete extent before entering
the local-interrupt critical scope. The request and result use the existing
architecture-neutral affine frame vocabulary; no physical address, pointer,
page-table record, or source instruction is exposed.

This is a systems-feature extension only. The executable profile now admits
the closed static shapes `(1, 1)` and `(3, 1)`. Each result remains one
indivisible affine extent. Dynamic requests, non-unit alignment, splitting,
and multiple simultaneously live extents remain rejected.

## Checked source and semantic model

The source checker admits the three-frame request only in its exact qualified
position. It requires an explicit success marker, whole-extent release, and
continuation with the returned allocator context. The original mapping and
active-translation probes still require exactly one frame. Negative tests
reject other counts, non-unit alignment, use of the three-frame extent in the
one-frame mapping path, wrong release identity, and a live extent at
disposition.

The executable semantic state now retains each live extent's exact request
through mapping and unmapping instead of representing ownership as a boolean.
This proves that one-frame mapping operations cannot silently consume the new
three-frame extent and that unmap returns the original request shape. The
architecture-independent allocator continues to model general aligned,
nonoverlapping extents; the initial executable profile remains deliberately
narrower.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/19`. The root passes the static frame count to
`topal_x86_systems_allocate_physical_frames`, retains it until release, and
clears the provider-private base and transient size state on consumption.

The sealed provider accepts counts one and three only, converts the count to a
byte extent with checked fixed geometry, and succeeds only when the complete
interval fits one E820 RAM run above the private bootstrap floor and below the
qualified 1 GiB identity-map boundary. Structural tests inspect both count
comparisons, the frame-to-byte conversion, full-extent E820 comparison, and
full-extent mapping bound.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/19` records 51
provider operations, 51 inspected placements, and a 153-transition semantic
trace. Root-object revision `topal.systems-root-object.x86_64/19` has 924 typed
relocations; the third allocator call, its failure edge, and the 28-byte marker
account for the increase. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged because the
handoff and entry contracts did not change.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/21` records 25
ordered markers, including `TOPAL_KERNEL_FRAME_EXTENT_OK`. The evidence binds:

- linked kernel:
  `cc804743b1a839810dcea899fb98444585333fa3e34ee6b1917f2766ea8dcd98`;
- boot image:
  `9882e957a3e8d428960e06f0e887feb2fab98f5609beb5994682faecd3760b38`;
- artifact provenance:
  `feaaaaef1aaf149fdd8fa8b073650de06ec463e65322d62234ca1646dab12fb3`;
- boot provenance:
  `d0dc881846e7248cc9d4c2b51f95d448616be0d0454805e5ae9b54c217d75c50`;
- serial observation:
  `08951a9eec1c20fa7b6c683dfd00cff1ebb97e87c78f86cba0e0bd471a7e2108`;
  and
- complete evidence record:
  `c29af196f01e6ae0394254704f30f91a6d009e08c24ed97ca933cd84fe88b2e9`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is a high-risk physical-memory ownership and privileged-provider change.
Review covered authority-order conformance, systems-only scope, whole-extent
affinity, request-shape preservation, complete-interval overflow and E820
bounds, structural provider inspection, negative tests, two physical
executions, and the complete repository test surface. Final validation uses:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
python3 -m unittest linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git diff --check
```

## Remaining boundary

The three frames are still one opaque indivisible extent. This increment does
not add subextent ownership, simultaneous live extents, dynamic allocation
sizes, mapping of a multi-frame extent, source-owned page tables, user mapping,
ELF loading, reclaim, SMP allocation, or general userspace. The next
dependency-ordered increment can define how a source-owned user address-space
builder consumes this qualified extent without weakening its ownership proof.
