# Bounded static-process shell child disposition

Status: approved systems-feature design (`TK-DEC-201`).

## Purpose

The pinned BusyBox shell installs a child-status disposition immediately after
observing its process identity. This increment represents that operation as a
portable process-owned disposition, before changing the sealed external
argument profile from `echo` to `sh`.

## Portable contract

The live initial process may install one child-status disposition after its
sealed PID-one observation. The request carries a complete readable candidate
whose handler and return-trampoline are executable process addresses, whose
restart policy is the approved child-status policy, and whose blocked set is
the approved signal set. The provider copies and validates the complete value,
retains only private disposition state, returns success, and advances one
private startup phase.

The provider requires an executable continuation, active checked address
space, retained initial-stack leaf, empty prior child disposition, and the
exact predecessor phase. Failure preserves the predecessor phase, empty
disposition, and every process resource. Terminal process completion validates
and clears the retained disposition before address-space rollback.

The clean fixture constructs the complete request on its stack after
`getpid`, requires success, clears its request bytes, and exits. The current
pinned external `echo` profile retains its newline terminal path. A future
pinned external shell profile may use the same transition after `getpid`.

## Architecture bindings

X86-64 binds the operation to Linux `rt_sigaction` syscall 13 with signal 17,
no old-action output, an eight-byte native signal-set size, `SA_RESTORER`, and
the approved child mask. The provider owns native field order, flag encoding,
register convention, and address checks. AArch64 or RISC-V may use its native
selector, layout, trampoline convention, and child-signal number while
realizing the same portable disposition lifecycle.

## Exclusions

This adds no signal generation, pending queue, delivery, frame construction,
`rt_sigreturn`, handler execution, restart execution, old-action publication,
arbitrary disposition, multiple process or thread, child creation, wait,
shell execution, container, or virtualization support.
