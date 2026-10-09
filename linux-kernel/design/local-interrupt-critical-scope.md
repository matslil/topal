# Local-interrupt critical-scope contract

## Decision

Local maskable-interrupt exclusion is an affine refinement of the current
processor context. Entry captures the exact prior state in opaque restoration
authority and returns a context on which local maskable interrupts are
disabled. Restoration consumes that context exactly once and returns the
context state that immediately preceded entry.

This resolves `TK-DEC-022` without exposing a flags value, interrupt-mask
register, controller state, register, or instruction. It specializes the
approved critical-state meaning in `TOPAL-SYSTEMS-CRITICAL-001`.

## Source lifecycle

```topal
context critical enter (
  domain is local-maskable-interrupts
)
  Ok critical then {
    critical console write "TOPAL_KERNEL_INTERRUPTS_MASKED"
    restored is critical restore
    restored console write "continued"
  }
  Error failure then {
    failure fatal "critical entry failed"
  }
```

Entry consumes `context`. The success continuation owns one `critical`
context, its processor identity, domain, nesting identity, and opaque exact
prior state. The failure continuation owns only fatal disposition authority in
the initial slice. `critical restore` consumes the success context and returns
`restored`; the entry context and critical context cannot be used afterward.

Every ordinary exit restores. A critical context cannot escape, suspend,
block, transfer processors, perform a disposition, or survive completion.
Nested scopes, where admitted, receive distinct identities and restore in
last-in-first-out order. Duplicate restore, out-of-order restore, restoration
on another processor, and unconditional enable are invalid.

## Exclusion boundary

The `local-maskable-interrupts` domain excludes only maskable interrupt
delivery to the current processor. It does not exclude:

- a non-maskable or machine-critical event;
- another processor;
- a device or DMA agent; or
- an already executing observer not ordered by the scope.

Algorithms needing those exclusions require separate atomic, ordering,
controller, device, DMA, or multi-processor protocols.

## Architecture pressure test

| Target family | Provider-private state and action | Common observation |
| --- | --- | --- |
| x86-64 | capture `RFLAGS.IF`, execute the qualified local disable operation, and on restore enable only when the captured bit was enabled | local maskable interrupts remain disabled throughout the scoped continuation and exact prior enablement is restored |
| AArch64 | capture the relevant `DAIF` interrupt-mask state, apply the qualified mask, and restore the captured state with required synchronization | the same current-processor domain and exact-state restoration |
| RISC-V | capture the qualified supervisor interrupt-enable state, clear it, and restore that captured state | the same current-hart domain and exact-state restoration |

The common operation does not identify a status register or promise that all
asynchronous events are excluded. Controller-specific masking is a separate
domain and cannot be inferred from local processor masking.

## Initial x86-64 slice

The first provider admits one non-nested scope in the bootstrap root. Its
entry facility captures the current flags privately, disables local maskable
interrupts, and returns opaque nonzero restoration authority. Its restore
facility validates that authority and restores only the captured interrupt
enable state. No source-visible numeric state or instruction spelling is
introduced.

Structural inspection requires distinct enter and restore symbols, retained
flags capture, disable, conditional-enable, and disabled-restoration paths,
success/fatal branches, and ordered semantic trace. Pinned QEMU evidence emits
`TOPAL_KERNEL_INTERRUPTS_MASKED` from inside the scope and continues through
the restored context.

Nested scopes, preemption control, controller masking, interrupt handlers,
multi-processor exclusion, and synchronization-library policies remain fail
closed in this initial provider slice.

This contract was approved in the project discussion before implementation.
