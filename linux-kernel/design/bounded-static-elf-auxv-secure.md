# Bounded static ELF `AT_SECURE` fact

This record resolves `TK-DEC-149` as the execution-security refinement of the
guarded static ELF stack. It is a systems-profile refinement only: checked
source composes the credential-qualified block with the sealed loader result,
while providers own auxiliary-vector tags, scalar widths, placement,
alignment, and mutation representation.

## Decision

`static image construction add stack secure mode` shall require the exact live
`CredentialStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-148`. It shall accept only that owner, that backing identity, value
`initial-execution-security`, and policy `bounded-linux-auxv-before-null`. It
shall produce one affine `SecureModeStackEntryStaticImageConstruction`
retaining every prior fact while adding one non-secure execution fact after the
credential group and before `AT_NULL`.

The fact shall agree with equal real/effective root credentials and the sealed
absence of set-user-ID, set-group-ID, file-capability, or security-module
transition processing. Source shall neither choose nor observe its numeric
encoding. Before success, the provider shall revalidate the complete
credential-qualified block and every inactive-translation obligation, clear
the complete backing, reconstruct the target-aligned block, perform target-
required visibility operations, and read back every word, pointer relation,
string byte, terminator, security relation, and otherwise-zero byte. Source
shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK` observation.

Source shall then invoke `static image construction remove stack secure mode`
with the exact owner, backing, and policy
`bounded-stack-secure-mode-qualification-rollback`. Removal shall validate the
complete extended block, reconstruct and completely validate the exact
credential-qualified block, and reproduce its affine owner. Source shall emit
exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK` observation before
credential removal.

Neither operation shall expose a secure-mode number, tag, stack pointer,
virtual or physical address, word encoding, offset, alignment, or write
mechanism. It shall not add clock, platform, or random facts, initialize TLS,
activate the hierarchy, construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall encode native `AT_SECURE=(23,0)` after
`AT_EGID=(14,0)` and before `AT_NULL`. Zero means that the sealed image did not
cross a privilege-changing execution boundary; it is not a source-selected
policy flag.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact credential group,
executable-name alias, RW/NX stack leaf, guard, final image leaves, inactive
hierarchy, and bootstrap root remain unchanged. No active-address invalidation
is permitted.

## Affine lifecycle and rollback

Only the exact credential-qualified block may gain the security fact. While
live, credential or prior-fact removal, ordinary entry clearing, leaf
revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact fact removal may restore the
credential-qualified owner. Any content, identity, agreement, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable fact is the non-secure result of the sealed executable transition,
not a source integer. X86-64, AArch64, and RISC-V use the same Linux semantic
fact through native process-entry layouts. Capability systems gain no memory
authority from it. Source sees only the semantic relationship and affine
lifecycle.

## Alternatives rejected

Omitting the fact was rejected because libc otherwise lacks an explicit
secure-execution result. Publishing one was rejected because no qualifying
privilege transition exists. A source-selected value was rejected because
loader security state is kernel policy. Combining clock, platform, and random
facts was rejected because their authorities and evidence differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_SECURE` fact. It
does not retain the fact after qualification, add clock, platform, or random
facts, initialize TLS, construct a user context, enter user mode, execute
BusyBox, or add a syscall.
