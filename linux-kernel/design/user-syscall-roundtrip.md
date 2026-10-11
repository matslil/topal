# Affine user-execution and syscall round-trip contract

This record resolves `TK-DEC-032` for the first transition between kernel and
user execution. It establishes the privilege and syscall boundary without
prematurely selecting an ELF loader, general process model, signal protocol,
or pointer-bearing Linux operation.

## Semantic boundary

A `ValidatedUserState A B` is an opaque affine description of one legal user
continuation in user address space `A` under ABI personality `B`. It contains
semantic instruction, stack, status, and argument state, but source cannot
inspect or manufacture register slots, descriptor values, privilege numbers,
entry frames, or return instructions. Validation proves that execution begins
in executable user memory with a writable non-executable user stack and that
all target return invariants hold.

A `SuspendedUserContext P A T B` owns that validated state for thread `T` on
processor `P`. Entering it consumes the current running kernel context and the
suspended user context. The transition either reaches one declared special
entry or fails before user execution; it is not an ordinary call and it cannot
return an unchecked machine frame.

User syscall entry produces one affine `UserSyscallContext P A T B I` with a
fresh entry identity `I`. Its accessors expose the ABI syscall number and six
arguments as untrusted semantic machine words. The source handler owns dispatch
policy. It must consume the entry through exactly one of these dispositions:

- `ReturnToUser(result)` encodes one ABI result, validates the exact saved user
  continuation again, and resumes only that continuation; or
- `TerminateUser(status)` consumes the user continuation and returns terminal
  thread status to the registered kernel continuation.

The backend owns the privilege transition, kernel entry stack, saved machine
state, speculation and security sequence, target validation, result register,
and legal fast or general return path. A provider does not select a syscall,
invent a result, or own Linux dispatch policy.

## First executable profile

The initial x86-64 profile creates one process/thread identity and one sealed
qualification-only user probe. Preparation consumes one allocator-owned
three-frame extent and assigns sealed code, stack, and leaf-translation roles.
The provider publishes one read-only executable user page and one writable,
non-executable user stack in a validated user address space; terminal
completion revokes those mappings and returns the exact whole extent for
release. The probe performs native Linux x86-64 syscall `39`
(`getpid`), verifies that the returned value is the source-owned initial PID,
and then performs syscall `231` (`exit_group`) with status zero. A mismatch
terminates with a distinct nonzero status.

The kernel personality recognizes only those two numbers in this profile.
`getpid` returns the process identity encoded according to the pinned Linux
7.2.9 x86-64 ABI. `exit_group` terminates the sole thread group and does not
return to user mode. Kernel source owns both semantic results; the generated
probe merely observes them. Qualification records user entry, both syscall
entries, the successful return after `getpid`, terminal status, and restoration
of the exact registered kernel continuation.

The probe is not an application format or a hidden executable loader. It is a
target-qualified physical witness for the common state transitions. Later ELF
loading must create the same validated semantic state from a checked Linux
image rather than depending on this probe representation.

## Cross-architecture boundary

X86-64 may lower the boundary with `SYSCALL` plus a validated `SYSRET` or
general interrupt return. AArch64 may use `SVC` and exception return, while
RISC-V may use `ECALL` and supervisor return. Their argument registers, saved
status, entry stacks, and return restrictions remain provider evidence. The
portable contract is the affine user continuation, semantic syscall request,
source-selected disposition, and exact return or termination identity.

No architecture is allowed to expose its register frame as the common user
context. Stronger target behavior may not weaken validation or make a malformed
continuation returnable.

## Rejected alternatives

- An ordinary function call cannot represent a privilege transition or the
  nonreturning terminal path.
- A source-visible trap frame would freeze one architecture's registers and
  permit construction of invalid return state.
- A provider-owned syscall dispatcher would move Linux policy and compatibility
  behavior below the checked source boundary.
- Beginning with a general ELF loader would combine image parsing, mapping,
  initial-stack construction, user entry, syscall entry, and return validation
  before the irreducible transition could be qualified independently.
- A private test syscall would avoid rather than test the pinned Linux ABI.

## Deliberate exclusions

This increment does not add arbitrary ELF loading, dynamic linking, executable
replacement, pointer-bearing syscalls, user-copy recovery, user page faults,
signals, restart, seccomp, audit, ptrace, TLS, extended state, fork/clone,
multiple processes or threads, preemption while in user mode, migration, SMP,
or a complete syscall table. The sealed virtual placements and probe are
qualification evidence only and grant no source access to target addresses or
instructions.
