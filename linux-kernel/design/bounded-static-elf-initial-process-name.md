# Bounded static ELF initial process-name query

## Decision

After confirming GNU RELRO protection, the pinned x86-64 static executable
issues `prctl(PR_GET_NAME, buffer)` and expects the 16-byte native process-name
field containing `busybox` followed by zero bytes. The bounded static-process
profile shall admit one exact query, publish the complete field to a checked
writable stack extent, return zero, and advance the private startup phase.

The logical name is sealed process metadata derived from the selected initial
executable identity. Source shall name the process-name query service, not the
Linux selector, native buffer width, bytes, virtual address, or syscall
registers. The provider shall validate the live RELRO-confirmed process,
executable continuation, active hierarchy, writable stack leaf, complete output
extent, and reusable-random service before publication. It shall retain no user
pointer or mutable name state, and terminal rollback shall remain complete.

## Portable boundary

The portable boundary is one immutable initial-process name and one complete
publication operation. AArch64, RISC-V, and capability-system providers may use
different native calls, task structures, buffer widths, address validation, or
copy mechanisms while preserving the same logical bytes, all-or-nothing output,
nonretention, lifecycle ordering, and rollback.

## Rejected alternatives

- Returning zero without writing the name was rejected because the caller
  immediately consumes the output field.
- Exposing `prctl`, `PR_GET_NAME`, the 16-byte layout, or a stack address to
  source was rejected because those are Linux and target-provider ABI details.
- Deriving the result from mutable user memory was rejected because process
  identity must remain sealed kernel metadata.
- General process-name mutation, arbitrary `prctl`, and task tables were
  rejected as broader than the observed single-process boundary.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact immutable process-name query. It does not add `PR_SET_NAME`,
other `prctl` operations, arbitrary output candidates, multiple processes,
threads, signals, procfs, BusyBox redistribution, containers, or a prompt.
