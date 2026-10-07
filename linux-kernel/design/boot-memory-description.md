# Portable boot-memory description contract

## Decision

The first kernel-foundation transition converts a provider-private boot
handoff into an architecture-independent physical-memory description. The
source operation is `context boot describe memory`. It consumes the entered
bootstrap context and is handled as an ordinary exhaustive `Result` decision:

```topal
context boot describe memory
  Ok described then {
    described console write "TOPAL_KERNEL_MEMORY_DESCRIBED"
    described fatal "complete"
  }
  Error failure then failure fatal "boot memory description failed"
```

The `Ok` binding is a `MemoryDescribedContext`. It retains all capabilities of
the entered bootstrap context and additionally owns the normalized physical
memory description. The `Error` binding is a fatal-only
`BootMemoryFailureContext`; it cannot continue boot, allocate, inspect the raw
handoff, or recover the consumed entered context. Neither binding may escape
the entry extent.

This decision resolves `TK-DEC-017`. It introduces no raw pointer, numeric
address, firmware-structure, register, or source-assembly vocabulary.

## Normalized description

The provider validates its target-specific handoff and produces disjoint,
nonempty, nonwrapping, page-qualified physical ranges. Each retained range has
one class:

- `allocatable`: ordinary volatile RAM available now;
- `reclaimable`: RAM unavailable until its named boot or firmware owner
  completes;
- `persistent`: nonvolatile memory requiring a later explicit policy;
- `reserved`: occupied or platform-owned memory;
- `unusable`: memory the provider reports as unsafe; or
- `unknown`: an unrecognized source classification, never treated as RAM.

Every range retains its source kind and provider provenance. Overlaps are
split into disjoint output ranges and resolved conservatively: any non-
allocatable claimant prevents the overlap from being allocatable, and
`unusable` dominates all other classes. Page-edge fragments which cannot form
a complete physical frame are retained as reserved. Malformed input, numeric
overflow, cyclic or out-of-bounds extension data, contradictory provider
facts, or absence of any allocatable frame produces the failure context.

Before producing range capabilities, the provider subtracts every live
bootstrap reservation,
including the boot adapter, transition page tables and descriptors, initial
stack, linked kernel image, bootstrap storage, boot-handoff data, command line,
initramfs when present, and any target-qualified firmware table still needed.
Reservation ownership and reclamation conditions remain explicit; a range is
not made allocatable merely because source has no direct reference to it.

Portable source receives capabilities for normalized physical ranges, not
their integer encodings. Later frame allocators consume those capabilities.
Any diagnostic count or size observation carries no access authority.

## Architecture pressure test

The common transition is the validation, conservative normalization,
reservation, and capability refinement—not a particular firmware table:

| Target family | Provider-private input | Common result |
| --- | --- | --- |
| x86-64 | Linux `boot_params`, E820, optional setup-data extensions, and adapter-owned ranges | normalized physical ranges plus explicit reservations |
| AArch64 | Device Tree and/or UEFI memory descriptors plus image and firmware reservations | the same normalized range classes and ownership facts |
| RISC-V | Device Tree, SBI/platform handoff facts, and reserved-memory declarations | the same normalized range classes and ownership facts |

An architecture may reject a handoff it cannot validate. It may not expose its
native descriptor layout through the portable operation or reinterpret an
unknown range as allocatable memory.

## Initial x86-64 executable slice

The initial provider accepts the Linux x86 boot-protocol 2.15 handoff carried
privately from `RSI`. It validates the zeropage E820 count and entries, rejects
wrapping or empty usable ranges, treats every unknown E820 type as reserved,
and proves that at least one page remains allocatable after generated image
reservations. A nonzero setup-data pointer fails closed in this first slice;
bounded extension-list traversal and provenance require separate qualification.

The checked source must perform this transition before console, debug-break,
bootstrap-allocation, or successful fatal disposition operations. The pinned
QEMU gate emits `TOPAL_KERNEL_MEMORY_DESCRIBED` only on the success path. Model,
artifact-inspection, and physical evidence remain separate qualification
layers.

This contract was approved in the project discussion before implementation.
