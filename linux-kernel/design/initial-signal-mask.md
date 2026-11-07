# Initial thread signal mask

This record resolves `TK-DEC-079` as a systems-profile refinement. It adds the
Linux blocked-signal state used by libc without claiming signal delivery or
changing core-language meaning.

## Baseline behavior

Linux 7.2.9 x86-64 assigns syscall 14 to `rt_sigprocmask`. Its native kernel
signal set is one unsigned 64-bit word and the size argument must be eight.
`SIG_BLOCK`, `SIG_UNBLOCK`, and `SIG_SETMASK` respectively add, remove, and
replace blocked bits. A null input is a query for which `how` is ignored. A
nonnull output receives the mask from before any update. Linux removes the
`SIGKILL` and `SIGSTOP` bits before installing a supplied set.

## Decision

The initial thread owns one private 64-bit blocked-signal word initialized to
zero. Linux-compatible `rt_sigprocmask(how, set, old-set, 8)` is admitted only
for the live context and a return continuation in the finalized executable
mapping. A nonnull input and nonnull output must each denote a complete owned
eight-byte readable or writable user extent respectively.

The provider first stages a supplied input word under fault recovery and
removes bits 8 and 18, corresponding to signals 9 and 19. It rejects an
unsupported `how` before changing state. It computes the requested block,
unblock, or replacement result privately, retains the prior word, commits the
new word, and then publishes the prior word through an optional output under
fault recovery, matching Linux's update-before-output ordering. A null input
leaves state unchanged and ignores `how`; a null output performs no user
write. Terminal teardown clears the private word.

The qualification application installs a word containing one ordinary signal
and both unblockable signals, proves the returned prior word was empty, queries
and proves that only the ordinary signal remained, unblocks it while proving
the prior word, and finally queries the empty result.

## Alternatives considered

Returning zero without storing state would let startup probes pass while later
mask-dependent behavior disagreed. Admitting only queries would not exercise
the libc update ABI. Adding signal actions or delivery now would require frame,
pending-queue, restart, and asynchronous-continuation policy that this
increment does not establish.

## Cross-architecture pressure

The portable contract is private per-thread blocked-signal state with
fault-contained native-layout transfer and Linux update ordering. Syscall
numbers, argument registers, native set size and encoding, and unblockable bit
positions remain ABI-provider facts. A future architecture provider can map
the same lifecycle to its native ABI without exposing registers or signal
frames to Topal source.

This increment does not admit signal generation, pending queues, action
registration, delivery, alternate stacks, interrupted-syscall restart,
`rt_sigreturn`, multiple threads, or a general signal subsystem.
