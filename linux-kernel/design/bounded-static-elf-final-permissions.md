# Bounded static ELF final permissions

This record resolves `TK-DEC-134` as the third inactive-construction
transition for the approved static ELF. It is a systems-profile refinement
only: checked source selects a sealed permission plan and consumes an opaque
populated construction, while the provider owns page-table representation,
target permission bits, publication barriers, and verification mechanics.

## Decision

`static image construction seal permissions` shall consume the exact live
`PopulatedStaticImageConstruction` and refine it into one affine
`SealedStaticImageConstruction`. The sealed request shall name only plan
`bounded-static-elf-wx-relro`; source shall not spell a virtual or physical
address, page ordinal, permission bit, page-table entry, cache instruction, or
translation-invalidation mechanism.

The operation shall:

- revalidate the exact 503-frame payload, disjoint metadata frame, populated
  bytes, 503 live initial RW/NX leaves, and nine absent entries;
- replace all 503 inactive leaves with the exact final page roles from the
  approved static-ELF page plan;
- ensure no leaf is simultaneously writable and executable;
- publish the complete inactive permission update as one provider transaction;
- read back all 503 leaves and verify their frame identity, order, final role,
  and the nine absent entries; and
- retain the sealed construction as the only authority able to revoke and
  return both extents.

The checked source shall publish exactly one
`TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK` witness while the sealed owner is
live. It shall then use the existing complete revocation protocol, release
returned metadata, reclaim the exact payload, and release that indivisible
extent. Failure is fatal and no partially sealed owner may escape.

## Exact bounded plan

The x86-64 provider shall realize these page-ordinal intervals relative to the
503-frame payload and virtual base `0x400000`:

| Ordinals | Pages | Final role | x86-64 leaf policy |
| --- | ---: | --- | --- |
| `[0, 1)` | 1 | headers | user RO/NX |
| `[1, 397)` | 396 | executable | user RO/X |
| `[397, 485)` | 88 | immutable data | user RO/NX |
| `[485, 493)` | 8 | GNU RELRO | user RO/NX |
| `[493, 503)` | 10 | writable data and BSS | user RW/NX |

The 97 RO/NX, 396 RO/X, and 10 RW/NX leaves shall retain their original
payload-frame identities and order. All nine remaining entries shall stay
absent. No active translation shall reference this leaf table during the
transition, so this increment requires no active-address-space invalidation.

## Affine lifecycle

Sealing consumes the populated identity; neither it nor any earlier
uninitialized or zeroed identity can be reused. Revocation in this profile
shall accept only the sealed refinement. Wrong owner or plan, wrong page role
or count, frame substitution, W+X, duplicate sealing, revocation before
sealing, missing or duplicate witness, or disposition with any construction
identity live is invalid. The fatal path remains the only rollback for a
provider failure during permission publication.

## Architecture pressure test

Role separation, W^X, immutable RELRO, complete readback, inactive atomic
publication, and affine refinement are portable policy. AArch64 providers may
encode AP, PXN, and UXN and perform data-cache clean plus instruction-cache
invalidation before future execution. RISC-V providers may encode R/W/X and
perform `fence.i`. Capability systems may seal executable and writable
capabilities separately. These mechanisms remain provider-owned and do not
alter the source transition.

## Alternatives rejected

Source-visible page loops were rejected because they expose target ordinals
and permission mechanics. Applying permissions during copy was rejected
because byte verification and W^X publication need separate rollback states.
Making every file-backed page executable was rejected because headers and
immutable data are not code. Leaving RELRO writable was rejected because the
approved ELF plan already identifies its immutable post-relocation interval.
Sampling leaves or checking only role counts was rejected because frame order
and every permission matter. Activating the leaf table in the same operation
was rejected because inactive construction and address-space attachment need
separate authority and failure boundaries.

## Deliberate bounds

This increment seals, verifies, revokes, and returns the inactive payload. It
does not retain the sealed owner across bootstrap phases, construct upper
translation levels, attach an address space, initialize TLS or a process
stack, publish an entry, or execute the image. General permission plans,
relocations, interpreters, demand paging, shared mappings, and writable RELRO
transitions remain out of scope.
