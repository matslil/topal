# Initial descriptor `fcntl` metadata

This record resolves `TK-DEC-119` by separating per-slot descriptor flags from
the access mode carried by the fixed open description introduced in
`TK-DEC-118`. It adds the metadata operations commonly used by libc without
admitting general `fcntl`, file-status mutation, or exec.

## Decision

Each of the three descriptor slots shall retain one private `FD_CLOEXEC` bit in
addition to its optional open-description mapping. All three initial bits are
clear. Closing a slot clears both its mapping and its flag. `dup` and `dup2`
shall always clear `FD_CLOEXEC` on the new destination mapping, including when
`dup2` replaces a live destination; equal live `dup2(old, old)` preserves the
existing bit because it performs no duplication.

Linux-compatible `fcntl(fd, F_GETFD)` shall return zero or `FD_CLOEXEC` for a
live slot. `fcntl(fd, F_SETFD, arg)` shall set the slot bit from
`arg & FD_CLOEXEC`, ignore other argument bits as Linux does, and return zero.
`fcntl(fd, F_GETFL)` shall return the shared open description's immutable
access mode: `O_RDONLY` for console input and `O_WRONLY` for console output or
error. Invalid or closed descriptors return `-EBADF`; unsupported commands
return `-EINVAL` without state change. Descriptor lookup precedes command
decoding, preserving Linux error precedence.

The ring-3 qualification shall observe `O_WRONLY` and clear descriptor flags
on output, set and read back `FD_CLOEXEC`, and reject an unsupported command.
After closing zero, `dup(1)` shall create slot zero with its flag clear while
leaving the source flag set. Closing one and restoring it with `dup2(0, 1)`
shall clear the destination flag. A later query of closed zero shall return
`-EBADF`. Terminal dispatch shall verify the two remaining mappings have clear
flags, clear the complete flag state, and then resume the kernel continuation.

Portable source owns command identities, flag masks, access-mode values, error
precedence, and duplication interactions. It observes no packed storage,
native syscall number, register, bit instruction, or open-description address.

## Architecture pressure test

Per-slot close-on-exec ownership, shared open-description access mode, command
semantics, lookup precedence, and duplication behavior are portable Linux-
personality meaning. X86-64 selects native `fcntl` syscall 72; AArch64 and
RISC-V select syscall 25. Providers may pack the three bits or use typed slot
records, but cannot expose their layout or move command policy out of source.

## Alternatives rejected

Storing `FD_CLOEXEC` on the open description was rejected because aliases may
carry different descriptor flags. Returning numeric-slot-derived access modes
was rejected because duplication permits output descriptions in slot zero.
Preserving the source flag during `dup`/`dup2` was rejected because Linux clears
close-on-exec on those new descriptors. Rejecting unknown `F_SETFD` argument
bits was rejected because Linux masks them. Dispatching the command before fd
lookup was rejected because it would return `EINVAL` where Linux returns
`EBADF`. Adding `F_SETFL`, locks, leases, ownership, seals, or notifications was
rejected because each changes separate shared object or asynchronous state.

## Deliberate bounds

This increment supports one process, one thread, three slots, `F_GETFD`,
`F_SETFD`, `F_GETFL`, `FD_CLOEXEC`, fixed `O_RDONLY`/`O_WRONLY`, duplication
flag clearing, and terminal clearing. It does not provide `F_DUPFD`,
`F_DUPFD_CLOEXEC`, `F_SETFL`, status flags beyond access mode, record locks,
leases, ownership, signals, seals, pipe sizing, exec-time close, fork
inheritance, dynamically opened descriptions, concurrency, or general `fcntl`.
