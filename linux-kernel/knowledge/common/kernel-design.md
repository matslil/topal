# Linux kernel design map

## LK-CORE-001 — Subsystem decomposition

The Linux implementation is a modular monolithic kernel. Its important design
boundaries for this project are:

- architecture entry, CPU discovery, traps, interrupts, and context switching;
- initialization, configuration, firmware, and device discovery;
- physical and virtual memory, allocators, mappings, reclamation, faults, and
  user access;
- tasks, threads, scheduling classes, signals, timers, synchronization, and
  deferred work;
- VFS, page cache, block I/O, filesystems, namespaces, and pseudo-filesystems;
- networking, sockets, protocol stacks, routing, filtering, and network devices;
- credentials, capabilities, namespaces, cgroups, LSM, audit, seccomp, and BPF;
- driver core, buses, device classes, power management, DMA, IOMMU, and IRQs;
- tracing, perf, debugging, crash handling, and observability; and
- KVM, vhost, VFIO, and other virtualization facilities.

These are research partitions, not mandatory Topal modules. The future design
maps responsibilities, state, authority, effects, and protocols before choosing
implementation boundaries.

## LK-CORE-BOOT-001 — Staged initialization

Linux boot crosses machine-defined entry state, architecture setup, early
memory and console setup, interrupt/time initialization, allocator and scheduler
availability, subsystem initialization, device probing, root filesystem
selection, and the first userspace process. Dependencies currently expressed
through Linux init-call ordering must become explicit Topal construction and
capability dependencies.

The Topal kernel must state which operations are legal in each boot phase and
how capabilities are refined. A subsystem cannot acquire allocation,
scheduling, interrupt, device, or filesystem authority merely because a global
initialization function happened to run earlier.

## LK-CORE-MM-001 — Memory management

The required responsibilities include firmware memory discovery, reserved
ranges, page-frame ownership, bootstrap and steady-state allocation, kernel
virtual mappings, per-process address spaces, page faults, copy-on-write,
file-backed mappings, page cache interaction, reclaim, shared memory, locking,
DMA/IOMMU constraints, executable mappings, and validated user access.

Topal semantic values cannot substitute for page and mapping identities. The
design needs explicit physical, virtual, user, kernel, device, and DMA address
spaces; mapping lifetimes and permissions; translation invalidation; faultable
access; and ownership transfer.

## LK-CORE-SCHED-001 — Execution, interrupts, and scheduling

Linux combines process/thread state, per-CPU execution, preemption, interrupt
and exception contexts, kernel threads, scheduler classes, wait queues,
completion, work queues, timers, RCU, and several lock/atomic mechanisms.

Topal must model external interrupts and timing as declared observations while
retaining defined behavior. Kernel-safe concurrency cannot depend on the current
portable-source assumption that compiler synthesis alone hides every atomic or
lock. The design must resolve preemption points, interrupt context, scoped
masking, per-CPU state, blocking legality, ordering, progress, reclamation, and
the relationship between kernel scheduling and Topal tasks.

## LK-CORE-VFS-001 — VFS and filesystem model

Linux VFS presents shared concepts for superblocks, mounts, dentries, inodes,
open file descriptions, file descriptors, paths, credentials, caches, and
filesystem operations while permitting filesystem-specific semantics. The
Topal design should retain explicit identities and protocols rather than copy C
operation tables mechanically.

External compatibility is driven by path resolution, mount/name namespace
behavior, descriptor semantics, stat data, permissions, caching observations,
locking, notification, memory mapping, and filesystem-specific UAPI. Internal
objects may differ if those observations and resource guarantees remain equal.

## Mapping method

For each subsystem, the Step 2 design records:

1. externally observable Linux contracts;
2. Linux internal responsibilities and dependency relationships;
3. accidental implementation structure that need not survive;
4. proposed Topal values, resources, effects, interfaces, protocols, and
   implementation plans;
5. architecture- or board-qualified providers;
6. error, cancellation, interruption, and teardown behavior; and
7. differential, conformance, stress, and fault-injection evidence.

