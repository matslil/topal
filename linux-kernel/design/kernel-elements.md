# Proposed Topal systems profile and kernel elements

This document records the minimal Topal changes approved for the architecture
in [the kernel design](topal-kernel-architecture.md). Authoritative language
meaning is now defined in `docs/systems-profile.md`, the
`TOPAL-REQ-SYSTEMS-*` requirements, and `spec/systems-profile.md`. Illustrative
names here remain explanatory where the authoritative records do not expose a
surface name.

## TK-SYSTEMS-001 — Separate, capability-gated profile

The recommended design is a selected `systems` language feature available only
to a freestanding root artifact. It extends, rather than weakens, portable
Topal:

- ordinary values remain immutable semantic values;
- layouts remain separate from values;
- effects, failures, resource identity, and authority remain explicit;
- inability to prove memory safety or race freedom remains rejection;
- an architecture model supplies facts but never runtime authority; and
- ordinary packages cannot mint machine, entry, address-space, interrupt,
  atomic, or artifact capabilities.

The profile admits external concurrency and privileged state only through the
closed elements below. It does not add `unsafe`, raw pointers, arbitrary
intrinsics, source assembly, foreign register variables, or a promise that any
numeric address can be dereferenced.

Selection changes the artifact and available sealed vocabulary, not the
meaning of existing portable declarations. A package using the profile cannot
also pretend to be an ordinary Linux process artifact.

## Semantic delta from current Topal

| Current rule/design | Kernel pressure | Proposed narrow extension |
| --- | --- | --- |
| architecture descriptions are authority-free | kernel must execute privileged operations | sealed runtime machine capabilities supplied only by entry/provider construction |
| source exposes no atomics or locks | scheduler, interrupt, reference, and lock-free protocols share state | typed atomic locations and scoped exclusion with a formal memory relation |
| Topal tasks own serial event state | hardware events and preemption arrive involuntarily | typed special-entry contexts and external observations |
| structured task cancellation is cooperative | signal, fault, process exit, and CPU reschedule alter execution | scheduler-owned suspended contexts and explicit competing transitions |
| layouts/locations cover addressed storage | page tables and user pointers involve distinct address spaces and recoverable faults | sealed address-space capabilities, mappings, and fault-contained transfer |
| compiler publishes a Linux process static PIE | kernel is loaded by firmware/bootloader and provides syscalls | freestanding kernel artifact profile with placement and special roots |
| fatal process termination uses host platform | kernel failure cannot call a host OS | nonreturning target-qualified fatal disposition |

## TK-ELEMENT-ENTRY-001 — Typed special entry

A special entry declaration is not an ordinary callable function. It names one
closed entry kind, target profile, input contract, legal execution context,
capability bundle, and disposition set. Initial kinds are:

- bootstrap CPU entry;
- secondary CPU entry;
- synchronous exception;
- external interrupt;
- syscall from user mode;
- non-maskable/machine-critical event; and
- resumed kernel thread.

The backend owns the physical symbol, section, alignment, stack transition,
saved registers, speculation/security sequence, unwind metadata, and return
instruction. Source receives an opaque typed context whose accessors expose
semantic facts such as cause, user instruction location, syscall arguments, or
fault address only when that entry kind defines them.

The handler returns one member of its closed disposition set, for example
`ResumeInterrupted`, `ReturnToUser`, `Schedule`, `DeliverSignal`, or `Fatal`.
The compiler verifies that required acknowledgement, context validation,
resource discharge, and mask/preemption state hold for that disposition. An
entry context cannot escape, be stored, cross a message boundary, or be
constructed in source.

Required implementation evidence includes target entry mechanism, exact
hardware-created state, stack rules, nested-entry policy, context layout,
permitted return paths, and artifact inspection. On x86-64 the backend may use
`SYSCALL`, IDT gates, IST, `SYSRET`, and `IRET`; those names do not occur in the
portable semantic contract.

## TK-ELEMENT-OBSERVATION-001 — External events and protocol choice

The systems profile admits nondeterminism only through a declared observation
or concurrent protocol. Interrupt arrival, device completion, clock progress,
user input, another CPU's ordered operation, and scheduler-visible wake events
each name a source capability, value/error layout, ordering constraints, and
trace identity.

When concurrently enabled transitions have no required order, the owning
protocol may observe any permitted linearization. Every outcome must preserve
its invariant and satisfy its external contract; replay evidence records the
selected events and linearization. This extends the current portable
schedule-equivalence rule only inside the systems profile's declared domains.
It does not add a general random choice, make data races meaningful, or allow
an optimizer to invent an observation.

Portable functions called by the kernel retain their existing deterministic
meaning. Variation becomes observable only when their inputs include one of
these declared observations or protocol results. Differential tests compare
the set and constraints of Linux-permitted outcomes rather than requiring one
fixed interleaving.

## TK-ELEMENT-MACHINE-001 — Sealed machine-provider operations

Privileged operations are closed typed provider interfaces, not a universal
`intrinsic` call. A provider operation states:

- semantic state consumed and produced;
- required privilege and CPU/device identity;
- execution-context and interrupt/preemption restrictions;
- fault behavior and recoverability;
- ordering and visibility relation;
- target evidence required for lowering; and
- model/interpreter behavior, which may be a checked abstract transition.

Portable operation families include installing a validated exception table,
activating an address space, invalidating translations in a named scope,
querying a qualified CPU identity/feature, routing an interrupt, waiting under
a declared wake contract, and making instruction changes visible.

Target-qualified families retain real differences: x86 port I/O and MSR state,
AArch64 system-register facilities, RISC-V CSR/SBI facilities, and later
VMX/SVM/EL2/hypervisor-extension operations. A target package may implement a
declared operation, but ordinary source cannot define a new operation or its
lowering.

## TK-ELEMENT-ADDRESS-001 — Address-space and mapping identities

The systems profile supplies sealed address families parameterized by owning
resource identity. Construction comes only from validated boot/firmware input,
allocation, mapping, bus resources, or another checked derivation. Required
families are physical, kernel virtual, user virtual, device/MMIO, DMA/IOMMU,
and firmware source.

An address range carries width and overflow-checked bounds. A location adds a
layout, access rights, lifetime, access sizes/alignment, cache policy, and
ordering domain. A mapping is a linear resource relating ranges under page
size, permission, ownership, and translation-provider evidence.

Operations permit contained offset/range derivation, mapping construction,
activation, protection change, unmapping, and translation invalidation. There
is no general integer-to-address conversion. A boundary decoder may create an
untrusted address candidate; only validation against a live address-space or
device resource creates a usable capability.

## TK-ELEMENT-USER-ACCESS-001 — Fault-contained user transfer

User access is a primitive boundary because a valid-looking user range can
fault or change mappings while the kernel operates. A transfer consumes:

- the current user-address-space observation capability;
- an untrusted candidate range and maximum byte extent;
- direction and selected layout/byte policy;
- a destination/source kernel-owned region; and
- an interruption and partial-progress policy derived from the Linux contract.

It returns validated data or a structured outcome containing fault and allowed
partial progress. The compiler/backend establishes the architecture-specific
access-enable and recovery region; source cannot install an arbitrary fault
resume address. Nested kernel faults outside such a region remain fatal.

The same abstraction covers x86 SMAP access control, AArch64 PAN/UAO choices,
and RISC-V SUM-related access without making any one control bit portable.

## TK-ELEMENT-ATOMIC-001 — Typed atomic locations

Kernel-safe shared state requires a source-visible operation stronger than the
current compiler-synthesized synchronization model. The proposed element is an
`AtomicLocation` over one fixed, target-supported scalar or tagged state layout
and one synchronization-domain identity. It is created only from exclusively
owned suitably aligned storage and consumed before that storage is released.

The closed operations are atomic load, store, exchange, compare/exchange, and
target-supported read-modify-write functions. Each operation states one of
these semantic orders:

| Order | Contract |
| --- | --- |
| atomic-only | indivisible access and one per-location modification order; no unrelated visibility edge |
| acquire | atomic-only plus observations after success occur after the release sequence observed |
| release | atomic-only plus prior observations become visible before a successful observer acquire |
| acquire-release | both relations for a modifying operation |
| sequential | acquire-release plus participation in one global order for sequential operations |

Compare/exchange separately declares its success and failure order; failure
cannot release. The operation also declares compiler, CPU, device, and DMA
domains involved. CPU atomics do not imply MMIO or DMA ordering.

Atomic operations permit nondeterministic winner selection only inside a
declared protocol whose every transition preserves its invariant. They do not
make data races defined: non-atomic conflicting access remains rejected, and a
mixed atomic/plain location is invalid unless a proved ownership transition
ends atomic access first.

The compiler selects x86 locked operations, AArch64 LSE or LL/SC loops, or
RISC-V A-extension/LR-SC implementations from qualified target evidence. A
fallback lock is permitted only when it satisfies context, progress, and
recursion requirements. Source never selects the instruction strategy.

## TK-ELEMENT-CRITICAL-001 — Scoped interrupt and preemption control

Masking and preemption are scoped state transitions tied to the current CPU,
not Boolean flags. Entering a critical scope returns an affine restoration
token recording the exact previous state and domain; every exit path consumes
it once. Tokens cannot cross CPUs, suspend, escape, or be restored out of
nesting order.

Distinct operations cover scheduler preemption, maskable local interrupts, a
specific interrupt source, and target-defined stronger domains. Masking local
interrupts does not disable NMIs, other CPUs, DMA, or devices and therefore
does not by itself prove exclusive access. The protected protocol must state
which producers are excluded.

Locks whose acquisition may block are illegal in interrupt and machine-critical
contexts. Interrupt-safe exclusion composes one lock protocol with the exact
mask scope and restoration token rather than using a separate ad hoc primitive.

## TK-ELEMENT-FENCE-001 — Visibility and maintenance operations

Ordering is expressed by semantic relation and scope. Separate closed families
cover CPU memory fences, device/MMIO ordering, DMA ownership visibility,
instruction synchronization, cache maintenance, and translation-maintenance
completion. A generic “full barrier” is not the primary interface because it
would hide both excessive cost and missing domain semantics.

Each operation names the observations ordered, direction, participating
agents, topology scope, and required completion. Architecture evidence maps
that contract to zero or more instructions. A zero-instruction lowering is
valid only when the selected target guarantees the complete relation.

## TK-ELEMENT-CONTEXT-001 — Scheduler context transfer

A suspended execution context is an opaque linear resource created by special
entry, initial-thread construction, or a prior context transfer. It owns its
kernel stack, saved machine state, extended-state policy, thread identity, and
address-space relationship.

The transfer operation consumes the current running-context capability and one
validated suspended context, executes scheduler-defined per-CPU and address-
space transitions, and resumes exactly one continuation. It does not return in
the ordinary call sense; a later transfer may resume the old context at its
typed continuation point.

Source cannot inspect register slots or manufacture a context. Debug/ptrace
adapters use a separately validated semantic user-state view. Backend evidence
proves the callee-saved, stack, extended-state, TLS/per-CPU, unwind, and address-
space rules for the target.

## TK-ELEMENT-FAULT-001 — Recovery and fatal disposition

A fault-recovery scope admits only a closed set of expected synchronous faults
for one operation, such as user transfer or a probed device access. The backend
binds generated fault sites to generated recovery continuations; source sees a
typed `Result`, not an instruction address or exception-table entry.

Recovery scopes cannot catch arbitrary kernel invariant failures, NMIs,
machine checks, stack corruption, or faults in cleanup unless their own closed
contract explicitly covers them. An unrecoverable disposition is a typed
nonreturning transition to the kernel fatal provider and cannot be treated as
process termination.

## TK-ELEMENT-DEVICE-001 — Device register and DMA protocol

Existing Topal layouts and locations provide the representation foundation.
The systems extension adds device access semantics that bind each location to
a register protocol: permitted widths, read/write behavior, side effects,
reserved-bit policy, ordering, and device-session lifetime.

DMA uses linear typestate rather than volatile buffers:

```text
CpuOwned -> Prepared -> DeviceOwned -> Completed -> CpuOwned
                                   \-> Failed ----/
```

Transitions record direction, mapping, cache maintenance, descriptor
visibility, device notification, completion source, and unmap/reclaim rules.
IOMMU and bounce-buffer implementations may differ without changing the
protocol. A device cannot retain a mapping past the session that granted it.

## TK-ELEMENT-STORAGE-001 — Kernel allocation and placement

The artifact profile provides bounded bootstrap storage and later allocator
capabilities. Allocation is fallible, qualified by region/pool, alignment,
physical/virtual requirements, context legality, and reclaim policy. The
compiler and standard library must not call a host allocator or Linux syscall.

Static placement attaches semantic requirements to a declaration: entry text,
read-only data, mutable data, per-CPU template, bootstrap-reclaimable data,
page-aligned tables, or a board/boot format requirement. Source does not spell
ELF section names or linker scripts. The artifact provider selects concrete
sections and proves placement constraints in the published image.

## TK-ELEMENT-ARTIFACT-001 — Freestanding kernel artifact

The kernel target is a distinct qualified artifact profile, not the current
`x86_64-unknown-linux-gnu` application target. It has:

- no host syscall, libc, dynamic loader, process `_start`, or implicit runtime;
- a backend-generated set of special roots and entry adapters;
- a fixed target/data-layout/relocation/code-model identity;
- explicit image, debug, map, and provenance outputs;
- atomic publication only after link and structural validation; and
- a packaging stage that creates the exact Step 3 boot image from the linked
  kernel payload.

The boot protocol and final container are selected with the QEMU machine. The
semantic profile does not assume Linux bzImage, Multiboot, or UEFI; each is an
artifact adapter with its own entry and placement evidence.

## What remains ordinary Topal/library design

The following should not become language primitives:

- scheduler policy, run queues, timers, wait queues, and work queues;
- mutex, spin-style, sequence, reference-count, epoch/RCU, and completion
  algorithms built over the atomic/critical primitives;
- page allocators, slabs, caches, VFS objects, protocols, and filesystems;
- syscall, ioctl, Netlink, socket, BPF, perf, io_uring, and KVM dispatch;
- PCI, virtio, TTY, block, network, ACPI, and Device Tree logic; and
- Linux error numbers, flags, structures, namespaces, and credentials.

These are typed libraries and kernel modules because their behavior can be
stated using the elements above. Making them syntax or backend intrinsics would
freeze Linux structure into the language and make architecture or policy
evolution harder.

## Adoption and implementation order

1. approve or revise the semantic families and their boundary with libraries
   (complete after PR #790);
2. update authoritative `docs/` and `se/` with the selected systems profile;
3. add normative rules and traceability in `spec/`;
4. implement architecture-independent semantic checking and model behavior;
5. implement the x86-64 backend/provider and kernel artifact publisher;
6. add negative, model, artifact-inspection, QEMU, and differential tests; and
7. qualify only the implemented x86-64 profile while retaining AArch64 and
   RISC-V as design conformance cases.
