# Step 4 opaque user-address-space lifecycle increment

## Outcome

User translation ownership is now a distinct opaque affine resource instead of
being attached directly to the initial user thread. The checked root prepares
one `initial-probe` address space from its three-frame extent, transfers that
space into user-probe state, receives a revoked space after terminal execution,
reclaims the exact extent, and releases it.

This is a systems-feature refinement only. It adds no grammar, numeric address,
pointer conversion, source instruction, or core-language rule. Process,
thread, processor, ABI, address-space, and physical-backing identities remain
separate throughout the lifecycle.

## Checked source and semantic model

The source checker admits only the exact closed sequence: three-frame
allocation, `initial-probe` address-space preparation, probe binding to that
space, user entry and terminal completion, revoked-space reclaim, exact extent
release, and final success marker. Negative tests reject wrong backing, wrong
layout, wrong prepared-space identity, missing completion binding, wrong
revoked-space reclaim, and wrong final release.

The architecture-independent model introduces affine
`PreparedUserAddressSpace` and `RevokedUserAddressSpace` resources with a fresh
identity and exact backing request. Preparation consumes frame ownership;
probe construction consumes the prepared space; completion returns a revoked
space rather than frames; and only reclaim returns the original
`PhysicalFrameRequest(3, 1)`. The semantic trace records both new transitions.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/21`. Address-space preparation writes the generated probe, creates the RX
code and RW/NX stack leaves in the allocator-owned leaf-table frame, publishes
the table, and invalidates the sealed virtual locations. A separate user-probe
operation validates that prepared state before configuring syscall-entry state
and binding the user context.

Terminal completion removes the hierarchy entry, orders the update, and
invalidates code, stack, and guard locations while retaining the opaque backing
identity in revoked state. A distinct reclaim provider validates that state,
clears retained ownership, and returns the exact base to the checked root for
ordinary affine release.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/21` records 54
provider operations, 54 inspected placements, and a 158-transition semantic
trace. Root-object revision `topal.systems-root-object.x86_64/21` has 961 typed
relocations, including the separate preparation and reclaim calls and their
fail-closed edges. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/23` records the
same 26 ordered compatibility markers. The final
`TOPAL_KERNEL_USER_BACKING_OK` marker remains reachable only after terminal
mapping revocation, revoked-space reclaim, exact extent return, and source
release. The artifact trace separately binds the prepared and reclaimed
address-space transitions. The evidence binds:

- linked kernel:
  `8682b320c9218a6064643ced1b712882d9e85188fdcb2a5b63ecf359554f8065`;
- boot image:
  `4841d3a85683742bb2ef3ccaca970b1bc6afefcb860f7bab562fca04fe7c1b99`;
- artifact provenance:
  `9c14f90a4c22d6dc8af5f0a52c79d0fba2fc076f6a60198f81c0e3a821994ba1`;
- boot provenance:
  `0f96181fbb15e86d967191197da130ee1d5db0fcbbec9510766fcf5f6e691d81`;
- serial observation:
  `fdb1c01e032b40867664b8edec4f24612676e6539ab3d5167e2cf09e1a807525`;
  and
- complete evidence record:
  `5b82866e1bad34905229914813fd730855719b53c1dff76909cc50bc6096630a`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is a high-risk affine ownership, page-table, and privilege-boundary
change. Review covered authority-order conformance, systems-only scope, fresh
address-space identity, exact state and backing transfer, fail-closed operation
ordering, target-private representation, mapping publication and revocation,
translation invalidation, structural provider inspection, negative tests, two
physical executions, and the complete repository test surface. Final
validation uses:

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

The `initial-probe` layout is still sealed and only one address space may be
live. This increment does not add general user mapping edits, ELF loading,
source-selected addresses or permissions, dynamic stack construction, multiple
processes, address-space switching, ASIDs/PCIDs, concurrent reclaim, SMP
shootdown, demand paging, or general userspace. The next dependency-ordered
increment can add an affine user-address-space mapping edit for one sealed
segment while preserving this lifecycle.
