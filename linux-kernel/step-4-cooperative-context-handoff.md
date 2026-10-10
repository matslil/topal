# Step 4 cooperative context-handoff increment

## Outcome

The checked kernel root now constructs two independently owned 16 KiB kernel
contexts and implements a bounded FIFO dispatcher in ordinary Topal source.
The cooperative worker enters, hands its suspended context back to the
dispatcher, resumes after the terminal worker has run and retired, then
retires itself. Both completed stacks are consumed by reclamation before the
bootstrap context reaches its fatal disposition.

The architecture-independent model distinguishes cooperative-once and
terminal entry protocols, retains both context and stack identities, and
rejects wrong ordering, result refinement, repeated handoff, retirement with
live obligations, and disposition with a live context. The sealed source
checker admits exactly the approved cooperative/terminal/cooperative sequence;
the x86 provider performs no runnable selection.

## Initial x86-64 provider

Provider revision `topal.provider.x86_64-qemu-pc-q35/13` uses private context
records at distinct bootstrap-storage offsets and disjoint stacks beginning at
offsets 4096 and 20480. Its symmetric transfer routine accepts only a suspended
or active qualified record. Selecting a suspended worker saves the dispatcher
stack and restores the worker stack; cooperative handoff saves the worker stack
and restores the exact dispatcher stack. Entry addresses, saved stack
positions, callee-saved registers, state tags, and instruction sequences remain
outside Topal source.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/13` records 45
linked semantic placements and a 91-transition trace. Root object revision
`topal.systems-root-object.x86_64/13` has 671 typed relocations, including two
context creates, four context transfers, two retirements, two reclamations,
and one reference to each generated worker entry. Boot adapter revision
`topal.boot-adapter.linux-x86-protocol-2.15-q35/5` is unchanged because both
workers are internal linked roots.

## Evidence

- architecture-neutral tests cover two-context FIFO dispatch and reject live
  retirement obligations;
- source tests cover both exact typed handlers, suspended and retired result
  classifiers, target identity, stack shape, markers, and reclamation;
- compiler tests cover the two entries and stack placements, symmetric
  save/select sequences, exact relocation ledger, linked placements, and
  semantic trace;
- the complete compiler library suite passes 188 tests; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced the linked kernel
  `710ff4cc664ff4f3c123f9a80713ec071751a87a8b21516dc5878ef57cf2a8b2`,
  boot image
  `b4f7abf6793262b204befea01e97cc9f1c2c3b14e2ea3ebe259eaa962bbf2b65`,
  artifact provenance
  `1015134581f9930b3942cb70867e43619bf03363b691af8163be7505892193fe`,
  boot provenance
  `280a075c3c87c820023dea1ed53dc34747fff0ce9ec1f82c0fba34754130a272`,
  and serial observation
  `a6193e7dd951676e1b8958efe55a1577cacf9e132dd3f00c6fffb9f82eef2da4`.

Gate schema `topal-kernel-toolchain-gate-qemu/14` records all nineteen ordered
markers. They prove cooperative entry and suspension, terminal entry and
retirement, cooperative resumption and retirement, and quiescence after final
stack reclamation and fatal halt.

This is high-risk continuation and stack-ownership work. Review combines the
authority-ordered design check, cross-architecture pressure test, affine
negative tests, exact machine-code and ELF inspection, deterministic artifact
publication, two physical QEMU runs, and full workspace validation.

## Remaining boundary

This increment does not add involuntary preemption, blocking or wakeup,
dynamic affine runnable collections, priorities, timeslicing, cancellation,
migration, SMP context ownership, user/kernel transitions, multiple address
spaces, floating-point/vector or debug state, TLS/per-CPU switching, guard
pages, stack growth, or cross-transfer unwinding. Those remain later reviewed
increments and are not implicit in this bounded cooperative dispatcher.
