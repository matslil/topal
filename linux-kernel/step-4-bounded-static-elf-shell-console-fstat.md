# Step 4 increment: bounded static-process shell console `fstat`

## Outcome

This increment publishes the approved immutable console metadata once on the
clean static-shell path after signal setup and before sealed command input. The
x86-64 provider accepts only `fstat(1, 0x7ffba0)` at the exact predecessor
phase, builds the complete native 144-byte value in private process staging,
checks the active writable stack mapping before mutation, copies and verifies
the result, clears and verifies every staging qword, and returns zero without
advancing the shell-input phase.

One private observation distinguishes the clean witness from the pinned
external echo-command profile. The later read and terminal paths require that
exact distinction; terminal handling clears it before the existing disposition
cleanup and complete affine rollback. The clean fixture validates every field,
padding byte, timestamp component, and unused word, erases the complete output,
then executes its existing sealed `echo topal\nexit\n` sequence. The external
BusyBox witness omits the optional branch and remains unchanged.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-CONSOLE-FSTAT-001` and decision
`TK-DEC-206` bind the exact transition while reusing
`TOPAL-SYSTEMS-USER-FSTAT-001`. Structural tests inspect syscall dispatch,
descriptor and candidate identity, all nonzero native fields, every private
staging slot, observation publication, full context restoration, and the clean
fixture's complete output checks.

Artifact `/185`, provider `/175`, provider object `/167`, root object `/174`,
boot adapter `/85`, and gate `/187` identify the qualified contract. During
qualification, the Rust clean fixture's two command-erasure instructions were
also corrected to carry complete four-byte zero immediates; the Python physical
witness already used those complete encodings, and byte-parity validation now
shows only its deliberate supplied-image sentinel differs.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`e048239fa1d27dca4a505bdd3797acdaaa2388f7a3c87ee2cd199cdb88ffc642`
and
`4cad169ab1433507cff47ed08aebf7f93b9ba7dd5160d7cb14e9ee024de8a45c`.
They bind boot image
`18e1d301fd45ea302c16bf158732ca1b693e4ea2bbb1ecd4710754c82a331067`,
distinguish the present and absent console-metadata observations, complete all
ordered markers, remain running afterward, and remain serially quiescent. The
serial transcripts retain SHA-256
`ea53b5b8ef51e503220e8295fc5927a1c0c7279ba1cb298c2a333625a6fa7306`
for the clean path and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`
for the pinned external path.

## Risk and remaining boundary

Risk is high because this adds a complete native UAPI publication inside the
privilege-transition path and extends its private context and teardown proof.
Review combines exact state-machine and byte-layout checks, focused language
and compiler tests, full-workspace validation, Python mirror parity, and clean
plus external QEMU runs.

This does not yet select the interactive BusyBox arguments or provide
`/dev/tty`, terminal probes on all standard descriptors, warning and banner
output, `MAILPATH` lookup, profile lookup, a prompt, refillable input, general
commands, filesystem-backed execution, process creation, containers, or
VM-host support.
