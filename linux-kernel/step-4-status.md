# Step 4 implementation status

Step 4 remains in progress. The Topal kernel boots and executes bounded static
userspace under pinned x86-64 QEMU. It does not yet replace the reference VM's
Linux kernel for distribution, Docker/Podman or VM-host qualification.

The approved implementation target remains x86-64. Other architectures inform
systems abstractions; ARM64 implementation is outside the current focus.

## Current executed capabilities

- An ordinary Topal Boolean/Enum policy library with source-bound code ownership,
  actual helper calls, native ABI and stack evidence, and representation adaptation.
- Ordinary Topal numeric deadline policy using shared arbitrary-precision algorithms,
  exclusive owned arena lending, allocation/stack proofs and complete temporary erasure.
- Generated privileged entry/return, checked memory/translation ownership,
  kernel contexts, notifications, clock/deadline preemption and complete rollback.
- A supplied static ELF in the qualified mapping profile, real user execution,
  caller-owned argv/environment, repeated exec and normal exit status.
- Shared file descriptions and offsets, descriptor flags/aliases, an immutable
  boot-root namespace, cwd/directory observations and prepared root/PID/UTS views.
- Signal registration/masks without user delivery; actual RTC seconds; borrowed
  monotonic time/resolution and relative sleeps with source-defined saturation.
- Unmodified pinned BusyBox utilities and interactive shell builtin/cwd/input
  cycles through shared services. Unknown syscall selectors return ENOSYS.

The [current runtime report](step-4-owned-numeric-runtime.md) records 40 successful
physical runs on one kernel, one positive representation-variant run, seventeen lab
contracts and bounded workspace validation. Each report identifies its executed scope and limitations.
A successful bounded witness does not establish general Linux compatibility.

## Remaining acceptance work

| Area | Required result | Current limit |
| --- | --- | --- |
| Freestanding Topal libraries | Kernel policy/subsystems in ordinary Topal modules over qualified machine elements, as approved in Step 2 | Finite Boolean/Enum and bounded allocating numeric source units; general/persistent allocation and imports remain; native adapters still carry substantial Linux policy |
| Processes and execution | Process/thread creation, wait, general executable/mapping selection, dynamic ELF and libraries | Sole prepared process, one pinned static executable/mapping profile |
| Signals and scheduling | Delivery, interruption/restart, multiple runnable users, lifecycle and synchronization | Registration/masks and kernel-context foundations; sole-thread waits |
| Filesystems/devices | Writable storage, root mount, general VFS and actual distribution device interfaces | Three immutable namespace objects and qualified serial/entropy/time hardware |
| Networking and isolation | Network stack, namespaces, credentials/capabilities, cgroups and other retained UAPI contracts | Research/design plus bounded initial-process views |
| Reference distribution | Replace the Step 3 kernel and run declared command-line corpus from its ordinary prompt | Toolchain-gate VM and static BusyBox corpus |
| Containers | Docker and Podman acceptance in the replacement-kernel VM | Reference Linux controls only |
| VM hosting | Declared guest VM operation through Linux-compatible virtualization interfaces | Guest execution under QEMU only |
| Full pinned UAPI | Complete retained interface inventory and regression/drift evidence | Explicitly qualified subsets; ENOSYS does not imply implementation |

These are existing initial-plan/Step 2 obligations, not new scope. See the
[Step 2 architecture](design/topal-kernel-architecture.md),
[gap analysis](design/topal-gap-analysis.md), [baseline](baseline.md) and
[Step 3 reference VM](step-3-report.md). Systems-only refinements proceed under
standing authorization; a necessary core-language design change still requires
a human decision. Human merge remains required, without pausing implementation
merely because a PR is open.

## Review packaging

The open Step 4 work is consolidated into one PR against the repository default
branch. Definition, specification, implementation and qualification stages remain
separate commits. Further Step 4 increments update that PR. Previously published
increment PRs retain their historical review/evidence references and are
superseded by the consolidated PR. No automatic merge is authorized.
