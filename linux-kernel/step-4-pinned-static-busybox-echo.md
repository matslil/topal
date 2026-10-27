# Step 4 increment: pinned static BusyBox `echo`

## Outcome

This increment advances the first real external application from silent
BusyBox `true` to BusyBox `echo`. The complete initial stack now seals the
semantic arguments `busybox`, `echo`; every auxiliary-vector reconstruction,
process-entry validation, syscall return, and rollback check preserves the same
strings and pointer relations until complete clearing.

With the sealed root identity, the pinned executable proceeds directly from
`getuid(0)` to `write(1, 0x5f8450, 1)` and `exit_group(0)`. The provider admits
that transition only for the external resident-header profile, revalidates the
exact anonymous backing, stages and verifies line feed, emits it once, clears
staging, and requires the common newline terminal phase. The clean-room fixture
retains its longer configuration and credential path before the same output
and terminal phase.

The external artifact remains outside the repository and is accepted only by
its approved SHA-256 digest. No executable bytes or host path enter evidence.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ECHO-PROFILE-001` and decision `TK-DEC-199`
bind the result to one exact argument profile and root-specific startup branch.
Negative systems-source tests reject the former profile name. Structural tests
inspect `echo\0` construction and every later reconstruction, the exact clean
and external predecessor phases, both profile-bound output candidates, common
newline terminal state, and removal of the former external early exits.

Artifact `/178`, provider `/168`, provider object `/160`, root object `/167`,
boot adapter `/78`, and gate `/180` identify the qualified contract. The root
contains 9,482 typed provider relocations, including 6,858 UART writes, and the
artifact records 65 placements and a 483-transition semantic trace.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. Their SHA-256
values are respectively
`50069871525b759748a7d3f55dbdcbbde4d4315d49493797e9fc21c87c1d1ec2`
and
`22a740cab2e7801a013f312f069317e0a29120d398d7ffc2af2176cd3dbcee90`.
Both pinned-QEMU runs complete every ordered marker, contain exactly one
physical newline, remain running afterward, and remain serially quiescent after
the terminal fatal disposition. They bind boot image
`a8a4e3178f486319a35626cc5caa316d4649780490f5f4961bac90e91d840a99`
and serial transcript
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`.
The external file is bound only by approved SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

## Risk and remaining boundary

Risk is high because this changes a process-entry ABI fact and makes a real
external executable produce an application-visible effect. Review combines
authority-ordered design and specification, a differential native trace for
the sealed-root branch, fail-closed structural checks, focused language and
compiler tests, full-workspace validation, and clean plus external QEMU runs.

This proves only empty-line `echo` from one digest-pinned static executable. It
does not add arbitrary arguments or output, a command parser, interactive
input, a shell, dynamic libc, Docker, Podman, or VM-host support.
