# Bounded affine runnable-queue contract

This record resolves `TK-DEC-029` for the first reusable scheduler-owned
runnable collection. It replaces the cooperative qualification sequence's
fixed suspended-context variables with a bounded FIFO value in ordinary Topal
source. Context switching remains a qualified provider mechanism; runnable
storage and selection remain portable scheduler policy.

## Portable source contract

A `BoundedKernelRunnableQueue P` is an opaque affine composite with a retained
fixed positive capacity. It owns every `SuspendedKernelContext P` enqueued in it
and retains the queue, provider, processor, and active-address-space identities
needed to validate later selection. It exposes no register, stack, continuation,
address, or target-layout representation.

Creating an empty queue establishes its capacity without allocating after
bootstrap. Enqueue consumes the prior queue and one suspended context. Success
produces the next queue with that context at its tail; a full result returns
both original obligations without duplication or loss. Dequeue consumes the
prior queue. A ready result produces the oldest suspended context and the
remaining queue; an empty result returns the unchanged queue. Closed occupancy
evidence may refine these results statically, but a refinement is valid only
when the checker proves the exact capacity and operation history.

The queue cannot contain a running, retired, or completed context. It cannot
mix processor, provider, or address-space identities. Copying, serialization,
use after consumption, duplicate enqueue, dropping a nonempty queue, and final
disposition with queued contexts are invalid. Destroying an empty queue
consumes its remaining queue obligation.

The initial executable queue has capacity two and follows this exact source
sequence:

1. create the empty queue and enqueue the cooperative worker followed by the
   terminal worker;
2. dequeue and transfer to the cooperative worker;
3. receive its suspended handoff and enqueue it at the tail, leaving terminal
   then cooperative FIFO order;
4. dequeue, transfer to, retire, and reclaim the terminal worker;
5. dequeue, transfer to, retire, and reclaim the cooperative worker;
6. consume the now-empty queue before bootstrap disposition.

The existing worker entry protocols and context-transfer outcomes are
unchanged. Full and empty paths remain represented in the general contract;
the initial source profile proves that neither path occurs.

## Representation and provider boundary

The queue is an ordinary checked Topal value, not a machine-provider operation.
The compiler may use a fixed local aggregate, stack slots, static storage, or
another equivalent representation when capacity and affine observations are
preserved. It may erase a statically known queue entirely, but artifact
evidence must retain the source queue transitions and prove that no provider
symbol or provider-owned run queue implements selection.

The x86-64, AArch64, and RISC-V context providers receive only the suspended
context selected by source. Their register sets, frame layouts, stack-switch
instructions, and extended-state policies remain private and are unaffected by
the FIFO representation.

## Initial boundary

The queue is fixed-capacity and bootstrap-preallocated. This slice does not add
unbounded or dynamically allocated runnable storage, involuntary preemption,
blocking or wakeup, priorities, timeslicing, cancellation, migration, SMP,
user contexts, multiple address spaces, floating-point/vector ownership,
TLS/per-CPU switching, stack growth, guard pages, or cross-transfer unwinding.
