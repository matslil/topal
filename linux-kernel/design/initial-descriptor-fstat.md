# Initial descriptor `fstat`

This record resolves `TK-DEC-120` as the first Linux file-metadata publication
for the fixed console descriptions. It gives libc a truthful character-device
view without introducing paths, inodes as mutable VFS objects, or a filesystem.

## Decision

The three fixed initial open descriptions shall reference one immutable console
device object. Its portable metadata is: one stable object identity, character-
device kind, owner user and group zero, permissions `0600`, link count one,
zero byte size and allocated blocks, preferred I/O size 4096, and zero-valued
access/modification/change timestamps because this profile has no persistent
filesystem clock metadata. Input, output, error, and all aliases shall publish
the same underlying object identity and device identity; their differing
access modes remain `fcntl` state and shall not change `stat` mode bits.

The x86-64 personality shall encode this metadata in the baseline native
144-byte `struct stat`: eight-byte device, inode, and link fields; four-byte
mode, UID, GID, and zero padding; eight-byte special-device, size, block-size,
block-count, six timestamp components, and three zero unused words. The Q35
console device shall use Linux ttyS0 special-device identity major 4, minor 64,
encoded as `0x4400`; the containing synthetic device identity is zero and the
stable inode identity is one.

Linux-compatible `fstat(fd, candidate)` shall look up a live descriptor before
user-memory access, construct the complete value in private kernel staging,
validate one overflow-free writable 144-byte user extent, and publish the
complete structure under fault recovery. It shall return zero only after the
whole value is visible and retain no user pointer. A negative, out-of-range, or
closed descriptor returns `-EBADF` without touching the candidate. An invalid,
read-only, wrapping, or cross-hole candidate returns `-EFAULT` without partial
publication.

The ring-3 qualification shall `fstat` output, validate every defined field and
all padding, duplicate output into slot zero, `fstat` the alias, and prove the
two structures have identical object/device/mode metadata. After closing the
last temporary slot-zero alias, another `fstat(0, …)` shall return `-EBADF`.
Source shall emit one ordered `TOPAL_KERNEL_USER_FSTAT_OK` witness after exact
return. Portable source owns semantic metadata and the selected UAPI layout;
it observes no syscall number, register, user address, copy instruction, or
provider storage representation.

## Architecture pressure test

Object identity, type, ownership, permissions, accounting, timestamps, lookup
precedence, and complete fault-contained publication are portable personality
meaning. X86-64 selects syscall 5 and its native layout. AArch64 and RISC-V use
`newfstatat` syscall 79 for equivalent descriptor-relative behavior and their
architecture-qualified layouts; neither target is required to imitate x86
padding or syscall selection. A future common `statx` service may supply the
semantic metadata to each personality adapter.

## Alternatives rejected

Reporting a regular file was rejected because standard streams are console
device descriptions. Giving each descriptor slot a different inode was
rejected because aliases and the three inherited descriptions reference the
same device object. Encoding read/write access in permission bits was rejected
because permissions and open status are distinct Linux state. Copying fields
directly to user memory while constructing them was rejected because a late
fault would expose a partial structure. Using host wall-clock timestamps was
rejected because the kernel has no approved persistent metadata clock policy.
Adding path lookup or mutable inode state was rejected as a separate VFS step.

## Deliberate bounds

This increment supports one immutable console object, native x86-64 `fstat`,
complete 144-byte publication, aliases, and exact `EBADF`/`EFAULT`. It does not
provide `stat`, `lstat`, `newfstatat`, `statx`, paths, mounts, mutable metadata,
terminal ioctls, filesystem timestamps, device registration, dynamically
opened objects, sockets, pipes, or general VFS inode state.
