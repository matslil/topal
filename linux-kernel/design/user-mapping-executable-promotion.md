# Affine user-mapping executable promotion

This record resolves `TK-DEC-043` as a systems-profile refinement. It adds an
explicit executable-authority boundary after read-only content verification.
It changes no core-language rule and exposes no address, descriptor bit,
barrier, cache operation, or invalidation instruction.

## Decision

`user mapping permit execute` consumes the exact verified RO/NX mapping and
returns an opaque executable RO/X refinement. Observable success requires
non-executable authority to be replaced without restoring write authority and
the target's instruction and translation completion contract to finish.
Unmap consumes the executable refinement.

```text
Mapped(RW/NX) -> Stored(RW/NX) -> Protected(RO/NX)
  -> Verified(RO/NX) -> Executable(RO/X) -> Unmapped
```

Both sequential generations perform the transition. Source cannot promote a
writable or unverified mapping, store or load through executable authority,
reuse a consumed refinement, or enter user execution before both executable
mappings are revoked and their frame is released.

## Failure and ordering

Promotion preserves mapping, address-space, backing, placement, memory kind,
user accessibility, contents, and read-only authority. It may only replace NX
with execute authority. Write and execute authority must never coexist.

Failure before publication leaves the verified RO/NX mapping live. Failure
after publication begins is fatal-only in this bounded profile; the provider
must make its best closed cleanup transition before reporting failure. A
provider must not report recoverable success with ambiguous, writable, or
stale instruction authority.

## Alternatives

Making the initial map executable would violate W^X during population.
Folding executable promotion into verification would hide authority creation
from affine checking. Arbitrary source-selected rights would prematurely
choose general loader, alias, capability-amplification, and memory-type policy.
One explicit RO/NX-to-RO/X transition proves the monotonic construction
sequence while keeping target mechanics private.

## Cross-architecture contract

The common operation grants execute only to already verified read-only
contents and completes stale translation and instruction-observation cleanup.
X86-64 may clear the leaf NX bit, fence publication, and use `INVLPG`; AArch64
may update execute-never attributes and use the required cache, DSB, TLBI, and
ISB sequence; RISC-V may update the executable PTE authority, perform required
`FENCE.I`, and use `SFENCE.VMA`. Exact bits, barriers, cache scope,
invalidation scope, and address-space tags remain provider-private.

The refinement remains bounded to the sealed `probe-guard` layout, one frame,
the fixed byte, RO/X output, and two sequential generations. It does not admit
permission relaxation back to writable, arbitrary rights, simultaneous
mappings, aliases, persistent image mappings, or an ELF loader.

## Initial x86-64 qualification

The generated provider clears only NX on the verified read-only guard leaf,
preserves present/user/read-only authority and physical identity, fences
publication, invalidates the sealed location, and advances provider-private
generation state. Structural inspection and physical execution prove the
transition in both generations before unmap.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.
