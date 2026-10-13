# Atomic-word location contract

## Decision

The initial atomic source element consumes one exclusively owned ordinary
region into one naturally aligned unsigned machine-word `AtomicLocation` in
the `cpu-shared` synchronization domain. Compare/exchange states independent
success and failure orders. Ending the atomic lifecycle consumes the location
and returns the same region, including the final word contents, to plain
ownership.

This resolves `TK-DEC-023` as a specialization of
`TOPAL-SYSTEMS-ATOMIC-001` and `TOPAL-SYSTEMS-ORDER-001`. It does not expose a
machine address, instruction width, opcode, register, exclusive monitor, or
retry strategy.

## Source lifecycle

```topal
atomic is region atomic word create (
  offset-bytes is 8,
  initial-value is 41,
  domain is cpu-shared
)
atomic compare exchange (
  expected is 41,
  desired is 42,
  success-order is acquire-release,
  failure-order is acquire
)
  Exchanged previous then {
    observed : Nat is atomic load (order is acquire)
    observed = 42
      true then {
        region is atomic end
        restored bootstrap release region
      }
      false then {
        region is atomic end
        restored bootstrap release region
        restored fatal "atomic load mismatch"
      }
  }
  Observed actual then {
    region is atomic end
    restored bootstrap release region
    restored fatal "atomic compare exchange lost"
  }
```

Construction consumes `region`; the name cannot be used while `atomic` is
live. The selected word is wholly contained in the region and aligned to the
qualified target word width. The entire region, rather than just the selected
bytes, remains under atomic ownership to make absence of overlapping plain
aliases explicit in the initial checker.

Compare/exchange returns `Exchanged previous` when the observed word equals
`expected`, replacing it with `desired` at one point in the location's
modification order. It returns `Observed actual` without modification
otherwise. Both branches are required even where a closed bootstrap test has
no competing writer. `atomic end` consumes the location only when no operation
is in flight and returns the original region identity for plain access or
release. Every ordinary continuation must end it; a terminal fatal disposition
may consume all remaining authority.

The first source slice admits only:

- an unsigned target machine word;
- one location at byte offset 8 of the 64-byte bootstrap test region;
- initial value 41 and desired value 42;
- the `cpu-shared` normal-memory domain;
- `AcquireRelease` success and `Acquire` failure for compare/exchange; and
- `Acquire` for the confirming load.

Those constants seal the first executable gate; they are not general language
limits. Plain access, region release, duplication, escape, a second live
operation, unsupported order/domain selection, out-of-bounds placement, or
misalignment fails before lowering.

## Ordering and domain boundary

Successful compare/exchange participates once in the location modification
order and supplies release and acquire relations. A failed comparison supplies
only the selected acquire observation and cannot release. The confirming load
is an acquire observation. A provider may strengthen these orders but cannot
weaken them or expose a result that the common model forbids.

`cpu-shared` covers coherent CPU access to ordinary normal memory. It makes no
claim about MMIO, device side effects, DMA visibility or ownership, firmware
memory, user memory, cache maintenance, instruction synchronization, or
translation maintenance. Those domains require their own typed operations.

## Architecture pressure test

| Target family | Provider-private implementation choices | Common observation |
| --- | --- | --- |
| x86-64 | naturally aligned 64-bit storage and a locked compare/exchange; ordinary aligned load under the qualified memory model | one indivisible success or observed failure, then an acquire observation |
| AArch64 | LSE compare-and-swap when qualified, or an exclusive load/store loop with the selected acquire/release semantics | the same result variants, modification order, and visibility edges; retries are unobservable |
| RISC-V | A-extension compare-and-swap when available, or a qualified LR/SC loop with required `aq`/`rl` behavior | the same result variants, modification order, and visibility edges; reservation failures are unobservable |

Progress is separate from ordering. A future concurrent protocol must state
whether it requires lock freedom, bounded retry, interrupt safety, or a
fallback lock. The initial single-processor bootstrap gate proves the atomic
effect and lowering shape but makes no contention-progress claim.

## Initial x86-64 slice

The initial provider uses one aligned 64-bit word at offset 8 in the generated
bootstrap storage. Generated root code obtains its private location and passes
it to typed provider facilities; no address becomes a Topal value. Creation
initializes the word while exclusive ownership is held. Compare/exchange uses a
locked 64-bit operation and branches on architectural success. The acquire
load requires no extra fence under the qualified x86-64 memory model, but it
remains a distinct semantic operation and placement.

Structural inspection requires typed create, compare/exchange, load, and end
placements; the locked operation; success and observed-failure branches; the
confirming load; and ownership return before bootstrap release. Pinned QEMU
evidence emits `TOPAL_KERNEL_ATOMIC_OK` only after successful exchange and
confirmation.

Concurrent agents, contention/progress qualification, other scalar widths,
store, exchange, additional read-modify-write functions, sequential order,
device/DMA atomicity, and AArch64 or RISC-V executable providers remain fail
closed.

This contract was approved in the project discussion before implementation.
