# Bounded static BusyBox shell empty-input profile

Status: approved systems-feature design (`TK-DEC-203`).

## Purpose

The digest-pinned BusyBox shell, launched with the already-sealed empty
environment and no input, proceeds from `getppid` through one system-identity
query, one current-directory query, one terminal probe, three
default-disposition observations, one ignored-disposition installation, and
one end-of-input read before exiting successfully. This increment admits that
complete bounded suffix and changes the external qualification argument from
`echo` to `sh`.

## Portable contract

The live initial process may perform exactly this ordered suffix:

1. receive the existing source-owned system identity in one complete output;
2. receive `/` as the sealed current directory in one bounded output;
3. observe that descriptor zero is not a terminal, with no output mutation;
4. observe default dispositions for interrupt and quit;
5. install the ignored quit disposition from a copied, validated request;
6. observe the default termination disposition;
7. read end-of-input from descriptor zero without modifying the destination;
8. exit with status zero.

The identity publication reuses the established six-field `new_utsname`
record. The current-directory publication is exactly `"/\0"`, retains no
pathname lookup or mutable directory state, and derives only from the sealed
root launch. Every transition requires the exact predecessor phase, executable
continuation, active checked address space, retained initial stack, and live
child disposition. Output dispositions are published as complete zeroed
values only into one checked writable stack extent. The ignored disposition
is retained as process-owned state until terminal validation and clearing.
The end-of-input result is permitted only for the sealed empty-input launch;
it retains no user pointer or input state. Failure preserves the predecessor
phase and every process resource.

The clean fixture may continue through a disjoint executable island after its
original contiguous probe area. The island is part of the same sealed
executable role; it grants no dynamic code publication or general branching
authority.

## Architecture bindings

X86-64 binds system identity to `uname`, current directory to
`getcwd(candidate, 4096)` returning two, the terminal probe to
`ioctl(0, TCGETS, candidate)` returning `-ENOTTY`, disposition operations to
`rt_sigaction` with native signal-set size eight, and end-of-input to
`read(0, candidate, 1023)` returning zero. The native action layout is four
eight-byte fields. AArch64 or RISC-V uses its own selectors, request value,
action layout, and calling convention. Portable policy names system identity,
sealed current directory, terminal capability, signal dispositions, and
sealed input state rather than those native encodings.

## Exclusions

This adds no mutable system identity, UTS namespace, directory traversal,
pathname lookup, mutable working directory, terminal mode, successful terminal
ioctl, general descriptor ioctl, signal generation or delivery, pending queue,
signal frame, `rt_sigreturn`, handler execution, mutable input stream, blocking
read, filesystem, process creation, job control, pipeline, command execution,
container, or virtualization support.
