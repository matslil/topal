# Step 4 affine user-mapping lifecycle increment

## Outcome

The sealed `probe-guard` transaction is now split into distinct affine map,
store, load, and unmap operations. Source holds a persistent opaque mapping
between those operations, advances it through checked refinements, and receives
the exact original frame only after explicit revocation and target completion.

This is a systems-feature refinement only. It adds no grammar, general pointer
or address conversion, source-visible page-table operation, instruction name,
or core-language rule. Layout, descriptor encoding, and translation completion
remain private to each target provider.

## Checked source and semantic model

The checked root applies these operations in order:

1. `topal.systems.memory.user-address-space.map/1` consumes the prepared
   address space's disjoint one-frame extent and returns a live mapping;
2. `topal.systems.memory.user-mapping.store-byte/1` consumes that mapping and
   returns its stored refinement;
3. `topal.systems.memory.user-mapping.load-byte/1` confirms the qualification
   byte and returns its verified refinement; and
4. `topal.systems.memory.user-address-space.unmap/1` consumes the verified
   mapping and returns the exact frame to source.

Only then may the root release the returned extent and publish
`TOPAL_KERNEL_USER_MAPPING_OK`. Negative cases reject the wrong stored or
verified mapping identity, unmapping before verification, early release,
missing or duplicate transitions, and incorrect marker ordering.

The architecture-independent authority model represents `UserMapping` as an
affine value with `Live`, `Stored`, and `Verified` stages. The address space
stays prepared and its original three-frame backing stays retained throughout.
No source-owned access survives unmap, and user-probe preparation requires the
mapping lifecycle to be complete.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/24`. Four generated provider functions implement the checked lifecycle. Map
verifies the prepared-space state, retained backing, one-frame alignment,
half-open interval disjointness, and empty sealed guard leaf before publishing
an RW/NX user PTE and completing invalidation. Store writes byte `0x6d`; load
reads and compares it. Unmap clears the leaf, completes invalidation, and
returns the exact retained frame identity.

Provider-private state keeps the address-space backing at offset `0x68`, its
lifecycle byte at `0x70`, and the live mapping frame at `0x78`. Structural
regression coverage verifies those disjoint fields and the publication,
access, revocation, completion, and return sequence. The prior one-shot
provider symbol remains an artifact-compatibility placement but is no longer
called by the checked root.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/24` records 58
planned provider operations, 59 inspected placements including the
compatibility placement, and a 166-transition semantic trace. Root-object
revision `topal.systems-root-object.x86_64/24` has 1,034 typed relocations.
Boot-adapter revision `topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is
unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/26` records 28
ordered markers. `TOPAL_KERNEL_USER_MAPPING_OK` is emitted only after the
separate unmap returned the exact frame. The later intentional guard-crossing
`copy_to_user` and `copy_from_user` faults prove that no stale guard mapping
survived. The evidence binds:

- linked kernel:
  `8a951cc34e61559d6ae42a771bbfc37af31b2ac17a264d39dc42a9cbb8a2fe14`;
- boot image:
  `8d3b841df38345905bcb516f4a02d68095d35c19f43724a312495b6b6a247a18`;
- artifact provenance:
  `0e385e62d61d9e8fde85b954a358a7677c385b0ed79a24fd777a9c00755f232b`;
- boot provenance:
  `a837eff6dc8fed959fb1a9219dbfde3511b7bb773a080d20fee3f7e2dd6229c2`;
- serial observation:
  `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
  and
- complete evidence record:
  `545907d0972340fbc6982f8d5493b2f2f1cb9816ee21b5252ea688e68ccddf94`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is high-risk mapping, revocation, and affine-ownership work. Review
covered authority-order conformance, systems-only scope, exact identity return,
refinement consumption, interval arithmetic, PTE rights, publication and
revocation ordering, failure cleanup, state-layout separation, target-private
representation, structural instruction inspection, negative tests, two
physical executions, and the complete repository test surface. Final
validation uses:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
scripts/run_bounded.py -- python3 -m unittest \
  linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git -c core.whitespace=trailing-space,space-before-tab diff --check
```

## Remaining boundary

The mapping is still restricted to one exact `probe-guard` layout, one frame,
fixed RW/NX rights, and a fixed byte qualification. This increment does not add
arbitrary placement or permissions, multiple mappings, aliases, executable
mappings, replacement, partial unmap, concurrent edits, file-backed mappings,
demand paging, copy-on-write, or an ELF image builder. Those require later
reviewed systems contracts while retaining the affine ownership and completion
invariants proved here.
