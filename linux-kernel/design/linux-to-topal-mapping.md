# Linux 7.2.9 to Topal-kernel mapping

This ledger maps Linux responsibilities and external contracts to the proposed
Topal-native architecture. It deliberately maps behavior and ownership, not C
types or function names. Stable `TK-MAP-*` identities survive an upstream
source move; their source evidence and compatibility obligations change when a
future baseline changes.

## Mapping record rule

Every detailed mapping derived from this ledger contains:

- stable mapping identity and owning Topal component;
- Linux 7.2.9 source/document/inventory evidence;
- externally observable contract and configuration conditions;
- Linux internal responsibility being understood, explicitly marked as
  non-normative implementation evidence;
- Topal values, resources, effects, capabilities, protocols, and lifetimes;
- architecture/board providers and failure/teardown behavior;
- subordinate interface identities and dependency mappings; and
- conformance, differential, stress, and artifact evidence.

One Linux source item may map to several Topal responsibilities, and several
Linux items may map to one Topal service. The ledger is therefore a traced
many-to-many relation rather than a directory translation plan.

## Architecture, boot, and execution

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-BOOT-001` | x86 image entry, boot parameters, early setup | artifact adapter validates the selected boot handoff; construction graph refines `Entered` through userspace-ready phases |
| `TK-MAP-CPU-001` | CPU discovery, feature enablement, per-CPU state | architecture provider supplies qualified facts; runtime CPU capabilities own per-CPU resources; common code never branches on compiler model values |
| `TK-MAP-ENTRY-001` | exception, interrupt, syscall, NMI entry/return | typed special entries and dispositions; backend owns physical frames and prologue/epilogue |
| `TK-MAP-SWITCH-001` | task context switch and extended state | scheduler owns opaque running/suspended contexts and invokes qualified context transfer |
| `TK-MAP-TIME-001` | clocksources, clock events, timers, timekeeping | typed clock/counter providers feed one kernel time service; Linux clock UAPIs adapt from that service |
| `TK-MAP-SMP-001` | CPU bring-up, IPIs, stop and synchronization | board/architecture CPU-start protocol plus routed interprocessor-event capabilities and explicit completion |
| `TK-MAP-FATAL-001` | panic, oops, machine failure, crash path | minimal nonreturning fatal provider separated from recoverable process/device errors |

Relevant Linux families include `arch/x86/boot/`, `arch/x86/kernel/`,
`arch/x86/entry/`, `kernel/time/`, `kernel/sched/`, and architecture interrupt
code. Their implementation organization is not copied.

## Memory and execution resources

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-PHYS-001` | firmware memory map, memblock, page ownership | validated physical ranges feed bootstrap and steady-state frame allocators with explicit reservations |
| `TK-MAP-VM-001` | page tables, VMAs, faults, COW, `mmap` | address-space service owns mappings and fault protocol; user-visible operations adapt Linux flags and results |
| `TK-MAP-USERCOPY-001` | `copy_*_user`, access checks, exception fixup | fault-contained typed user transfer; no direct user-pointer dereference |
| `TK-MAP-ALLOC-001` | slab/page/vmalloc and allocation contexts | pool/region capabilities state fallibility, context legality, alignment, address family, and reclaim policy |
| `TK-MAP-CACHE-001` | page cache, writeback, reclaim | VFS/memory shared page-object protocol with explicit dirty/writeback/reclaim states |
| `TK-MAP-SYNC-001` | atomics, spinlocks, mutexes, seqlocks, refcounts | minimal atomic/critical elements plus verified synchronization libraries scoped by context and protected invariant |
| `TK-MAP-RCU-001` | RCU publication and deferred reclamation | epoch-domain library with read participation, publication, grace completion, and lifetime evidence |
| `TK-MAP-WAIT-001` | wait queues, completions, futex foundation | exact wait identity connects condition transition, scheduler blocking, wake, timeout, signal, and cancellation winners |

Relevant Linux families include `mm/`, `include/linux/mm*.h`, allocator code,
`kernel/futex/`, `kernel/rcu/`, and synchronization headers/source. Linux macro
shape is not an API requirement.

## Processes, security, and isolation

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-PROCESS-001` | task/process/thread identity and lifecycle | separate `Process` and `Thread` resources with parent, group, scheduler, signal, and namespace protocols |
| `TK-MAP-ELF-001` | native ELF exec and initial process ABI | `LinuxX86_64ProcessAbi` validates/maps image, interpreter, stack, auxv, TLS, vDSO/vvar |
| `TK-MAP-SIGNAL-001` | dispositions, masks, frames, restart | signal service owns pending state; ABI adapter constructs/validates frames and return dispositions |
| `TK-MAP-PTRACE-001` | ptrace/regsets/core state | debugger capability mediates stopped-thread semantic views and exact Linux layouts |
| `TK-MAP-CRED-001` | credentials, Linux capabilities, permission checks | immutable credential snapshots and object-specific policy calls at operation commit points |
| `TK-MAP-NS-001` | user, mount, PID, network, IPC, UTS, cgroup, time namespaces | typed namespace resources provide views and local handle resolution; no universal namespace map |
| `TK-MAP-CGROUP-001` | cgroup v2 hierarchy/controllers/accounting | hierarchy and controller protocols attach resource policy/accounting to process groups |
| `TK-MAP-SECCOMP-001` | syscall filtering and observation | Linux personality evaluates selected filter before service authority and records exact result path |
| `TK-MAP-LSM-001` | security hooks and audit interactions | later policy-provider interface at named semantic decisions; no replication of scattered C hook calls required |

## Files, I/O, and IPC

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-FD-001` | fd tables and open file descriptions | namespace-local integer lookup returns rights-limited handle to a reference-counted open description |
| `TK-MAP-VFS-001` | mounts, paths, dentries, inodes, superblocks | VFS services use separate namespace, cache identity, persistent object, open-description, and filesystem-provider resources |
| `TK-MAP-POLL-001` | poll/epoll readiness and wakeup | readiness subscriptions have exact source, generation, edge/level policy, wake, close, and removal states |
| `TK-MAP-AIO-001` | AIO and io_uring submission/completion | protocol-owned rings/requests validate shared layouts, ownership, registration, cancellation, and completion once |
| `TK-MAP-PIPE-001` | pipes, splice-family transfer | bounded buffer and transfer protocols preserve partial progress, blocking, signal, and descriptor semantics |
| `TK-MAP-IPC-001` | SysV/POSIX IPC and shared memory | namespace-owned objects and mappings with Linux permission/lifetime/notification adapters |
| `TK-MAP-FS-001` | concrete filesystems and pseudo-filesystems | provider interfaces implement VFS protocols; pseudo-files are generated semantic views with path-specific mutation rules |

## Networking and programmable interfaces

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-SOCKET-001` | socket families/types/protocols/options | socket object carries protocol state; Linux layouts/options adapt to typed network services |
| `TK-MAP-NETDEV-001` | network devices, queues, NAPI, namespace view | device session and packet-ownership protocol feed per-namespace network service |
| `TK-MAP-NETLINK-001` | Netlink families and TLV policies | generated per-family decoders/encoders and command state machines, not one generic untyped message path |
| `TK-MAP-ROUTE-001` | addressing, route/neighbour, filtering | immutable/published tables plus synchronized update services under namespace and credential identities |
| `TK-MAP-BPF-001` | BPF command API, verifier, maps, programs, hooks | Linux adapter validates versioned command layouts; verifier produces bounded typed execution evidence before attachment |
| `TK-MAP-PERF-001` | perf events, rings, counters, sampling | capability-gated event objects and shared-ring protocol tied to architecture/provider evidence |

## Devices, firmware, and power

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-DEVICE-001` | driver core device/driver/bus/class | distinct description, device, bus membership, driver, class endpoint, and resource identities |
| `TK-MAP-PROBE-001` | match, probe, deferred probe, remove | dependency-aware binding transaction returns a complete live session or rolls back every acquisition |
| `TK-MAP-FWNODE-001` | ACPI/OF property abstraction | common typed queries retain ACPI/DT provenance and binding-specific validation |
| `TK-MAP-DT-001` | FDT parse, bindings, address/IRQ translation, population | immutable validated tree plus binding decoders and explicit population policy from `LK-DT-*` |
| `TK-MAP-ACPI-001` | ACPI tables, namespace, resources, control methods | validated table/namespace service with narrowly authorized method/provider operations |
| `TK-MAP-PCI-001` | PCI enumeration/resources/capabilities/MSI/error/power | PCI bus service grants function, BAR, DMA, interrupt, reset, and configuration capabilities |
| `TK-MAP-IRQ-001` | IRQ domains, descriptors, handlers, affinity | routed interrupt capabilities join controller source, vector, handler context, acknowledgement, and lifecycle |
| `TK-MAP-DMA-001` | DMA API, scatter/gather, IOMMU | linear DMA ownership/mapping protocol qualified by device and translation domain |
| `TK-MAP-POWER-001` | device links, runtime/system PM, reset, clocks | supplier/consumer graph and explicit quiesce/suspend/resume/reset state protocols |
| `TK-MAP-VIRTIO-001` | virtio transport, features, queues, devices | transport session negotiates a typed protocol before derived virtqueues/function endpoints exist |
| `TK-MAP-TTY-001` | console, TTY, termios, line discipline | serial device session feeds TTY state machine and Linux terminal interface |
| `TK-MAP-BLOCK-001` | block queues, partitions, flush/discard/error | request/queue protocol with buffer ownership and exact completion; publishes block endpoint |

## Linux personality and virtualization profiles

| ID | Linux responsibility/contract | Topal owner and mapping |
| --- | --- | --- |
| `TK-MAP-SYSCALL-001` | 385 native table rows and subordinate protocols | generated number dispatcher plus reviewed per-operation adapters and service calls |
| `TK-MAP-UAPI-001` | installed headers/layouts/flags/ioctls | generated identity ledger and explicit validated layouts; headers do not become internal types |
| `TK-MAP-ABI-FS-001` | `/proc`, `/sys`, `/dev`, cgroupfs and other views | path-ledger providers over live kernel objects, permissions, namespaces, poll, and mutation protocols |
| `TK-MAP-VDSO-001` | vDSO/vvar symbols and data update | ABI publisher builds mapped ELF/data views from time/CPU providers with fallback syscalls |
| `TK-MAP-CONTAINER-001` | Docker/Podman prerequisites | acceptance profile over namespaces, cgroup v2, seccomp, mounts/storage, networking, capabilities, terminals, and pseudo-filesystems |
| `TK-MAP-QEMU-001` | QEMU software-emulation host | application profile over process, memory, file, signal, time, threading, and device interfaces; independent of KVM |
| `TK-MAP-KVM-001` | `/dev/kvm` fd/ioctl/mmap protocol | separate virtualization service with typed VM/vCPU/memory-slot/routing resources and x86 provider |

## Linux patterns intentionally not copied

| Linux implementation pattern | Topal replacement |
| --- | --- |
| embedded common C structures and container recovery | explicit composition and identities |
| ambient globals and init-call ordering | constructed contexts and checked boot dependency graph |
| integer handles passed internally | typed resources; integers exist only at UAPI/bus boundaries |
| callback operation tables without protocol state | typed interfaces plus lifecycle/typestate evidence |
| error-pointer encoding and pervasive negative errno | internal `Result`; encode Linux result only at boundary |
| unchecked casts, flexible arrays, and raw user pointers | selected layouts, bounds, untrusted candidates, validated transfers |
| manual cleanup labels | owned acquisitions and explicit rollback/teardown protocols |
| ad hoc flag words internally | closed semantic variants/sets; exact bit layouts at UAPI/device boundaries |
| architecture macros and inline assembly | sealed machine providers and backend-generated entry/artifact support |
| configuration removing untracked interfaces | explicit conditional dispositions in the compatibility ledger |

These replacements are not promises that cleanup or concurrency becomes
automatic. Hot removal, partial I/O, restart, and external disappearance still
require explicit protocol transitions.

## Baseline update procedure

When moving from Linux 7.2.9 to a future release:

1. regenerate source inventories and identify added, removed, or changed
   entries and file digests;
2. update runtime probes for the new pinned configuration and board;
3. classify each change as external-contract drift, internal-only source drift,
   configuration/profile drift, or extractor drift;
4. attach external drift to existing `TK-MAP-*` identities or create a new
   stable mapping when the responsibility is genuinely new;
5. revise Topal protocols only when behavior changed—not because Linux moved a
   function or structure;
6. rerun prior conformance suites against both old and new reference evidence
   where the change requires behavioral comparison; and
7. publish a disposition report linking every generated difference to its
   mapping, implementation impact, and test evidence.

A source-path move with unchanged behavior updates provenance only. A changed
Linux internal algorithm does not force a Topal rewrite unless it reveals a
missed contract, safety condition, or required behavior. A new UAPI command or
observable file never disappears into an “internal change” classification.
