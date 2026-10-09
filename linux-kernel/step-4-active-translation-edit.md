# Step 4 active translation-edit increment

## Outcome

The checked kernel root now changes its active translation space through the
approved exclusive affine transaction. A map edit consumes one frame extent
into an inaccessible provisional mapping; commit publishes that mapping and
returns the refined active context. A later unmap edit consumes the live
mapping into an unavailable provisional extent; commit completes invalidation
before returning releasable frame ownership. Every admitted failure is
fatal-only.

The architecture-independent model and source checker retain edit kind,
provider and allocator provenance, the sealed normal read-write,
non-executable policy, provider-selected placement, provisional states, and
the two refined active contexts. They reject wrong policy or resource identity,
access before map commit, release before unmap commit, duplicate or incomplete
edits, and continuation through an obsolete context.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/6` implements four typed
edit operations. The provider uses private allocation floors to keep the
payload and two metadata frames above the three-frame active root without
overlap. Map staging clears a page-directory/page-table pair and installs one
present, writable, NX 4 KiB leaf. Map commit publishes that completed subtree
through PDPT entry 1, exposing only the opaque private 1 GiB window to generated
root code.

Unmap staging validates the live parent. Unmap commit clears that parent,
orders the removal, executes local `invlpg` for the private window, and only
then returns success so the root can restore frame ownership. Source names no
virtual or physical address, page-table level, descriptor bit, register, or
instruction. NX enablement, table construction, publication, removal,
invalidation, register-indirect access, distinct provider relocations, and
non-overlapping floors are structurally inspected.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/6` records 25
provider-plan identities, 22 linked semantic placements, and a 39-transition
trace. Gate schema `topal-kernel-toolchain-gate-qemu/7` binds those revisions
and the unchanged boot-adapter revision `/2`.

## Evidence

- the semantics suite passes affine publish/remove behavior, provenance,
  policy, edit-kind, and ownership tests;
- the systems source suite passes the complete accepted lifecycle and negative
  policy/resource/context cases;
- all 188 compiler unit tests pass, including exact generated hierarchy,
  NX, publication, invalidation, dependency, relocation, placement, and trace
  inspection;
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `30fae0ed21aa5ad9bf056f02b9d3a7dab46120aa53a699d4c81373e611545f79`,
  boot image `f43ff6a6f2e083813f6cf7070de55dbca73841f4ce01f9be3247d4f825691db6`,
  artifact provenance `b9f9f1ea58e8e37ea5c07b2f378cf67a7d75520ab2615e757d8b4d1e64ccbb06`,
  boot provenance `1e6d8b30df3b74e6cb725b11b8296415bab31944037f594cd56c9d17031b334e`,
  and serial observation
  `2f18ea449a3a3c62c85bd6eb428c63253d499e72a7286427fdc9724593159855`.

The serial stream emits `TOPAL_KERNEL_TRANSLATION_EDITED` only after the
dynamic mapping has been committed, written, reloaded, staged for removal,
invalidated, and returned for release. It then continues through debug-break
recovery and ordinary kernel-owned memory access before remaining quiescent in
the fatal halt.

## Remaining boundary

This increment admits one local 4 KiB mapping on the initial uniprocessor
execution path. Multiple simultaneous mappings, metadata reclamation,
permission changes, executable or device mappings, user address spaces,
remote TLB shootdown, SMP activation, and AArch64/RISC-V lowering remain
fail-closed future work.
