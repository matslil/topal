# Step 4 increment: bounded static-process named-root glob match

## Outcome

This increment refines sealed root with one immutable `bin` directory child.
Both profiles retain exact `echo /*\n`, receive coherent `.`, `..`, and `bin`
records, observe EOF, explicitly close the root description, and publish the
application-selected glob match `/bin\n`.

The provider keeps root inode three and child inode four, updates root link
count to three, and exposes the child only through enumeration. It validates
the clean stack or pinned BusyBox heap candidates, stages every returned and
selected byte before publication, erases staging, and retains the existing
private affine lifecycle. No child lookup, child open, arbitrary pathname,
general glob evaluator, host directory, mutable filesystem, output stream, or
general VFS is introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-NAMED-ROOT-GLOB-MATCH-001` and decision
`TK-DEC-227` bind the transition. Structural qualification inspects all five
input publications, coherent parent metadata, exact child inode and type,
three ordered 24-byte `linux_dirent64` records, zero padding, 72-byte result,
explicit EOF and close, exact five-byte selected output, clean and external
candidates, private staging and erasure, the complete lifecycle, later `pwd`,
terminal clearing, and rollback.

Artifact `/206`, provider `/196`, provider object `/188`, root object `/195`,
boot adapter `/106`, and gate `/206` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
3,216-byte continuation with five reads, five clean prompts, root `chdir`,
named root enumeration, selected glob output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`99e2425e33bf71630e056ced1fc470e950171dcda53cf8bce4e6a4b26221b789`
and
`22b74fc5e676258f00c55e75772831ccdb16981f9006b8f899fcfac4e4435b96`.
They bind boot image
`44242c2bf08e70e568ae9d80968b10af64e2aaa9b6662eca1e00ad264f849304`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
complete every ordered marker, remain running afterward, and remain serially
quiescent. Their serial transcript hashes are
`8891dcf3bb6c37d79cd84eac8800dd7dfde99497872eaac2f499522b7e90e5e9`
and
`bcdbd6736d336e7ac2fcbc2bfa5fee33c80e31af536e4c34212cfdce5a02c1b8`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because adding a child changes parent metadata, native record
content, and application control flow while descriptor state, candidate
authority, staging, observations, address-space state, and teardown remain
bounded. Review combines exact records and parent metadata, clean and external
mapping profiles, EOF and close, application-selected output, affine
ownership, terminal cleanup, rollback, and physical runtime evidence.

This proves one successful glob match for one immutable named directory child.
It does not provide child lookup or open, child contents, arbitrary patterns or
paths, file contents, mutation, multiple processes, containers, or VM-host
support.
