# Step 4 increment: bounded static-process shell exit command

## Outcome

This increment runs the pinned static BusyBox executable as `busybox sh` with
an empty environment and one sealed `exit\n` command. After the previously
qualified shell startup sequence, the provider admits only the exact standard
input read, copies the five command bytes into the checked application buffer,
and accepts only `exit_group(0)` after that read.

The input has a private single-owner lifecycle. The provider publishes the
sealed bytes only after validating the complete read request, and the terminal
transition erases and proves erasure of all five bytes before releasing the
owner. The clean witness validates the same return value and command bytes,
erases them itself, proves the cleared range, and terminates. The pinned
external BusyBox witness parses the command and exits without application
output.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-EXIT-COMMAND-001` and decision
`TK-DEC-204` bind the exact bounded transition. Negative source tests reject
another startup or input policy. Structural tests inspect the syscall dispatch,
checked destination, private input owner, exact command publication, phase
progression, terminal admission, erasure, rollback, and revision bindings.

Artifact `/183`, provider `/173`, provider object `/165`, root object `/172`,
boot adapter `/83`, and gate `/185` identify the qualified contract.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`d7fc8d31337e4d67a08695d2101ce929c6bed3563f60e7f2e25cd9c5b2b9ac2c`
and
`c4c33c3172fd2b463ace77cc650d7f58701643220783d4ac055edca76bcdd91a`.
They distinguish the clean sealed-command witness from the external BusyBox
shell terminal, complete every ordered marker, remain running afterward, and
remain serially quiescent. The clean witness retains its earlier one-byte
newline probe; the external shell emits no newline. Both bind boot image
`7fa53a2bfc670248176b32bc5902c88b8618c857e409988ffc4123f2f2da5d72`.
Their serial transcripts bind respectively to
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`
and
`8fe946139fc397c9bf92082b215ddc2332e7f55d795f0a8bbacaa2eafed45f86`.

## Risk and remaining boundary

Risk is high because this first advances native static shell execution through
application-visible command input while preserving checked memory publication,
single ownership, terminal ordering, and rollback. Review combines exact
state-machine checks, focused language/compiler tests, full-workspace
validation, and clean plus external QEMU runs.

This does not yet provide a prompt, general command parsing, filesystem-backed
command execution, process creation, pipes, job control, general file
descriptors, Docker, Podman, or VM-host support.
