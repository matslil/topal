# Step 4 increment: bounded static-process shell standard-terminal probes

## Outcome

This increment adds the three ordered terminal-capability observations that
follow the clean shell's `/dev/tty` absence result. The clean fixture issues
`ioctl(fd, TCGETS, candidate)` for standard error, standard output, and standard
input in that order and requires `-ENOTTY` from every call. It places a sentinel
in the checked stack candidate, verifies the sentinel after each result, and
erases and verifies it before sealed command input.

The x86-64 provider requires the completed console-`fstat` and `/dev/tty`
observations, exact unchanged shell-input predecessor phase, live dispositions,
checked hierarchy, exact next descriptor and request, and a complete 36-byte
writable initial-stack extent. It does not dereference or modify the candidate.
One private byte counts zero through three; clean input and terminal rollback
require three, while the pinned external BusyBox echo profile continues to
require the complete observation qword to remain zero.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-STANDARD-TERMINAL-PROBES-001` and decision
`TK-DEC-208` bind the transition. Structural tests inspect all three native
syscall sequences, the untouched-candidate witness, exact predecessor qword
states, private count publication, complete-count requirements, and full
context restoration.

Artifact `/187`, provider `/177`, provider object `/169`, root object `/176`,
boot adapter `/87`, and gate `/189` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and records the
same 107-byte probe and no-output sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`d141686269585299d085393aece7ba12a3ca5dd5d22c6acba334d0946af761fc`
and
`e354879fcf9e82841482e8da1581f969ecc57ce77b7a3316c5465e58da235c1d`.
They bind boot image
`f3709f9908a086033a8433b1ceb3c116f3ff8ea494098f772e9158c6c7b4411e`,
distinguish the clean ordered three-miss witness from the absent external
observation, complete all ordered markers, remain running afterward, and remain
serially quiescent. Both profiles passed twice. Their serial transcript hashes
remain
`ea53b5b8ef51e503220e8295fc5927a1c0c7279ba1cb298c2a333625a6fa7306`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites, full
Cargo workspace suite, formatting check, and diff check all pass through the
bounded validation runner.

## Risk and remaining boundary

Risk is high because this refines a shared native syscall selector into two
phase-specific contracts and extends ordered private process state. Review
combines exact state and descriptor checks, complete candidate-bound checks,
no-output structural and physical witnesses, full-workspace validation, mirror
parity, and repeated clean plus external QEMU runs.

This does not provide terminal attributes, successful terminal operations,
arbitrary `ioctl`, warning or banner output, `MAILPATH` lookup, profile lookup,
a prompt, refillable input, general commands, filesystem-backed execution,
process creation, containers, or VM-host support.
