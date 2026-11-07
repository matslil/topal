# Bounded affine user-mapping lifecycle

This record resolves `TK-DEC-040` as a systems-profile refinement. It splits
the already qualified `probe-guard` round trip into source-visible affine
map, store, load, and unmap transitions without exposing an address,
page-table representation, or target instruction and without changing the
core language.

## Decision

One prepared user address space may consume its exact disjoint one-frame
extent into a fresh affine `UserMapping`. The mapping is fixed to the sealed
`probe-guard` layout and RW/NX user rights. A store consumes the live mapping
and returns a stored refinement; a load consumes that refinement, confirms the
fixed qualification byte, and returns a verified refinement. Unmap consumes
the verified mapping and returns the exact original extent only after every
derived access has been revoked and target translation completion has
finished.

```topal
mapped-scratch is context user address space map (
  address-space is user-space,
  backing is scratch-frame,
  layout is probe-guard
)
stored-scratch is context user mapping store byte mapped-scratch
verified-scratch is context user mapping load byte stored-scratch
returned-scratch is context user address space unmap (
  address-space is user-space,
  mapping is verified-scratch
)
context frames release returned-scratch
context console write "TOPAL_KERNEL_USER_MAPPING_OK"
```

The prepared address-space identity and its original backing remain unchanged.
The additional frame is owned by the mapping from successful map publication
until successful unmap completion. Source cannot release the frame, bind the
address space into user execution, duplicate the mapping, skip a refinement,
or use a consumed refinement.

## Failure and ordering

Map success is observable only after publication and target completion. Store
success is observable only after the fixed byte is written. Load success is
observable only after the same byte is read and compared. Unmap success is
observable only after revocation and target completion make every derived
access impossible.

Failure before publication returns no mapping. Failure after publication
begins is fatal-only in this bounded profile; the provider must make its best
closed cleanup transition before reporting failure. Wrong identities,
overlap, another layout or permission set, store before map, load before
store, unmap before verification, duplicate operations, stale access, and a
recoverable disposition with a live mapping fail closed.

## Cross-architecture contract

The common lifecycle is `Prepared + Extent -> Mapped -> Stored -> Verified ->
Prepared + Extent`, with publication and revocation completion at the two
ownership boundaries. X86-64 may use an RW/NX leaf PTE and `INVLPG`; AArch64
may use a stage-1 descriptor with DSB/TLBI/ISB completion; RISC-V may use a
leaf PTE and `SFENCE.VMA`. Page sizes, descriptor bits, virtual placement,
ASIDs/PCIDs, barriers, invalidation scope, and access instructions remain
provider-private.

The refinement is deliberately bounded. It does not yet admit arbitrary
layouts, permissions, multiple mappings, aliases, executable mappings,
concurrent edits, mapping replacement, partial unmap, or a reusable ELF image
builder. Later contracts may generalize these dimensions without weakening
the affine ownership or completion rules proved here.

## Initial x86-64 qualification

The provider uses the existing sealed `probe-guard` leaf, stores the live
mapping's physical identity only in provider-private state, and implements the
four transitions as distinct generated functions. The later deliberate
guard-crossing user-copy faults prove that unmap restored the guard before
user execution.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.
