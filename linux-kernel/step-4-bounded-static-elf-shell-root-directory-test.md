# Step 4 increment: bounded static-process root-directory test

## Outcome

This increment inserts exact `test -d /\n` between the qualified `cd /\n`
and `pwd\n` commands. Both profiles consume five ordered publications through
one checked command buffer and issue x86-64 `newfstatat(AT_FDCWD, "/", stat,
0)` for an application-owned exact `/\0` pathname.

The provider publishes a complete 144-byte native `struct stat` value for one
synthetic root directory: inode 3, link count 2, mode `040755`, root ownership,
4,096-byte block size, and coherent zero values for the remaining qualified
fields. The shell evaluates the predicate silently, then continues to the
already qualified root `pwd` result and exact `exit\n` teardown. No general
pathname lookup, VFS, mutable directory, or fabricated storage backend is
introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-ROOT-DIRECTORY-TEST-001` and decision
`TK-DEC-223` bind the transition. Structural qualification inspects all five
input publications, six affine owner states, syscall-262 dispatch, exact
`AT_FDCWD` and zero flags, the complete pathname and output extents, every
published metadata word, clean and external prompt behavior, profile-specific
`pwd` continuation, private staging and erasure, terminal clearing, and
rollback.

Artifact `/202`, provider `/192`, provider object `/184`, root object `/191`,
boot adapter `/102`, and gate `/202` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
2,886-byte continuation with five reads, five clean prompts, root `chdir`, root
metadata observation, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`1e0ef066d156b5d14219f222fdf307588a40e6efe83e959b5e3aa6b0c77ce967`
and
`c1ada35bd7ffb35be9f8dfccb4c94610f4b82e703ef8ae31432de89befdbe9b2`.
They bind boot image
`d65a80af4a6c19733b3d12e192bbc38fa7af75571bc53c86e87fc37342eeeca0`,
distinguish the complete clean metadata witness from the pinned BusyBox
application witness, complete every ordered marker, remain running afterward,
and remain serially quiescent. Their serial transcript hashes are
`bdaef8750019fee9ce838a70b846700c845b823b103d841bf918dfd57af17416`
and
`94292fe0ae15f72d0d1413f0a70ea60fbb74574b9970d273b3fa467fc349eb4d`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because a real application consumes an ABI-native metadata value
while pathname, destination, transfer, observation, and teardown authority must
remain bounded. Review combines exact profile, predecessor, command, pathname,
flags, destination, native layout, ownership, prompt, terminal, rollback, and
physical runtime evidence.

This proves one successful metadata lookup for the already sealed root. It
does not provide arbitrary paths, failures beyond the existing bounded startup
misses, mutable metadata, a general command loop, dynamic applications,
multiple processes, containers, or VM-host support.
