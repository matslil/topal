# Initial standard-descriptor close

This record resolves `TK-DEC-117` as the first state-changing Linux file-
descriptor operation for the initial process. It turns the already qualified
standard input, output, and error numbers into a bounded live descriptor table
without prematurely defining VFS objects, open-file descriptions, or general
descriptor allocation.

## Decision

The initial Linux syscall-dispatch policy shall name exactly three initially
live descriptors, numbered zero through two, and Linux error `EBADF` as nine.
The provider shall retain one private live/closed state for each descriptor.
Portable source owns the table bound and error policy but observes no bitset,
native object address, syscall number, register, or target storage layout.

Linux-compatible `close` shall accept a live descriptor in that closed set,
atomically make it unavailable to later dispatch, and return zero. A negative,
out-of-range, or already closed descriptor shall return `-EBADF` without
changing any state. The one-thread signal-free profile cannot interrupt the
transition, so it introduces neither `EINTR` nor an ambiguous close outcome.

The existing bounded `read` and `write` paths shall consult the same private
state before input consumption, user-memory access, or external output. A
closed descriptor therefore returns `-EBADF` without another effect. Terminal
process teardown shall clear the complete private descriptor state before the
exact kernel continuation can resume.

The ring-3 qualification shall preserve every prior call, then close standard
input after its last successful read, require zero, attempt another read from
that descriptor and require `-EBADF` without consuming input, close it again
and require `-EBADF`, and only then call `exit_group(0)`. An ordered kernel
witness after exact return shall establish successful close-state teardown.

## Architecture pressure test

Descriptor identity, live/closed state, close atomicity, `EBADF`, integration
with later descriptor lookup, and teardown are portable Linux-personality
meaning. X86-64 selects native syscall 3 and its entry registers. AArch64 and
RISC-V select native syscall 57 and their own entry mechanisms. Providers may
encode the bounded state differently, but source never observes that encoding
or treats a Linux descriptor as a Topal reference or machine address.

## Alternatives rejected

Returning success without retained state was rejected because a repeated close
would falsely succeed and later I/O could still use the descriptor. A general
source-owned descriptor collection was rejected because allocation, sharing,
reference counts, inheritance, and object types are not yet designed. Closing
standard error without exercising another operation was rejected because it
would not prove that descriptor lookup shares the new state. Adding `EINTR`
was rejected because the qualified one-thread process has no deliverable
signal or interruption point during close.

## Deliberate bounds

This increment supports one process, one thread, descriptors zero through two,
one successful close of standard input after its final input use, repeated-
close rejection, closed-descriptor rejection by bounded `read`, and terminal
state clearing. It does not provide `open`, `openat`, `dup`, `fcntl`, descriptor
allocation, open-file-description sharing, filesystem objects, sockets, pipes,
polling, asynchronous I/O, close-on-exec, fork or exec inheritance, concurrent
close, signals, cancellation, device-specific release, or deferred writeback
errors.
