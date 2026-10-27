# Bounded static ELF `AT_EXECFN` fact

This record resolves `TK-DEC-147` as the executable-name refinement of the
guarded static ELF stack. It is a systems-profile refinement only: checked
source composes the program-header-qualified block with its existing sealed
argument identity, while providers own the auxiliary-vector tag, in-stack
pointer, address, word, placement, alignment, and mutation representation.

## Decision

`static image construction add stack executable name` shall require the exact
live `ProgramHeaderStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-146`. It shall accept only that owner, that backing identity, value
`initial-executable-name`, and policy `bounded-linux-auxv-before-null`. It shall
produce one affine `ExecutableNameStackEntryStaticImageConstruction` retaining
all prior facts while adding one `AT_EXECFN` fact after the coherent program-
header group and before `AT_NULL`.

The fact shall designate the exact existing NUL-terminated `busybox` string
already used by `argv[0]`; it shall not create a second name or accept a source
pointer. Before success, the provider shall revalidate the complete program-
header-qualified block, same-leaf string and pointer relations, resident ELF
metadata, and every inactive-translation obligation. It shall clear the
complete backing, reconstruct the target-aligned block, perform target-required
visibility operations, and read back every word, pointer relation, string
byte, terminator, alias relation, and otherwise-zero byte. Source shall emit
exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK` observation.

Source shall then invoke `static image construction remove stack executable
name` with the exact owner, backing, and policy
`bounded-stack-executable-name-qualification-rollback`. Removal shall validate
the complete extended block, reconstruct and completely validate the exact
program-header-qualified block, and reproduce its affine owner. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK` observation
before program-header removal.

Neither operation shall expose the tag, pointer, string address, stack pointer,
virtual or physical address, word encoding, offset, alignment, or write
mechanism. It shall not add credential or platform facts, initialize TLS,
activate the hierarchy, construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall encode native `AT_EXECFN` tag 31 with value `0x7fffe0`, the
exact existing `busybox` string address, after `AT_PHNUM=(5,11)` and before
`AT_NULL`. It shall prove equality with `argv[0]`, same-leaf containment, all
eight `busybox\0` bytes, and separation from the `true` string.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned. The exact RW/NX stack leaf, guard,
final image leaves, inactive hierarchy, and bootstrap root remain unchanged.
No active-address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact program-header-qualified block may gain the executable-name
fact. While live, program-header or prior-fact removal, ordinary entry clearing,
leaf revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact removal may restore the
program-header-qualified owner. Any content, alias, bounds, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable fact is the existing initial executable-name object and its
identity with `argv[0]`, not an integer pointer. AArch64 and RISC-V providers
use their Linux process-entry encoding. Capability systems derive a bounded
read-only string capability without fabricating authority. Source sees only
the semantic relationship and affine lifecycle.

## Alternatives rejected

Passing a pointer or address through checked source was rejected as forgeable.
Copying a second string was rejected because it weakens alias identity and
creates unnecessary lifetime. Allowing a name unrelated to `argv[0]` was
rejected because this bounded launch profile already selected one sealed
executable identity. Combining credentials, platform, clock, secure-mode, and
random facts was rejected because their authorities and evidence differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one `AT_EXECFN` fact. It
does not retain the fact after qualification, add credential, platform, clock,
secure-mode, or random facts, initialize TLS, construct a user context, enter
user mode, execute BusyBox, or add a syscall.
