# Step 4 bounded affine runnable-queue increment

## Outcome

The checked kernel root now stores its two suspended worker contexts in a
source-owned `BoundedKernelRunnableQueue InitialProcessor`. It creates a
capacity-two queue, enqueues cooperative then terminal, dequeues cooperative,
re-enqueues its suspended handoff at the tail, then dequeues terminal and
cooperative. Both retired stacks are reclaimed and the empty queue is consumed
before bootstrap disposition.

The architecture-independent model implements general full and empty results
which retain every queue and context obligation. It rejects zero capacity,
identity mismatch, duplicate enqueue, nonempty consumption, wrong FIFO order,
and disposition with a live queue. The sealed source profile statically proves
the three successful dequeues and all queue transitions.

## Initial x86-64 lowering

Queue operations are compile-time checked source policy. They advance root
typestate and the artifact semantic trace but generate no provider symbol,
call, relocation, or instruction. The x86 context provider therefore remains
`topal.provider.x86_64-qemu-pc-q35/13`, and the linked kernel bytes remain
identical to the preceding cooperative-handoff increment.

Root object revision `topal.systems-root-object.x86_64/14` retains 671 typed
relocations and the same 45 linked semantic placements. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/14` records a 99-transition trace,
including one queue creation, three enqueues, three dequeues, and one empty
consumption. Structural tests prove that none of the four queue identities
appears in the 45-operation provider plan or the root's undefined-provider
dependency set.

## Evidence

- architecture-neutral tests exercise FIFO ownership and full, empty, and
  nonempty-consumption outcomes;
- source tests cover the exact classifier, capacity, enqueue/dequeue order,
  selected context identity, and empty consumption;
- compiler tests cover erased queue lowering, unchanged provider dependencies,
  the 99-transition semantic trace, and deterministic artifact publication;
- the complete compiler library suite passes 188 tests; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical evidence:
  linked kernel
  `710ff4cc664ff4f3c123f9a80713ec071751a87a8b21516dc5878ef57cf2a8b2`,
  boot image
  `b4f7abf6793262b204befea01e97cc9f1c2c3b14e2ea3ebe259eaa962bbf2b65`,
  artifact provenance
  `c5fb9baf3da815bf0735be37171f6db164ac611cfb4c6fe20791193cc65cec04`,
  boot provenance
  `280a075c3c87c820023dea1ed53dc34747fff0ce9ec1f82c0fba34754130a272`,
  and serial observation
  `a6193e7dd951676e1b8958efe55a1577cacf9e132dd3f00c6fffb9f82eef2da4`.

Gate schema `topal-kernel-toolchain-gate-qemu/15` retains all nineteen ordered
runtime markers. The unchanged linked-kernel and serial hashes provide direct
evidence that the portable source queue changes scheduler ownership and audit
semantics without introducing a hidden x86 queue mechanism.

This is high-risk affine-ownership and scheduler-policy work. Review combines
authority-ordered design propagation, cross-architecture pressure testing,
general queue negative tests, sealed source-order checks, provider-boundary and
ELF inspection, deterministic publication, two physical QEMU runs, and full
workspace validation.

## Remaining boundary

This increment does not add unbounded or dynamically allocated runnable
storage, involuntary preemption, blocking or wakeup, priorities, timeslicing,
cancellation, migration, SMP, user contexts, multiple address spaces,
floating-point/vector ownership, TLS/per-CPU switching, guard pages, stack
growth, or cross-transfer unwinding.
