# Traversal admission and invocation control

This continues `TK-DEC-282` Stage 2 under the approved systems-only direction.
The provider constructs actual invocation context and runs the three native
roles; a test initializer is insufficient evidence for those mechanisms.

The initial view-bearing controller accepts an actual owned byte extent, three
immutable word-bounded Nat facts, actual allocation-owned persistent history,
the private temporary/stack domains and the caller's empty result frame. Its
closed source entry has `(KernelByteRegion, Nat, Nat, Nat) -> Boolean`, with the
first Nat the countdown. This is an initial native interface, not a core type or
recursion restriction. Caller extraction must prove whole values fit the facts;
it must never truncate a larger natural. Other signatures retain their separate
component evidence and need their own owning-caller qualification.
This initial owner is nonempty and exclusively read-write; geometry cannot
upgrade a read-only grant or establish those caller permissions.

Admission requires mapped exclusive ownership independently of address geometry.
It validates all disjoint extents, immutable input/frame layout, zero private/
stack/output/recovery state, nonfatal inactive history and native nonwrapping
generation/counter budgets. The maximum step count follows the reconstructed
natural guard and minimum decrement; all source helpers and replacement roots
are charged. Every refusal precedes any byte, generation or history mutation.

A fresh history is entirely zero. Successful admission initializes its next
frame/loan counters to two and advances generation to one; later invocations
retain actual next counters and advance the actual generation without reset.
The root frame/loan numbered one belongs to the new generation, matching the
owned reference model. It is distinct from earlier-generation roots. Private
context, both-bank roots and current-view metadata are constructed from the
selected code, caller facts and actual owner/history, rather than caller labels.

The controller executes source, checkpoint and completion through their fixed
native target entries. It accepts only the expected private outcomes, retains
the original history/output pins independently of mutable context and never
returns an admitted protocol failure as normal completion. Performed owner
stores and issuance persist; failure erases output and reaches trusted private
cleanup/nonreturning fatal handling. A normal source Boolean remains an actual
result: the owning kernel caller must separately validate its own invariant
before resuming or releasing its resource.

The controller runs on its owning caller's stack while the three inner roles
reuse the private stack sequentially. Its complete frame/call/argument/alignment
cost, saved state, input/output, recovery and retired spills must be charged and
physically reclaimed by the target kernel caller. Native controller execution
does not prove those obligations, root-stack/interrupt context, actual kernel
ownership or publication. The next integration must bind the real typed root
callsite, native objects, kernel resource lifetime and VM observations. Existing
qualified acyclic profiles remain unchanged. AArch64/RISC-V implementations must
preserve the same value, lifetime and admission ordering with their own targets.
