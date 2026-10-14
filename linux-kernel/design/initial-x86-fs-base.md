# Initial x86-64 user FS-base control

This record resolves `TK-DEC-075` as a target-qualified systems-profile
refinement. It provides the initial thread-pointer operation required by
x86-64 libc without adding a core-language rule or exposing machine registers
to Topal source.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall number 158 to `arch_prctl` and operation
codes `0x1002` and `0x1003` to `ARCH_SET_FS` and `ARCH_GET_FS`. `SET_FS`
selects the user FS base and `GET_FS` copies that value to the supplied user
word. Success returns zero. Other operation codes are outside this increment.

## Decision

The sealed initial-thread personality accepts `ARCH_SET_FS` only when:

- the user context is live;
- the return continuation is within the finalized executable mapping; and
- the base is zero, or its complete one-byte candidate lies in an already-owned
  readable user mapping and is a canonical user address.

It retains the selected base as private initial-thread state and installs it
as the user FS base before returning. `ARCH_GET_FS` requires the same context
and continuation checks plus a complete writable eight-byte destination. It
copies the retained value through the existing fault-contained user-output
path. Both operations return zero on success. Unsupported operation codes
return `-EINVAL`; invalid base candidates return `-EPERM`; invalid output
candidates return `-EFAULT` without changing retained state.

The qualification image sets FS to an address in its RW/NX data mapping, gets
the value into its writable stack, compares the complete word, restores FS to
zero, and gets and compares zero. Data-role teardown clears private FS state
before mapping revocation.

## Alternatives considered

Exposing x86 MSR writes or a named physical FS register to source would put
instruction selection and privilege policy at the wrong abstraction level.
Treating TLS as portable process state would erase genuine ABI differences:
AArch64 and RISC-V use different thread-pointer mechanisms and syscall
surfaces. A target-qualified semantic operation keeps native policy in the ABI
personality while leaving physical installation to its provider.

## Cross-architecture pressure

The portable concept is one thread-pointer value tied to a live user thread;
the Linux `arch_prctl` protocol is specifically x86-64. An AArch64 or RISC-V
personality may implement its ABI's thread-pointer setup without pretending to
support x86 operation codes. No source-visible syscall number, instruction,
register name, MSR number, saved-frame layout, or kernel storage address is
introduced.

This increment does not admit GS-base controls, CPUID controls, tagged-address
policy, dynamic TLS allocation, multiple threads, cloning, context-switch TLS
restoration, signals, ptrace, or a general `arch_prctl` family.
