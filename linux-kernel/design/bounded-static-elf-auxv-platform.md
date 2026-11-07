# Bounded static ELF `AT_PLATFORM` fact

This record resolves `TK-DEC-151` as the native-platform refinement of the
guarded static ELF stack. It is a systems-profile refinement only: checked
source composes the clock-rate-qualified block with a sealed platform identity,
while providers own native strings, auxiliary-vector tags, pointers, placement,
alignment, and mutation representation.

## Decision

`static image construction add stack platform` shall require the exact live
`ClockTickRateStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-150`. It shall accept only that owner, that backing identity, value
`native-userspace-platform`, and policy `bounded-linux-auxv-before-null`. It
shall produce one affine `PlatformStackEntryStaticImageConstruction` retaining
every prior fact while adding one provider-consistent native-platform fact
after `AT_CLKTCK` and before `AT_NULL`.

The fact shall report an already selected userspace ABI; it shall not select a
machine provider. Source shall neither choose nor observe its string, pointer,
tag, or encoding. Before success, the provider shall revalidate the complete
clock-rate-qualified block and every inactive-translation obligation, clear
the complete backing, reconstruct the target-aligned block, perform target-
required visibility operations, and read back every word, pointer relation,
string byte, terminator, identity separation, and otherwise-zero byte. Source
shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK` observation.

Source shall then invoke `static image construction remove stack platform`
with the exact owner, backing, and policy
`bounded-stack-platform-qualification-rollback`. Removal shall validate the
complete extended block, reconstruct and completely validate the exact clock-
rate-qualified block, and reproduce its affine owner. Source shall emit exactly
one `TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK` observation before
clock-rate removal.

Neither operation shall expose a platform string, pointer, tag, stack address,
word encoding, offset, alignment, or write mechanism. It shall not add random
or hardware-capability facts, initialize TLS, activate the hierarchy, construct
a user context, or enter the image.

## Exact x86-64 realization

The provider shall encode native `AT_PLATFORM=(15, pointer-to-"x86_64\0")`
after `AT_CLKTCK=(17,100)` and before `AT_NULL`. The complete platform string
shall reside inside the initialized stack block and remain distinct from the
`busybox` executable-name object; `argv[0]` and `AT_EXECFN` shall retain exact
alias identity only with the latter.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact clock, security,
credential, and executable-name facts, RW/NX stack leaf, guard, final image
leaves, inactive hierarchy, and bootstrap root remain unchanged. No active-
address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact clock-rate-qualified block may gain the platform fact. While
live, clock-rate or prior-fact removal, ordinary entry clearing, leaf
revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact fact removal may restore the
clock-rate-qualified owner. Any content, identity, aliasing, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable fact is a provider-consistent native userspace-platform identity,
not the x86 string or pointer. X86-64, AArch64, and RISC-V providers lower their
qualified native strings through their process-entry layouts. Capability
systems gain no memory authority from the pointer representation. Source sees
only the semantic identity and affine lifecycle.

## Alternatives rejected

Omitting the fact was rejected because libc and applications consume the native
platform identity. An integer platform code was rejected because Linux exposes
a string pointer. Aliasing the executable-name string was rejected because the
objects have different meanings. Source-selected or runtime target selection
was rejected because provider choice precedes process-entry construction.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_PLATFORM` fact.
It does not retain the fact after qualification, add random or capability
facts, initialize TLS, construct a user context, enter user mode, execute
BusyBox, or add a syscall.
