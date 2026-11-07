# Bounded resident user-image mappings

This record resolves `TK-DEC-045` as a systems-profile refinement. It permits
the two approved role-specific user mappings to coexist while a bounded image
is assembled. It changes no core-language rule and exposes no address,
permission bit, descriptor, allocator representation, or target instruction.

## Decision

The initial prepared address space admits exactly two separately allocated,
disjoint one-frame mapping extents. Source constructs the sealed `probe-code`
mapping first and retains its `ExecutableCodeMapping`. While that mapping is
resident, source allocates a second frame and constructs the sealed
`probe-data` mapping through `ImmutableDataMapping`.

```text
allocate code -> map/protect/verify/execute code (resident)
allocate data -> map/protect/verify/seal data (both resident)
unmap/release data -> unmap/release code -> prepare user probe
```

The finalized mappings remain affine owners of their exact backing extents.
Finalization makes the current construction extent unavailable to source but
does not revoke the mapping. Data is revoked and released first; code is then
revoked and released. User-context preparation requires both mappings and both
construction extents to be gone.

## Bounded authority and failure contract

The provider must prove that address-space backing, code backing, and data
backing are pairwise disjoint complete intervals. Each mapping has a fresh
opaque identity, a distinct sealed placement, and its previously approved
role authority. Code remains RO/X while data construction occurs; data remains
RO/NX. Neither mapping authorizes access through the other role.

The profile rejects a third construction mapping, allocation before code
finalization, reuse of either backing extent, swapped layouts or roles,
cross-role unmap, code teardown while data remains resident, release while a
mapping owns the extent, user preparation with either mapping resident, and
any recoverable exit with live construction authority.

Revocation and target-required completion remain per mapping. Failure before
publication returns no mapping. Once a permission or executable publication
begins, the sealed operation remains fatal-only as in the existing profile.

## Alternatives and cross-architecture pressure

Reusing one frame sequentially cannot establish that code and data can coexist
or remain disjoint. A general mapping collection, arbitrary virtual ranges, or
source-selected segment flags would prematurely define an image loader and
alias policy. Splitting one multi-frame token would require a broader physical-
extent partition facility not needed by this bounded proof.

The common semantics are the same on x86-64, AArch64, and RISC-V: two opaque
affine mappings with disjoint backing, sealed roles, and explicit per-mapping
revocation. Providers privately select placements, table representation,
permission encodings, publication barriers, instruction synchronization, and
translation invalidation.

This increment does not retain mappings into user execution, execute loaded
application code, define arbitrary layouts or aliases, relocate an image, or
parse ELF. Those remain later systems-profile work.
