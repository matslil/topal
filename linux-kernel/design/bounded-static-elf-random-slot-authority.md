# Bounded static ELF random-delivery slot authority

This record resolves `TK-DEC-154` as the precondition for adding secret
`AT_RANDOM` material to the guarded static ELF stack. It is a systems-profile
refinement only: checked source composes the load-metadata-qualified stack with
one sealed random-delivery-slot policy, while providers own native placement,
pointer encoding, and stack representation.

## Decision

`static image construction authorize stack random slot` shall require the exact
live `LoadMetadataStackEntryStaticImageConstruction` and nested backing from
`TK-DEC-153`. It shall accept only that owner, that backing, value
`initial-auxv-random-object`, extent 16, and policy
`bounded-linux-auxv-random-before-null`. It shall produce one affine
`RandomSlotAuthorizedStackEntryStaticImageConstruction` retaining every prior
fact while authorizing one later atomic delivery of native `AT_RANDOM` tag 25
and one distinct aligned 16-byte object.

Authorization shall not write a tag, pointer, random byte, or placeholder. The
provider shall instead revalidate the complete load-metadata-qualified block,
prove that the future auxiliary entry and object can be placed inside the
existing RW/NX stack leaf without overlapping the vector, argument pointers,
platform string, executable-name string, guard, or another mapping, and prove
that the entire future object is currently zero. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK` observation after success.

Source shall then invoke `static image construction revoke stack random slot`
with the exact owner, backing, and policy
`bounded-stack-random-slot-qualification-rollback`. Revocation shall revalidate
the unchanged complete block and zero destination, reproduce the exact
load-metadata-qualified owner, and emit exactly one
`TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK` observation before load-
metadata removal.

Neither operation shall expose an auxiliary tag, object address, stack address,
offset, alignment, byte, word encoding, or write mechanism. It shall not
consume entropy, add `AT_RANDOM`, make the stack entry-ready, initialize TLS,
activate the hierarchy, construct a user context, or enter the image.

## Exact x86-64 realization

The provider shall reserve one naturally aligned 16-byte object immediately
below the fixed platform and executable-name strings and enough downward space
for one future `(25,pointer)` auxiliary entry before `AT_NULL`. The currently
qualified deterministic entry block remains byte-for-byte unchanged.

Authorization and revocation shall each completely read back all 512 backing
qwords and validate every current tag/value pair, pointer relation, string byte,
terminator, and otherwise-zero byte. They shall also validate both future object
qwords as zero and prove the future enlarged aligned block remains inside the
single stack page. They perform no store and require no active-address
invalidation. The exact capability masks, load metadata, RW/NX stack leaf,
guard, final image leaves, inactive hierarchy, and bootstrap root remain
unchanged.

## Affine lifecycle and rollback

Only the exact load-metadata-qualified block may gain random-slot authority.
While authority is live, deterministic-fact removal, ordinary entry clearing,
leaf revocation, backing release, activation, teardown, duplicate
authorization, or recoverable disposition is invalid. A later refinement may
consume the authority and opaque entropy output exactly once; this increment
permits only exact revocation back to the load-metadata-qualified owner. Any
content, identity, extent, placement, translation, zero-state, or readback
failure is fatal.

## Architecture pressure test

The portable authority means that one Linux `AT_RANDOM` entry and one distinct
16-byte in-stack object have a proved destination, not that an architecture
shares x86 stack offsets or page tables. AArch64 and RISC-V providers choose
their own aligned placement and native pointer encoding while proving the same
extent, non-overlap, zero-state, and guarded-stack obligations. Capability
systems gain no address authority from the reservation. Source sees only the
semantic identity, extent, sealed policy, and affine lifecycle.

## Alternatives rejected

Writing a zero `AT_RANDOM` placeholder was rejected because the pointer would
publish an initialized-looking but non-random ABI object. Reserving a source-
visible address was rejected because it would permit pointer forgery and bind
portable policy to one layout. Combining reservation with entropy extraction
was rejected because construction currently precedes the entropy pipeline and
because placement proof and secret delivery have separate failure and review
boundaries. Reusing either string object was rejected because Linux requires a
distinct 16-byte random object.

## Deliberate bounds

This increment authorizes, witnesses, revokes, and verifies one future
16-byte initial-process random object and one future `AT_RANDOM` entry. It does
not mutate the deterministic stack, consume or expose entropy, retain authority
past the qualification rollback, initialize TLS, construct a user context,
enter user mode, execute BusyBox, or add a syscall.
