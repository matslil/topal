# Step 4 sealed user-mapping round-trip increment

## Outcome

The initial user-address-space profile now consumes one disjoint source-owned
frame into a closed map/access/unmap transaction. The provider installs that
frame at the sealed `probe-guard` location with user RW/NX rights, confirms a
fixed byte round trip, revokes the mapping, completes target invalidation, and
returns the exact frame to source before user-context binding.

This is a systems-feature refinement only. It adds no grammar, general pointer
or address conversion, source-visible page-table operation, or core-language
rule. Placement, descriptor encoding, and translation completion remain
private to each target provider.

## Checked source and semantic model

The checked root applies
`topal.systems.memory.user-address-space.verify-mapping/1` exactly once to the
prepared address space, its live disjoint one-frame extent, and the sealed
`probe-guard` layout. It must release the returned extent and publish
`TOPAL_KERNEL_USER_MAPPING_OK` before preparing user execution. Negative cases
reject a wrong backing identity, another layout, early release, a missing or
duplicate transaction, and incorrect marker ordering.

The architecture-independent authority model changes the address space from
prepared to verified while temporarily consuming and then returning the same
extent identity. User-probe preparation requires the verified state and no
remaining source-owned frame. This proves revocation and ownership ordering
without modelling an x86 PTE or exposing a reusable mapping API.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/23`. The generated provider verifies the prepared-space state, retained
three-frame backing, one-frame alignment, half-open interval disjointness, and
an empty sealed guard leaf. It publishes an RW/NX user PTE, executes the
required fence and invalidation, stores and reloads byte `0x6d` through the
user mapping, clears the leaf, and repeats completion before success. Its
failure-after-publication path also revokes and invalidates before returning.

The provider keeps the 64-bit backing identity at state offset `0x68` and the
address-space lifecycle byte at the disjoint offset `0x70`. Structural
regression coverage protects that layout after physical debugging exposed an
overlap in the first lowering attempt.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/23` records 55
provider operations, 55 inspected placements, and a 163-transition semantic
trace. Root-object revision `topal.systems-root-object.x86_64/23` has 1,025
typed relocations. Boot-adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/25` records 28
ordered markers. `TOPAL_KERNEL_USER_MAPPING_OK` is emitted only after the
temporary leaf has been revoked and the disjoint frame returned. The later
intentional guard-crossing `copy_to_user` and `copy_from_user` faults prove
that the guard was not left mapped. The evidence binds:

- linked kernel:
  `c553d1401f0307063893b0fa8ea2937f7be8289b0c348f973a179011b539e804`;
- boot image:
  `b68d0706b673eb9e7eaad10caf72002526326b8eb3687e6461841a231bf2414b`;
- artifact provenance:
  `66701acb3daf36220faf48489ac0cbffeba23a4d304496a8be7cca47b40a92a1`;
- boot provenance:
  `1aa654b4f5c014a6d54530b82aa883e17f39130a6aab45c03a22cff0ea664d60`;
- serial observation:
  `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
  and
- complete evidence record:
  `9671c0cdfdc924dd10f4540261d7e034db65fe4e6da48ecd037b4c4109902600`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is high-risk mapping, revocation, and affine-ownership work. Review
covered authority-order conformance, systems-only scope, exact identity
return, interval arithmetic, PTE rights, publication and revocation ordering,
failure cleanup, state-layout separation, target-private representation,
structural instruction inspection, negative tests, two physical executions,
and the complete repository test surface. Final validation uses:

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

Only one exact `probe-guard` transaction is qualified. This increment does not
add a general mapping-edit API, arbitrary virtual placement or permissions,
multiple pages, concurrent edits, aliases, file-backed mappings, demand
paging, copy-on-write, or multiple address spaces. Those require later
reviewed systems contracts while retaining the same architecture-neutral
ownership and completion invariants.
