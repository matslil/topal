# Initial-process scalar identity syscalls

This record resolves `TK-DEC-074` as a systems-profile refinement. It exposes
the initial process's already-approved identity through a closed family of
pointer-free Linux-compatible syscalls. It changes no core-language rule and
does not introduce a general process or credential service.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall numbers 102, 104, 107, 108, 110, and 186 to
`getuid`, `getgid`, `geteuid`, `getegid`, `getppid`, and `gettid`. Together
with the existing `getpid` syscall 39, these calls return scalar identity facts
without user-memory transfer.

## Decision

The sealed initial-process personality returns:

- PID one from `getpid`;
- TID one from `gettid`;
- parent PID zero from `getppid`; and
- real/effective UID and GID zero from `getuid`, `geteuid`, `getgid`, and
  `getegid`.

Every dispatch requires the live initial user context and a return instruction
inside the finalized executable mapping. Arguments are ignored as Linux does
for these zero-argument interfaces; no pointer or mutable identity state is
retained. The four credential results must agree with the zero-valued
`AT_UID`, `AT_EUID`, `AT_GID`, and `AT_EGID` facts on the initial stack.

The qualification image invokes all seven calls, checks every result, and
traps before terminal completion on mismatch. Existing ordered post-return
witnesses therefore cover the complete family without adding provider-owned
output.

## Alternatives considered

Returning the values from one synthetic combined operation would not exercise
the Linux syscall surface. Adding a mutable task table or credential object
would prematurely select lookup, lifetime, namespace, synchronization, and
authorization policy. The closed immutable family reuses identity facts that
the initial-process profile already owns.

## Cross-architecture pressure

The portable contract is coherence among process, thread, parent, and
credential facts. It does not prescribe syscall numbers, registers,
instructions, task-structure layout, namespace representation, or credential
storage. Each ABI provider maps its native syscall numbers and return
convention to the same initial identity.

This increment does not admit multiple processes or threads, process groups,
sessions, supplementary groups, user namespaces, credential mutation,
capabilities, signals, process lookup, scheduling policy, or permission checks.
