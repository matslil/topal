# Step 4 increment: bounded static-process bin-busybox header read

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read\n`.
After every previously qualified transition, both the clean fixture and pinned
external BusyBox issue four ordered one-byte reads on descriptor three,
receive exact artifact-bound `0x7fELF`, advance the description from offset
zero to four, emit `busybox-read\n`, retain the description across `pwd`, and
revoke it at terminal process teardown.

Native tracing exposed a profile distinction hidden by the original read-only
model. The pinned external BusyBox issues two `TCGETS` requests and one
`TCSETS` request that receive `ENOTTY`, then four exact `poll(POLLIN)` and read
pairs, followed by one more `TCGETS` and `TCSETS` miss. The clean fixture reads
directly. The provider models only those two qualified sequences: it validates
the complete terminal and poll candidates, orders all five misses, makes each
readiness fact single-use, clears it on its matching read, and revokes the
probe counter with the description. This does not add general terminal-control
or polling authority.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-HEADER-READ-001` and decision
`TK-DEC-235` bind the transition. Structural qualification inspects the exact
263-byte command, five input publications, the external profile's exact
terminal-probe and readiness sequence, four read arguments and results,
`0x7fELF` identity, complete writable candidates, offset progression, exact
thirteen-byte selected output, staging erasure, lifecycle states twenty-two
through twenty-seven, later `pwd`, terminal revocation, and rollback.

Artifact `/214`, provider `/204`, provider object `/196`, root object `/203`,
boot adapter `/114`, and gate `/214` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte artifact extent and is
byte-identical to the Rust fixture over the full 6,294-byte continuation,
which contains the four application-issued reads, selected output, retained
description, and terminal revocation paths.

The pinned external read and output candidates are wholly contained in the
retained low heap below `0x00600000`; its terminal and poll candidates are in
the affine RW/NX stack. Qualification proves each complete candidate and the
saved syscall continuation before helper scratch registers are used. The clean
profile proves its consecutive stack candidates without admitting the
external-only probe sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`f7e2ea0879464024a17f420e6b905f1fbe7075131efc892bbf5795bd3655c5cb`
and
`dcbe6d613ab62778d631229ef0941f2dd9b42464fc91c069f58c1577f5caaa54`.
They bind boot image
`6ae6443651eb3301f3c9b1e8c4405821b60bf2400bc1edac7f08778edd90ca77`
and linked kernel
`151df90250a02951eb884c4daa074e0fad595aa090720f7c7ab2381ba2428087`,
distinguish direct clean reads from the pinned BusyBox probe and readiness
sequence, observe exact `0x7fELF` transfer and `busybox-read` output, complete
every ordered marker, remain running afterward, and remain serially quiescent.
Their serial transcript hashes are
`fefe420286a009542201bc00748c8300387bbf1edcac68baff46506bc8d5680d`
and
`6c99b6e293aed84b2ac95ab773640f49c0481c3eb599e0553ecd438535c96fdd`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because application-visible content transfer mutates user memory,
advances a live description, and depends on profile-specific syscall ordering.
Review combines exact ABI arguments, sealed artifact provenance, complete
candidate proofs, immutable description ownership, single-use readiness,
ordered failure behavior, staging erasure, output identity, terminal cleanup,
rollback, and clean plus external physical runtime evidence.

This proves four exact one-byte reads of one immutable sealed object plus only
the observed bounded probe and readiness behavior. It does not provide general
reads, EOF, seeking, terminal control, polling, executable mappings, execution,
arbitrary opens, mutation, multiple processes, containers, or VM-host support.
