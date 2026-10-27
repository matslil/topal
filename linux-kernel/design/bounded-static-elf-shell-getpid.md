# Bounded static-process shell `getpid`

Status: approved systems-feature design (`TK-DEC-200`).

## Purpose

The pinned BusyBox shell path queries its process identity immediately after
the already qualified root-identity observation. This increment reuses the
existing sealed PID-one process fact at the static-process boundary before the
argument profile changes from `echo` to `sh`.

## Portable contract

The live initial process may observe its sealed process identity once. The
provider requires an executable continuation, active checked address space,
retained initial stack, and coherent PID-one process profile; returns identity
one; retains no argument, pointer, or mutable process state; and advances only
the private startup phase.

The clean fixture performs the observation after its newline witness. The
pinned external `echo` profile retains its terminal newline path, while the
future external `sh` profile binds the same operation directly to `getuid(0)`.
Only the header-selected predecessor is valid. Failure preserves all process
resources and the preceding phase.

## Architecture bindings

X86-64 binds the operation to Linux syscall 39 and returns one in the native
result register. AArch64 or RISC-V uses its native Linux selector and calling
convention. The portable abstraction is observation of the sealed process
identity, not a syscall number or register.

## Exclusions

This adds no PID allocation, namespace, parent query, task table, signal
semantics, multiple process, fork, clone, wait, shell profile, or container.
