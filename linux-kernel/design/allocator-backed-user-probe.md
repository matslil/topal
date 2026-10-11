# Allocator-backed user-probe ownership contract

This record resolves `TK-DEC-036` as a systems-profile refinement of the
initial affine user-execution contract. It replaces fixed provider-owned user
pages with one allocator-owned opaque backing extent. It adds no grammar,
numeric address, pointer conversion, source instruction, or core-language
rule.

## Decision

The initial user-probe preparation consumes one live three-frame
`PhysicalFrameExtent` together with the process identity. Success transfers
the whole extent into the returned `SuspendedUserContext`; source cannot access
or release it while the user execution owns it. The target provider assigns
the frames these sealed roles:

1. one read-only executable user page;
2. one writable, non-executable user stack page; and
3. one user leaf-translation page which is not user-accessible.

The source spelling remains ordinary Topal construction syntax:

```topal
user-probe is context user probe prepare (
  process-identity is 1,
  backing is user-frames
)
user-completed is context user context enter user-probe
returned-frames is context user execution complete user-completed
context frames release returned-frames
```

The later [opaque user-address-space lifecycle](opaque-user-address-space.md)
separates preparation and reclaim from user-thread state while retaining this
exact ownership boundary.

Terminal completion consumes the exact completed execution, revokes every
user mapping derived from the extent, performs the provider completion and
translation-invalidation contract, and returns the original indivisible
three-frame extent. Only then may source release it. Preparation with a wrong
shape, release while user execution owns the extent, completion before
terminal restoration, a different returned extent, or disposition with live
backing is invalid.

## Abstraction boundary

Source owns allocation policy, process identity, the enter/complete lifecycle,
and final release. The provider owns target page-table layout, user virtual
placement, publication ordering, privilege-transition state, invalidation,
and generated probe bytes. This keeps the allocation visible to checked
source without exposing page-table records or requiring source to split an
extent merely to assign target-private roles.

The returned extent remains indivisible. This refinement therefore does not
admit a general subextent operation, multiple simultaneously source-live
extents, dynamic allocation, or source-selected user virtual addresses.

## Cross-architecture contract

The common contract is ownership transfer from one qualified backing extent
into a validated user address space and exact ownership return after mappings
are unreachable. X86-64 may use one base-page leaf table plus code and stack
frames. AArch64 may use an appropriate translation-table level and selected
granule; RISC-V may use the appropriate Sv leaf-table level. A target whose
qualified geometry cannot represent the three roles with three base frames
must reject this profile rather than silently allocate hidden frames.

Every provider must prove executable-versus-writable separation, a
non-executable stack, non-user-accessible translation metadata, an unmapped
guard, complete revocation, required translation invalidation, and exact
extent return. No architecture exposes its PTE format or return registers to
portable source.

## Initial x86-64 qualification

The selected extent is already constrained to one E820 RAM run within the
qualified identity mapping. Preparation writes the probe through the
supervisor identity mapping, constructs the user leaf table in the third
frame, publishes code and stack leaves into the active hierarchy, and
invalidates the fixed user virtual locations before entry. The previous
provider-owned `0x00600000` and `0x00601000` physical backing is removed; those
numbers remain only sealed user virtual placements.

After `exit_group(0)` restores the exact kernel continuation, completion
removes the user hierarchy entry, invalidates code, stack, and guard virtual
locations, clears retained backing identity, and returns the original extent
for source release. A pinned-QEMU marker is emitted only after that release.

This systems-only refinement was approved in the project discussion before
implementation.
