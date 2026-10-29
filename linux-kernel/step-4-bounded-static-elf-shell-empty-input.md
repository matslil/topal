# Step 4 increment: bounded static-process shell empty input

## Outcome

This increment runs the pinned static BusyBox executable as `busybox sh` with
an empty environment and sealed end-of-file input. After the previously
qualified `getppid` boundary, the provider supplies deterministic `uname` and
root-`getcwd` results, rejects terminal discovery with `-ENOTTY`, exposes the
selected default signal dispositions, installs the exact ignored `SIGQUIT`
disposition, returns EOF from the selected read, and accepts only
`exit_group(0)` after that complete sequence.

The clean witness executes the same observable suffix from a disjoint scratch
region and restores every retained stack and process fact before rollback. The
pinned external witness reaches the shell terminal without application output.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-EMPTY-INPUT-001` and decision
`TK-DEC-203` bind the exact bounded transition. Negative source tests reject
another startup or policy. Structural tests inspect the new syscall dispatch,
phase progression through 27, deterministic outputs, exact signal actions,
retained hierarchy and heap checks, clean witness continuation, terminal
admission, rollback, and revision bindings.

Artifact `/182`, provider `/172`, provider object `/164`, root object `/171`,
boot adapter `/82`, and gate `/184` identify the qualified contract.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`e438b63f2253f65d1eb9ecd88d1f5e04482d69259f74f696df0a6ada789a434c`
and
`198b0fdd213d0e74336f9d6875e9792c6dff7565a70b13d64b86d1624fa54463`.
They distinguish the clean empty-input witness from the external BusyBox shell
terminal, complete every ordered marker, remain running afterward, and remain
serially quiescent. The clean witness retains its earlier one-byte newline
probe; the external shell emits no newline. Both bind boot image
`3a62bb75ea22a649233047f0f8c23b3bfb59bc96a158edd89466fe30696d9649`.
Their serial transcripts bind respectively to
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`
and
`8fe946139fc397c9bf92082b215ddc2332e7f55d795f0a8bbacaa2eafed45f86`.

## Risk and remaining boundary

Risk is high because this advances native static-process execution across
identity, working-directory, terminal, signal, and input interfaces while
preserving rollback. Review combines exact state-machine checks, focused
language/compiler tests, full-workspace validation, and clean plus external
QEMU runs.

This does not yet provide a prompt, command parsing, filesystem-backed command
execution, process creation, pipes, job control, general file descriptors,
Docker, Podman, or VM-host support.
