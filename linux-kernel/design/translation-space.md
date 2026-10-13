# Translation-space construction and activation contract

## Decision

The fourth kernel-memory transition replaces the boot adapter translation
through an exclusive builder lifecycle: `TranslationUpdate` to inactive
`TranslationSpace` to refined active bootstrap context. Source expresses a
semantic template and page policy, never a table entry, physical backing
address, table level, descriptor bit, register, or maintenance instruction.

The exact initial source form is:

```topal
memory translation begin (
  template is bootstrap-equivalent,
  page-policy is provider-selected
)
  Ok update then {
    memory translation commit update
      Ok space then {
        memory translation activate space
          Ok translated then {
            translated console write "TOPAL_KERNEL_TRANSLATION_ACTIVE"
            translated fatal "complete"
          }
          Error failure then {
            failure fatal "translation activation failed"
          }
      }
      Error failure then {
        failure fatal "translation commit failed"
      }
  }
  Error failure then {
    failure fatal "translation construction failed"
  }
```

This decision resolves `TK-DEC-020` by refining the approved exclusive-builder
and address-space activation meaning in `TOPAL-SYSTEMS-MAPPING-001`.

## Affine lifecycle

`translation begin` borrows the live allocator and reserves provider-selected
backing. Success returns one exclusive affine update which owns that backing
and the semantic template snapshot. `translation commit` consumes a validated
update and returns one inactive affine space. `translation activate` consumes
that space and the current translation authority, completes the target
publication/activation protocol, and returns the only context admitted for
subsequent operations.

Every initial error path is fatal-only. In particular, activation failure does
not guess whether old or new target state is active. Updates and spaces cannot
be duplicated, stored, escaped, crossed between providers, committed twice,
activated twice, or abandoned at a disposition. Provider backing cannot be
released while any update, inactive space, or active context owns it.

`bootstrap-equivalent` preserves the current qualified bootstrap coverage and
permissions. It grants no new user, device, DMA, firmware, writable-executable,
or address-conversion authority. `provider-selected` permits different backing
counts, granularities, and table shapes only when their observable coverage,
access, lifetime, and completion semantics are identical.

## Architecture pressure test

| Target family | Provider-private construction and activation | Common observation |
| --- | --- | --- |
| x86-64 | allocate a four-level root hierarchy, populate 2 MiB bootstrap identity leaves, publish entries, and load the root through the qualified control-state operation | equivalent bootstrap coverage becomes active; the returned context is the sole continuing authority |
| AArch64 | select granule/levels, populate descriptors and MAIR/TCR-compatible attributes, clean/publish as required, install TTBR state, and complete required barriers/TLBI | the same update, commit, activation, coverage, permission, and ownership lifecycle |
| RISC-V | select supported Sv mode, populate PTE hierarchy, publish it, install `satp`, and complete required `SFENCE.VMA` scope | the same lifecycle and observations |

The common contract never promises a particular number of levels or backing
frames, identity translation, huge-page size, descriptor layout, or activation
instruction.

## Initial x86-64 slice

The provider selects three contiguous complete 4 KiB E820 RAM frames above the
16 MiB bootstrap floor and wholly below the existing first-GiB identity map.
The inactive space owns those frames. Commit zeroes and populates a PML4, PDPT,
and page directory whose 512 2 MiB leaves preserve the boot adapter's first-GiB
identity coverage. Activation loads the replacement root and returns only
after the architecture-defined serialization of that control-state change.

Structural inspection requires distinct backing selection, table zeroing and
population, root activation, success/fatal branches, and ordered semantic
trace. Pinned QEMU evidence emits `TOPAL_KERNEL_TRANSLATION_ACTIVE` only after
the replacement has been activated and ordinary kernel execution continues.

Arbitrary mapping edits, permission changes, NX enablement, user spaces,
per-processor spaces, multiple active spaces, unmap, shootdown, and translation
backing reclamation remain fail-closed future work.

This contract was approved in the project discussion before implementation.
