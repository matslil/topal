# Step 4 increment: bounded static-process named-root directory condition

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo bin-directory\n`. Both the clean fixture and
the pinned external BusyBox first enumerate the immutable `bin` child, then
issue `newfstatat(AT_FDCWD, "/bin", ..., 0)`, receive complete directory
metadata, select the true branch, publish `bin-directory\n`, and continue
through the previously qualified `pwd` and exit sequence.

The provider admits exactly one child pathname and one successful condition.
It binds inode four, link count two, directory mode `0755`, and 4 KiB block
size; validates the clean stack or pinned BusyBox heap candidates; stages and
reads back all metadata and output; erases staging; and advances the private
affine lifecycle before any successor. No arbitrary lookup, symbolic link,
child open, child contents, mutable filesystem, general condition evaluator,
host directory, or general VFS is introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-NAMED-ROOT-DIRECTORY-CONDITION-001` and
decision `TK-DEC-228` bind the transition. Structural qualification inspects
the complete 44-byte command, exact `/bin` pathname and terminator, syscall
arguments, clean and external candidate extents, complete 144-byte metadata,
exact fourteen-byte selected output, staging erasure, lifecycle states six
through eight, later `pwd`, terminal clearing, and rollback.

Artifact `/207`, provider `/197`, provider object `/189`, root object `/196`,
boot adapter `/107`, and gate `/207` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
3,629-byte continuation with five reads, five clean prompts, root `chdir`,
named root enumeration, child metadata query, selected conditional output,
root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`ace297d07d2978a0729514679344f327acd99fa2b950c851645f969d628c9279`
and
`626f68c8ddaf7b5c6b05c2907e516009ddc8e8e454f8df214aaac4a248a8bed7`.
They bind boot image
`b461433d24ea85391110ad4ecdc0f1f4ca831520d981a5906d6e57dc85425653`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe complete `/bin` metadata and the selected condition, complete every
ordered marker, remain running afterward, and remain serially quiescent. Their
serial transcript hashes are
`b81e55804b8a664f55a265dfc8500ab89757b389b19aa65212e9a1d31d809c17`
and
`9ba398cfda6ff93a21918fdcb450b1995623f542c69d861954efad921b49a2ae`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where
applicable.

## Risk and remaining boundary

Risk is high because a successful named lookup now affects application control
flow while pathname authority, metadata, candidate mappings, descriptor state,
staging, observations, and teardown remain bounded. Review combines exact
pathname and metadata checks, clean and external mapping profiles, selected
output, affine ownership, terminal cleanup, rollback, and physical runtime
evidence.

This proves one successful directory test for one immutable named child. It
does not provide child open or contents, arbitrary paths, false conditions,
general shell conditions, mutation, multiple processes, containers, or VM-host
support.
