# Bounded static ELF load-metadata facts

This record resolves `TK-DEC-153` as the interpreter-base and ELF-flags
refinement of the guarded static ELF stack. It is a systems-profile refinement
only: checked source composes the hardware-capability-qualified block with one
sealed static-executable metadata identity, while providers own native values,
auxiliary-vector tags, placement, and mutation representation.

## Decision

`static image construction add stack load metadata` shall require the exact
live `HardwareCapabilitiesStackEntryStaticImageConstruction` and nested backing
from `TK-DEC-152`. It shall accept only that owner, that backing identity,
value `static-executable-load-metadata`, and policy
`bounded-linux-auxv-before-null`. It shall produce one affine
`LoadMetadataStackEntryStaticImageConstruction` retaining every prior fact
while atomically adding coherent `AT_BASE` and `AT_FLAGS` facts after
`AT_HWCAP2` and before `AT_NULL`.

The facts shall report the already validated absence of a program interpreter
and the accepted executable's already validated architecture flags. They shall
not select a loader, relocation policy, personality, or executable. Source
shall neither choose nor observe either value, tag, or encoding. Before
success, the provider shall revalidate the complete capability-qualified block
and every inactive-translation obligation, clear the complete backing,
reconstruct the target-aligned block, perform target-required visibility
operations, and read back every word, pointer relation, string byte, metadata
fact, terminator, and otherwise-zero byte. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK` observation.

Source shall then invoke `static image construction remove stack load metadata`
with the exact owner, backing, and policy
`bounded-stack-load-metadata-qualification-rollback`. Removal shall validate
the complete extended block, reconstruct and completely validate the exact
hardware-capability-qualified block, and reproduce its affine owner. Source
shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK` observation before
hardware-capability removal.

Neither operation shall expose a loader base, ELF flags, tag, stack address,
word encoding, offset, alignment, header field, or write mechanism. It shall
not add random data or a vDSO, initialize TLS, activate the hierarchy,
construct a user context, or enter the image.

## Exact x86-64 realization

For the accepted static `ET_EXEC` image, the provider shall encode native
`AT_BASE=(7,0)` and `AT_FLAGS=(8,0)` after `AT_HWCAP2=(26,0)` and before
`AT_NULL`. The zero base follows from the validated absence of `PT_INTERP`; the
zero flags follow from the accepted `EM_X86_64` ELF header. The nonzero tags
make both facts distinct from the terminating zero tag despite their zero
values.

Addition and removal shall each clear all 512 backing qwords before
reconstruction, execute one store fence, and completely read back the result.
Addition shall enlarge the aligned entry block downward without moving or
overlapping the platform or executable-name strings. The exact capability
masks, platform string, earlier facts, RW/NX stack leaf, guard, final image
leaves, inactive hierarchy, and bootstrap root remain unchanged. No active-
address invalidation is permitted.

## Affine lifecycle and rollback

Only the exact capability-qualified block may gain the load-metadata group.
While live, capability or prior-fact removal, ordinary entry clearing, leaf
revocation, backing release, activation, teardown, duplicate addition, or
recoverable disposition is invalid. Only exact group removal may restore the
capability-qualified owner. Any content, identity, value, ordering,
translation, or readback failure is fatal.

## Architecture pressure test

The portable facts are Linux process-entry loader-base and ELF-flags results,
not x86 header offsets or loader mechanics. AArch64 and RISC-V providers derive
values from their validated image and interpreter choices. Capability systems
gain no address authority from a zero or nonzero base representation. Source
sees only the semantic identity and affine lifecycle.

## Alternatives rejected

Omitting the facts was rejected because Linux publishes them even for a static
image. Treating zero values as absent was rejected because `AT_NULL` is
identified by its tag, not its value. Source-selected values were rejected
because validation already determines both results. Reusing the executable
entry as `AT_BASE` was rejected because the latter describes an interpreter,
which this profile forbids.

## Deliberate bounds

This increment adds, witnesses, removes, and verifies one coherent
`AT_BASE`/`AT_FLAGS` group. It does not retain the facts after qualification,
admit an interpreter or dynamic loading, add random data or a vDSO, initialize
TLS, construct a user context, enter user mode, execute BusyBox, or add a
syscall.
