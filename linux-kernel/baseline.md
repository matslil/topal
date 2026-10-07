# Linux compatibility baseline

## LK-BASELINE-001 — Pinned upstream release

The initial compatibility baseline is Linux `7.2.9`, published as the latest
stable release on 2026-10-03 and selected on 2026-10-04.

| Field | Value |
| --- | --- |
| Release | `7.2.9` |
| Git identity | stable-tree tag `v7.2.9` |
| Source archive | `linux-7.2.9.tar.xz` |
| SHA-256 | `b4c5dfbe51a364a6c7f03869200f88c8e1f77403539005f14b7fc6bc91b8d8ba` |
| Release authority | [kernel.org front page](https://www.kernel.org/) |
| Checksum authority | [signed kernel.org checksums](https://cdn.kernel.org/pub/linux/kernel/v7.x/sha256sums.asc) |
| Source | [stable Linux tree at `v7.2.9`](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/?h=v7.2.9) |

The archive is an input, not a repository dependency. Tooling shall verify the
signed checksum record and archive digest before using it. The archive and its
expanded tree shall remain outside version control.

## LK-SCOPE-001 — Implementation architecture

The first implementation supports:

- an x86-64 kernel;
- native x86-64 LP64 userspace;
- little-endian execution; and
- one version-pinned QEMU machine profile selected by the VM phase.

The i386 and x32 compatibility ABIs are not part of this baseline. Supporting
an x86-64 processor does not by itself add those distinct userspace ABIs.
AArch64, RISC-V, and other architectures are comparison inputs for language and
compiler abstraction; they are not implementation targets.

## LK-SCOPE-002 — Meaning of complete public-interface support

The eventual compatibility target is every userspace-visible interface present
in Linux 7.2.9 for native x86-64, including conditional interfaces when their
owning subsystem is available. The inventory shall cover at least:

- native system calls, calling convention, errors, restart behavior, and
  userspace argument layouts;
- ELF process entry, auxiliary vectors, TLS, vDSO/vvar, signals, ptrace, and
  register-set layouts;
- exported UAPI constants, structures, enums, flags, ioctls, socket options,
  Netlink families, BPF commands, perf events, and KVM interfaces;
- files, filesystems, mounts, extended attributes, asynchronous I/O, `io_uring`,
  notification, IPC, synchronization, process, scheduler, memory, time, network,
  credential, capability, namespace, cgroup, security, and administration
  behavior;
- the observable files and links supplied by `/proc`, `/sys`, `/dev`, cgroupfs,
  security-oriented pseudo-filesystems, and subsystem-specific filesystems;
- device interfaces for implemented device classes and the Linux-defined
  absence or unsupported results for devices and features not present in the
  qualified machine; and
- userspace-facing virtualization and container prerequisites.

The generated source inventory begins with `make headers_install`, native
x86-64 syscall tables, vDSO symbols, `Documentation/ABI`, UAPI directories, and
the selected kernel configuration. It is extended by sources that define
runtime-discovered or text interfaces not completely represented by headers.

“Complete” does not mean that nonexistent hardware is fabricated. An interface
whose Linux behavior depends on configuration, privilege, hardware, or a
runtime capability shall expose the same discovery behavior and Linux result
for the same qualified environment.

## LK-SCOPE-003 — Latest-only behavior

Qualification compares against Linux 7.2.9. It does not promise:

- interfaces removed before 7.2.9;
- behavior introduced after 7.2.9;
- internal kernel C APIs, internal symbols, loadable-module ABI, Kconfig names,
  or Linux implementation layout; or
- a behavior from an older release when 7.2.9 intentionally changed it.

Linux 7.2.9 itself retains many historical interfaces for compatibility. Those
remain in scope because they are part of the selected release. “Latest only”
does not authorize deleting an old interface that the baseline still exposes.

## LK-SCOPE-004 — Conformance and implementation status

Each inventory entry has one implementation disposition:

- `required`: unconditionally visible in the qualified baseline;
- `conditional`: required when its recorded configuration, capability, device,
  privilege, or protocol precondition holds;
- `absent-by-profile`: the reference machine lacks the owning facility and
  Linux-compatible discovery/absence behavior is required;
- `implemented`: Topal kernel behavior has conformance evidence;
- `qualified`: differential and independent tests cover the recorded contract;
  or
- `blocked`: a named Topal, toolchain, dependency, or design decision prevents
  implementation.

An entry cannot disappear because an extractor, document, or test omitted it.
Every baseline bump performs a source-to-source inventory diff and requires an
explicit disposition for additions, removals, and semantic changes.

## LK-SCOPE-005 — Distribution and application tests

Distribution boot and application tests are evidence, not the definition of
the ABI. The VM phase will pin a small reference root filesystem, package
versions, libc variants, and executable test corpus. Passing that corpus does
not mark an unexercised interface qualified.

The acceptance ladder is:

1. static initramfs and command prompt;
2. static native x86-64 application corpus;
3. dynamic libc, loader, TLS, and signal corpus;
4. reference distribution boot and service corpus;
5. container runtime profiles;
6. QEMU software-emulation host profile; and
7. KVM host and optional nested-virtualization profiles.

## LK-SCOPE-006 — Provenance and licensing boundary

The Topal repository is currently released under the Unlicense, while Linux is
distributed under its own licensing terms. This research library records
facts, short descriptions, stable identifiers, and primary-source links. It
does not copy Linux implementation code.

No Linux source, derived implementation, generated UAPI copy, firmware, or
third-party binary may be committed until its license, required notices,
provenance, and compatibility with the intended destination are reviewed. This
is a project gate, not a conclusion about whether a particular interface fact
or independently written implementation is copyrightable.

The approved [kernel licensing and provenance policy](design/license-and-provenance.md)
resolves `TK-DEC-011`: first-party kernel work remains under the Unlicense and
uses a clean-room implementation boundary. Any later exact UAPI or third-party
import remains under its reviewed upstream terms and must carry an immutable,
digest-bound import record.
