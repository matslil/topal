# Bounded static ELF stack-backing extent

This record resolves `TK-DEC-140` as the physical-resource boundary between an
opaque guarded stack slot and later stack initialization/mapping. It is a
systems-profile refinement only and reuses the generic affine physical-frame
allocator; checked source owns one opaque frame extent and never observes a
physical or virtual address.

## Decision

`static image construction reserve stack backing` shall be admitted only while
the exact `StackSlotAuthorizedStaticImageConstruction` from `TK-DEC-139` is
live. It shall accept that owner, `frame-count` one, and `alignment-frames` one,
and shall produce one affine page-sized physical extent nested beneath the
unchanged stack-slot owner.

The allocator/provider shall prove the complete returned frame is allocatable,
page-aligned, and disjoint from the static ELF payload, leaf metadata,
translation hierarchy, boot resources, every earlier reservation, and every
non-allocatable firmware interval. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK` observation, then release the
exact extent before stack-slot revocation. Whole release shall restore the
unchanged slot-authorized construction; partial release, an alternative shape,
or a second live backing extent is invalid.

Reservation does not install a translation leaf, clear or initialize the
frame, transfer it into the construction, derive a stack pointer, or change
the inactive address space. The frame remains ordinary opaque physical backing
until released.

## Exact x86-64 realization

The checked root shall retain the existing hierarchy identity while requesting
one unit-aligned frame at or above the first frame after the exact four-frame
hierarchy. The existing bounded E820 allocator shall validate the complete
candidate interval and all boot reservations. This monotonic floor proves
disjointness from the earlier payload, leaf metadata, and hierarchy extents
without publishing any base.

The one-frame backing identity may occupy a target-private register only while
the extent is live. Whole release shall clear that identity and restore the
retained hierarchy identity before any slot or entry transition. No provider
object change is required because this refinement composes the existing
qualified affine allocation and release operations.

## Affine lifecycle and rollback

Only the slot-authorized owner may nest one stack-backing extent. While the
extent is live, duplicate allocation, stack-slot or entry revocation,
activation, detach, resource return, or recoverable disposition is invalid.
Only exact whole release restores the slot-authorized state. Allocation
failure is fatal because executable and slot authority are already live.

## Architecture pressure test

One opaque, page-sized, suitably aligned physical backing extent is portable
policy. AArch64, RISC-V, IOMMU-backed systems, and capability machines may use
different frame descriptors, colors, zones, ownership tags, or capability
bounds. Providers must retain those mechanics and any numeric address while
proving whole-resource disjointness and affine release.

## Alternatives rejected

Returning a physical address was rejected because source could forge mappings.
Allocating before slot selection was rejected because the resource would lack
its virtual-layout purpose and rollback order. Reusing a payload, metadata, or
hierarchy frame was rejected because their independent lifetimes and roles are
already live. Zeroing or mapping during reservation was rejected because
backing acquisition, initialization, and leaf publication need separate
failure boundaries. Retaining the frame after slot revocation was rejected
because its purpose-derived authority would outlive the slot.

## Deliberate bounds

This increment reserves, witnesses, and releases one opaque stack-backing
frame. It does not zero the frame, install a stack leaf, initialize process
entry contents or TLS, construct a user context, enter user mode, execute
BusyBox, or add a syscall.
