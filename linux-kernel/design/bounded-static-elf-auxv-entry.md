# Bounded static ELF `AT_ENTRY` fact

This record resolves `TK-DEC-145` as the executable-entry auxiliary-vector
refinement of the guarded static ELF stack. It is a systems-profile refinement
only: checked source composes an existing opaque entry authority with the
page-size-qualified entry block, while providers own tag, address, word,
placement, alignment, and mutation representation.

## Decision

`static image construction add stack entry fact` shall require the exact live
`PageSizeStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-144`. It shall accept only that owner, that backing identity, value
`authorized-static-elf-entry`, and policy `bounded-linux-auxv-before-null`.
It shall produce one affine `ExecutableEntryStackEntryStaticImageConstruction`
retaining all prior authorities and process-entry facts while adding one
`AT_ENTRY` fact immediately after `AT_PAGESZ` and before `AT_NULL`.

Before success, the provider shall revalidate the complete page-size block,
the opaque entry authority against the resident static ELF header and exact
executable leaf, and every inactive-translation obligation. It shall clear the
complete backing, reconstruct a target-aligned block with the authorized entry
fact, perform target-required visibility operations, and read back every word,
pointer relation, string byte, terminator, executable-entry relation, and
otherwise-zero byte. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK` observation.

Source shall then invoke `static image construction remove stack entry fact`
with the exact owner, backing, and policy
`bounded-stack-entry-fact-qualification-rollback`. Removal shall validate the
complete extended block, reconstruct and completely validate the exact
page-size block from `TK-DEC-144`, and reproduce its affine owner. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK` observation
before page-size removal.

Neither operation shall expose the auxiliary-vector tag, numeric executable
entry, stack pointer, virtual or physical address, word encoding, offset,
alignment, or write mechanism. It shall not add another auxiliary-vector fact,
initialize TLS, activate the hierarchy, construct a user context, or enter the
image.

## Exact x86-64 realization

The provider shall encode native `AT_ENTRY` tag 9 with the exact retained ELF
entry `0x4a6580`, after `AT_PAGESZ=(6,4096)` and before `AT_NULL`. It shall
prove the value lies in the existing executable interval and corresponds to
the existing entry authority. The reconstructed entry pointer shall remain
16-byte aligned, with unchanged `busybox` and `true` pointer/string relations.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The exact RW/NX stack leaf, guard, final image leaves, inactive hierarchy, and
qualified bootstrap root shall remain unchanged. No active-address
invalidation is permitted.

## Affine lifecycle and rollback

Only the exact page-size-qualified block with its retained entry authority may
gain the executable-entry fact. While live, page-size removal, ordinary entry
clearing, leaf revocation, backing release, entry-authority revocation,
activation, teardown, duplicate addition, or recoverable disposition is
invalid. Only exact removal may restore the page-size-qualified owner. Any
content, containment, authority, ordering, translation, or readback failure is
fatal.

## Architecture pressure test

The portable fact is the executable entry selected by the validated image, not
an integer address. AArch64 and RISC-V providers use their Linux process-entry
encoding and executable mapping rules. Capability systems derive a bounded
executable capability without fabricating or widening authority. Source sees
only the semantic relationship and affine lifecycle.

## Alternatives rejected

Passing the ELF integer through checked source was rejected because a forgeable
number would bypass entry authority and architecture-specific executable
capability rules. Copying the file-header value without resident executable-
leaf validation was rejected because metadata alone does not grant execution
authority. Combining program-header, credential, platform, clock, secure-mode,
and random facts was rejected because their authorities and evidence differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_ENTRY` fact. It
does not retain that fact after qualification, add program-header, credential,
platform, clock, secure-mode, or random facts, initialize TLS, construct a user
context, enter user mode, execute BusyBox, or add a syscall.
