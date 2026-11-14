# Step 4 increment: bounded static-process shell `/dev/tty` open miss

## Outcome

This increment reports the truthful absence of the optional controlling-terminal
pathname immediately after the clean static shell's console metadata check. The
clean fixture issues exactly `openat(AT_FDCWD, "/dev/tty", O_RDWR, 0)` and
requires `-ENOENT` before sealed command input. The x86-64 provider admits that
exchange only at the established predecessor phase, validates the immutable
nine-byte pathname and every native argument, and returns the miss without
allocating a descriptor, retaining lookup state, or advancing the phase.

The completed console-`fstat` and `/dev/tty`-miss observations occupy two
ordered bytes of one already qualified private qword. Later clean input and
terminal rollback require both facts; terminal handling clears the complete
qword. The pinned external BusyBox echo profile omits both optional interactive
prerequisites and continues to require the all-zero observation.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-DEV-TTY-OPEN-MISS-001` and decision
`TK-DEC-207` bind the transition. Structural tests inspect native syscall 257,
the exact low 32-bit `AT_FDCWD` value, pathname candidate and bytes, `O_RDWR`,
zero mode, exact `-ENOENT`, observation publication, clean-fixture sequencing,
and full context restoration.

Artifact `/186`, provider `/176`, provider object `/168`, root object `/175`,
boot adapter `/86`, and gate `/188` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and records the
same immutable path and 31-byte open/miss exchange.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`726707e5d45c33992e5fc812742694cb44eda4725245f1964dbd4f1727c3a632`
and
`54dd123e8652db9cc261a48689e1b94310a77039759f4d2e3adf31bf3bd0df18`.
They bind boot image
`53b389f2d812f2c45d384239599087822d88aebdf590332f6a60c00bd5311aba`,
distinguish the clean `ENOENT` witness from the absent external observation,
complete all ordered markers, remain running afterward, and remain serially
quiescent. Both profiles passed twice. Their serial transcripts retain SHA-256
`ea53b5b8ef51e503220e8295fc5927a1c0c7279ba1cb298c2a333625a6fa7306`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites, full
Cargo workspace suite, formatting check, and diff check all pass through the
bounded validation runner.

## Risk and remaining boundary

Risk is high because this extends the native syscall boundary and the ordered
private process state used by later input and teardown. Review combines exact
argument and byte-layout checks, state-ordering checks, full-workspace
validation, Python mirror parity, and repeated clean plus external QEMU runs.

This does not provide `/dev/tty`, descriptor allocation, general pathname
lookup, terminal probes on the three standard descriptors, warning or banner
output, `MAILPATH` lookup, profile lookup, a prompt, refillable input, general
commands, filesystem-backed execution, process creation, containers, or
VM-host support.
