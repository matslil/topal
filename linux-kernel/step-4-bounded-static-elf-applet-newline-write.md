# Step 4 increment: bounded static ELF applet newline output

## Outcome

This increment adds the exact one-byte output operation required by the pinned
BusyBox `echo` startup path. After credential normalization, the clean-room
static process places one line-feed byte in a disjoint part of its retained
initial stack leaf, calls Linux x86-64 `write(1, candidate, 1)`, receives one,
clears the temporary byte, restores its stack pointer, and completes the
existing affine process rollback.

The provider admits only descriptor one, length one, the exact profile-bound
candidate, the expected executable continuation, and the next private startup
phase. It revalidates the active address-space hierarchy and candidate backing,
copies the byte into private kernel staging, verifies line feed, publishes the
staged byte once, clears the staging register, and returns one. The clean
profile binds the candidate to retained stack backing; the future external
`echo` profile binds it to the pinned anonymous program-break backing.

The digest-pinned external BusyBox `true` applet retains its earlier terminal
path and emits no newline. This preserves the real-process regression boundary
without claiming that the currently sealed applet executes the new operation.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-APPLET-NEWLINE-WRITE-001` and decision
`TK-DEC-198` bind the result to one staged line-feed publication. Negative
source tests reject another startup vocabulary, policy, marker, and order.
Structural tests inspect native syscall 1 dispatch, phase-specific admission,
descriptor and length checks, both exact profile candidates, retained-stack and
anonymous-backing validation, byte staging and value validation, one UART
publication, staging erasure, result one, and header-dependent terminal state.
The clean fixture exercises the complete operation; the external run protects
the preceding real process and rollback behavior.

Artifact `/177`, provider `/167`, provider object `/159`, root object `/166`,
boot adapter `/77`, and gate `/179` identify the qualified contract. The root
contains 9,482 typed provider relocations, including 6,858 UART writes, and the
artifact records 65 placements and a 483-transition semantic trace.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. Their SHA-256
values are respectively
`7436b0ef08a44d9eda77e8209bfc542cb451d67f9e045b1cfbbe46000472168c`
and
`d03371b5576e298395eae5e8ba380b24dad827aecc06ba421329d5d92a171d16`.
Both pinned-QEMU runs complete every ordered marker, remain running afterward,
and remain serially quiescent after the terminal fatal disposition. They bind
the common boot image
`1eec5815266a8d622a2449bf3ca6e8560997406782e9cafb0dcf8243a166715b`.
The clean transcript contains exactly one newline and has SHA-256
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`;
the external transcript contains none and retains SHA-256
`8fe946139fc397c9bf92082b215ddc2332e7f55d795f0a8bbacaa2eafed45f86`.
The external file remains outside the repository and is bound only by approved
SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

## Risk and remaining boundary

Risk is high because this introduces an application-visible output effect and
depends on page-table validation and private staging across a native syscall
return. Review combines authority-ordered design and specification,
fail-closed native structural checks, focused language/compiler tests,
full-workspace validation, and clean plus external QEMU executions.

This proves only one profile-bound line-feed write. It does not add arbitrary
write buffers, general descriptor behavior, a shell, Docker, Podman, or
VM-host support. Switching the sealed external applet from `true` to `echo`
remains a later increment.
