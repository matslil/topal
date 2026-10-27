# Step 4 increment: bounded static-process shell `getpid`

## Outcome

This increment reuses the existing sealed PID-one process identity at the
bounded static-process boundary. The clean fixture calls Linux x86-64
`getpid` after its newline output, requires result one, and then exits. The
provider validates the executable continuation, active hierarchy, retained
stack leaf, header-selected predecessor, and result phase without retaining a
native argument or creating mutable task state.

The current pinned external BusyBox `echo` profile still terminates after its
newline. The provider also seals the future external BusyBox `sh` predecessor
from `getuid(0)` to the same identity observation, but this increment does not
yet change the external argument profile or claim shell execution.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-GETPID-001` and decision `TK-DEC-200`
bind the result to one PID-one observation. Negative source tests reject
another startup or policy. Structural tests inspect syscall 39 dispatch, the
clean result-one probe, both header-selected predecessor phases, active-root
and retained-stack validation, phase 17 progression, terminal admission, and
revision bindings.

Artifact `/179`, provider `/169`, provider object `/161`, root object `/168`,
boot adapter `/79`, and gate `/181` identify the qualified contract. The root
contains 9,482 typed provider relocations, including 6,858 UART writes, and the
artifact records 65 placements and a 483-transition semantic trace.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`cd6f73bd2b8e26e170eedd247a23ef286ddcc79d8e87bfbcf4cf9ebd0fda7335`
and
`92a25d7992314081766055f2777538a038c82014bfd90f7c13a0d56cddb3bc81`.
They distinguish the clean PID-one witness from the external echo terminal
boundary, complete every ordered marker, contain exactly one newline, remain
running afterward, and remain serially quiescent. Both bind boot image
`7ec0638f23618a7a7ac3087e491864ec54b741de36aa2fd3e14cb2d3bbca8d17`
and serial transcript
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`.

## Risk and remaining boundary

Risk is high because this extends native static-process dispatch and prepares
a different real application path. Review combines the pre-existing scalar
identity contract, exact phase checks, focused language/compiler tests,
full-workspace validation, and clean plus external QEMU runs.

This adds no PID allocation, `getppid`, signals, startup-file lookup, terminal
discovery, shell input, Docker, Podman, or VM-host support.
