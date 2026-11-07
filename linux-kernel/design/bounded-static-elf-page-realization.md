# Bounded static ELF page realization

This record resolves `TK-DEC-126` as the checked bridge from the immutable
static-ELF load plan to a later affine frame and mapping lifecycle. It refines
compiler-owned planning only; it does not allocate frames, publish leaves, or
make the pinned BusyBox runtime-selectable.

## Decision

The x86-64 compiler shall derive one immutable page-realization plan only from
a successful bounded static-ELF load plan. The current qualification envelope
shall require the four page-rounded load intervals to be exactly adjacent, so
every page of the combined span has one declared role and no undeclared gap is
accidentally mapped.

The realization plan shall name:

- one future affine physical-frame request whose count equals the complete
  load span and whose alignment is one base frame;
- one ordered page record for every virtual page, mapping its zero-based frame
  ordinal to that virtual page;
- an initial RW/NX construction permission for every page;
- an exact entry-stage permission of RO/NX, RO/X, or RW/NX for every page;
- RELRO as staged RW/NX pages inside the original writable role until the
  exact checked post-relocation protection transition;
- one whole-frame zero operation preceding four exact file-copy operations;
  and
- complete copy, zero, permission, entry, TLS, and RELRO facts inherited from
  the validated load plan without reparsing ELF bytes.

The pinned shape therefore requests one indivisible 503-frame extent and emits
503 ordered page records: 396 RO/X pages, 89 RO/NX pages, and 18 RW/NX pages
including eight staged RELRO pages. Zeroing the complete extent before segment
copies constructs all padding and BSS deterministically. Entry-stage permission
publication shall remain a later affine operation; no page may be executable
during construction and no entry-stage or final page may combine W and X.
`TK-DEC-193` refines the later runtime lifecycle so the eight RELRO pages become
RO/NX only at the exact checked protection request.

## Architecture pressure test

Ordered page roles, zero-before-copy construction, staged RELRO, and W^X are
portable policy. A single physically contiguous 503-frame extent and one
x86-64 512-leaf table are qualification bounds, not general application-loader
requirements. AArch64 or RISC-V may use different table levels and executable-
coherence operations while preserving the same page-role plan. Capability
architectures must derive bounded mapping capabilities from the affine extent
instead of turning frame ordinals or ELF integers into ambient authority.

## Alternatives rejected

Mapping the complete span with one final permission was rejected because it
would violate W^X or make data executable. Mapping gaps as readable zero pages
was rejected because the input did not declare those pages. Copying segments
before complete zeroing was rejected because page padding and BSS could expose
allocator residue. Allocating four independent extents was rejected for this
first gate because it complicates rollback and teardown before a general
mapping collection exists. Re-parsing ELF during realization was rejected
because it would create two sources of truth for already validated facts.

## Deliberate bounds

This increment derives and tests the page-realization plan only. It does not
extend the source-level frame request beyond the existing executable profile,
allocate 503 frames, install a leaf table, copy the third-party binary, apply
permissions, initialize process TLS, build an application stack, enter BusyBox,
or add startup syscalls. Sparse images, demand paging, copy-on-write, PIE,
relocations, shared objects, and images spanning more than one x86-64 leaf
table remain outside the envelope.
