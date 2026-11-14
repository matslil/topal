# Bounded static ELF stack-leaf publication

This record resolves `TK-DEC-142` as the mapping boundary between completely
zeroed stack backing and later Linux process-entry initialization. It is a
systems-profile refinement only: checked source owns an affine mapped-stack
construction, while providers own leaf-table representation, addresses,
permission bits, publication ordering, and invalidation mechanics.

## Decision

`static image construction publish stack leaf` shall require the exact live
`ZeroedStackBackingStaticImageConstruction` and nested one-frame backing from
`TK-DEC-141`. It shall accept only that owner, that backing identity, the
existing opaque stack-slot authority, `access` equal to
`writable-non-executable`, and `extent` equal to `complete`. The operation
shall borrow the live extent and produce one affine
`PublishedStackLeafStaticImageConstruction` retaining the unchanged entry and
slot authorities, complete-zero invariant, and exact published-leaf invariant.

Before success, the provider shall revalidate the complete inactive address
space, final image leaves, absent guard, absent selected slot, and completely
zero backing. It shall atomically publish exactly one user-accessible RW/NX
leaf from the selected slot to the selected backing, perform all target-
required visibility operations, and read back the complete selected entry and
all neighboring absence/final-image obligations. The hierarchy shall remain
inactive throughout.

Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK` observation and then invoke
`static image construction revoke stack leaf` with the exact owner, backing,
and policy `bounded-stack-leaf-qualification-rollback`. Revocation shall
remove exactly that leaf, perform all target-required invalidation and
visibility operations, verify the selected slot and guard absent, verify the
backing remains completely zero, and reproduce the zeroed-backing
construction. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK` observation before whole
backing release.

Publication shall not expose a virtual or physical address, leaf ordinal,
page-table representation, permission bit, stack pointer, or invalidation
mechanism. It shall not write process-entry contents, initialize TLS, activate
the hierarchy, construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall require the same inactive hierarchy, 503 final image leaves,
entry authority, ordinal-510 guard, and ordinal-511 slot established by
`TK-DEC-139`. It shall publish leaf ordinal 511 as the exact backing identity
with present, user, writable, and NX attributes and no executable permission.
It shall preserve ordinal 510 absent and preserve all 503 image leaves and
seven other absent entries byte-for-byte.

Publication and revocation shall each use one store fence before complete
readback. Because the hierarchy is inactive and has never exposed this leaf,
revocation requires no active-address-space invalidation. The provider shall
nevertheless verify that the current root remains the qualified bootstrap root
and differs from the static hierarchy before and after both operations.

## Affine lifecycle and rollback

Only the exact zeroed-backing owner may publish. While the stack leaf is live,
backing release, stack-slot or entry revocation, activation, detach, hierarchy
or construction-resource return, duplicate publication, initialization, or
recoverable disposition is invalid. Only exact leaf revocation may restore the
zeroed-backing state admitted by whole backing release. Any owner, backing,
slot, guard, permission, hierarchy, root, ordering, or readback failure is
fatal.

## Architecture pressure test

Publishing a completely initialized backing object through a typed writable,
non-executable mapping is portable policy. AArch64 and RISC-V providers may
use different leaf encodings, break-before-make rules, cache maintenance, and
translation fences. Capability systems may bind a bounded stack capability
instead of a numeric leaf. Those target mechanisms and addresses remain below
the checked-source interface.

## Alternatives rejected

Combining publication with zeroing was rejected because confidentiality
initialization must succeed before user visibility. Leaving the leaf installed
after qualification was rejected because later process-entry construction
needs its own rollback boundary. Activating the hierarchy to test the mapping
was rejected because exact inactive readback suffices and activation would
prematurely grant access. Source-visible leaf flags or addresses were rejected
because they would make target representation forgeable.

## Deliberate bounds

This increment publishes, witnesses, revokes, and verifies one inactive RW/NX
stack leaf. It does not retain the mapping after qualification, initialize
`argc`/`argv`/`envp` or auxiliary-vector contents, initialize TLS, derive a
stack pointer, construct a user context, enter user mode, execute BusyBox, or
add a syscall.
