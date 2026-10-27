# Bounded static ELF initial configuration-stat miss

Status: approved systems-feature design (`TK-DEC-196`).

## Purpose

The pinned static BusyBox applet startup path probes `/etc/busybox.conf` after
its real-user-identity query. The current kernel has no filesystem, but this
optional lookup must fail as Linux userspace expects before later scalar
identity and output work can run. This increment adds one absence fact without
claiming a VFS or a general metadata interface.

## Portable contract

The live initial process may make one ordered metadata lookup after the sealed
real-UID witness. The portable service:

1. recognizes the complete immutable absolute path `/etc/busybox.conf`;
2. requires a complete writable native metadata destination;
3. reports that the optional configuration object is absent;
4. publishes no metadata bytes and retains no pathname, destination, or lookup
   state; and
5. advances only the private startup phase, preserving every earlier process
   resource and the complete terminal rollback contract.

The provider owns native syscall selection, directory-relative constants,
pathname encoding checks, metadata layout and size, argument registers, return
encoding, and address validation. Other paths, flags, base directories,
destinations, phases, duplicates, and results fail closed.

## Initial x86-64 binding

The provider binds the service to Linux x86-64 `newfstatat` number 262,
`AT_FDCWD` (`-100`), flags zero, an 18-byte NUL-terminated pathname, a complete
144-byte writable `struct stat` destination in the retained stack page, and
`-ENOENT` (`-2`). The clean-room image stores the path in its sealed executable
role, reserves stack scratch below the initial vector, verifies the exact
result, restores its stack pointer, and exits. The external resident-header
variant also admits the exact immutable pathname occurrence already present in
the digest-pinned image, but the currently qualified `true` applet may terminate
immediately after `getuid` without issuing the optional probe.

The selected header variant is provider-private and determines which exact
immutable pathname candidate is valid. It may also determine whether terminal
completion is admitted directly after `getuid` or only after the absence
witness. Source sees one architecture-neutral startup profile rather than
addresses, a native syscall sequence, or a union of register-level cases.

## Cross-architecture mapping

An AArch64 or RISC-V provider would bind the same sealed-path absence fact to
its Linux syscall selector, argument registers, `AT_FDCWD` encoding, native
metadata layout, user-range validation, and return convention. It need not
reuse x86-64 syscall numbering, instruction encoding, virtual addresses, or
page-table traversal. The abstraction boundary is the ordered absence result
and ownership behavior, not `newfstatat` machinery.

## Exclusions

This increment adds no directory traversal, symlink resolution, inode or mount
state, descriptor allocation, permissions, metadata publication, mutable
filesystem contents, generic pathname service, additional process or thread,
BusyBox redistribution, command output, shell, or prompt.
