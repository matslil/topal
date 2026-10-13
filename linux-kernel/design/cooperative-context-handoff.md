# Cooperative kernel-context handoff contract

This record resolves `TK-DEC-028` for the first executable cooperative
scheduler sequence. It extends the approved opaque context-transfer semantics
from one terminal round trip to two independently owned kernel threads and a
bounded FIFO dispatcher written in ordinary Topal source. The provider still
implements only context mechanism; it does not select runnable work.

## Portable source contract

Context transfer is symmetric. It consumes the running-context authority and
one matching `SuspendedKernelContext P`, suspends the current continuation,
and resumes the selected continuation. When the suspended continuation is
later resumed, the transfer point yields exactly one affine outcome:

- a suspended outcome restores its running authority and returns the context
  which handed control back; or
- a retired outcome restores its running authority and owns the terminal
  context and stack until consuming reclamation.

The general semantic outcome is a sealed sum. The initial executable profile
statically refines that sum from each closed entry protocol: the first
transfer to `cooperative-thread` returns a suspended context, its second
transfer returns a completed context, and a transfer to `terminal-thread`
returns a completed context. Source cannot forge the refinement or reinterpret
one outcome as the other.

`kernel context retire to target` consumes the running context and one matching
suspended target at a checker-proved transfer-safe point. It makes the running
context terminal and resumes the target with the retired context and stack in
one `CompletedKernelContextTransfer P`. Reclamation consumes that completion,
returns the retired stack to its originating pool, and restores ordinary use
of the resumed running context.

The initial dispatcher constructs two contexts from disjoint checked 16 KiB
bootstrap-reclaimable regions. Its source order is fixed and auditable:

1. transfer to the cooperative worker;
2. receive that worker as suspended after its explicit handoff;
3. transfer to the terminal worker, receive its retirement, and reclaim it;
4. transfer back to the cooperative worker, receive its retirement, and
   reclaim it; and
5. enter the bootstrap fatal disposition with no live context or stack.

The cooperative worker writes its entry marker, transfers to the suspended
dispatcher, writes its resumed marker after the dispatcher selects it again,
and retires to the newly suspended dispatcher. The terminal worker writes its
entry marker and retires directly to the dispatcher. The checker rejects a
different order, wrong target, target reuse after consumption, overlapping
stack ownership, a mismatched outcome classifier, retirement with a live
obligation, and final disposition with an unreclaimed context.

This bounded sequence is qualification scaffolding, not a privileged FIFO
operation. A later ordinary kernel scheduler may replace the fixed variables
with an affine runnable collection once that collection contract is approved.

## Provider boundary

The provider receives only qualified create, transfer, retire, and reclaim
operations plus static entry identities. It saves and restores the state
required at a provider call boundary, selects the target's private stack, and
records the exact running, suspended, or terminal identity transition. It does
not inspect priorities, own a run queue, or choose the next context.

The portable contract applies equally to x86-64, AArch64, and RISC-V. Their
callee-saved register sets, status state, stack conventions, and instruction
sequences remain provider evidence. Only the x86-64 QEMU provider is executable
in this increment.

## Initial boundary

Both workers stay on `InitialProcessor`, retain the active kernel address
space, and transfer only with local maskable interrupts disabled. The slice
does not add involuntary preemption, blocking or wakeup, dynamic run queues,
priority, timeslicing, cancellation, migration, SMP, user contexts,
floating-point or vector ownership, TLS/per-CPU switching, stack growth,
guard pages, or cross-transfer unwinding.
