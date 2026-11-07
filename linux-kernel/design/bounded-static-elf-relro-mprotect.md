# Bounded static ELF RELRO `mprotect`

## Decision

After aligning its program break, the pinned x86-64 static executable issues
`mprotect(0x005e5000, 0x8000, PROT_READ)`. The bounded static-process profile
shall admit this exact request, confirm that it names the eight page-rounded
GNU RELRO pages of the validated image, return zero, and advance the private
startup phase before terminal exit.

The loader publishes those pages writable and non-executable for bounded
static self-relocation before this request. The provider shall validate every
named leaf and the live process state, replace all eight leaves with their
read-only and non-executable form while retaining exact frame identity,
complete target visibility and invalidation for the interval, read back every
leaf, and only then return success. This preserves W^X throughout and makes the
observed request the unique transition to immutable RELRO.

Before success the provider shall validate the aligned current break, sealed
executable continuation, active address-space hierarchy, exact RELRO interval,
all eight present user RW/NX leaves with their original affine frame identity,
complete heap and disjoint stack state, and reusable-random service. Failure
shall preserve every resource and the prior startup phase. Terminal rollback
shall remain complete and shall require the new phase.

## Portable boundary

Source names one bounded protection transition over the image role selected
by the checked ELF plan. It does not expose a syscall number, native protection
bits, virtual address, page size, register convention, leaf-table format, or
how a target edits or invalidates translations. AArch64, RISC-V, and capability-system
providers may represent the same immutable role differently while preserving
the exact interval, non-writability, non-executability, frame identity, ordered
completion, and rollback.

## Rejected alternatives

- Sealing RELRO before this call was rejected because the pinned external
  executable performs bounded self-relocation inside the checked role.
- Returning zero without validating the selected image role was rejected
  because it would fabricate success for corrupted or differently mapped
  process state.
- Returning success without changing and invalidating all eight leaves was
  rejected because it would leave post-relocation data writable.
- General multi-range `mprotect`, executable permission, splitting, protection
  keys, and VMA policy were rejected as broader than this observed boundary.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact read-only protection transition for the staged GNU RELRO
role. It does not support other ranges, lengths, protection combinations,
general `mprotect`, dynamic linking, faults, signals, concurrency, fork/exec
inheritance, BusyBox redistribution, containers, or a prompt.
