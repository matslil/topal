# Opaque user-address-space lifecycle contract

This record resolves `TK-DEC-037` as a systems-profile refinement of the
allocator-backed user-probe contract. It separates user translation ownership
from user-thread state without exposing addresses, page-table structure, or a
new core-language facility.

## Decision

The systems profile represents a prepared user address space as an opaque
affine resource. The initial lifecycle is:

```topal
user-space is context user address space prepare (
  backing is user-frames,
  layout is initial-probe
)
user-probe is context user probe prepare (
  process-identity is 1,
  address-space is user-space
)
user-completed is context user context enter user-probe
revoked-space is context user execution complete user-completed
returned-frames is context user address space reclaim revoked-space
context frames release returned-frames
```

Preparation consumes one qualified backing extent, validates the sealed layout,
publishes every provider-private translation required by that layout, and
returns `PreparedUserAddressSpace A`. The address-space identity is fresh and
opaque. Initial probe preparation consumes that space into the suspended user
context. The context owns it through entry, syscall returns, and termination.

Terminal completion revokes every user mapping, performs the provider
translation-completion contract, and returns `RevokedUserAddressSpace A`.
Only `user address space reclaim` may consume that revoked space and return its
exact original indivisible extent. Reclaim before revocation, use after
consumption, cross-identity substitution, mapping or frame reuse while the
space is live, and disposition with any live prepared or revoked space are
invalid.

The initial sealed preparation is fatal-only if a provider invariant fails
after it begins publishing target state. It does not expose a recoverable
partially built address space.

## Abstraction boundary

Source owns allocation, address-space lifetime, association with the user
execution, terminal reclamation, and final frame release. The provider owns
table levels, descriptors, virtual placement, publication, active-root
integration, invalidation, and target return state. A user context owns its
address space while runnable or active, but address-space identity remains
distinct from process, thread, processor, and ABI identity.

The `initial-probe` layout remains a closed qualification template. It is not a
general map operation and does not admit source-selected addresses, arbitrary
permissions, ELF segments, dynamic stack construction, concurrent mapping
edits, address-space switching, or multiple live spaces.

## Cross-architecture contract

The portable observation is one affine transition from opaque physical
backing to a prepared user address space, exclusive transfer into a user
context, complete revocation, and exact backing return. Every provider must
prove executable-versus-writable separation, a non-executable stack,
non-user-accessible translation metadata, an unmapped guard, publication
before entry, complete revocation, and required invalidation before reclaim.

X86-64 may install a leaf table into its active hierarchy. AArch64 may use a
translation-table level and selected granule, and RISC-V may use an Sv leaf
level. Different descriptor formats, activation registers, ASIDs, PCIDs, and
maintenance instructions remain target evidence. A provider whose qualified
geometry cannot represent the sealed layout from the supplied extent must
reject the profile rather than allocate hidden backing.

## Initial x86-64 qualification

`user address space prepare` consumes the already qualified three-frame
extent. It writes the generated probe to the first frame, uses the second for
the RW/NX stack, builds the leaf table in the third, publishes the table into
the active bootstrap hierarchy, and invalidates the sealed code, stack, and
guard locations. User-probe preparation then binds process, thread, processor,
ABI, and kernel-continuation state to that opaque space without accepting a
physical base.

Terminal user completion restores the registered kernel continuation and
returns the address-space state only after removing the hierarchy entry and
invalidating the same locations. Reclaim clears provider retention and returns
the exact extent to source. A pinned-QEMU marker is emitted only after ordinary
frame release.

This systems-only refinement was approved by the standing project instruction
to proceed autonomously when core-language meaning is unchanged.
