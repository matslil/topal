# Step 4 opaque kernel-mapping increment

## Outcome

The checked kernel root now consumes its affine `PhysicalFrameExtent` through
the approved `memory kernel map frames (...)` decision. Success produces one
opaque `KernelMapping`; only that capability admits bounded byte access.
`memory kernel unmap mapping` consumes the mapping and returns the original
extent, which is then released. Mapping failure and a mismatched byte both
terminate through the fatal disposition without exposing an address.

The architecture-independent model records rights, execution policy, memory
kind, provider provenance, opaque physical and virtual identities, byte
contents, and ownership state. It rejects unsupported policy, cross-allocator
unmap, frame release while mapped, uninitialized or out-of-bounds access, and
completion with live mapping authority.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/4` adopts the qualified
bootstrap identity mapping without making identity mapping part of the common
contract. The E820 selector now returns one complete 4 KiB RAM frame above the
16 MiB reservation floor and wholly below the existing 1 GiB identity-map
limit. Generated root code retains that provider-private base, stores byte
`165` through it, reloads and compares the byte, logically unmaps the extent,
and clears the retained base on release.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/4` records the
four mapping operations in the provider plan, linked placements, and canonical
semantic trace. Structural tests require the selector bound and distinct
register-indirect store/load/compare sequence; a constant-folded observation
cannot satisfy them.

## Evidence

- all 64 semantics-library tests pass, including mapping ownership, policy,
  provenance, bounds, initialization, unmap, and mapped-frame release checks;
- all 11 systems source-checker tests pass, including the exact sealed policy,
  affine identities, static one-frame bounds, exhaustive results, cleanup on
  both comparison paths, and no live authority at disposition;
- all 188 compiler-library tests pass, including the 18-operation provider
  map, closed provider object, physical-address return, root indirect access,
  15 linked placements, and 24-transition artifact trace;
- gate schema `topal-kernel-toolchain-gate-qemu/5` records provider and
  artifact revisions `/4` and the unchanged boot-adapter revision `/2`; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `cb6099426e62e92bc10bb664d8ff4f993b443f6b225994846ceea0124628720e`,
  boot image `545a90e3eedfd21747cdabbe533d43f5f0a06150d5a1c643aabc892ddf291ca7`,
  artifact provenance `b3ec783b52a44cb26b9e6bddc4f0415f2d39e3bc09b71e52a29b23627e0ecb18`,
  boot provenance `fd5ffbf600f5ae3231f107a610fe79c1a65b4b6ee036b1d5310caf967e2d014a`,
  and serial observation
  `c811173f42d1a84581efdd3ad44ac5c64496e1e475a05c742e1bf4447eb2058e`.

The serial stream emits `TOPAL_KERNEL_FRAME_MAPPED` only after the indirect
store/load comparison succeeds, between allocation and release markers, then
remains quiescent while QEMU stays in the fatal halt.

## Remaining boundary

This increment qualifies one normal, read-write, non-executable kernel mapping
under the initial x86-64 bootstrap provider. General page-table construction,
translation activation, permission changes, shootdown, multiple simultaneous
mappings, dynamic offsets, user mappings, device memory, DMA, executable
mappings, reclamation, and other boot paths remain fail-closed future work.
