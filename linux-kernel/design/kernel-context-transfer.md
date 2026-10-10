# Kernel-context transfer contract

This record resolves `TK-DEC-027` for the first executable scheduler-context
transition. It specializes the approved opaque context-transfer semantics with
one provider-created kernel thread, one transfer from the bootstrap
continuation, terminal retirement of that thread at a statically safe point,
and resumption of the caller. Stack pointers, register slots, continuation
addresses, and save/restore instructions remain outside Topal source.

## Portable source contract

`SuspendedKernelContext P` is affine authority for one nonrunning kernel
continuation bound to processor identity `P`. It owns its kernel stack, saved
machine state, thread identity, address-space relationship, and declared
extended-state policy. It is neither a function nor a serializable value and
cannot be copied, inspected, widened, migrated, or used as an address.

Initial construction consumes one exclusive kernel-owned ordinary region and
one static typed kernel-thread entry. The region must meet the selected
provider's checked size and alignment requirements. Construction creates the
initial continuation without running it. The provider may reject an
unsupported region or entry before artifact publication; the admitted sealed
request is infallible at runtime after those checks.

Context transfer consumes the current running-context capability and exactly
one matching suspended context. It suspends the caller at a typed continuation
point and resumes the selected context. It is not an ordinary call or return,
even when a backend uses call/return-shaped instructions internally. A later
qualified transfer or terminal thread disposition may resume the caller.

The initial worker entry receives its running `KernelThreadContext P` and the
affine suspended caller. Its terminal `kernel context retire to caller`
disposition is legal only at a checker-proved transfer-safe point with no live
resource, critical-scope, interrupt, recovery, or cleanup obligation. It
consumes both contexts, marks the worker terminal, and resumes exactly that
caller. The resumed caller obtains one affine
`CompletedKernelContextTransfer P`, which temporarily owns both the running
caller authority and the retired worker's stack. `kernel context reclaim`
consumes the completion, returns the stack to its originating monotonic pool,
and yields the resumed caller context. Neither operation is general thread
cancellation.

The initial source shape is deliberately closed:

```topal
kernel-thread-handler is fn (
  context : KernelThreadContext InitialProcessor,
  caller : SuspendedKernelContext InitialProcessor
) -> KernelThreadDisposition InitialProcessor
  context console write "TOPAL_KERNEL_CONTEXT_ENTERED"
  context kernel context retire to caller

deadline-resumed bootstrap allocate (
  byte-count is 16384,
  alignment-bytes is 16,
  placement is bootstrap-reclaimable
)
  Ok stack-region then {
    worker : SuspendedKernelContext InitialProcessor is
      deadline-resumed kernel context create (
        stack is stack-region,
        entry is kernel-thread
      )
    completed : CompletedKernelContextTransfer InitialProcessor is
      deadline-resumed kernel context transfer worker
    completed console write "TOPAL_KERNEL_CONTEXT_RESUMED"
    context-resumed is completed kernel context reclaim
    context-resumed fatal "toolchain gate complete"
  }
  Error problem then {
    deadline-resumed fatal "kernel context stack allocation failed"
  }
```

The systems artifact binds exactly one `kernel-thread` resumed-thread entry to
the handler. Creation, transfer, retirement, resumption, and reclamation carry
one context identity and one stack-region identity in the semantic trace.

## Provider boundary

The portable abstraction is an ownership-preserving continuation transfer,
not an exposed calling convention. The x86-64 provider may save the required
callee-saved state and stack position, install a private initial frame, and
resume through a private continuation. An AArch64 provider may save `SP`,
`x19`–`x30`, and qualified system state; a RISC-V provider may save `sp`, `ra`,
and `s0`–`s11`. Those sets are examples of target evidence, never source
vocabulary.

The initial transfer stays on `InitialProcessor`, retains the active address
space, requires local maskable interrupts to be disabled at the transfer
boundary, and declares no floating-point, vector, debug-register, TLS, or
per-CPU-base transition. A provider must fail closed if those assumptions do
not hold. Future providers may extend the saved-state policy only through a
separately qualified contract.

## Initial boundary

This increment admits exactly one 16 KiB bootstrap-reclaimable stack, one
static kernel-thread entry, one caller-to-worker transfer, terminal retirement
with no live worker resources, one caller resumption, and one stack reclaim.
It does not admit involuntary preemption, scheduling policy, run queues,
priority, timeslicing, blocking, general yield, arbitrary cancellation,
multiple threads, migration, SMP, address-space switching, user contexts,
floating-point or vector ownership, TLS/per-CPU switching, stack growth,
unwinding across transfer, or userspace ABI behavior.
