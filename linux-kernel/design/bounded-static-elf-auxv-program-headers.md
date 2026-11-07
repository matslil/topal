# Bounded static ELF program-header facts

This record resolves `TK-DEC-146` as the resident-program-header refinement of
the guarded static ELF stack. It is a systems-profile refinement only: checked
source composes the existing executable-entry-qualified block with opaque
resident ELF metadata authority, while providers own tags, addresses, sizes,
counts, words, placement, alignment, and mutation representation.

## Decision

`static image construction add stack program headers` shall require the exact
live `ExecutableEntryStackEntryStaticImageConstruction` and nested backing
from `TK-DEC-145`. It shall accept only that owner, that backing identity,
value `resident-static-elf-program-headers`, and policy
`bounded-linux-auxv-before-null`. It shall produce one affine
`ProgramHeaderStackEntryStaticImageConstruction` retaining every prior fact
while adding the coherent `AT_PHDR`, `AT_PHENT`, and `AT_PHNUM` group after
`AT_ENTRY` and before `AT_NULL`.

Before success, the provider shall revalidate the complete entry-qualified
block, resident static ELF header and complete program-header table, exact
header leaf, and every inactive-translation obligation. It shall derive the
resident table identity without accepting a source address, clear the complete
backing, reconstruct the target-aligned block, perform target-required
visibility operations, and read back every word, pointer relation, string
byte, terminator, header/table relation, and otherwise-zero byte. Source shall
emit exactly one `TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK` observation.

Source shall then invoke `static image construction remove stack program
headers` with the exact owner, backing, and policy
`bounded-stack-program-headers-qualification-rollback`. Removal shall validate
the complete extended block, reconstruct and completely validate the exact
entry-qualified block from `TK-DEC-145`, and reproduce its affine owner. Source
shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK` observation before
entry-fact removal.

Neither operation shall expose an auxiliary-vector tag, program-header
address, entry size, entry count, stack pointer, virtual or physical address,
word encoding, offset, alignment, or write mechanism. It shall not add
credentials or platform facts, initialize TLS, activate the hierarchy,
construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall derive resident `AT_PHDR=(3,0x400040)`, encode
`AT_PHENT=(4,56)` and `AT_PHNUM=(5,11)`, and place the group after
`AT_ENTRY=(9,0x4a6580)` and before `AT_NULL`. It shall prove that the complete
eleven-row table begins at ELF `e_phoff=64`, has the header-declared entry size
and count, lies in the exact immutable header leaf, and is the table already
validated by the static ELF load plan.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
The entry pointer remains 16-byte aligned; the `busybox` and `echo` strings and
all prior facts remain unchanged. The exact RW/NX stack leaf, guard, final
image leaves, inactive hierarchy, and bootstrap root remain unchanged. No
active-address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact entry-qualified block with resident metadata authority may gain
the program-header group. While live, entry-fact removal, page-size removal,
ordinary entry clearing, leaf revocation, backing release, entry-authority
revocation, activation, teardown, duplicate addition, or recoverable
disposition is invalid. Only exact group removal may restore the entry-
qualified owner. Any content, identity, bounds, ordering, translation, or
readback failure is fatal.

## Architecture pressure test

The portable fact is one coherent reference to the validated resident program-
header table and its shape, not three unrelated integers. AArch64 and RISC-V
providers use their Linux ELF conventions. Capability systems derive a bounded
read-only table capability without fabricating or widening authority. Source
sees only the semantic relationship and affine lifecycle.

## Alternatives rejected

Passing three source integers was rejected because it would permit unrelated
or forged metadata. Deriving only the address from the ELF header was rejected
because entry size and count must agree with the validated complete table.
Adding the facts separately was rejected because Linux consumers interpret
them as one identity-and-shape group and partial rollback creates invalid
intermediate states. Combining credentials, platform, clock, secure-mode, and
random facts was rejected because their authorities and evidence differ.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one coherent program-
header fact group. It does not retain the group after qualification, add
credential, platform, clock, secure-mode, or random facts, initialize TLS,
construct a user context, enter user mode, execute BusyBox, or add a syscall.
