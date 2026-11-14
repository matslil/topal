# Step 4 increment: bounded static-process system-profile open

## Outcome

This increment adds the clean shell's read-only system-profile open immediately
after its post-banner effective-user query. The clean fixture issues native
`openat(AT_FDCWD, "/etc/profile", O_RDONLY, 0)`, requires exact descriptor
three, and proceeds only after the result is validated.

The x86-64 provider accepts the open only at the exact unchanged shell-input
phase with every prior interactive observation complete. It validates the
immutable NUL-terminated path, directory-relative origin, read-only flags,
zero mode, active hierarchy, retained initial-stack leaf, and live process
dispositions. Success publishes one private description for an immutable,
empty, launch-owned profile object at the lowest free descriptor. Sealed input
and terminal rollback require that description; rollback revokes it and proves
it absent. The pinned external BusyBox echo profile requires this and all
earlier optional observations to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PROFILE-OPEN-001` and decision
`TK-DEC-215` bind the transition. Structural tests inspect the exact native
syscall sequence, immutable path bytes, predecessor observation, arguments,
descriptor result, private-description publication, successor gating,
terminal revocation, and full context restoration.

Artifact `/194`, provider `/184`, provider object `/176`, root object `/183`,
boot adapter `/94`, and gate `/196` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and
carries the same 1,723-byte continuation shape, exact 28-byte open sequence,
and immutable profile path.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`70638bad4d6a104d52bfcf8c188e2f531f69e0a6abb1798e1292921e3f27fc15`
and
`0e843d8c53e1787d10e941ec5751e2631a54b6cce5086eebbd5e76c488c3ad9d`.
They bind boot image
`2d9f3e5da3fd867cc2a32edf68f66a8c01b0dc96b1766067b384be3ccda608a7`,
distinguish the clean descriptor-three witness from the absent external
observation, complete all ordered markers, remain running afterward, and
remain serially quiescent. Both profiles passed twice. Their serial transcript
hashes are
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites,
full Cargo workspace suite, formatting check, and diff check all pass through
the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because this introduces the first live filesystem-like descriptor
into the ordered application-controlled shell lifecycle. Review combines exact
phase, predecessor, path, argument, lowest-free descriptor, publication,
successor, and revocation checks; physical transcript evidence; full-workspace
validation; and repeated clean plus external QEMU runs.

This does not provide profile metadata, profile reads, descriptor close,
general VFS lookup, a prompt, refillable input, general commands, filesystems,
processes, containers, or VM-host support.
