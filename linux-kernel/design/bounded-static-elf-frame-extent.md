# Bounded static ELF frame extent

This record resolves `TK-DEC-127` as the first executable-qualified physical
resource for the static-ELF page plan. It widens only the systems profile's
closed affine allocation envelope; it does not change core-language meaning or
grant mapping authority.

## Decision

The initial x86-64 qualification profile shall admit the exact static request
`(frame-count: 503, alignment-frames: 1)` in addition to the existing one-,
two-, and three-frame shapes. Success returns one indivisible
`PhysicalFrameExtent`. Source may neither split nor address it and shall return
the complete extent to the creating allocator before continuing.

The checked root shall allocate this extent only after returning the earlier
three-frame probe, emit exactly one
`TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK` observation while the extent is live,
then release it whole. The x86-64 provider shall use checked frame-to-byte
conversion, select one page-aligned interval completely inside one normalized
E820 RAM run, exclude the boot resource and retained reservations, remain
below the qualified first-GiB bootstrap mapping ceiling, and fail without
publishing partial ownership.

The count 503 is derived from the immutable static-ELF page plan. It is a
closed executable qualification shape, not a new general upper bound and not a
portable page-size constant. Requests for four through 502 or 504 and above
remain rejected by this profile.

## Architecture pressure test

Affine whole-extent ownership, exact release, fallibility, and opaque physical
identity are portable. The count and first-GiB selection limit belong to the
current x86-64 gate. AArch64 and RISC-V providers may qualify the same page
count using their selected base-frame granule and platform memory description;
capability machines must not turn the extent into ambient integer authority.

## Alternatives rejected

Treating every count through 503 as qualified was rejected because only four
shapes have executable evidence. Allocating 503 independent frames was rejected
because the current source model has no bounded affine collection or rollback
protocol. Hiding the allocation inside the ELF loader was rejected because it
would move resource policy and ownership below checked source. Reusing the
three-frame address-space backing was rejected because the application image
has a distinct lifetime and exact extent.

## Deliberate bounds

This increment allocates, observes, and releases the extent without mapping or
touching it. It does not install 503 leaves, copy the executable, publish final
permissions, initialize TLS, enter BusyBox, or relax the single-live-extent
rule. Physical scatter/gather, fragmentation recovery, dynamic page counts,
subextents, paging, and concurrent application images remain outside scope.
