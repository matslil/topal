# Active translation-edit contract

## Decision

Active translation changes use one exclusive affine transaction. A map edit
consumes a physical-frame extent into a provisional inaccessible mapping and
commit returns a refined active context while making that mapping live. An
unmap edit consumes the live mapping into a provisional unavailable extent and
commit returns the next refined context only after target-qualified
invalidation completes.

This resolves `TK-DEC-021` without exposing a virtual address, page-table
format, page size, descriptor, invalidation address, register, or instruction.
It refines the already approved builder and mapping meaning in
`TOPAL-SYSTEMS-MAPPING-001`.

## Map lifecycle

```topal
active translation edit begin
  Ok edit then {
    edit kernel map frames (
      rights is read-write,
      execution is denied,
      memory-kind is normal,
      placement is provider-selected
    )
      Ok mapping then {
        edit translation commit
          Ok edited then {
            mapping byte store (offset-bytes is 0, value is 60)
            edited fatal "complete"
          }
          Error failure then {
            failure fatal "translation edit commit failed"
          }
      }
      Error failure then {
        failure fatal "translation edit mapping failed"
      }
  }
  Error failure then {
    failure fatal "translation edit construction failed"
  }
```

Begin consumes the active context into one `TranslationEdit`; the old context
cannot be used while the edit is live. Map consumes one frame extent and
retains its physical provenance, rights, memory kind, extent, active-space
identity, and provider identity in a provisional `KernelMapping`. The mapping
cannot be accessed before commit. Commit validates the complete candidate,
publishes it with provider-required ordering, consumes the edit, returns a
refined active context, and changes the mapping to live.

Every initial error path is fatal-only. Duplicate or nested edits, wrong-space
mapping, frame release after provisional map, mapping access before commit,
old-context use, commit twice, edit escape, and disposition with a live edit
fail closed.

## Unmap lifecycle

Unmap begins another exclusive edit and consumes the live mapping into its
original provisional frame extent. Neither the old mapping nor the returned
extent can be used while removal is pending. Commit removes the translation,
performs the complete target invalidation protocol, consumes the edit, returns
the next refined active context, and restores ordinary releasable frame
ownership. A failed or indeterminate removal commit is fatal-only; source never
guesses whether stale translations remain usable.

## Architecture pressure test

| Target family | Provider-private edit and invalidation | Common observation |
| --- | --- | --- |
| x86-64 | allocate hierarchy backing, populate a 4 KiB leaf, publish the parent entry, clear it on removal, order table writes, and execute the qualified local invalidation | commit changes one opaque mapping between provisional/live/removed states and refines the active context |
| AArch64 | choose granule and levels, publish descriptors with required barriers, then apply the required VA/ASID TLBI and completion barriers | the same exclusive transaction, mapping visibility, invalidation completion, and ownership return |
| RISC-V | choose an Sv hierarchy, publish/clear PTEs, then apply the required `SFENCE.VMA` address/ASID scope | the same transaction and observations |

No common promise selects local versus broadcast invalidation. An SMP provider
must satisfy the full affected-processor scope before returning reusable frame
ownership.

## Initial x86-64 slice

The provider admits one 4 KiB normal, read-write, non-executable mapping in a
reserved opaque kernel window. It reserves two additional complete E820 RAM
frames for a page-directory and page-table subtree without overlapping the
active root, existing image, or mapped payload frame. Map commit links that
subtree into the active root only after it is complete. Unmap commit removes
the parent entry, publishes the removal, executes local invalidation for the
private window, and only then restores the payload extent for release.

Structural inspection requires non-overlapping allocation floors, distinct
edit/map/commit/unmap provider paths, complete table construction, publication
and removal stores, the exact invalidation instruction, success/fatal branches,
and ordered semantic trace. Pinned QEMU evidence writes through the new opaque
mapping, reloads the sentinel, unmaps and invalidates it, releases the frame,
and continues through the refined context.

Concurrent edits, multiple live dynamic mappings, user mappings, device
memory, executable mappings, permission changes, remote shootdown, arbitrary
placement, and metadata reclamation remain fail closed.

This contract was approved in the project discussion before implementation.
