# Bounded static-process shell bin-busybox read-only open

Status: approved systems-feature design (`TK-DEC-234`).

## Purpose

The nonempty condition proves coherent metadata for the sealed
`/bin/busybox` artifact, but it does not create a file description. This
increment extends the third command with exact
`exec 3</bin/busybox; echo busybox-open`, admits the pinned application's
read-only open of that exact object as descriptor three, and exposes the
selected success edge as `busybox-open\n`. It does not read, map, or execute
file contents.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open\n`,
`pwd\n`, and `exit\n`, with unchanged affine input ownership and prompt
distinction. Every predecessor transition shall complete before the open.

After the nonempty output, the application may open exact
`/bin/busybox\0` relative to `AT_FDCWD` with read-only flags and mode zero.
The provider shall validate the calling process, command owner, complete
pathname, candidate extent, artifact-bound inode-five identity, and absence
of a live nonstandard description. It shall then bind the lowest free
descriptor, three, to one process-owned immutable read-only description at
offset zero. It shall retain no user pointer and shall not consult a host
filesystem.

The application shall consume the successful open to select one
standard-output write containing exact `busybox-open\n`. The provider shall
validate all thirteen readable bytes, stage and compare them before
publication, return thirteen, erase staging, and record one private ordered
open-output fact. The following prompt and `pwd` shall require that fact.

The observed pinned application does not explicitly close descriptor three.
The description shall therefore remain live across `pwd` and be revoked by
terminal process teardown. Teardown shall prove and clear the description,
the output fact, and every predecessor command-time fact before returning all
affine resources.

## X86-64 binding

The open binds to syscall 257 with `AT_FDCWD=-100`, exact
`/bin/busybox\0`, flags zero, mode zero, and result three. Selected output
binds to syscall 1 on descriptor one with count and result thirteen. Input
results become 11, 5, 216, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Exact pathname identity, immutable read-only description ownership,
lowest-free descriptor selection, offset zero, checked transfers, ordered
observation, and teardown revocation are portable Linux-personality policy.
X86-64, AArch64, and RISC-V may select different syscall encodings, calling
conventions, mapping layouts, and transfer mechanisms. No native selector,
address, register, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Provider-owned confirmation and an implicit descriptor were rejected because
they would bypass the application-issued open. Forcing an explicit close was
rejected because the observed pinned application retains the description
until exit. A host pathname was rejected because the sealed boot artifact is
the authoritative object.

Adding a content read, mapping, execution, mutation, arbitrary path lookup,
or a general open implementation in the same increment was rejected because
each adds separately observable authority that requires independent
qualification.

## Deliberate bounds

This increment proves one exact read-only open of the artifact-bound sealed
`/bin/busybox`, one selected output, and terminal revocation of the live
description. It does not add reads, seeks, mapping, execution, arbitrary
opens, mutation, dynamic applications, multiple processes, containers, or
virtualization.
