# Reusable bounded static-process execution

Status: approved systems-feature direction under the standing authorization;
implementation and qualification pending (`TK-DEC-239`).

## Purpose

The same-image `true` replacement proves an image lifetime but compares every
argument with a compiled constant and reconstructs that constant. Standard
userspace requires runtime values to cross the syscall boundary. This
refinement replaces those payload-specific checks with bounded transfer,
stack construction, console output, and exit services. The sealed executable
remains `/bin/busybox`; executable selection and dynamic linking follow later.

## Argument ownership and failure

The exec adapter resolves the sealed executable object, then copies a
null-terminated native argument vector containing one through eight arguments
into private kernel storage. Complete strings, including terminators, occupy
at most 256 bytes in total. Empty arguments are valid. Argument zero must name
the selected executable; later argument contents are application values,
including bytes without special shell meaning. No applet-name or output-text
oracle belongs in the exec service. The four inherited environment entries
remain those qualified by `TK-DEC-238` in this increment.

Every vector entry and string byte is checked against the live readable user
mapping set before access. Null or inaccessible vector/string candidates
return Linux `EFAULT`. An unterminated vector or strings exceeding the stated
limits return `E2BIG`. A complete unsupported executable pathname returns
`ENOENT`; a pathname without a terminator in 256 bytes returns `ENAMETOOLONG`.
An empty argument vector, invalid argument zero, or environment outside the
retained four-entry contract returns `EINVAL`. Rejected requests return
the original process-image owner unchanged: its memory, mappings, descriptors,
TLS, signals, random generator, command state and continuation remain live.
Staging is erased on every rejection. No user pointer survives validation.

Only a fully copied and checked argument aggregate can commit replacement.
The image reset from `TK-DEC-238` remains indivisible. The provider constructs
argc, argv pointers and terminators, environment and auxiliary vectors from
the staged values; it neither substitutes payload bytes nor retains pointers
into the old image. It reserves fresh stack space with the target ABI's
alignment and consumes staging before entering the new image.

## Reusable startup, console, and termination

Replacement startup continues to use the qualified libc operations, but
stack-local outputs are validated by complete extents in the retained stack
rather than by one hard-coded address determined by argc. Credential facts
are read from the constructed auxiliary-vector identity for this generation.
These changes do not grant arbitrary syscall, mapping or credential authority.

Once replacement startup has established its ordinary process state, console
`write` accepts standard output or standard error and zero through 256 bytes
from any complete readable user extent. It stages all bytes before publishing
any, returns the requested count, and preserves the syscall continuation.
Bad descriptors return `EBADF`; inaccessible nonempty candidates return
`EFAULT`; a request beyond the bound returns `EINVAL`. Empty writes return
zero without dereferencing the candidate. Payloads and output counts are
derived from the request rather than compared with qualification strings.

The process may terminate with a native integer exit status; the adapter
retains its low eight bits as the observable process status. Cleanup revokes
inherited descriptor three and every replacement resource regardless of that
status. Qualification may expose a clearly kernel-owned status witness after
application output and before kernel rollback; this does not manufacture
application output or emulate a waiting parent.

## Input and physical qualification

The last shell refill receives one bounded newline-terminated command from
the declared serial input capability. It does not construct a selected exec
command inside the provider. The harness supplies the command after the
static-process entry marker and supplies the existing later probe input only
at its established resource marker. Other shell services retain their
predecessor bounds; this is not yet unrestricted interactive shell support.

The qualification corpus includes successful external `true`, nonzero external
`false`, and external `echo` with multiple runtime arguments and changed
payload bytes. Clean fixtures additionally exercise empty arguments, boundary
counts, nonempty data in rejected requests, inaccessible candidates, missing
terminators and aggregate overflow, then successfully exec on the unchanged
owner. Evidence checks exact application output, status, argument transfer,
staging erasure, fresh image state, descriptor inheritance and full rollback.

## Portability, risk, and alternatives

Portable meaning is checked executable authority plus an immutable bounded
argument aggregate, a fresh image owner, a console description and byte
transfer, and a completed process with an eight-bit status. Native pointer
width, vector encoding, initial-stack alignment, auxiliary-vector placement,
syscall registers and return mechanisms stay in the ABI provider. AArch64 and
RISC-V can preserve the ownership/error/byte observations with different ABI
realizations. No core-language grammar, task semantics, raw pointers, source
assembly or general privileged intrinsic is introduced.

Risk is high because a value-dependent copy replaces constant-sized transfer
and introduces recoverable pre-commit failures. Review must check every extent,
count, arithmetic operation and cleanup edge, the commit boundary, preservation
of the kernel caller frame, register and SIMD reset, and separation of staging
from pristine snapshots and user mappings. Both negative and positive runtime
evidence are required before claiming qualification.

Separate per-applet dispatchers would preserve the payload oracle. A full VFS,
arbitrary ELF loader, fork/wait, containers and virtualization would combine
independent authority and lifetime changes. This refinement establishes shared
services needed by those later parts of the original plan.
