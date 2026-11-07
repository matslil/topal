# Step 4 increment: bounded static-process bin-busybox read-only open

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open\n`.
After every previously qualified condition, both the clean fixture and pinned
external BusyBox issue exact
`openat(AT_FDCWD, "/bin/busybox", O_RDONLY, 0)`, receive descriptor three,
emit `busybox-open\n`, retain the description across `pwd`, and revoke it at
terminal process teardown.

The provider binds descriptor three to one process-owned immutable read-only
description of artifact-bound inode five at offset zero. It validates the
complete pathname in the retained low-heap mapping or exact clean-fixture
candidate, stages and compares all thirteen selected output bytes, erases the
staging, and retains no user pointer. No content read, seek, mapping,
execution, mutation, host filesystem, arbitrary pathname, or general VFS
authority is added.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-READ-ONLY-OPEN-001`
and decision `TK-DEC-234` bind the transition. Structural qualification
inspects the exact 216-byte command, five input publications, exact open
arguments and result, complete `/bin/busybox\0` pathname, artifact identity,
descriptor-three ownership, exact thirteen-byte selected output, staging
erasure, lifecycle states twenty-one and twenty-two, later `pwd`, terminal
revocation, and rollback.

Artifact `/213`, provider `/203`, provider object `/195`, root object `/202`,
boot adapter `/113`, and gate `/213` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte artifact extent and is
byte-identical to the Rust fixture over the full 5,951-byte continuation,
which contains the application-issued open, selected output, retained
description, and terminal revocation paths.

The pinned external pathname and output candidates are wholly contained in
the retained low heap below `0x00600000`. Qualification proves the complete
relevant RW/NX leaf set without requiring unrelated high-heap mappings and
validates the saved syscall continuation before helper scratch registers are
used.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`d9ae0b42564162de2fccb019dc2d2a41d38a3267c7512bb2fb77a9ae8daabeff`
and
`f187e7b39d8a3f82745846bcf9161f54b12b281c3dc3119e46c6679f465727e1`.
They bind boot image
`a00ff9c817f19bdff5c82949120004f1c1ffd039c0c31c0da4115c9f2b6a6bcd`
and linked kernel
`591006b5007ebef4d2b1763aaa20b7a93652e5b1a9d50a4ed914c8f923baf66b`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the exact read-only open and `busybox-open` output, complete every
ordered marker, remain running afterward, and remain serially quiescent.
Their serial transcript hashes are
`74a848fee51b79d5a4c41298028872f61827270a102d0dfb27e835fc046355ed`
and
`65e071cb8f702666f7179990124d804417cae40a658101f03083952aa8f45945`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because a successful application-issued open creates a live
description and affects visible control flow. Review combines exact ABI
arguments, sealed artifact provenance, candidate-scoped mapping proof,
immutable description ownership, output identity, descriptor lifetime,
terminal cleanup, rollback, and clean plus external physical runtime evidence.

This proves one successful read-only open of one immutable sealed object. It
does not provide file content reads, seeking, executable mappings, execution,
arbitrary opens, mutation, multiple processes, containers, or VM-host
support.
