# Initial console `TIOCGWINSZ`

This record resolves `TK-DEC-122` as the first Linux terminal-geometry query
for the fixed serial console. It completes the immutable terminal-discovery
foundation without inventing geometry that the current transport does not
provide.

## Decision

The console object shared by the three initial open descriptions shall own one
immutable window-size value. Because the pinned ISA serial device and current
boot contract provide no row, column, or pixel geometry, all four components
shall be zero. Every description and alias shall expose the same value.

The x86-64 personality shall accept
`ioctl(fd, TIOCGWINSZ=0x5413, candidate)` and publish the complete native
eight-byte `struct winsize`: four little-endian unsigned 16-bit zero values for
rows, columns, horizontal pixels, and vertical pixels. Zero means unspecified;
it does not imply a zero-area display or prevent later userspace policy from
selecting dimensions through a separately designed mutation path.

Dispatch shall retain the established descriptor-before-request-before-pointer
precedence. Invalid or closed descriptors return `-EBADF`; other unsupported
live-console requests retain `-ENOTTY`; an accepted request with an invalid,
read-only, wrapping, cross-hole, or incomplete eight-byte candidate returns
`-EFAULT`. A valid query shall construct the complete value in private staging,
publish it under fault recovery, return zero only after complete visibility,
and retain no pointer.

The ring-3 qualification shall query input and output into disjoint writable
extents, validate all eight bytes, prove both values identical, and after the
final close of slot zero reject a query there with `-EBADF`. Source shall emit
one ordered `TOPAL_KERNEL_USER_WINSIZE_OK` witness after exact return.

## Architecture pressure test

Unspecified geometry, shared console identity, error precedence, and complete
fault-contained publication are portable personality meaning. X86-64 selects
syscall 16; AArch64 and RISC-V select syscall 29. Each qualified personality
owns the request and aggregate layout without exposing registers or ioctl
encoding to portable source.

## Alternatives rejected

Inventing 80-by-24 text geometry was rejected because the serial transport has
no such source. Reading host pseudo-terminal geometry was rejected because it
would make the guest contract depend on harness state and break reproducible
qualification. Combining `TIOCSWINSZ` was rejected because mutation, authority,
change notification, and `SIGWINCH` are separate behavior. Treating the query
as `TCGETS` metadata was rejected because Linux defines distinct layouts and
requests.

## Deliberate bounds

This increment supports immutable `TIOCGWINSZ` for the shared console, complete
eight-byte publication, aliases, and exact `EBADF`/`ENOTTY`/`EFAULT`. It does
not provide `TIOCSWINSZ`, host geometry import, `SIGWINCH`, sessions, process
groups, job control, ptys, display devices, or a general ioctl registry.
