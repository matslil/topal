# Deadline-preemptible kernel-context transfer contract

This record resolves `TK-DEC-030` for the first involuntary scheduler return.
It integrates one absolute deadline with one selected kernel context while
keeping runnable selection in ordinary Topal source. The machine provider
saves and restores target state, but it neither owns the runnable queue nor
chooses the context which runs next.

## Portable source contract

`kernel context transfer until deadline` consumes the running dispatcher
authority, one matching `SuspendedKernelContext P`, and one `Deadline C`. It
atomically establishes the caller's exact suspended continuation as the sole
preemption target, arms the deadline event, and resumes the selected context.
The target, dispatcher, processor, provider, address-space, clock, deadline,
and event identities remain bound until the transfer resolves.

Atomic establishment means that delivery cannot become visible between arming
the event and installing its preemption target. An already-expired deadline is
immediately deliverable only after that target exists. The selected context
runs with local maskable interrupt delivery admitted by this operation; the
dispatcher later resumes with its exact prior interrupt state. Source cannot
inspect or manufacture the saved interrupt state, continuation, event route,
or target binding.

The matching typed deadline handler must first consume its ordinary completion
obligation. Its `preempt current kernel context` disposition is legal only
while the matching preemptible transfer is active. That disposition consumes
the completed event and running target, captures the target as one opaque
suspended context, and resumes exactly the registered dispatcher. The resumed
dispatcher receives one affine `PreemptedKernelContextTransfer P C` owning its
restored running authority, the suspended target, and the scheduled/observed
delivery evidence. Consuming that outcome returns the suspended target for
source-owned enqueue.

The handler cannot inspect the interrupted frame, choose a runnable context,
access the queue, redirect to a different dispatcher, or resume normally after
choosing preemption. A preemption disposition without a matching active
transfer, before event completion, for another clock/event/processor, or after
the transfer resolved is invalid. A preempted outcome cannot be copied,
dropped, or reinterpreted as cooperative or terminal completion.

Later ordinary transfer to the suspended target restores the exact interrupted
machine continuation and its frozen affine source obligations. The provider
also establishes the newly suspended dispatcher as that continuation's return
target. Thus a subsequently retiring worker returns to the currently running
scheduler instance without retaining or duplicating the dispatcher context
which was made running by preemption.

## Initial executable sequence

The first executable profile uses the existing capacity-two runnable queue and
the existing one-shot deadline source:

1. create and enqueue a deadline-preemptible worker followed by a terminal
   worker;
2. dequeue the first worker and transfer to it until the one-shot deadline;
3. let the worker enter a closed preemptible test region which cannot yield or
   retire before matching deadline delivery;
4. complete that deadline in its typed handler and preempt to the exact
   dispatcher;
5. consume the preempted outcome and enqueue the suspended worker at the tail;
6. dequeue, retire, and reclaim the terminal worker;
7. dequeue the preempted worker, restore its interrupted continuation, let it
   retire, and reclaim it; and
8. consume the empty queue before bootstrap disposition.

The resulting source-selected order remains preemptible, terminal,
preemptible. The closed test region exists only to make the first physical
preemption deterministic; it is not a general wait, sleep, or scheduler policy
operation. Its sealed `kernel context await deadline preemption` operation is
legal only inside this closed entry protocol. It does not return on initial
entry: the matching deadline first preempts the context, and only a later
source-selected transfer restores the interrupted continuation and lets the
operation complete.

## Provider boundary and architecture pressure

On x86-64 the provider privately retains the interrupt entry frame and all
state required to resume the interrupted kernel continuation, acknowledges the
deadline source, switches to the registered dispatcher stack, and later
restores the saved frame before interrupt return. AArch64 retains its exception
return state and required general registers; RISC-V retains its supervisor trap
return state and required general registers. Each target proves that the
dispatcher becomes eligible before delivery is enabled and that resumption is
exact.

Those frame sets, status fields, interrupt-mask operations, stack-switch
instructions, and return instructions are architecture evidence. The portable
operation exposes only the same typed preempted outcome and exact-identity
rules. In every provider, source dequeues the target before dispatch and
re-enqueues the interrupted context after return.

## Initial boundary

This slice admits one processor, one active address space, one outstanding
deadline, one registered dispatcher, and one deadline-preempted worker. It
does not add cancellation, rearming, periodic ticks, general timeslices,
priority, fairness, blocking or wakeup, nested preemption, migration, SMP,
user contexts, floating-point/vector ownership, TLS/per-CPU switching, stack
growth, guard pages, or cross-transfer unwinding.
