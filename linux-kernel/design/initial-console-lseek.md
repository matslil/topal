# Initial console `lseek`

This record resolves `TK-DEC-123` as the first Linux file-position operation
for inherited descriptions. It makes the existing console object truthfully
non-seekable without introducing file offsets before seekable objects exist.

## Decision

Every open description of the shared initial console shall have stream
semantics and no file-position state. The Linux personality shall implement
`lseek(fd, offset, whence)` for the bounded three-slot descriptor table.

Dispatch shall resolve the integer slot before interpreting `whence`. Invalid
or closed slots return `-EBADF`, including when `whence` is invalid. For a live
slot, `whence` values from `SEEK_SET=0` through `SEEK_HOLE=4` are accepted by
the generic syscall layer and then return `-ESPIPE` because the console is not
seekable. A live slot with any larger unsigned `whence` returns `-EINVAL`
before object dispatch. The signed 64-bit offset is not interpreted for a
non-seekable console and no position is created or changed.

The ring-3 qualification shall require `-ESPIPE` for `SEEK_CUR` on both input
and output, `-EINVAL` for an unsupported `whence` on a live output, and
`-EBADF` for the same unsupported `whence` after slot zero closes. Source shall
emit one ordered `TOPAL_KERNEL_USER_LSEEK_OK` witness after exact return.

## Architecture pressure test

Descriptor identity, validation precedence, stream non-seekability, and the
absence of position state are portable Linux-personality meaning. X86-64
selects syscall 8 and passes `fd`, signed offset, and unsigned `whence` in its
native argument registers. AArch64 and RISC-V select syscall 62. Providers
encode those conventions without exposing registers or syscall numbers to
portable source.

## Alternatives rejected

Returning zero for a position query was rejected because it would report a
seekable object and create state that terminal reads and writes do not use.
Returning `-EINVAL` for every live-console call was rejected because Linux
distinguishes a valid seek mode on a non-seekable stream as `-ESPIPE`.
Validating `whence` before descriptor lookup was rejected because Linux gives
an invalid or closed descriptor `-EBADF` precedence. Adding a general file
position to every open description was rejected because pipes, sockets, and
terminals are streams while regular files require separately designed shared
offset and synchronization behavior.

## Deliberate bounds

This increment supports x86-64 `lseek` classification for the bounded initial
console descriptions and aliases, with exact `EBADF`, `EINVAL`, and `ESPIPE`.
It does not provide seekable files, shared offsets, `pread`/`pwrite`,
directories, block devices, pipes, sockets, or a general VFS seek operation.
