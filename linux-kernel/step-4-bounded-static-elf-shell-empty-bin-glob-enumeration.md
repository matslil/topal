# Step 4 increment: bounded static-process empty-bin glob enumeration

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*\n`. Both the clean fixture and the
pinned external BusyBox enumerate the immutable `bin` child, verify its
directory metadata, open the exact `/bin/` directory, observe only `.` and
`..`, reach directory EOF, close the descriptor, publish the unmatched glob
literally as `/bin/*\n`, and continue through the previously qualified `pwd`
and exit sequence.

The provider admits exactly one child-directory open with flags `0x90800`, one
complete metadata result, one 48-byte two-record directory publication, one
EOF, and one close. It validates the clean stack or pinned BusyBox heap
candidates, stages and reads back all metadata, directory records, and output,
erases staging, and advances the private affine lifecycle before any
successor. No arbitrary lookup, named child beneath `/bin`, symbolic link,
mutable filesystem, general glob engine, host directory, or general VFS is
introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-EMPTY-BIN-GLOB-ENUMERATION-001` and
decision `TK-DEC-229` bind the transition. Structural qualification inspects
the complete 37-byte command, exact `/bin/` pathname and terminator, open flags,
clean and external candidate extents, complete 144-byte metadata, exact `.` and
`..` directory records, EOF, close, exact seven-byte literal output, staging
erasure, lifecycle states six through thirteen, later `pwd`, terminal clearing,
and rollback.

Artifact `/208`, provider `/198`, provider object `/190`, root object `/197`,
boot adapter `/108`, and gate `/208` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
4,074-byte continuation with five reads, five clean prompts, root `chdir`,
named root enumeration, child metadata query, empty child enumeration, literal
glob output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`b18f5f9ed3eb57d0ac0d05263f3b671e8522469be9acbf2085ed72fa6ffadf5e`
and
`7a3a2721dbe335c3c7acde871fc0f7a1c5e908f913b4ee9f386b8098c417a4f1`.
They bind boot image
`e2079c0d0a8bc08e087533e7b6ed7623a57899912b774fa78dd17438b9240524`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the exact directory open, complete metadata, two records, EOF, close,
and literal output, complete every ordered marker, remain running afterward,
and remain serially quiescent. Their serial transcript hashes are
`56966ea009d72a34cac1b1a2aabef66843bfd453b91ebb6f2e561f34c861c184`
and
`47359ed1e0484ea1c35ddc638f45e7615add67f5399b24ba5ca7a89724d72178`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where
applicable.

## Risk and remaining boundary

Risk is high because a successful child-directory open and enumeration now
affect application output while pathname authority, descriptor state,
metadata, candidate mappings, directory records, staging, observations, and
teardown remain bounded. Review combines exact pathname and flag checks, clean
and external mapping profiles, record layout and offsets, EOF, close, literal
output, affine ownership, terminal cleanup, rollback, and physical runtime
evidence.

This proves one empty immutable directory enumeration for one named child. It
does not provide entries beneath `/bin`, arbitrary paths, general glob
matching, mutation, multiple processes, containers, or VM-host support.
