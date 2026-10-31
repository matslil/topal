# One-shot affine kernel-completion contract

This record resolves `TK-DEC-031` for the first blocking and wakeup lifecycle.
It establishes a portable completion primitive without prematurely selecting
general wait-queue, futex, timeout, signal, or cancellation policy.

## Semantic boundary

A `KernelCompletion P I` has one processor/provider identity `P`, one fresh
completion identity `I`, one affine wait endpoint, and one affine signal
endpoint. The endpoints may move independently but cannot be copied, forged,
serialized, or substituted across identities. The completion is one-shot: it
accepts at most one waiter and one signal.

The signal endpoint records readiness persistently. If signalling linearizes
before the matching block attempt, the worker observes completion and does not
sleep. If blocking linearizes first, the operation captures the worker as one
opaque `BlockedKernelContextTransfer P I` and resumes only the exact dispatcher
registered by the transfer. Signalling that blocked outcome consumes the
signal endpoint and yields one `SuspendedKernelContext P` suitable for a
source-owned runnable queue. These alternatives prevent a lost wakeup.

The wait endpoint is owned by the worker protocol. Blocking consumes it; later
redispatch resumes immediately after that block operation. The completed
worker may then retire normally. The provider owns target context capture and
restoration only. It does not select a runnable context, own a run queue, or
invent a signal.

## Context and ordering rules

The dispatcher shall bind its exact continuation before the worker may block.
The transition which publishes a blocked context and the transition which
records a matching signal have one declared linearization order. Every outcome
retains processor, provider, address-space, completion, context, stack, and
continuation identities.

Blocking is legal only in a process or kernel-thread context with no live
machine-critical, interrupt-entry, or incompatible affine obligation. A hard
interrupt cannot block. Signalling may be implemented by eligible ordinary or
deferred source code; an interrupt handler must first complete its own entry
obligation and transfer only the typed signal evidence permitted by a later
extension.

## Initial executable sequence

The first x86-64 sequence is deliberately closed:

1. create one one-shot completion and its matching wait/signal endpoints;
2. create a block-once worker and one terminal worker;
3. enqueue both and select the blocking worker first;
4. let that worker enter, block once, and return a blocked outcome to the exact
   dispatcher;
5. select, retire, and reclaim the terminal worker;
6. consume the signal endpoint and blocked outcome to obtain the awakened
   suspended worker, then enqueue it;
7. select the awakened worker, resume immediately after its block, retire and
   reclaim it; and
8. consume the empty runnable queue and completed one-shot state.

The source-selected order is blocking, terminal, awakened. The initial profile
statically proves the block-first branch. Architecture-neutral tests also
cover the signal-first branch and reject repeated signal, repeated wait, wrong
identity, wrong dispatcher, and live completion obligations at disposition.

## Architecture pressure test

The common contract names completion readiness, one linearization order,
blocked-context ownership, exact dispatcher return, and later source-selected
redispatch. It does not name an instruction.

| Target | Possible private lowering | Portable result |
| --- | --- | --- |
| x86-64 | context-state transition with interrupts masked across publication | blocked or already-complete outcome |
| AArch64 | context-state transition with DAIF/GIC-qualified exclusion | same outcome and identities |
| RISC-V | context-state transition with status/controller-qualified exclusion | same outcome and identities |

`HLT`, `WFI`, `WFE`, monitor instructions, event registers, interrupt masks,
and context-frame layouts remain provider evidence. The semantic primitive is
blocking on a one-shot completion, not a portable low-power instruction.

## Explicit exclusions

This increment does not add multiple waiters, repeated signals, arbitrary
predicate wait queues, spurious wakeups, timeouts, cancellation, process
signals, priority ordering, cross-processor wakeups, SMP, user contexts, or
device-removal races. Those require later contracts that preserve the endpoint
and context ownership established here.
