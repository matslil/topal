# Bounded static ELF address-space attachment

This record resolves `TK-DEC-136` as the first address-space ownership
transition for the approved static ELF. It is a systems-profile refinement
only: checked source names opaque sealed image and hierarchy owners, while the
provider owns translation levels, entry representation, bootstrap mappings,
publication barriers, and invalidation mechanics.

## Decision

`static image construction attach address space` shall consume the exact live
`SealedStaticImageConstruction` and four-frame hierarchy extent and produce
one affine `AttachedStaticImageConstruction`. It shall accept only layout
`bounded-static-elf-inactive-address-space`. The operation shall:

- revalidate the 503-frame payload, final leaf table, four-frame hierarchy,
  their exact adjacency/disjointness, and the complete final leaf roles;
- prove that the hierarchy root is not the active translation root;
- zero and read back all four hierarchy frames before construction;
- construct the complete qualified supervisor bootstrap/MMIO hierarchy and
  link the existing leaf table at the static image's user virtual interval;
- atomically publish and read back every present and absent hierarchy entry;
  and
- preserve payload bytes and every final leaf without granting entry or
  activation authority.

Source shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK`
witness while the attached owner is live.

`static image construction detach address space` shall consume only that exact
attached owner. It shall revalidate the whole inactive hierarchy and final
leaf table, again prove the root inactive, clear all four hierarchy frames,
publish the removal, and verify every hierarchy byte zero before producing one
affine `DetachedStaticImageConstruction`. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK` witness.

The detached owner shall permit `static image construction reclaim` with
resource `hierarchy` exactly once. The complete returned four-frame extent
shall be released before the existing metadata then payload revocation and
release sequence, which shall name the detached owner. Failure at any attach
or detach check is fatal; no partially constructed hierarchy or derived owner
may escape.

## Exact x86-64 inactive hierarchy

The four adjacent hierarchy frames are, in order, PML4, PDPT, low-memory page
directory, and qualified-MMIO page directory. The provider shall construct:

- PML4 entry zero referencing the PDPT with present/write/user traversal;
- PDPT entry zero referencing the low-memory directory with
  present/write/user traversal;
- PDPT entry three referencing the MMIO directory with supervisor
  present/write traversal;
- 512 low-memory 2 MiB supervisor bootstrap leaves matching the approved
  identity policy, except directory entry two shall reference the existing
  static leaf table with present/write/user traversal;
- qualified HPET and local-APIC 2 MiB mappings at MMIO directory entries 502
  and 503; and
- every other PML4, PDPT, and MMIO-directory entry absent.

The linked static leaf table remains exactly 97 user RO/NX, 396 user RO/X, ten
user RW/NX leaves, then nine absent entries. Attachment changes no payload
frame, leaf role, or byte. The inactive root is not an activation token.

## Affine lifecycle and rollback

Hierarchy reservation refines the sealed ownership state without changing the
image. Attachment consumes both live authorities. Detachment consumes the
attached authority and keeps all resources inside the detached rollback owner.
Hierarchy reclaim and whole release restore a detached sealed-image owner;
only then may leaf metadata and payload be revoked and returned. Duplicate,
substituted, stale, out-of-order, split, or live-at-disposition authority is
invalid.

## Architecture pressure test

Inactive construction, complete readback, explicit activation separation,
affine detach, and hierarchy zero-before-return are portable policy. AArch64
providers may use different levels and table attributes and must perform
required cache maintenance. RISC-V providers may construct Sv39/Sv48 roots and
use the required fences. Capability systems may construct sealed address-space
objects instead of integer tables. Source names only the opaque owners and
sealed layout; table levels, indices, flags, roots, cache instructions, and
fences remain provider-private.

## Alternatives rejected

Activating during attachment was rejected because construction failure must be
recoverable without switching execution roots. Copying the active hierarchy
was rejected because it would import mutable and unreviewed mappings. Leaving
allocator bytes outside explicitly written entries was rejected because stale
translations could become authority. Rebuilding the final leaf table was
rejected because attachment must preserve the already verified image.
Returning hierarchy frames without zeroing and readback was rejected because
provider access and stale mapping state would remain ambiguous. Publishing an
entry together with attachment was rejected because address-space ownership
and execution authority require separate review boundaries.

## Deliberate bounds

This increment constructs, verifies, detaches, zeros, and returns one inactive
address space. It does not activate the root, initialize TLS, allocate or map a
process stack, publish an instruction pointer, enter user mode, retain the
address space, or run BusyBox. Those remain later bounded transitions.
