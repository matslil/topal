# Step 4 translation-space activation increment

## Outcome

The checked kernel root now constructs and activates a replacement translation
space through the approved affine lifecycle. `translation begin` reserves
provider-owned backing and returns one exclusive `TranslationUpdate`;
`translation commit` consumes it into one inactive `TranslationSpace`; and
`translation activate` consumes that space and returns the only bootstrap
context admitted for subsequent operations. Every initial failure path is
fatal-only.

The architecture-independent model retains the sealed
`bootstrap-equivalent` template, `provider-selected` page policy, provider and
backing provenance, and update/space/active states. The source checker rejects
different policies, duplicated or wrong affine resources, activation before
commit, continuation through the old context, incomplete result coverage, and
a non-fatal failure continuation.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/5` implements three
distinct generated operations. Begin selects three contiguous complete 4 KiB
E820 RAM frames above the 16 MiB reserved floor and wholly below the qualified
first-GiB bootstrap identity limit. Commit zeroes those frames and constructs a
PML4, PDPT, and page directory with 512 present, writable 2 MiB leaves. Activate
validates the root and performs the qualified CR3 transition before returning.

The source names no physical address, table level, descriptor, instruction,
register, or maintenance sequence. Structural tests require three-page
selection, complete table zeroing and leaf population, publication fencing,
three distinct provider calls and failure branches, the retained CR3
instruction, and consumption of the inactive-space token.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/5` records 21
provider-plan identities, 18 linked semantic placements, and a 28-transition
trace containing begin, commit, activation, and the post-activation marker in
source order.

## Evidence

- all architecture-independent semantics tests pass, including backing
  ownership, provider provenance, insufficient backing, and wrong-provider
  activation;
- all focused systems source-checker tests pass, including sealed request,
  affine lifecycle, exhaustive results, refined-context continuation, and
  fatal-only failures;
- focused provider and linked-artifact tests pass, including exact generated
  instruction sequences, root dependencies, relocation counts, placement
  records, transition trace, and repeatable publication;
- gate schema `topal-kernel-toolchain-gate-qemu/6` binds provider and artifact
  revisions `/5` and the unchanged boot-adapter revision `/2`; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical linked
  kernel `6799d37e3daf32a27dfb4def70ae479955ecfd565a147b02211eef3ddf8910da`,
  boot image `39eeffec17e9eda29008e4e63edc1ccc6ef5271680000ffaaf01ceb1fc1acbac`,
  artifact provenance `7fc85c600790d7eed3e41ae576d5b40b82c7b2df0d18c879018a06b635c9e709`,
  boot provenance `123129808fa29d7b39526486c044ff260bfd04185fbd5a489ecd90b9a4f3ac0d`,
  and serial observation
  `53250183afc70b5e3f09a5a792eeb825357c530dba48ce8153bc7ae96ed154c3`.

The serial stream emits `TOPAL_KERNEL_TRANSLATION_ACTIVE` only after the new
root has been loaded, then continues through debug-break recovery and ordinary
kernel-owned memory access. It remains quiescent while QEMU stays in the fatal
halt.

## Remaining boundary

This increment replaces only the bootstrap-equivalent translation on the
initial x86-64 provider. Arbitrary mapping edits, permission changes, NX
enablement, user spaces, multiple active or per-processor spaces, shootdown,
backing reclamation, and AArch64/RISC-V lowering remain fail-closed future
work.
