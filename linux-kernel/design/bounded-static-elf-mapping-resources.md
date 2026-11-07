# Bounded static ELF mapping resources

This record resolves `TK-DEC-128` as the affine resource boundary between the
qualified 503-page static-ELF plan and later leaf publication. It refines only
the systems profile. It adds no core-language element and exposes neither a
physical address nor an architecture page-table representation to source.

## Decision

The initial x86-64 qualification profile shall keep the exact 503-frame static
ELF payload extent live while requesting one additional unit-aligned physical
frame for translation metadata. The two successful allocations create two
distinct affine owners:

- `static-elf-frames` owns all 503 payload frames as one indivisible extent;
- `static-elf-leaf-table-frame` owns exactly one metadata frame; and
- the allocator/provider proves the complete extents are disjoint from each
  other, the boot resource, retained reservations, and non-RAM ranges.

The source sequence is closed and ordered. It allocates the payload first,
publishes the existing payload witness, allocates metadata while the payload
remains live, publishes exactly one
`TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK` witness, releases the metadata
extent, and finally releases the payload extent. A failure of either request
enters the existing fatal disposition. No success path may retain only one
member, reverse the releases, or combine the disjoint extents into a fictitious
contiguous owner.

The provider allocation interface receives an opaque exclusion interval from
the checked root. For the nested metadata request that interval is the entire
503-frame payload. Checked interval construction, alignment, non-overlap, E820
containment, boot-resource exclusion, and the first-GiB ceiling all precede
ownership publication. The exclusion is provider evidence and never becomes a
Topal integer, pointer, or comparable identity.

## Abstraction boundary

Portable source asks for application-image payload and translation-metadata
resources; it does not name PML4, PDPT, PD, PT, PTE, register encodings, or a
physical address. The x86-64 provider may later consume the one-frame metadata
extent as the single 512-entry leaf table covering the checked
`0x0040_0000..0x005f_7000` window. This increment grants no such consumption or
publication authority: both extents remain source-owned and are returned
unchanged.

The pair deliberately remains two affine resources. The payload frame count is
derived from application pages, whereas translation metadata depends on the
target translation format. Keeping them separate allows a future provider to
request a different checked metadata shape without changing the application
page plan or inventing hidden allocation.

## Architecture pressure test

All providers must preserve explicit fallibility, distinct affine identity,
complete-interval disjointness, and exact rollback. The x86-64 gate qualifies
one 4 KiB metadata frame because all 503 leaves share one two-MiB leaf window.
An AArch64 provider may require several table levels or a different granule;
RISC-V may require a different checked hierarchy; a capability machine may
retain bounded capabilities instead of integer intervals. Those target facts
belong to provider-specific plans below the common resource roles.

## Alternatives rejected

Taking a frame from the payload extent was rejected because it would falsify
the one-frame-per-application-page plan and require unsupported splitting.
Hiding the table allocation inside the provider was rejected because rollback
and memory accounting would disappear below checked source. Treating the two
extents as contiguous was rejected because the allocator promises only
disjointness. Reusing an existing probe table was rejected because its virtual
window and lifetime differ. Exposing page-table entry construction in source
was rejected as an x86 representation leak.

## Deliberate bounds

This increment proves simultaneous ownership and exact unwind only. It does
not transfer either extent into a static-image construction owner, zero or copy
payload bytes, install leaf entries, publish final W^X permissions, initialize
TLS or stack state, enter BusyBox, or provide general multiple-live-extent
allocation. Only the exact nested `(503, 1)` then `(1, 1)` sequence is admitted.
