# Opaque kernel-mapping contract

## Decision

The third kernel-memory transition consumes an owned physical-frame extent
into one opaque kernel mapping. Source requests semantic access policy rather
than a physical base, virtual base, page-table format, or target instruction:

```topal
memory kernel map frames (
  rights is read-write,
  execution is denied,
  memory-kind is normal
)
  Ok mapping then {
    mapping byte store (offset-bytes is 0, value is 165)
    observed : Nat is mapping byte load (offset-bytes is 0)
    frames is memory kernel unmap mapping
    memory frames release frames
    memory fatal "complete"
  }
  Error problem then {
    memory fatal "kernel mapping failed"
  }
```

This decision resolves `TK-DEC-019`. It refines the already approved
`TOPAL-SYSTEMS-MAPPING-001` meaning with source vocabulary and an executable
initial slice. It introduces no pointer, numeric address, page-table record,
register, instruction, or source-assembly vocabulary.

## Ownership and access

`kernel map` borrows the live allocator context and consumes one
`PhysicalFrameExtent`. Success returns one affine `KernelMapping` which owns
that exact extent and records:

- opaque physical and provider-selected kernel-virtual extent identities;
- read/write/execute rights and normal-memory policy;
- owner, lifetime, and translation-provider provenance; and
- the complete byte bound derived from the target-qualified frame geometry.

The mapping alone grants access. Its byte load and store operations borrow the
live mapping, require a static in-bounds offset in the initial slice, and
retain ordinary byte contents. The approved initial request is exactly
read-write, execution denied, normal memory. It grants no user, device, DMA,
firmware, volatile, atomic, or executable authority.

`kernel unmap` consumes the mapping, revokes its access authority, completes
the provider's required translation/lifetime protocol, and returns exactly the
original opaque extent. Only then may the allocator release the frame. Frame
release while mapped, mapping duplication or escape, use after unmap,
cross-provider use, writable execution, out-of-bounds access, and disposition
with a live mapping are invalid.

An explicit map failure preserves only the live allocator context needed for
the selected failure action. The initial boot program consumes that context
through `fatal`; it cannot recover, duplicate, or release an extent whose
provider mapping result is unknown.

## Architecture pressure test

The portable contract is mapping ownership and access, not one translation
mechanism:

| Target family | Provider-private implementation | Common observation |
| --- | --- | --- |
| x86-64 | adopt the sealed bootstrap identity translation for a selected frame below its qualified limit | opaque read/write, non-executable normal mapping; unmap returns the frame |
| AArch64 | adopt or construct a translation with selected granule, attributes, and required TLBI completion | the same rights, byte bounds, lifetime, and reuse rules |
| RISC-V | adopt or construct an Sv-mode translation with required `SFENCE.VMA` scope/completion | the same rights, byte bounds, lifetime, and reuse rules |

Adoption is permitted only when the provider proves that ordinary source has
no alias capability and cannot access the range after the mapping token is
consumed. Numeric equality in an identity mapping does not equate the physical
and kernel-virtual semantic families.

## Initial x86-64 executable slice

The provider selects one complete 4 KiB E820 RAM frame above the 16 MiB
bootstrap reservation floor and wholly below the adapter's existing 1 GiB
identity-map limit. The selected base remains in backend-private machine state.
Generated root code accesses byte zero only while its mapping token is live,
then logically unmaps, returns, and releases the extent. Structural inspection
requires the selector bound, register-indirect byte store/load, and ordered
map/unmap semantic trace. Pinned QEMU evidence emits
`TOPAL_KERNEL_FRAME_MAPPED` only after the stored byte is read back.

Page-table builders, activation, permission changes, translation shootdown,
multiple mappings, dynamic offsets, executable mappings, and user mappings
remain fail-closed future work.

This contract was approved in the project discussion before implementation.
