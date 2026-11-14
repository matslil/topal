# Affine user-mapping permission tightening

This record resolves `TK-DEC-042` as a systems-profile refinement. It inserts
an explicit permission-tightening boundary between population and verification
of each bounded user mapping. It changes no core-language rule and exposes no
address, descriptor bit, barrier, or invalidation instruction.

## Decision

`user mapping protect read only` consumes the exact stored RW/NX mapping and
returns an opaque read-only/NX refinement. Publication of that refinement is
observable only after writable authority has been revoked and the target's
translation-completion contract has finished. The subsequent fixed-byte load
consumes the protected refinement and returns the verified mapping accepted by
unmap.

```text
Mapped(RW/NX) -> Stored(RW/NX) -> Protected(RO/NX)
  -> Verified(RO/NX) -> Unmapped
```

Both admitted sequential generations perform this transition. Source cannot
store through the protected refinement, load through the stale stored
refinement, unmap before verification, reuse any consumed refinement, or enter
user execution before both protected mappings are revoked and their frame is
released.

## Failure and ordering

Protection must preserve mapping identity, address-space identity, backing
identity, placement, memory kind, user accessibility, non-executable policy,
and contents. It may only remove write authority. Success requires target
completion sufficient to prevent a later access through stale writable
translation state.

Failure before the permission update leaves the stored mapping live. Failure
after the update begins is fatal-only in this bounded profile; the provider
must make its best closed cleanup transition before reporting failure. A
provider must not report recoverable success with ambiguous or writable
authority.

## Alternatives

Leaving the qualification RW/NX would not establish the permission boundary
needed by immutable image data and eventual executable mappings. Folding the
change into load would hide write revocation from affine checking. Allowing
arbitrary source-selected permissions would prematurely choose executable,
device-memory, alias, and capability-amplification policy. One explicit
RW-to-RO transition proves monotonic authority reduction without those choices.

## Cross-architecture contract

The common operation removes write authority and completes stale-translation
revocation. X86-64 may replace the leaf permissions, fence publication, and
use `INVLPG`; AArch64 may update a stage-1 descriptor and use the required
DSB/TLBI/ISB sequence; RISC-V may update the leaf PTE and use the required
`SFENCE.VMA`. Exact bits, barriers, invalidation scope, and address-space tags
remain provider-private.

The refinement remains bounded to the sealed `probe-guard` layout, one frame,
the fixed byte, read-only/NX output, and two sequential generations. It does
not admit permission widening, executable authority, arbitrary rights,
simultaneous mappings, aliases, persistent image mappings, or an ELF loader.

## Initial x86-64 qualification

The generated provider clears only the writable permission of the active guard
leaf, preserves present/user/NX and the physical identity, fences publication,
invalidates the sealed location, and advances provider-private generation
state. Structural inspection and physical execution prove the transition in
both generations before load and unmap.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.
