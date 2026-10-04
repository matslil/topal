# Proposed Topal-native kernel architecture

This is the approved Step 2 kernel design. Stable `TK-KERNEL-*` identities
describe kernel responsibilities. Authoritative Topal meaning is defined by
the repository-root systems-profile design, requirements, and specification;
the originating element analysis remains in
[the systems-profile design record](kernel-elements.md).

## TK-KERNEL-ARCH-001 — Design objective

The kernel is a new, capability-oriented monolithic kernel whose observable
native x86-64 behavior is compatible with Linux 7.2.9. Linux source structure
is research input, not an implementation template. Compatibility adapters
preserve Linux ABIs while internal interfaces use typed values, explicit
resources, effects, state transitions, and failure domains.

“Monolithic” describes deployment and privileged address-space placement, not
ambient authority. Core services and drivers may call each other directly only
through checked interfaces and granted capabilities. The first implementation
uses one kernel image because Linux compatibility, device throughput, and the
current toolchain do not justify imposing an IPC boundary between every
subsystem.

The design has five invariants:

1. no source assembly, instruction strings, clobber lists, raw register
   allocation, or generic privileged intrinsic;
2. no numeric address, descriptor, process ID, interrupt number, or device
   property grants authority merely because its bits are known;
3. every externally supplied byte sequence is validated through a selected
   layout/protocol before it becomes an internal semantic value;
4. every mutable or asynchronous resource has one owner or a checked shared
   synchronization protocol and an explicit teardown path; and
5. Linux-visible behavior is translated only at compatibility boundaries;
   internal errors and states are not forced into errno-shaped integers.

## TK-KERNEL-LAYERS-001 — Layered dependency direction

The image is organized into layers whose dependencies point downward:

| Layer | Responsibility | May depend on |
| --- | --- | --- |
| artifact and target | freestanding image, sections, relocation, entry stubs, target evidence | compiler/backend only |
| machine substrate | boot state, CPUs, traps, MMU, interrupts, time, context transfer | artifact/target and architecture providers |
| kernel core | storage, scheduler, synchronization, credentials, object identities, observability | machine substrate |
| resource frameworks | VFS, network, IPC, device, power, firmware, driver services | kernel core and granted machine services |
| Linux personality | syscalls, ELF, signals, UAPI protocols, pseudo-filesystems, vDSO | resource frameworks and kernel core |
| board and drivers | runtime discovery and concrete device protocols | resource frameworks and machine substrate capabilities |

The Linux personality is not beneath every internal object. It is an adapter
from Linux 7.2.9 representations and operations to kernel services. A native
internal caller does not construct a fake syscall or ioctl to use a service.

Architecture-specific implementations live behind semantic machine-provider
interfaces. Common code may request “activate this validated address space” or
“invalidate these translations in this scope”; it may not request a control
register write by number. Operations with no honest common meaning, such as
x86 port I/O, remain in a target-qualified provider.

## TK-KERNEL-BOOT-001 — Construction graph and capability refinement

Boot is a checked construction graph, not a bag of globally ordered init
functions. Each phase consumes capabilities from the preceding phase and
returns a more capable state:

1. **Entered:** validated boot handoff, one CPU, bounded bootstrap storage, and
   a backend-created boot context;
2. **Memory described:** firmware memory input is validated and reserved ranges
   are separated from allocatable physical frames;
3. **Kernel mapped:** final kernel mappings, fault entry, and safe translation
   operations are active;
4. **Core local:** allocation, per-CPU state, interrupt controller, monotonic
   clock source, and scheduler foundations exist on the bootstrap CPU;
5. **Multiprocessor:** secondary CPUs have validated per-CPU state and enter
   the scheduler through the typed secondary-entry path;
6. **Discovered:** firmware and buses have produced device/resource identities;
7. **Mounted:** required storage, VFS, root filesystem, and initial namespace
   exist; and
8. **Userspace:** the first Linux process image is validated and entered.

Constructors declare their required phase and resources. Cycles, missing
providers, and use of a capability before its phase are build-time errors when
static and boot diagnostics otherwise. A failed optional device does not roll
back the core; a failed required root dependency produces a structured boot
failure before the userspace transition.

## TK-KERNEL-EXEC-001 — Execution-context taxonomy

The kernel distinguishes semantic execution contexts rather than exposing
target stack frames:

| Context | May block | General allocation | May be preempted | Principal completion |
| --- | --- | --- | --- | --- |
| boot/secondary entry | no until scheduler phase | phase-qualified | no | refine boot state |
| process/kernel thread | yes | yes | policy-qualified | schedule or return |
| syscall entry | yes after full entry | yes | policy-qualified | return/signal/reschedule |
| synchronous user fault | conditionally | conditionally | policy-qualified | resume, signal, or terminate |
| synchronous kernel fault | only inside declared recovery boundary | no by default | no by default | recover or fatal |
| hard interrupt | no | bounded interrupt-safe only | no | acknowledge and return/defer |
| deferred work | yes when its queue permits | yes | yes | complete/cancel/reschedule |
| NMI/machine-critical | no | no | no | minimal record/recover/fatal |

Each entry handler receives an opaque context view and only capabilities legal
in that row. A source handler cannot cast one context into another. Deferring
work transfers a typed payload and resource obligation into a scheduler-owned
queue; it does not make blocking legal in the original interrupt context.

A Linux process, thread, and scheduler entity are kernel resources and are not
ordinary Topal `Task` values. Existing Topal tasks remain useful for selected
serialized services, but their serial event processor, structured
cancellation, and schedule-equivalence rules cannot model involuntary CPU
preemption, signal delivery, or a hard interrupt without the proposed systems
profile.

## TK-KERNEL-STATE-001 — State, ownership, and synchronization

Immutable semantic values remain the default at interfaces. Kernel state is
held by resource objects with explicit identities and protocols:

- exclusively owned objects permit in-place representation reuse while their
  owner capability is live;
- ownership may move through queues or lifecycle states;
- read-mostly snapshots publish immutable versions;
- shared mutable locations require a synchronization-domain capability and an
  operation supported by that domain; and
- interrupt, DMA, userspace, and device access remain separate effect and
  ordering domains even when they touch the same physical bytes.

The systems profile proposes checked atomic locations and scoped exclusion.
Mutexes, spin-style exclusion, sequence counters, reference counts, epoch/RCU
reclamation, wait queues, and completions are libraries with verified
implementations over those minimal elements. The language does not standardize
one universal lock or expose target fence instructions.

Every shared object states:

- the synchronization domain and protected invariant;
- which contexts may operate on it;
- whether an operation may block or retry;
- ordering/progress evidence and target assumptions;
- lifetime and reclamation protocol; and
- behavior during cancellation, process exit, device removal, and fatal error.

## TK-KERNEL-MEMORY-001 — Address spaces and mappings

Address identity is separated from numeric representation. The initial design
uses distinct opaque families for physical frames, kernel virtual locations,
user virtual locations qualified by an address-space identity, MMIO locations,
DMA addresses qualified by a device/IOMMU domain, and firmware source ranges.

A mapping resource records source and destination ranges, page layout,
permissions, cache policy, owner, lifetime, and translation provider. Page
tables are edited through an exclusive builder; activation and invalidation
consume validated mapping evidence and architecture-provider evidence. A
numeric equality between two address families conveys no alias or access
authority.

User pointers are untrusted Linux ABI values, never Topal references. A
user-transfer operation combines the current process address-space capability,
direction, maximum extent, selected UAPI layout, and a recoverable fault
boundary. It returns copied/validated data, partial progress where the Linux
contract permits it, or a structured fault. Kernel code cannot directly load
through a user pointer.

Executable mappings additionally connect VFS state, page-cache identity,
credentials, W^X/security policy, invalidation, and instruction visibility.
These relationships are kernel services, not properties encoded in a pointer.

## TK-KERNEL-SCHED-001 — Threads, scheduling, and asynchronous work

The scheduler owns CPU execution capabilities and suspended thread contexts.
Only the backend-qualified context-transfer operation can consume one running
context and activate one validated suspended context. Ordinary code cannot
name a stack pointer or continuation register set.

The core model distinguishes:

- `Process`: address space, credentials, namespaces, descriptor table, signal
  group, limits, and child/wait relationships;
- `Thread`: scheduling state, signal mask, TLS/user context, kernel stack, and
  process membership;
- `Cpu`: current thread, per-CPU storage, interrupt/preemption state, and
  scheduler run state; and
- `Work`: a bounded or cancellable deferred operation with its resource
  obligations and context requirements.

Blocking is a state transition that atomically connects the protected
condition, wait registration, and scheduler disposition. Wakeup changes a
waiter's state through its exact wait identity; it is not an untyped hint.
Timeout, signal interruption, process exit, and device removal are explicit
competing transitions so a waiter cannot be resumed twice.

## TK-KERNEL-OBJECT-001 — Kernel objects and Linux handles

Kernel objects have unforgeable internal identities. Linux-visible integers
and names are namespace-local handles resolved at the boundary:

- a file descriptor maps to an open-file-description capability;
- a PID maps through a PID namespace to process/thread observation authority;
- a path resolves through a mount and name namespace under credentials;
- an interface index, port, inode number, device number, or IPC identifier is
  interpreted only by its owning namespace/service; and
- KVM, BPF, io_uring, perf, and device handles carry protocol state in their
  owning kernel object rather than in the integer.

Lookup grants only the rights specified by the Linux operation and current
credentials. Closing or removing a handle updates the namespace mapping but
does not destroy an object still retained by another reference. Reclamation
occurs after the owning lifetime/epoch protocol proves no access remains.

## TK-KERNEL-UAPI-001 — Linux compatibility personality

The native x86-64 personality contains:

1. a backend-generated syscall entry adapter implementing
   `LK-X64-SYSCALL-001`;
2. a number dispatcher generated from the pinned syscall ledger;
3. per-operation decoders that validate Linux 7.2.9 flags, structures,
   pointers, lengths, time forms, and subordinate commands;
4. internal service calls using semantic values and capabilities;
5. result encoders implementing partial progress, interruption/restart, errno,
   output layout, and side effects; and
6. signal/ptrace/audit/seccomp observation at their specified commit points.

The same pattern applies to ioctl, Netlink, socket options, BPF, perf,
io_uring, KVM, and filesystem-shaped interfaces. A transport number selects a
protocol; it does not define that protocol. Unknown flags, reserved fields,
size negotiation, extension rules, privilege, and configuration are validated
before internal authority is granted.

ELF loading, initial stack construction, vDSO/vvar, TLS, signals, ptrace, and
core state share one `LinuxX86_64ProcessAbi` adapter. This prevents independently
implemented paths from disagreeing about register, extended-state, auxiliary
vector, or return validation.

## TK-KERNEL-SECURITY-001 — Credentials and policy

Credentials are immutable snapshots replaced through checked transitions.
Operations receive a subject credential and object capability explicitly or
through a process execution context. Namespace views, capabilities, resource
limits, seccomp, audit, and later LSM decisions remain distinct policy inputs.
One successful check cannot be reused as blanket authority for another object
or after a credential transition.

Linux-compatible discretionary checks and capability results are external
requirements. The internal design may additionally use Topal information-flow
and authority evidence, but it must not expose behavior that contradicts the
pinned Linux personality. Security-relevant denial, partial completion, and
audit ordering are part of each operation contract.

## TK-KERNEL-DRIVER-001 — Devices and drivers

The device framework implements the identities and lifecycle in
`LK-DRIVER-*`. Discovery creates a description and resource candidates;
binding selects a driver; successful probe returns a live device session plus
published functional endpoints. Failed probe returns no endpoints and unwinds
every acquired resource.

MMIO, port I/O, interrupts, DMA, clocks, resets, firmware properties, and bus
configuration are narrow capabilities. DMA buffers move through explicit CPU,
prepared, device, completed, and reclaimed states. Device removal first closes
admission, then masks observations, drains/cancels work and transfers, updates
published endpoints, and finally releases resources.

ACPI, Device Tree, PCI, and architecture providers retain their provenance.
Binding-specific decoders may produce a common resource request, but a driver
cannot access an arbitrary firmware property or physical address.

## TK-KERNEL-FAILURE-001 — Failure containment

Expected failures use typed results internally and translate to Linux errno or
protocol-specific output only at the compatibility boundary. Faults are
classified as:

- user input/access faults, contained by the current operation;
- recoverable machine faults inside an explicitly declared recovery boundary;
- device/protocol failures owned by a device session;
- process-fatal failures translated to signal/exit behavior; and
- kernel invariant or unrecoverable machine failures, which enter the minimal
  fatal path.

The fatal path requires no allocation, lock acquisition, general scheduler, or
filesystem. It records bounded diagnostics when a qualified sink is available,
stops other CPUs under a target-defined protocol, and halts/reboots according
to board policy. It cannot masquerade as an ordinary `Result` continuation.

## TK-KERNEL-OBSERVE-001 — Testing and observability

Every external boundary has a trace identity independent of Linux internal
function names. Differential tests compare return values, output bytes,
partial progress, signals, filesystem/protocol observations, ordering, and
discovery behavior against the pinned Linux VM.

Trace, debug, and crash facilities are capability-gated and bounded. A disabled
facility must not change application behavior. Architecture artifact tests
inspect entry stubs, sections, relocations, undefined symbols, stack rules,
privileged instruction placement, and absence of host-Linux runtime calls.
