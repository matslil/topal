# Bounded static ELF RELRO `mprotect`

## Decision

After aligning its program break, the pinned x86-64 static executable issues
`mprotect(0x005e5000, 0x8000, PROT_READ)`. The bounded static-process profile
shall admit this exact request, confirm that it names the eight page-rounded
GNU RELRO pages of the validated image, return zero, and advance the private
startup phase before terminal exit.

The loader has already sealed those pages read-only and non-executable before
process entry. The request is therefore an idempotent Linux protection change:
the provider shall validate every named leaf and the live process state but
shall not rewrite a translation or invalidate the target. This preserves the
approved rule that RELRO is never writable after publication while presenting
the result expected by the executable.

Before success the provider shall validate the aligned current break, sealed
executable continuation, active address-space hierarchy, exact RELRO interval,
all eight present user RO/NX leaves with their original affine frame identity,
complete heap and disjoint stack state, and reusable-random service. Failure
shall preserve every resource and the prior startup phase. Terminal rollback
shall remain complete and shall require the new phase.

## Portable boundary

Source names one bounded protection confirmation over the image role selected
by the checked ELF plan. It does not expose a syscall number, native protection
bits, virtual address, page size, register convention, leaf-table format, or
whether a target must edit translations. AArch64, RISC-V, and capability-system
providers may represent the same immutable role differently while preserving
the exact interval, non-writability, non-executability, frame identity, ordered
completion, and rollback.

## Rejected alternatives

- Making RELRO writable until this call was rejected because the approved
  loader publishes the image only after final W^X and RELRO sealing.
- Returning zero without validating the selected image role was rejected
  because it would fabricate success for corrupted or differently mapped
  process state.
- Rewriting identical leaf entries and invalidating the interval was rejected
  because no observable protection transition requires either operation.
- General multi-range `mprotect`, executable permission, splitting, protection
  keys, and VMA policy were rejected as broader than this observed boundary.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact read-only protection confirmation for the sealed GNU RELRO
role. It does not support other ranges, lengths, protection combinations,
general `mprotect`, dynamic linking, faults, signals, concurrency, fork/exec
inheritance, BusyBox redistribution, containers, or a prompt.
