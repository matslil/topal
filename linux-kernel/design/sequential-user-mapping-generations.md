# Sequential affine user-mapping generations

This record resolves `TK-DEC-041` as a systems-profile refinement. It permits
the exact frame returned by the bounded affine `probe-guard` lifecycle to enter
one second complete lifecycle under the same prepared address space. It changes
no core-language rule and exposes no address, descriptor, or target operation.

## Decision

The initial profile admits exactly two sequential generations:

```text
Prepared + Extent
  -> Mapped(1) -> Stored(1) -> Verified(1) -> Prepared + Extent
  -> Mapped(2) -> Stored(2) -> Verified(2) -> Prepared + Extent
```

Each successful map creates a fresh opaque mapping identity. Unmap consumes the
matching verified generation, revokes every access derived from it, completes
the target translation contract, and returns the exact original extent. The
second map consumes only that returned extent; it cannot observe, revive, or
alias the first generation.

Source releases the frame and prepares user execution only after the second
unmap. A stale mapping refinement from generation one cannot be stored,
loaded, unmapped, compared with, or substituted for generation two. The
prepared address-space identity and its original backing remain unchanged.

## Failure and ordering

The two generations are totally ordered. Generation two cannot begin until
generation one's revocation and completion have made its accesses impossible.
Release between generations, reuse before completion, identity reuse, skipped
or duplicated access, cross-generation substitution, and recoverable failure
with either generation published fail closed.

Publication failure before authority becomes observable returns no mapping.
Failure after publication begins remains fatal-only, with the same best closed
cleanup obligation as the first affine lifecycle.

## Alternatives

Keeping a single-use mapping would not test provider-state reset or stale-token
exclusion. Admitting two simultaneously live mappings would prematurely choose
alias, overlap, invalidation-scope, and multi-extent policy. A provider-owned
loop would hide freshness and ownership from the source checker. Two explicit
sequential generations prove the needed reusable boundary without making any
of those later choices.

## Cross-architecture contract

Fresh identity and complete revocation separate the generations on every
target. X86-64 may reuse a leaf only after clearing it and completing `INVLPG`;
AArch64 may reuse a descriptor after the required DSB/TLBI/ISB sequence; and
RISC-V may reuse a leaf after the required `SFENCE.VMA`. Descriptor bits,
addresses, ASIDs/PCIDs, barrier choice, and invalidation scope remain
provider-private.

The refinement remains bounded to one frame, one sealed `probe-guard` layout,
RW/NX rights, the fixed byte, and two sequential generations. It does not admit
simultaneous mappings, aliases, arbitrary layouts or permissions, replacement,
partial unmap, persistent image mappings, or an ELF loader.

## Initial x86-64 qualification

The generated provider reuses the existing guard leaf and provider-private
frame slot only after the first unmap clears both. Its state machine
distinguishes both publications, accesses, and revocations. The checked root
performs both generations explicitly; later guard-crossing user-copy faults
continue to prove that the leaf was absent before user execution.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.
