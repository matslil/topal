# Step 4 one-shot kernel-completion increment

## Outcome

The checked kernel root now creates one affine one-shot completion and a third
kernel worker. Source selects that worker from its capacity-three FIFO, binds
the exact dispatcher before entry, and receives the blocked worker back. It
then runs and reclaims the terminal worker, consumes the matching signal
authority, enqueues the awakened worker, and eventually selects it again. The
worker resumes immediately after its completion wait, retires, and is
reclaimed before the completed one-shot state and empty queue are consumed.

The architecture-independent reference model separately covers signal-first
readiness, proving that a signal cannot be lost when it linearizes before the
wait. Negative tests reject mismatched completion and dispatcher identities,
blocking with incompatible obligations, altered source order, wrong queue
capacity, and incomplete affine lifecycles.

## Initial x86-64 lowering

Provider revision `topal.provider.x86_64-qemu-pc-q35/15` qualifies the
completion-bound transfer as an opaque context-state operation. The closed
single-processor block-first gate uses the existing private stack-switch
mechanism: the generated completion worker transfers to its registered
dispatcher at the wait, and only a later source-selected transfer restores the
worker after that call. Completion creation, signalling, queue selection, and
consumption remain source-owned typestate and emit no provider-owned scheduler
or run queue.

Provider-object revision `topal.provider-object.x86_64-qemu-pc-q35/15` retains
private 32-byte context records at bootstrap-storage offsets 80, 112, and 144.
The three 16 KiB stacks begin at offsets 4096, 20480, and 36864 and fit within
the unchanged 64 KiB bootstrap pool. Root-object revision
`topal.systems-root-object.x86_64/15` has 799 typed relocations. Artifact
revision `topal.systems-artifact.x86_64-qemu-pc-q35/15` records 46 provider
operations and a 119-transition semantic trace.

## Evidence

- semantic tests cover block-first, signal-first persistent readiness, exact
  dispatcher identity, wrong completion identity, incompatible obligations,
  redispatch, retirement, reclamation, and final completion consumption;
- source tests cover the exact third handler, completion create/block/signal/
  consume forms, capacity-three FIFO order, and all affine cleanup;
- compiler tests cover the completion provider plan, three context records and
  stacks, generated entry symbol, typed relocations, semantic trace, and
  deterministic publication; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical evidence:
  linked kernel
  `ddd23d390d4858c1985e1b9a30ab534bd8cdc60eaa50751a134a89544d685325`,
  boot image
  `331d05b887ea46052a17332cc55772895afd9a7874abdaa5fb7d63bec190aaf5`,
  artifact provenance
  `73cd553912b7d9222932abd142360426bc804791bb8f2c6541de7b7244e8131a`,
  boot provenance
  `63c771fed5d0dccce0147079c580c7ca05c0255885104571dbd0d7e9b372edb5`,
  and serial observation
  `df9bab87ebe50a9580c90d9d053dd3ab9abe73191f137a3b3c500a6c4f06cc4d`.

Gate schema `topal-kernel-toolchain-gate-qemu/17` records 21 ordered markers.
They prove completion-worker entry before terminal execution, terminal
retirement before signalling and awakened redispatch, and worker continuation
after the wait before final retirement. QEMU remains live and serial output is
quiescent after the fatal disposition.

## Remaining boundary

This increment remains single-processor and single-address-space with one
waiter and one signal. It does not add multiple waiters, repeated signals,
predicate wait queues, spurious wakes, timeouts, cancellation, process
signals, priorities, cross-processor wakeup, SMP, user contexts, or a
provider-owned scheduler.
