# Bounded contiguous physical-frame extent refinement

This record resolves `TK-DEC-035` as a systems-profile refinement of the
existing affine physical-frame allocator. It does not add grammar, expose an
address, or change portable Topal semantics.

## Decision

The initial executable x86-64 profile admits a second static allocation shape:
three target-profile base frames aligned to one frame. The request returns one
affine `PhysicalFrameExtent`, not three independently usable values. Source
must release the complete extent to the allocator which created it before any
later bootstrap transition or disposition.

The checked qualification sequence is:

```topal
context frames allocate (
  frame-count is 3,
  alignment-frames is 1
)
  Ok extent then {
    context console write "TOPAL_KERNEL_FRAME_EXTENT_OK"
    context frames release extent
    // continue with the returned allocator context
  }
  Error problem then context fatal "multi-frame allocation failed"
```

This is a closed profile qualification of `TOPAL-SYSTEMS-FRAMES-001`. The
general contract already defines nonzero counts and power-of-two alignment;
the refinement supplies executable evidence for one additional static count.
Counts other than one and three, non-unit alignment, dynamic requests,
splitting an extent, and multiple simultaneously live extents remain rejected.

## Why one contiguous extent

The next source-owned address-space increment needs a bounded group of frames
while preserving opaque physical identity. One contiguous affine extent keeps
the allocation and release obligation atomic and lets later mapping work add
an explicit, reviewed split or subextent operation if independently owned
frames are actually required.

Three independent allocations were rejected for this increment because the
current executable checker deliberately permits only one live extent. A
provider-created user-memory bundle was also rejected because it would hide
allocation policy and ownership in the target backend. General dynamic counts
were deferred because they require source proofs or explicit bounded failure
behavior beyond this fixed qualification.

## Cross-architecture contract

The number three counts the base-frame unit selected by the target profile; it
does not mean 12 KiB in portable source. An x86-64 provider uses three
contiguous 4 KiB frames. A future AArch64 provider uses three frames of its
qualified translation granule, and a RISC-V provider uses three frames of its
qualified page/Sv profile. Every provider must prove that the complete extent
is aligned, lies within one allocatable normalized run, does not wrap or cross
a retained reservation, and remains opaque to source.

## Initial x86-64 qualification

The sealed provider accepts only `(1, 1)` and `(3, 1)`. It derives the required
byte extent with checked arithmetic, scans the bounded normalized E820 input,
and succeeds only when the whole candidate lies in RAM above the bootstrap
reservation floor and within the qualified bootstrap mapping. The root
lowering records the live extent's count, requires exact whole-extent release,
and clears provider-private base and size state after consumption.

The three-frame probe is sequential with the existing one-frame mapping
probes. It therefore qualifies multi-frame extent geometry and ownership, but
not simultaneous live extents, subextent access, page-table construction, or
user mapping. Those remain separate fail-closed increments.

This systems-only refinement was approved in the project discussion before
implementation.
