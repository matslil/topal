# Resident user-image readiness witness

This record resolves `TK-DEC-046` as a systems-profile qualification
refinement. It makes the already approved bounded resident-image state
physically observable before teardown. It changes no core-language rule and
adds no general image, address, permission, or loader interface.

## Decision

Source emits the exact `TOPAL_KERNEL_USER_IMAGE_READY` console marker after
generation-two immutable-data sealing and before either finalized mapping is
unmapped. At that boundary the address space owns exactly one sealed RO/X
`ExecutableCodeMapping` and one sealed RO/NX `ImmutableDataMapping` backed by
separately allocated, pairwise-disjoint one-frame extents.

```text
finalize code (resident) -> finalize data (both resident)
  -> emit image-ready witness
  -> unmap/release data -> unmap/release code
```

The marker is a qualification witness, not a new ownership transition. Console
output preserves both affine mapping owners and all provider state. The source
checker requires the marker at this exact construction boundary, and the
architecture-independent model rejects it before data finalization or after
either teardown begins. Existing role-specific unmap and release ordering
continues unchanged.

## Bounded authority and failure contract

The witness may be emitted only when both approved mappings are finalized,
resident, distinct, and backed by extents disjoint from one another and the
address-space backing. It grants no read, write, execute, mapping, teardown, or
transfer authority and exposes no virtual or physical address, target
permission encoding, page-table representation, or mapping identity.

A missing, duplicate, renamed, early, or late witness is invalid for this
qualification. Failure to emit the witness follows the existing console-write
failure contract and cannot make partially constructed image state usable.
All construction authority remains subject to fatal-only completion after
permission publication begins.

## Alternatives and cross-architecture pressure

Inferring readiness from later teardown markers would not physically
distinguish simultaneous residency from sequential construction. Retaining the
mappings into user execution would conflate a construction qualification with
loader and execution policy. A new first-class readiness operation would add
unnecessary systems vocabulary when an exact checked console witness can
observe the existing semantic state without changing it.

The boundary has the same meaning on x86-64, AArch64, and RISC-V: one finalized
immutable code mapping and one finalized immutable data mapping coexist before
revocation. Providers retain private choices for placement, page-table format,
permission encoding, instruction synchronization, translation invalidation,
and console transport.

This increment does not retain mappings into user execution, transfer image
ownership, execute loaded application code, define arbitrary segment
collections, relocate an image, parse ELF, or provide demand paging.
