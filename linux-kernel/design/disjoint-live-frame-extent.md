# Bounded disjoint live-frame-extent contract

This record resolves `TK-DEC-038` as a systems-profile refinement of affine
physical-frame allocation. It qualifies one additional source-owned extent
while a prepared user address space owns its backing, without exposing either
physical base or adding a core-language facility.

## Decision

After the initial `PreparedUserAddressSpace` consumes its exact three-frame
backing, the same live allocator may service one static `(1, 1)` request. The
returned `PhysicalFrameExtent` must be disjoint from the complete backing of
the prepared address space and from every reservation and non-allocatable
range. Source owns the new extent normally and must release it before consuming
the prepared address space into user-probe state.

The initial source sequence is closed:

```topal
user-space is context user address space prepare (
  backing is user-frames,
  layout is initial-probe
)
  context frames allocate (
    frame-count is 1,
    alignment-frames is 1
  )
    Ok scratch-frame then {
      context frames release scratch-frame
      context console write "TOPAL_KERNEL_FRAME_DISJOINT_OK"
      user-probe is context user probe prepare (
        process-identity is 1,
        address-space is user-space
      )
      ...
    }
    Error problem then context fatal "disjoint frame allocation failed"
```

Allocation failure preserves the allocator but the sealed qualification root
uses a fatal disposition. Success does not grant access to the address-space
backing, split it, or make its identity source-observable. The additional
extent cannot enter user execution in this increment.

## Ownership and ordering

The allocator retains exclusive authority over free physical memory while the
prepared address space owns its exact three-frame extent. A successful nested
allocation creates a second simultaneously live extent with a distinct affine
identity. Non-overlap is an invariant of allocator state, not a comparison
performed by portable source.

Release consumes only the one-frame extent. Probe preparation remains invalid
until that release completes. Wrong request shape, allocation before address-
space preparation in this probe, a third live extent, release of address-space
backing, cross-allocator release, overlap, and disposition with the scratch
extent live are invalid.

## Cross-architecture contract

Every provider must prove that the new extent is disjoint from all provider-
retained extents, irrespective of page size, firmware-memory format, or
allocation algorithm. X86-64 may compare against a retained physical interval;
AArch64 and RISC-V may use allocator metadata, a free-range tree, bitmap, or
another qualified representation. The exclusion witness remains private and
cannot become a portable address or pointer.

The common contract requires only request shape, distinct affine identity,
non-overlap, exact release, and preservation of the prepared address space. It
does not require first-fit behavior or a stable physical selection.

## Initial x86-64 qualification

The generated root asks the provider to exclude the three-frame interval
retained by the prepared user-address-space state. The E820 selector checks the
complete new interval against that exclusion as well as the existing RAM and
qualified identity-map bounds. The one-frame result is released before user
context binding. A marker is emitted only after that release; the existing
user execution then proves the prepared address space remained intact.

This systems-only refinement was approved by the standing project instruction
to proceed autonomously when core-language meaning is unchanged.
