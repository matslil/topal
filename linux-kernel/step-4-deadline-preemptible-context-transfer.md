# Step 4 deadline-preemptible context-transfer increment

## Outcome

The checked kernel root now binds its first source-selected runnable context to
one absolute deadline and exact dispatcher continuation. The worker enters a
sealed preemption region; the deadline handler completes the event, suspends
the interrupted worker, and returns to that dispatcher. Source takes the
resulting affine suspended context, re-enqueues it behind the terminal worker,
and later selects it again. Redispatch restores the retained interrupt
continuation, after which the worker resumes and retires normally. Both stacks
and the empty runnable queue are consumed before fatal disposition.

The architecture-independent model exposes no interrupt frame, register set,
stack pointer, vector, timer route, or instruction. It retains context, caller,
deadline-event, queue, and stack identities and rejects wrong bindings,
premature take or restore, repeated preemption, ordinary return from the sealed
region, and disposition with live affine obligations.

## Initial x86-64 lowering

Provider revision `topal.provider.x86_64-qemu-pc-q35/14` combines the existing
HPET/I/O-APIC one-shot event with private context state. The sealed await
operation admits interrupts only across an atomic enable-and-halt window. The
deadline entry saves the complete interrupt register image, completes the
device event, saves its handler stack as the worker continuation, and restores
the dispatcher stack. Selecting that worker later restores the handler stack;
the generated entry returns through the retained interrupt frame to the
instruction after the wait. Source still owns every runnable-queue decision.

Provider-object revision `topal.provider-object.x86_64-qemu-pc-q35/14` adds
typed await and preempt-current functions. Root-object revision
`topal.systems-root-object.x86_64/14` has 636 typed relocations, including one
deadline arm, one completion, three ordinary context transfers, one sealed
await, and one interrupt-driven preemption. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/14` records 45 provider operations
and a 97-transition trace covering deadline binding, interruption, dispatcher
resumption, affine take, queue re-enqueue, and exact restoration.

## Evidence

- architecture-neutral tests cover one-shot preemption/restoration and reject
  identity, ordering, and repeated-transition violations;
- source tests cover the exact typed handler, sealed await region, preempting
  disposition, affine take, queue order, marker order, and final reclamation;
- compiler tests cover provider mappings, exact generated sequences, typed
  dependencies, relocation counts, semantic trace, and deterministic artifact
  publication; and
- two pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs produced identical evidence:
  linked kernel
  `e8512257a3bba4c87d87cd6f9ca36b2a85024c0398b5da9e0b837dcaa469f0b8`,
  boot image
  `b6fa9e64c48c03cc39984ebcc21c6b044f335f972318d5b051a0e1f5b23534b0`,
  artifact provenance
  `d9ae5fee9929e67b4ac0bc7df26d3feab8c7519ab39bac102aaa75f22bef3a29`,
  boot provenance
  `32c7b70bc8886a682e8c1dc56722c2071c25a9d04c376f3c7229d73af7498d43`,
  and serial observation
  `23e4699bcd61274c3b862e3a3f55f85ef2c53e74a55256c5976e38c6c37bf5b8`.

Gate schema `topal-kernel-toolchain-gate-qemu/16` records eighteen ordered
markers. Their order proves worker entry before deadline delivery, dispatcher
execution after preemption, terminal-worker completion before redispatch, and
continuation of the preempted worker before its retirement. QEMU remains live
and the serial stream remains quiescent after the final fatal disposition.

This is high-risk interrupt, affine-ownership, and continuation work. Review
combines authority-ordered design propagation, x86/AArch64/RISC-V pressure
testing, negative state-machine tests, exact machine-code and ELF inspection,
deterministic publication, two physical QEMU runs, and full workspace
validation.

## Remaining boundary

This increment admits one processor, one address space, one deadline, one
registered dispatcher, and one preempted worker. It does not add general
blocking or wakeup, multiple timers, periodic scheduling, priorities,
timeslices, cancellation, migration, SMP ownership, user contexts,
floating-point/vector or debug-state ownership, TLS/per-CPU switching, guard
pages, stack growth, or cross-transfer unwinding.
