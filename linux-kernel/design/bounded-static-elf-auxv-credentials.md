# Bounded static ELF credential facts

This record resolves `TK-DEC-148` as the process-credential refinement of the
guarded static ELF stack. It is a systems-profile refinement only: checked
source composes the executable-name-qualified block with the sealed initial
process profile, while providers own auxiliary-vector tags, scalar widths,
placement, alignment, and mutation representation.

## Decision

`static image construction add stack credentials` shall require the exact live
`ExecutableNameStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-147`. It shall accept only that owner, that backing identity, value
`initial-process-credentials`, and policy `bounded-linux-auxv-before-null`. It
shall produce one affine `CredentialStackEntryStaticImageConstruction`
retaining every prior fact while adding one coherent real/effective user/group
identity group after `AT_EXECFN` and before `AT_NULL`.

The facts shall agree with the provider-sealed credential identity of the
prepared PID-one process profile. The bounded profile admits only the sealed
privileged identity; source shall neither supply nor observe its numeric
encoding. Before success, the provider shall revalidate the complete
executable-name-qualified block and every inactive-translation obligation,
clear the complete backing, reconstruct the target-aligned block, perform
target-required visibility operations, and read back every word, pointer
relation, string byte, terminator, credential relation, and otherwise-zero
byte. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK` observation.

Source shall then invoke `static image construction remove stack credentials`
with the exact owner, backing, and policy
`bounded-stack-credentials-qualification-rollback`. Removal shall validate the
complete extended block, reconstruct and completely validate the exact
executable-name-qualified block, and reproduce its affine owner. Source shall
emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK` observation before
executable-name removal.

Neither operation shall expose credential numbers, tags, stack pointers,
virtual or physical addresses, word encodings, offsets, alignment, or write
mechanisms. It shall not add platform, clock, secure-mode, or random facts,
initialize TLS, activate the hierarchy, construct a user context, or enter the
image.

## Exact x86-64 realization

The provider shall encode native `AT_UID=(11,0)`, `AT_EUID=(12,0)`,
`AT_GID=(13,0)`, and `AT_EGID=(14,0)` in that order after
`AT_EXECFN=(31,0x7fffe0)` and before `AT_NULL`. The four facts form one atomic
group and all agree with the sealed PID-one root profile.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact executable-name alias,
RW/NX stack leaf, guard, final image leaves, inactive hierarchy, and bootstrap
root remain unchanged. No active-address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact executable-name-qualified block may gain the credential group.
While live, executable-name or prior-fact removal, ordinary entry clearing,
leaf revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact group removal may restore the
executable-name-qualified owner. Any content, identity, agreement, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable fact is one coherent credential identity matching the prepared
process profile, not four source integers. AArch64 and RISC-V providers use
their Linux process-entry encoding. Capability systems gain no memory
authority from the scalar facts. Source sees only the semantic relationship
and affine lifecycle.

## Alternatives rejected

Passing four source integers was rejected because it permits forged or
inconsistent process identity. Adding the facts separately was rejected
because partial credential groups create invalid intermediate states. Making
root identity implicit was rejected because Linux consumers observe these
facts independently. Combining platform, clock, secure-mode, and random facts
was rejected because their authorities and evidence differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one credential fact
group. It does not retain the group after qualification, add platform, clock,
secure-mode, or random facts, initialize TLS, construct a user context, enter
user mode, execute BusyBox, or add a syscall.
