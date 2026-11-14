# Bounded static ELF prepared-process retention

This record resolves `TK-DEC-155` as the lifetime bridge between deterministic
static ELF construction and the later entropy pipeline. It is a systems-profile
refinement only: checked source transfers the complete random-slot-authorized
construction into one opaque affine prepared-process owner while providers and
the compiler retain its resource representation.

## Decision

`static image construction retain prepared process` shall require the exact
live `RandomSlotAuthorizedStackEntryStaticImageConstruction` and nested stack
backing from `TK-DEC-154`. It shall accept only that owner, that backing, value
`bounded-static-elf-initial-process`, and policy
`bounded-static-elf-pre-entropy-retention`. It shall atomically consume the
construction-facing owners and produce one affine
`RetainedStaticElfInitialProcess` containing the sealed image, inactive
hierarchy, entry authority, guarded RW/NX stack, complete deterministic process
table, and random-delivery-slot authority.

Retention shall not release, alias, expose, activate, or mutate any resource.
It shall close the construction allocator channel so later unrelated kernel
work can own its own resources without forging access to the prepared process.
Before success, the provider shall completely revalidate the same obligations
as random-slot authorization and bind the opaque resource tuple to the retained
owner. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK` observation.

This qualification increment shall then invoke
`static image construction restore prepared process` with the exact retained
owner and policy `bounded-static-elf-retention-qualification-rollback`.
Restoration shall prove no retained resource or byte changed, reopen only the
exact construction-facing random-slot-authorized owner and nested backing, and
emit exactly one `TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK`
observation before random-slot revocation.

Neither operation shall expose resource addresses, stack representation,
registers, retention slots, page tables, or bytes. It shall not consume entropy,
publish `AT_RANDOM`, activate the address space, construct a user context, or
enter the image.

## Exact x86-64 realization

The lowering shall retain the provider-private payload, metadata, hierarchy,
leaf, entry, and stack-backing identities in compiler-controlled root state
that survives ordinary provider calls. It shall clear construction-facing
allocator state without returning any frame to the allocator. Restoration
shall require the exact retained state and restore the exact one-frame nested
stack-backing obligation.

Retention and restoration shall each completely validate the inactive root,
all hierarchy and leaf entries, resident ELF metadata, deterministic process
table, both future random-object qwords, fixed strings, guard, and all otherwise
zero stack bytes. They perform no store, fence, active-address invalidation,
frame release, or address-space activation.

## Affine lifecycle and rollback

Only the exact random-slot-authorized construction may become the retained
initial process. While retained, construction mutation, deterministic-fact or
slot revocation, backing release, address-space activation, teardown, duplicate
retention, or bootstrap completion is invalid. Later refinements may perform
unrelated kernel work or consume the retained owner together with opaque entropy
output; this increment permits only exact restoration. Any identity, content,
translation, lifetime, or readback failure is fatal.

## Architecture pressure test

The portable object is a retained prepared Linux process, not an x86 register
set or page-table root. AArch64 and RISC-V providers may keep the affine
resource tuple in different compiler-owned storage while preserving the same
non-aliasing, inactivity, survival, and restoration obligations. Capability
systems can represent the retained owner directly without converting
capabilities to integers.

## Alternatives rejected

Leaving construction resources implicitly live across arbitrary operations was
rejected because it obscures which owner may mutate or release them. Returning
frames to the allocator was rejected because the prepared process still owns
their contents and mappings. Exposing a tuple of addresses was rejected because
it permits forgery and binds source to one architecture. Moving entropy
acquisition into the loader was rejected because it would couple portable
driver and cryptographic policy to image construction.

## Deliberate bounds

This increment retains, witnesses, restores, and verifies one prepared static
initial process. It does not yet keep that owner across another source
operation, consume entropy, publish `AT_RANDOM`, initialize TLS, construct a
user context, enter user mode, execute BusyBox, or add a syscall.
