# Linux userspace interface map

## LK-UAPI-001 — Inventory boundary

Linux userspace compatibility is a collection of protocols, not a syscall-name
checklist. The authoritative baseline inputs include:

- `arch/x86/entry/syscalls/syscall_64.tbl` and native entry code;
- `include/uapi`, `arch/x86/include/uapi`, and generated exported headers;
- `make headers_install` output for the selected x86 configuration;
- `Documentation/ABI/{stable,testing,obsolete,removed}`;
- userspace API documentation, subsystem protocol specifications, and vDSO
  symbol/version definitions;
- pseudo-filesystem producers and device/subsystem ioctl definitions; and
- runtime probes against the pinned Linux reference VM.

The inventory records availability conditions and behavior, not merely names
and numeric constants. Primary orientation sources are the Linux
[ABI description](https://docs.kernel.org/admin-guide/abi.html),
[exported-header documentation](https://docs.kernel.org/kbuild/headers_install.html),
and [userspace API index](https://docs.kernel.org/userspace-api/index.html).

## LK-UAPI-SYSCALL-001 — Native system-call contract

For each native x86-64 syscall, record:

- number, entry convention, arguments, widths, pointed-to layouts, direction,
  alignment, and accepted flag space;
- return and error encoding, partial completion, blocking, interruption,
  cancellation, restart, and timeout behavior;
- memory, file-descriptor, credential, namespace, scheduling, and other side
  effects;
- privilege, capability, LSM, seccomp, resource-limit, and configuration gates;
- relationships with vDSO, libc wrappers, ioctls, Netlink, or filesystem
  alternatives; and
- conformance and negative tests.

The native table is necessary but insufficient: multiplexed operations such as
`ioctl`, `fcntl`, `prctl`, `arch_prctl`, `bpf`, `perf_event_open`, socket
options, and `io_uring` expose subordinate protocols with their own versioned
structures and state machines.

## LK-UAPI-PROCESS-001 — Program, process, thread, and signal boundary

Compatibility includes ELF loading and mapping, `PT_INTERP`, initial stack and
auxiliary vectors, credentials, TLS, vDSO/vvar, process and thread creation,
exec, exit, wait, ptrace, register sets, core dumps, signal disposition and
masks, native signal frames, alternate stacks, restart behavior, robust futex
lists, CPU affinity, scheduling, and resource accounting.

The Topal kernel must distinguish user pointers from kernel locations, validate
all transfers, and recover from user-memory faults without turning malformed
input into kernel failure. Native x86-64 is the only userspace ABI in the first
profile; x32 and i386 layouts and signal frames are excluded.

## LK-UAPI-FD-001 — File descriptors, VFS, filesystems, and events

The shared file-description model affects files, directories, sockets, pipes,
terminals, devices, event objects, process handles, namespaces, and KVM. Record
open-file-description sharing, flags, offsets, credentials, polling readiness,
advisory locking, asynchronous ownership, inheritance, close behavior, and
cross-process transfer.

The interface inventory covers path resolution, mounts, filesystem statistics,
permissions, ACLs, extended attributes, leases, inotify/fanotify, epoll,
eventfd, signalfd, timerfd, pidfd, AIO, `io_uring`, splice-like transfer, and the
baseline filesystems and storage stack selected by later qualification.

## LK-UAPI-NET-001 — Networking and Netlink

Record socket families, types, protocols, address layouts, ancillary data,
options, shutdown and error queues, readiness, namespaces, packet filtering,
virtual network devices, routing, and interface configuration. Netlink is a
family of bidirectional TLV protocols rather than one generic message format;
each family, command, attribute policy, dump, multicast group, capability gate,
and extension rule needs an entry. See the Linux
[Netlink introduction](https://docs.kernel.org/userspace-api/netlink/intro.html).

## LK-UAPI-DEVICE-001 — Device and subsystem interfaces

`ioctl` is a transport for many unrelated UAPIs. Inventory each request by
owning file type, command encoding, native structure layout, pointer direction,
variable-length rule, lifetime, concurrency, capability, and error behavior.
The global command-number table is an index rather than a complete contract;
see the Linux [ioctl guidance](https://docs.kernel.org/userspace-api/ioctl/ioctl-number.html).

Device UAPI coverage follows the complete target even though implementation is
staged. A device absent from the qualified board must be absent in the same
discoverable way; it is not silently represented by a partial fake device.

## LK-UAPI-PSEUDOFS-001 — Text and filesystem-shaped interfaces

Generated header analysis cannot find all public behavior. Track `/proc`,
`/sys`, `/dev`, cgroupfs, securityfs, configfs, tracefs, and other configured
filesystem-shaped interfaces by path pattern, file type, contents, parsing,
permissions, namespace view, poll behavior, and side effects. Debug-only
interfaces are still recorded and marked with their upstream stability class.

## LK-UAPI-CONTAINER-001 — Container prerequisites

Docker and Podman are application profiles over many interfaces. Their ledger
must include clone/unshare/setns, user/mount/PID/network/IPC/UTS/time/cgroup
namespaces, UID/GID maps, capabilities, seccomp, cgroup v2, mounts and
propagation, overlay or selected snapshot storage, virtual networking,
Netfilter or selected firewall integration, pseudo-filesystems, terminals,
resource limits, and LSM interactions.

Rootful and rootless profiles are distinct. A successful `hello-world` run is
an integration checkpoint, not complete container qualification.

## LK-UAPI-KVM-001 — Virtual-machine host API

KVM is a late, separately qualified UAPI based on `/dev/kvm`, file descriptor
lifetimes, ioctls, userspace memory slots, shared `mmap` state, virtual CPU
threads, interrupt routing, in-kernel devices, capability discovery, and
x86-specific processor state. The primary contract is the Linux
[KVM API](https://docs.kernel.org/virt/kvm/api.html).

Running QEMU with software emulation does not qualify KVM. Running the Topal
kernel as a guest does not qualify it as a VM host. Nested virtualization also
requires the outer hypervisor to expose suitable hardware facilities and is a
third, optional profile.

