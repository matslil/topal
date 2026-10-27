# Bounded static ELF applet newline write

Status: approved systems-feature design (`TK-DEC-198`).

## Purpose

After its credential normalization, the pinned BusyBox `echo` startup path
publishes one line-feed byte to standard output. This increment adds that exact
process-entry dependency before changing the sealed argument profile from
`true` to `echo`.

## Portable contract

Following the credential-tail witness, the live initial process may publish
exactly one line-feed byte to standard output. The provider requires descriptor
one, length one, an executable continuation, and the profile-bound candidate
byte in an already proved readable process mapping. It revalidates the active
address space and that candidate's exact backing before reading, copies the
byte into private kernel staging, verifies value line feed, publishes only the
staged byte, returns one, discards the pointer and staging, and advances only
its private startup phase.

The clean profile binds the candidate to its temporary byte in the retained
initial stack leaf. The external profile binds it to the pinned applet's
observed byte in the earlier anonymous program-break extent. Native addresses
remain provider-private. No mapping may change concurrently in this
single-context profile. A wrong descriptor, length, value, candidate, mapping,
continuation, phase, or credential state is a fatal violation of the sealed
startup profile and produces no output.

The digest-pinned external `true` applet retains its header-bound early
terminal path. It is regression evidence in this increment and does not yet
claim to execute the output operation.

## X86-64 binding

The provider binds the operation to Linux x86-64 `write` syscall 1 with `rdi`
descriptor, `rsi` candidate, and `rdx` length. The clean-room fixture places
one line-feed byte in its retained stack leaf, issues the exact request after
`setuid(0)`, requires result one, clears the byte, restores its stack pointer,
and exits. Native registers, page-table traversal, UART access, and private
phase representation remain provider facts.

## Cross-architecture mapping

AArch64 or RISC-V binds the same bounded output effect to its native Linux
selector, argument/result convention, readable-mapping proof, and console
adapter. The abstraction is complete validation and private staging before a
single output effect, not an x86 instruction or UART port.

## Exclusions

This increment adds no general descriptor table, arbitrary payload, partial or
blocking write, file, pipe, socket, signal, retry, application prompt, shell,
BusyBox redistribution, container, or virtualization-host support.
