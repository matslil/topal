# Step 4 increment: bounded static-process bin-busybox reopen

## Outcome

This increment retains the exact 294-byte explicit-close command and replaces
the following `pwd` publication with exact
`exec 3</bin/busybox;pwd\n`. After both mappings of the predecessor description
have been closed and its last reference destroyed, the clean fixture and the
pinned external BusyBox both issue a second exact read-only open of
`/bin/busybox`. The provider reuses lowest-available descriptor three for a
distinct immutable description generation at offset zero, retains it through
the application-selected `pwd`, and revokes it exactly once at process exit.

No provider-owned reopen confirmation is emitted. The five input results are
11, 5, 294, 24, and 5, and the existing clean-prompt versus external-no-prompt
distinction remains observable. Native tracing fixed the longer publication's
external command-time console-stat and `TCGETS` candidates at `0x007f_f240`
and `0x007f_f1a0` respectively.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-REOPEN-001` and decision
`TK-DEC-237` bind the transition. Structural qualification checks exact old-
generation absence, second-open ABI arguments and result, lowest-slot reuse,
fresh description identity and offset, clean auxiliary state, application-
selected continuation, candidate extents, prompt ordering, terminal
revocation, command-buffer erasure, and affine rollback.

Artifact `/216`, provider `/206`, provider object `/198`, root object `/205`,
boot adapter `/116`, and gate `/216` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte artifact extent and is
byte-identical to the Rust fixture over the complete 6,645-byte continuation.
Both profiles validate the same portable description-generation and mapping
lifecycle; only the predecessor profile-specific probes and prompts differ.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`ca5d13f78c732dd3a19743def09981f6b4bf0e2bd258f2a060c976c3c552756d`
and
`534df00cd196fc2bd02d35128cbe37296266dcac29fc4fc08d611c8acdccb722`.
They bind boot image
`311a3b000cf3918186b43b4f96072e4f739b8a9b56d6ec07ed622091792fb06d`
and linked kernel
`ee3886201e1bb75a95bc0099c10586d5a77ff7c70a9834a0e7ce6ab38872e4d6`,
observe the exact reopen and terminal-revocation profiles, complete every
ordered marker, remain running afterward, and remain serially quiescent. Their
serial transcript hashes are
`d4107591069569de5cb08c67a4d94c118a5a9bab8f553fb7742201e9228ec2eb`
and
`503965eac7e475e287e32ab8d2b7dd59b1e6cf187ecb6848621bdc1a7522c769`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and duplicate clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because this increment changes application-visible descriptor
generation, reuse, retention, and destruction ordering. Review combines exact
ABI arguments, immutable artifact identity, prior-generation absence,
lowest-available allocation, fresh offset and auxiliary state, complete
candidate proofs, application output identity, terminal cleanup, rollback,
and clean plus external physical runtime evidence.

This proves one exact reopen of one sealed immutable path after one exact
last-close transaction. It does not provide general path lookup, arbitrary
open flags, a general descriptor table, seeking, later reads, mapping,
execution, mutation, multiple processes, containers, or VM-host support.
