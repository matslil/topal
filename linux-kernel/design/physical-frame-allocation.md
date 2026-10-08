# Affine physical-frame allocation contract

## Decision

The second kernel-foundation transition turns a normalized boot-memory
description into one exclusive physical-frame allocator. Source spells the
transition as `context memory create frame allocator` and handles its result
exhaustively:

```topal
described memory create frame allocator
  Ok memory then {
    memory frames allocate (
      frame-count is 1,
      alignment-frames is 1
    )
      Ok frames then {
        memory console write "TOPAL_KERNEL_FRAME_ALLOCATED"
        memory frames release frames
        memory fatal "complete"
      }
      Error problem then memory fatal "frame allocation failed"
  }
  Error failure then {
    failure fatal "frame allocator creation failed"
  }
```

The transition consumes the `MemoryDescribedContext`. Its success binding is a
`FrameAllocatorContext` which retains the preceding bootstrap capabilities and
owns all normalized allocatable-frame authority. Its error binding is a
fatal-only `FrameAllocatorFailureContext`. Neither result may escape the entry
extent, and the consumed context is invalid on both paths.

This decision resolves `TK-DEC-018`. It introduces no pointer, numeric address,
page-table record, register, instruction, or source-assembly vocabulary.

## Allocation and ownership

`frames allocate` borrows one live allocator context and requests a nonzero
frame count and a power-of-two alignment measured in frames. Success returns
one affine `PhysicalFrameExtent` with allocator identity, extent identity,
frame count, alignment, and provider provenance. Its physical base remains
opaque. The extent grants ownership of frames, but it grants no byte access,
mapping, device, DMA, firmware, or user-memory authority.

The allocator never overlaps live extents or any reservation retained by the
boot-memory description. An exhausted or malformed request selects the
explicit `Error` action without invalidating the allocator. Allocation policy
and free-range data structures are ordinary Topal library concerns; the
language contract permits any deterministic choice whose observable success,
failure, alignment, non-overlap, and ownership behavior is equivalent.

`frames release` consumes one live extent and returns it only to the allocator
identity which produced it. Double release, cross-allocator release, use after
release, extent escape, and any bootstrap disposition with a live extent are
invalid. Coalescing is an allocator implementation choice and cannot merge
across a live reservation or distinct non-allocatable provenance.

The initial executable slice accepts only the closed request for one frame
aligned to one frame. Dynamic sizes, multiple simultaneously live extents,
mapping an extent, and reclaiming boot-owned ranges remain fail-closed until
their separate implementations and evidence exist.

## Architecture pressure test

The common abstraction is ownership of target-qualified complete frames, not
one page-table format:

| Target family | Provider-private geometry | Common allocator meaning |
| --- | --- | --- |
| x86-64 | 4 KiB base pages, E820-derived RAM, later x86 page-table modes | affine complete-frame extents from normalized allocatable ranges |
| AArch64 | selected translation granule, Device Tree and/or UEFI descriptors | the same count, alignment, ownership, and release obligations |
| RISC-V | selected page granule and Sv mode, Device Tree/SBI facts | the same opaque extent and allocator provenance |

A target profile fixes its base-frame geometry before allocator creation. A
portable request counts those frames and cannot infer their numeric byte size
or use one target's frame extent under another target identity.

## Initial x86-64 executable slice

The sealed x86 provider privately retains the validated Linux `boot_params`
handoff. For the one-frame request it selects a complete E820 RAM frame above
the closed 16 MiB bootstrap reservation floor and reports failure if no such
frame exists. The compiler enforces that only one extent is live, that release
precedes every later disposition, and that source never observes the selected
physical base. The pinned QEMU gate emits `TOPAL_KERNEL_FRAME_ALLOCATED` only
after the provider succeeds and the affine extent is established.

This contract was approved in the project discussion before implementation.
