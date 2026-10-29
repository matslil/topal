# Bounded static-process shell `getppid`

Status: approved systems-feature design (`TK-DEC-202`).

## Purpose

The pinned BusyBox shell queries its parent identity immediately after
installing the child-status disposition. This increment reuses the existing
sealed no-parent fact at the static-process boundary before changing the
external argument profile from `echo` to `sh`.

## Portable contract

The live initial process may observe its sealed parent identity once after the
child disposition is installed. The provider requires an executable
continuation, active checked address space, retained initial stack, live child
disposition, and exact predecessor phase; returns parent identity zero;
retains no argument, pointer, lookup result, or mutable task state; and
advances only the private startup phase. Failure preserves the predecessor
phase, disposition, and every process resource.

The clean fixture performs the observation after installing its disposition.
The pinned external `echo` profile retains its terminal newline path, while a
future external `sh` profile may bind the same operation after its disposition
installation. Only the exact predecessor is valid.

## Architecture bindings

X86-64 binds the operation to Linux syscall 110 and returns zero in the native
result register. AArch64 or RISC-V uses its native Linux selector and calling
convention. The portable abstraction is observation of the sealed absence of a
parent in the qualification process model, not a syscall number or register.

## Exclusions

This adds no process lookup, parent process, PID allocation, namespace, task
table, child creation, wait, signal delivery, multiple processes or threads,
shell execution, container, or virtualization support.
