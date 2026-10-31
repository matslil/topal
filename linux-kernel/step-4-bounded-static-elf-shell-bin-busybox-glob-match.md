# Step 4 increment: bounded static-process bin-busybox glob match

## Outcome

This increment retains the sealed shell command
`echo /*; test -d /bin && echo /bin/*\n` and adds one immutable `busybox`
regular-file directory entry beneath `/bin`. Both the clean fixture and the
pinned external BusyBox preserve the root match and `/bin` metadata, open and
describe exact `/bin/`, enumerate `.`, `..`, and `busybox`, reach directory
EOF, close the descriptor, publish the successful match as
`/bin/busybox\n`, and continue through the previously qualified `pwd` and exit
sequence.

The provider admits exactly one inode-five regular-file directory identity,
one 80-byte three-record publication, one EOF, one close, and one thirteen-byte
matched output. It validates the clean stack or pinned BusyBox heap candidates,
stages and reads back every directory and output byte, erases staging, and
advances the private affine lifecycle before any successor. No file lookup,
metadata, content, opening, execution, arbitrary child, host directory, or
general VFS is introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-GLOB-MATCH-001` and decision
`TK-DEC-230` bind the transition. Structural qualification inspects the
unchanged 37-byte command, exact `/bin/` pathname and open flags, clean and
external candidate extents, complete inode-four metadata, exact native `.`,
`..`, and `busybox` directory records, EOF, close, exact thirteen-byte matched
output, staging erasure, lifecycle states six through thirteen, later `pwd`,
terminal clearing, and rollback.

Artifact `/209`, provider `/199`, provider object `/191`, root object `/198`,
boot adapter `/109`, and gate `/209` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
4,150-byte continuation with five reads, five clean prompts, root `chdir`,
named root enumeration, child metadata query, three-record child enumeration,
matched output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`bc2ffddcf292ba5705e7907b06f6f1760b3dfcc7a51cb880326abb613a877e0d`
and
`f1824b395392eebee617f73d79e7504f9783d2d075ca9effb128024f363c1387`.
They bind boot image
`d69964edbba43448c27e8717e7d47de0d0d9af0b3db4de9ae409e9286d295c09`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the exact three-record enumeration, EOF, close, and matched output,
complete every ordered marker, remain running afterward, and remain serially
quiescent. Their serial transcript hashes are
`2aaae6c11b44982f9fbfa5398549ab3c226f21223e6522e416b05ac7ee227620`
and
`0d5b8b191e2bcbd41205cb9e6deee64c2227f24dc885d5de97d02071717c9010`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where
applicable.

## Risk and remaining boundary

Risk is high because a newly visible nested regular-file identity changes
application glob selection and output while child authority, descriptor state,
candidate mappings, native records, staging, observations, and teardown remain
bounded. Review combines exact record identity, layout, type and offset checks,
clean and external mapping profiles, EOF, close, matched output, affine
ownership, terminal cleanup, rollback, and physical runtime evidence.

This proves one successful nested match for one immutable regular-file
directory entry. It does not provide file lookup, metadata or contents,
opening or execution, arbitrary paths, mutation, multiple processes,
containers, or VM-host support.
