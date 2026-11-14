# Step 4 increment: bounded static-process bin-busybox explicit close

## Outcome

This increment extends the sealed shell command with exact
`exec 3<&-; echo busybox-close`. After every previously qualified transition,
both the clean fixture and pinned external BusyBox duplicate live descriptor
three to close-on-exec descriptor ten, close descriptor three while retaining
the shared immutable description, close descriptor ten and destroy the last
description reference, then emit exact application-owned `busybox-close\n`.
The following prompt and `pwd` prove that both mappings and the description are
absent.

Native tracing fixed the external shell transaction as syscall 72 with
descriptor three, `F_DUPFD_CLOEXEC=0x406`, minimum ten, and result ten, followed
by syscall 3 on descriptors three and ten with zero results. The provider
models only this ordered transaction. It retains one shared identity, rights,
and offset-four description until the last mapping is removed; revokes the
description, offset, probe count, and readiness facts exactly once; and records
a distinct output-publication fact before admitting the next prompt.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-EXPLICIT-CLOSE-001` and
decision `TK-DEC-236` bind the transition. Structural qualification inspects
the complete 294-byte command, five input publications, exact duplicate
arguments and result, ordered mapping removal, shared-description lifetime,
last-close destruction, exact fourteen-byte output, staging erasure, prompt
distinction, later `pwd`, terminal proof of absence, and affine rollback.

Artifact `/215`, provider `/205`, provider object `/197`, root object `/204`,
boot adapter `/115`, and gate `/215` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte artifact extent and is
byte-identical to the Rust fixture over the complete 6,517-byte continuation.
Both profiles validate the same portable mapping and description lifecycle;
only the predecessor header-read terminal and readiness behavior remains
profile-specific.

The clean and external write candidates remain wholly inside their qualified
RW/NX stack or retained low-heap extents. The provider validates each complete
candidate, source continuation, profile, lifecycle, owner, mapping identity,
rights, offset, and command disposition before changing retained state or
publishing output.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`f63bc5f2c4b9ab24c1a92718f1bf2cebf479c2f2416df8b15a5dff4862baf0f8`
and
`e58569e30c782c8fe6e633affeedd67eaa427019e5de1058068821738d5f3143`.
They bind boot image
`83d995a17ebb0bb83200aa88d92e33b5e57ce89b48dfac0f0c866dfdab8b4a4a`
and linked kernel
`7938ae34ed7c69e6c8b2a21fae877e7df7303933d6c923b476f898071353a3fd`,
observe exact duplicate, close, absence, and `busybox-close` profiles, complete
every ordered marker, remain running afterward, and remain serially quiescent.
Their serial transcript hashes are
`d4107591069569de5cb08c67a4d94c118a5a9bab8f553fb7742201e9228ec2eb`
and
`503965eac7e475e287e32ab8d2b7dd59b1e6cf187ecb6848621bdc1a7522c769`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because this increment changes application-visible descriptor
ownership and destruction ordering. Review combines exact ABI arguments,
lowest-available result, shared-offset preservation, mapping lifetime,
last-reference revocation, complete candidate proofs, ordered failure
behavior, staging erasure, output identity, terminal cleanup, rollback, and
clean plus external physical runtime evidence.

This proves one exact close-on-exec duplicate and two exact ordered closes for
one immutable sealed description. It does not provide general `fcntl`, an
arbitrary descriptor table, exec-time close-on-exec processing, arbitrary
close, seeking, mapping, execution, mutation, multiple processes, containers,
or VM-host support.
