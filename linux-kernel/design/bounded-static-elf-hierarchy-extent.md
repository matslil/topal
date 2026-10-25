# Bounded static ELF hierarchy extent

This record resolves `TK-DEC-135` as the resource boundary immediately before
static-image address-space attachment. It is a systems-profile refinement only:
checked source owns an opaque affine frame extent, while the provider owns its
physical placement and future translation-table representation.

## Decision

While the exact `SealedStaticImageConstruction` remains live, the initial
x86-64 profile shall admit one typed
`static image construction reserve hierarchy` request with
`frame-count` five and `alignment-frames` one. Success shall produce one
indivisible opaque extent reserved solely for the static image's future upper
translation hierarchy. It shall not refine, consume, expose, or alias the
sealed payload or leaf-table ownership.

The provider shall select the complete five-frame extent:

- inside one normalized E820 RAM interval and below the first-GiB bootstrap
  ceiling;
- page aligned and overflow-free;
- disjoint from the 503-frame payload, one-frame leaf table, retained boot
  resource, active translation metadata, and every other live allocation; and
- after the leaf-table extent in the qualified deterministic selection.

Source shall publish exactly one
`TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK` witness while both the hierarchy
extent and sealed construction are live. This increment shall then release the
complete hierarchy extent before revoking the sealed construction. Failure is
an implicit fatal provider edge; partial release, splitting, escape, reuse, or recoverable completion
with either authority live is invalid.

## Resource and rollback boundary

The five frames are future translation metadata, not payload and not an
implicit address space. Allocation grants no byte access, mapping authority,
entry authority, activation token, or execution right. Whole release restores
the exact sealed-construction state, after which the existing leaf-table then
payload revocation order remains unchanged.

This separate allocation boundary keeps later hierarchy construction
transactional. A future attachment operation may consume both the sealed image
and hierarchy extent; a failed or deliberately detached operation can return
the hierarchy without inventing hidden provider ownership.

## Architecture pressure test

Explicit caller-owned affine metadata and disjoint placement are portable
policy. The count of five is an x86-64 profile fact: one PML4, one PDPT, one
low-memory page directory, one page directory for the qualified MMIO window,
and one high leaf table that keeps the bounded user stack disjoint from later
heap growth. AArch64 may need a different number of translation-table pages for its
granule and level choice; RISC-V may need a different Sv39/Sv48 hierarchy; a
capability system may use sealed mapping objects rather than page-table pages.
Those target representations and counts stay in their profiles. Future source
attachment shall name the opaque hierarchy extent and a sealed layout, never a
table level, entry index, permission bit, or physical address.

## Alternatives rejected

Hiding hierarchy allocation inside the provider was rejected because failure
and teardown would lack source-owned affine resources. Reusing the leaf-table
frame was rejected because upper and leaf metadata have distinct lifetimes and
sizes. Taking frames from the payload was rejected because it would invalidate
the exact 503-page plan. Allocating after revocation was rejected because the
resource would no longer be transactionally paired with the sealed image.
Allocating a general variable-sized hierarchy was rejected because only one
bounded x86-64 shape is qualified. Constructing or attaching the hierarchy in
the same increment was rejected so allocation, construction, and activation
retain separate failure and rollback boundaries.

## Deliberate bounds

This increment allocates, witnesses, and returns the future hierarchy extent.
It does not initialize translation metadata, link the final leaf table, attach
or activate an address space, retain the hierarchy beyond the witness, build a
stack or TLS image, publish the static entry, or execute the application.
