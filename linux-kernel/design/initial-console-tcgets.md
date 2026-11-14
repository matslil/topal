# Initial console `TCGETS`

This record resolves `TK-DEC-121` as the first Linux terminal-control query for
the fixed console descriptions. It lets libc identify inherited descriptors as
terminals without claiming line-discipline behavior the bounded serial path
does not yet implement.

## Decision

The one immutable console object shared by the three initial open descriptions
shall also own one immutable raw terminal-attribute value. Input, output,
error, and every alias shall expose identical attributes independent of access
mode or descriptor-local flags.

The x86-64 personality shall accept `ioctl(fd, TCGETS, candidate)` with native
request `0x5401` and publish the native 36-byte `struct termios`. Its four
32-bit flag words are `c_iflag=0`, `c_oflag=0`,
`c_cflag=B38400|CS8|CREAD|CLOCAL` (`0x8bf`), and `c_lflag=0`; line discipline is
zero. The 19 control bytes are zero except `VMIN=1`. These values truthfully
describe the current exact-fill, no-conversion, no-echo, no-signal serial path;
they do not promise canonical input, output newline conversion, or editing.

Dispatch shall resolve a live descriptor before command decoding and decode
the request before candidate access. An invalid or closed descriptor returns
`-EBADF`; any request other than `TCGETS` on the live console returns `-ENOTTY`
without touching the candidate. `TCGETS` requires one overflow-free complete
writable 36-byte extent, constructs the whole value in private staging, and
publishes it under fault recovery. Invalid, read-only, wrapping, cross-hole,
or incomplete candidates return `-EFAULT`; success returns zero and retains no
user pointer.

The ring-3 qualification shall query standard input and output into disjoint
writable extents, validate every byte and prove both structures identical,
reject an unsupported live-console request with `-ENOTTY`, and after closing
slot zero reject `TCGETS` there with `-EBADF`. Source shall emit one ordered
`TOPAL_KERNEL_USER_TCGETS_OK` witness after exact return. Portable source owns
terminal meaning and selected UAPI layout; it observes no syscall number,
register, user address, copy instruction, or provider storage representation.

## Architecture pressure test

Terminal identity, raw attributes, lookup/command/pointer precedence, and
complete fault-contained publication are portable personality meaning. X86-64
selects syscall 16, request `0x5401`, and its native termios layout. AArch64 and
RISC-V also use syscall 29 and `TCGETS`, but their qualified personality
adapters remain responsible for request and layout facts rather than exposing
register or ioctl encoding to portable source.

## Alternatives rejected

Returning `ENOTTY` for the inherited console was rejected because the device
is observably a serial terminal. Advertising conventional canonical and echo
flags was rejected because the current provider does not implement that line
discipline. Treating unknown requests as `EINVAL` was rejected because Linux
terminal ioctl discovery uses `ENOTTY`. Copying fields while constructing them
was rejected because a late fault could expose a partial structure. Adding
`TCSETS` in the same increment was rejected because mutable terminal policy
requires later read/write behavior to honor the selected state.

## Deliberate bounds

This increment supports immutable `TCGETS` for one shared raw console object,
complete 36-byte publication, aliases, and exact `EBADF`/`ENOTTY`/`EFAULT`.
It does not provide `TCSETS*`, `termios2`, window size, process groups, sessions,
job control, canonical input, echo, signal generation, flow control, modem
state, break handling, polling, dynamically opened ttys, ptys, or a general
ioctl registry.
