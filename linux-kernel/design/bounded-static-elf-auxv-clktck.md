# Bounded static ELF `AT_CLKTCK` fact

This record resolves `TK-DEC-150` as the userspace-accounting-rate refinement
of the guarded static ELF stack. It is a systems-profile refinement only:
checked source composes the secure-mode-qualified block with a sealed Linux
ABI fact, while providers own auxiliary-vector tags, scalar widths, placement,
alignment, and mutation representation.

## Decision

`static image construction add stack clock tick rate` shall require the exact
live `SecureModeStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-149`. It shall accept only that owner, that backing identity, value
`linux-user-hz`, and policy `bounded-linux-auxv-before-null`. It shall produce
one affine `ClockTickRateStackEntryStaticImageConstruction` retaining every
prior fact while adding one userspace accounting-rate fact after `AT_SECURE`
and before `AT_NULL`.

The fact shall denote Linux's baseline `USER_HZ` contract. It shall not be
derived from or interpreted as the qualified HPET period, a scheduler timer,
or a target counter. Source shall neither choose nor observe its numeric
encoding. Before success, the provider shall revalidate the complete
secure-mode-qualified block and every inactive-translation obligation, clear
the complete backing, reconstruct the target-aligned block, perform target-
required visibility operations, and read back every word, pointer relation,
string byte, terminator, clock identity, and otherwise-zero byte. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK` observation.

Source shall then invoke `static image construction remove stack clock tick
rate` with the exact owner, backing, and policy
`bounded-stack-clock-tick-rate-qualification-rollback`. Removal shall validate
the complete extended block, reconstruct and completely validate the exact
secure-mode-qualified block, and reproduce its affine owner. Source shall emit
exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK` observation before
secure-mode removal.

Neither operation shall expose a rate number, tag, stack pointer, virtual or
physical address, word encoding, offset, alignment, or write mechanism. It
shall not add platform or random facts, initialize TLS, activate the hierarchy,
construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall encode native `AT_CLKTCK=(17,100)` after
`AT_SECURE=(23,0)` and before `AT_NULL`. The value is Linux's userspace
accounting frequency, not a hardware or scheduling frequency.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact security and credential
facts, executable-name alias, RW/NX stack leaf, guard, final image leaves,
inactive hierarchy, and bootstrap root remain unchanged. No active-address
invalidation is permitted.

## Affine lifecycle and rollback

Only the exact secure-mode-qualified block may gain the accounting-rate fact.
While live, secure-mode or prior-fact removal, ordinary entry clearing, leaf
revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact fact removal may restore the
secure-mode-qualified owner. Any content, identity, agreement, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable fact is Linux's baseline userspace accounting rate, not a source
integer or hardware timer. X86-64, AArch64, and RISC-V can lower the same Linux
semantic fact through their native process-entry layouts. Capability systems
gain no memory or timer authority from it. Source sees only the semantic
relationship and affine lifecycle.

## Alternatives rejected

Omitting the fact was rejected because libc exposes it through
`sysconf(_SC_CLK_TCK)`. Deriving it from HPET or scheduler frequency was
rejected because those clocks have different semantics. A source-selected
integer was rejected because ABI construction remains kernel policy. Combining
platform or random facts was rejected because their authorities and evidence
differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_CLKTCK` fact. It
does not retain the fact after qualification, add platform or random facts,
initialize TLS, construct a user context, enter user mode, execute BusyBox, or
add a syscall.
