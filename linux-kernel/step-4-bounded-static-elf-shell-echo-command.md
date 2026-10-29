# Step 4 increment: bounded static-process shell echo command

## Outcome

This increment runs the pinned static BusyBox executable as `busybox sh` with
an empty environment and one sealed `echo topal\nexit\n` command sequence. The
provider publishes all sixteen input bytes through the checked read, validates
and stages the shell's six-byte `topal\n` result from its caller-selected heap
extent, publishes that result to standard output, and accepts only
`exit_group(0)` afterward.

The sealed input retains its private single-owner lifecycle. Terminal handling
requires the completed-output phase, erases and proves erasure of both
eight-byte command words, and preserves the existing disposition cleanup and
complete rollback. The clean witness validates the same input, output, and
erasure sequence from its checked stack extent. The pinned external witness
executes the real BusyBox built-in and emits exactly `topal\n`.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-ECHO-COMMAND-001` and decision
`TK-DEC-205` bind the exact bounded transition. Negative source tests reject
another startup or policy. Structural tests inspect checked input and output
extents, the complete heap mapping, staging before publication, exact command
and output bytes, ownership consumption, phase progression, terminal erasure,
rollback, and revision bindings.

Artifact `/184`, provider `/174`, provider object `/166`, root object `/173`,
boot adapter `/84`, and gate `/186` identify the qualified contract.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`7193cd656f2cf40c41847a8374e7f39dc1cd4807ec014db41b79b0c2460d11c6`
and
`fca63b690846fe86c7cea1ae2aa87292f6ed028f9d8eeac34fc023e94b5a99f4`.
They distinguish the clean sealed-command witness from the external BusyBox
shell, complete every ordered marker, remain running afterward, and remain
serially quiescent. The clean witness emits its earlier one-byte newline and
the new `topal\n`; the external shell emits only `topal\n`. Both bind boot image
`71702de0ccc67fdc23c4fafedd74e1958d56982f8ccbb65953ee4fc634470a56`.
Their serial transcripts bind respectively to
`ea53b5b8ef51e503220e8295fc5927a1c0c7279ba1cb298c2a333625a6fa7306`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

## Risk and remaining boundary

Risk is high because this first executes an output-producing shell command and
publishes a caller-selected application extent while preserving complete heap
identity validation, staging, ordering, ownership, erasure, and rollback.
Review combines exact state-machine checks, focused language/compiler tests,
full-workspace validation, and clean plus external QEMU runs.

This does not yet provide an interactive prompt, general commands,
filesystem-backed command execution, process creation, pipes, job control,
general file descriptors, Docker, Podman, or VM-host support.
