# Sealed user mapping round-trip contract

This record resolves `TK-DEC-039` as a bounded systems-profile qualification.
It uses the disjoint source-owned frame to prove one user-address-space
map/access/unmap transaction without exposing a virtual or physical address and
without changing the core language.

## Decision

One `PreparedUserAddressSpace` may borrow its exact disjoint live one-frame
extent for the sealed `probe-guard` mapping round trip. The operation consumes
the extent during the transaction, installs one provider-selected RW/NX user
mapping, writes and reloads a fixed qualification byte through that mapping,
revokes the mapping, completes target-required invalidation, and returns the
exact original extent. The prepared address space remains affine and cannot be
bound into user execution until the returned extent is released.

```topal
returned-scratch is context user address space verify mapping (
  address-space is user-space,
  backing is scratch-frame,
  layout is probe-guard
)
context frames release returned-scratch
context console write "TOPAL_KERNEL_USER_MAPPING_OK"
```

Success is observable only after the store/load comparison, revocation, and
translation completion all succeed. Failure preserves no mapping authority and
the qualification root enters `fatal`. Source cannot access the temporary
mapping, select its placement, alter its rights, retain an alias, or recover a
partially completed transaction.

## Ownership and ordering

Before the operation, the address-space owner retains its original three-frame
backing and source owns the additional one-frame extent. During the operation,
the provider temporarily owns the additional extent and the derived mapping.
Success returns the same extent identity to source only after mapping access is
impossible. The prepared address-space identity and backing are unchanged.

Wrong address-space or extent identity, another request shape, another layout,
overlap, use or release during the transaction, failed byte confirmation,
incomplete invalidation, duplicate verification, probe binding before release,
and disposition with the extent live fail closed. The initial profile admits
the round trip exactly once.

## Cross-architecture contract

The common observation is an atomic semantic transaction: a disjoint extent is
temporarily mapped with user RW/NX authority, a byte round trip succeeds, all
derived access is revoked, target translation completion occurs, and the exact
extent returns. X86-64 may use a leaf PTE plus `INVLPG`; AArch64 may use a stage-1
descriptor with the required DSB/TLBI/ISB sequence; RISC-V may use a leaf PTE
and the required `SFENCE.VMA`. Page-table formats, addresses, ASIDs/PCIDs, and
instructions remain provider-private.

This contract does not choose a reusable general mapping API. It qualifies the
ownership and completion invariants needed before later work separates begin,
map, access, unmap, and commit into general affine operations.

## Initial x86-64 qualification

The provider uses the previously unmapped sealed `probe-guard` leaf, verifies
that the extra frame is outside the prepared backing, publishes an RW/NX user
leaf, invalidates the sealed location, performs the byte store/load, clears the
leaf, and invalidates again before success. The later intentional boundary-
crossing user-copy faults prove that the guard location was not left mapped.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.
