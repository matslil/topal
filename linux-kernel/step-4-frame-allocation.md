# Step 4 affine physical-frame allocation increment

## Outcome

The checked kernel root now consumes its `MemoryDescribedContext` through the
approved `memory create frame allocator` decision. The resulting
`FrameAllocatorContext` accepts the closed initial request for one frame
aligned to one frame, produces one affine opaque extent, and requires that
extent to be released before later bootstrap work or a disposition.

The architecture-independent semantic model normalizes allocator-owned free
runs, selects aligned nonoverlapping extents deterministically, retains
provider provenance, rejects malformed and exhausted requests without losing
allocator state, prevents cross-allocator and double release, coalesces only
compatible released runs, and refuses completion while an extent remains
live. No numeric physical address, mapping authority, page-table format, or
machine instruction enters the common model or Topal source.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/3` maps the common
transitions to three sealed responsibilities:

- allocator creation retains the previously validated E820 frame authority;
- allocation invokes `topal_x86_systems_allocate_physical_frames`, which
  privately scans the bounded Linux zeropage E820 table for a complete 4 KiB
  RAM frame above the 16 MiB bootstrap reservation floor; and
- release consumes the compiler-owned opaque extent state.

The root-object lowering tracks allocator creation and live-extent state,
admits only the approved `(frame-count: 1, alignment-frames: 1)` request,
branches to the nonreturning fatal provider when selection fails, and rejects
lowering that ends with a live extent. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/3` requires the selector symbol in
the linked ELF and map, records its semantic placement, and includes create,
allocate, and release in the canonical semantic trace.

## Evidence

- all 62 semantics-library tests pass, including aligned allocation,
  nonoverlap, reuse after release, exhaustion, malformed requests, provenance,
  wrong-allocator release, and live-extent completion rejection;
- all 10 systems source-checker tests pass, including wrong contexts,
  unsupported counts and alignments, wrong extent release, and nonfatal
  failure rejection;
- all 188 compiler library tests pass, including the 14-operation provider
  map, closed provider object, typed selector relocation, linked-symbol
  inspection, 16 MiB reservation floor, and 19-transition artifact trace;
- the gate schema is `topal-kernel-toolchain-gate-qemu/4` and records provider
  revision `/3`, artifact revision `/3`, and the unchanged qualified Linux
  boot-adapter revision `/2`; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU reproductions produced identical
  digests: linked kernel
  `cc86d4088f1ca806a849cc58d2827f150a191f84143b6bdfe22a99c67fcae75e`,
  boot image
  `d4c97b8da21dc2fc4b2147b29954f06ffd412987b75c6d19dca702ed7f888eba`,
  artifact provenance
  `57291617fe54ce058bec8cd35a51856539d7d2918843c509f3dd2902fa8f5259`,
  boot provenance
  `221dfafb78733ce838b483b2ca5b629ca17e31174e65f02aacf6b4cf40bc5c76`,
  and serial observation
  `ae75dfaee6df7587a0f14b8cc5e6a34f0c807c741ddf506f4c6b020191dce3c3`.

The serial stream emitted `TOPAL_KERNEL_FRAME_ALLOCATED` before the existing
memory-description, boot, resumed-fault, and checked-memory markers, then
remained quiescent while QEMU stayed in the fatal halt.

## Remaining boundary

This increment established allocation ownership, not memory access. The
subsequent [opaque kernel-mapping increment](step-4-kernel-mapping.md) adds
capability-mediated access without exposing an address. Multiple simultaneously
live extents, dynamic request sizes, general page-table construction, reclaim,
setup-data extension traversal, and other boot paths remain fail-closed future
work.
