# Step 4 increment: bounded static-process shell `getppid`

## Outcome

This increment reuses the sealed no-parent fact after the bounded child
disposition installation. The clean fixture invokes Linux x86-64 `getppid`,
requires result zero, and exits. The provider validates the executable
continuation, active hierarchy, retained stack, exact predecessor, and live
child disposition without performing process lookup or retaining identity
state.

The existing fixture request clearing was mechanically compacted to keep the
entire qualification program inside its checked executable interval. The
pinned external BusyBox `echo` profile remains unchanged and terminates after
its newline; a future external `sh` profile may use the new transition.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-GETPPID-001` and decision `TK-DEC-202`
bind the zero-parent result. Negative source tests reject another startup or
policy. Structural tests inspect syscall 110 dispatch, the clean zero-result
probe, live disposition checks, active hierarchy and stack validation, phase
19 progression, terminal admission, and revision bindings.

Artifact `/181`, provider `/171`, provider object `/163`, root object `/170`,
boot adapter `/81`, and gate `/183` identify the qualified contract.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`2bfd062015b1f53a16d32d97297ebc56cf856250baeaabaa34369d1ed21b3e9b`
and
`65f884ad9441049c85ce21f9e8e728084d9e08ce3023511f75dfc04de9736ab9`.
They distinguish the clean no-parent witness from the external echo terminal
boundary, complete every ordered marker, contain exactly one newline, remain
running afterward, and remain serially quiescent. Both bind boot image
`ee4d5a013c651de50d8500e84ae949dc3dec493760df48bd5723fc4648b40414`
and serial transcript
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`.

## Risk and remaining boundary

Risk is high because this advances native static-process dispatch while
preserving previously retained signal state. Review combines exact phase and
state checks, focused language/compiler tests, full-workspace validation, and
clean plus external QEMU runs.

This adds no process lookup, parent task, PID allocation, namespace, child
creation, wait, filesystem query, terminal discovery, shell input, Docker,
Podman, or VM-host support.
