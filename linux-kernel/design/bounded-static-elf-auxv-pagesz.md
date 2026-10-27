# Bounded static ELF `AT_PAGESZ` fact

This record resolves `TK-DEC-144` as the first nonterminating auxiliary-vector
refinement of the guarded static ELF stack. It is a systems-profile refinement
only: checked source selects a sealed semantic fact, while providers own the
tag and word encodings, stack placement, pointer representation, alignment,
and mutation mechanics.

## Decision

`static image construction add stack pagesz` shall require the exact live
`InitializedStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-143`. It shall accept only that owner, that backing identity, value
`provider-base-page`, and policy `bounded-linux-auxv-before-null`. It shall
produce one affine `PageSizeStackEntryStaticImageConstruction` retaining all
entry, slot, leaf, backing, argument, environment, and termination facts while
adding one provider-consistent `AT_PAGESZ` fact immediately before `AT_NULL`.

Before success, the provider shall validate the complete minimal block and all
translation obligations, reconstruct an aligned complete entry block with the
new fact, perform target-required visibility operations, and read back every
word, pointer relation, string byte, terminator, and otherwise-zero byte.
Source shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK`
observation.

Source shall then invoke `static image construction remove stack pagesz` with
the exact owner, backing, and policy
`bounded-stack-pagesz-qualification-rollback`. Removal shall completely
validate the extended block, reconstruct and completely validate the exact
minimal block from `TK-DEC-143`, and reproduce its affine owner. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK` observation
before existing complete stack-entry clearing.

Neither operation shall expose the auxiliary-vector tag, numeric page size,
stack pointer, virtual or physical address, word encoding, offset, alignment,
or write mechanism. It shall not add another auxiliary-vector fact, initialize
TLS, activate the hierarchy, construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall use native `AT_PAGESZ` tag 6 with value 4096, place it after
the empty environment terminator and before `AT_NULL`, and retain the exact
`busybox` and `echo` pointer/string relations. The reconstructed entry pointer
shall remain 16-byte aligned. Addition and removal shall each clear the
complete backing before reconstruction, use one store fence, and completely
read back the resulting frame.

The exact RW/NX stack leaf, absent guard, 503 final image leaves, seven other
absent leaves, inactive hierarchy, and qualified bootstrap root shall remain
unchanged. No active-address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact initialized minimal block may gain the page-size fact. While the
fact is live, ordinary stack-entry clearing, leaf revocation, backing release,
activation, teardown, duplicate addition, or recoverable disposition is
invalid. Only exact removal may restore the minimal initialized owner admitted
by complete clearing. Any content, placement, owner, backing, translation,
ordering, or readback failure is fatal.

## Architecture pressure test

The portable fact is the provider's Linux base-page size, not the number 4096.
AArch64 and RISC-V providers may select a different configured base-page value
and their own process-entry encoding. Capability systems retain bounded
in-stack capabilities. Source observes only the semantic fact and affine
lifecycle.

## Alternatives rejected

Passing a source integer was rejected because the provider and prepared
address space must agree on page size. Mutating only two words without
reconstructing and rereading the complete block was rejected because changed
entry placement and pointer relations require whole-object proof. Adding all
deterministic auxiliary-vector facts at once was rejected because executable
metadata, credentials, platform identity, and clock policy have distinct
authorities and compatibility evidence.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_PAGESZ` fact. It
does not retain that fact after qualification, add executable, credential,
platform, clock, secure-mode, or random facts, initialize TLS, construct a user
context, enter user mode, execute BusyBox, or add a syscall.
