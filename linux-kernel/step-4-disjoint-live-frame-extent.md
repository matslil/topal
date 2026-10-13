# Step 4 disjoint live-frame-extent increment

## Outcome

The initial allocator profile now admits one source-owned frame while a
prepared user address space owns its exact three-frame backing. The two
affine resources coexist, remain disjoint, and are consumed in a closed order:
source releases the additional extent before the prepared address space can be
bound into the user probe.

This is a systems-feature refinement only. It adds no grammar, pointer or
address conversion, source instruction, or core-language rule. Physical
selection and the non-overlap witness remain private to each target provider.

## Checked source and semantic model

The checked root nests one exact `(frame-count: 1, alignment-frames: 1)`
allocation under the prepared address-space scope. Its success arm must release
the returned extent, publish `TOPAL_KERNEL_FRAME_DISJOINT_OK`, and only then
consume the prepared space into the user-probe lifecycle. Its failure arm must
preserve the allocator context and enter `fatal`.

The architecture-independent authority model retains the three-frame backing
in `AddressSpacePrepared` state while ordinary allocator ownership holds the
one-frame extent. It rejects a third live extent and rejects user-probe
preparation until ordinary extent ownership is empty. Negative checking also
rejects the wrong nested request, release identity, and marker.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/22`. The root obtains the opaque address-space record through generated
bootstrap storage and passes its retained backing base only as a private
provider-call exclusion. Portable source never observes that value.

The E820 selector treats both intervals as half-open, advances a colliding
one-frame candidate to the end of the exact three-page exclusion, recomputes
the complete candidate interval, and then reapplies the existing RAM-run and
one-GiB identity-map bounds. The exclusion path admits only the sealed
one-frame request. Structural inspection proves both interval comparisons, the
advance, overflow branches, and restoration of the callee-saved register on
success and failure.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/22` records 54
provider operations, 54 inspected placements, and a 161-transition semantic
trace. Root-object revision `topal.systems-root-object.x86_64/22` has 994 typed
relocations, including five allocator calls, the private exclusion-state load,
and the new marker and fail-closed edge. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/24` records 27
ordered markers. `TOPAL_KERNEL_FRAME_DISJOINT_OK` appears after both ownership
domains coexist and the source-owned extent is released; the later user-copy,
syscall, backing-reclaim, and release markers prove that the excluded prepared
space remained intact. The evidence binds:

- linked kernel:
  `b256e76f9e62a049181304761d57de359486a2be52b7dc5bc1a7b15092560f74`;
- boot image:
  `83a58f78918a1821e02d4074a59883f05d756d4fad81440f82aefcee063db906`;
- artifact provenance:
  `41e9feeb04a3273d8552d6ea18fc2d3aa869c3b2e9a5e643cdff7b2872e566bb`;
- boot provenance:
  `be184f7c5357c77bf01c0db962931c0563a272513235d482dea1a287a1e8ab98`;
- serial observation:
  `8aad996bb9ba92fec6afe1c60b682ab0131b83e32acdeaad8a730a25d19a353f`;
  and
- complete evidence record:
  `b79fe5ca0a98393ab9b6dbb2f676931c8289531f0d441f1049a226cc2382e21b`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is a high-risk affine-ownership and physical-selection change. Review
covered authority-order conformance, systems-only scope, simultaneous but
separate ownership, half-open interval arithmetic, overflow and bounds checks,
callee-saved register preservation, exact release ordering, target-private
representation, structural instruction inspection, negative tests, two
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

Only the exact prepared three-frame backing plus one transient source-owned
frame is qualified. This increment does not add general allocator concurrency,
dynamic requests, extent splitting, arbitrary live-extent sets, source-visible
physical identity, multiple address spaces, or user mapping edits. The next
dependency-ordered increment can consume a disjoint extent into one sealed
user-address-space mapping edit while preserving opaque ownership.
