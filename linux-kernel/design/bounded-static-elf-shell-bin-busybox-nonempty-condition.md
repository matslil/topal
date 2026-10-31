# Bounded static-process shell bin-busybox nonempty condition

Status: approved systems-feature design (`TK-DEC-233`).

## Purpose

The executable condition proves pathname identity, mode, and credentials but
still describes the sealed `/bin/busybox` inode as empty. That prevents its
metadata from coherently representing the exact digest-checked static ELF
already carried by the boot resource. This increment binds the inode size to
that artifact, extends the third command with exact
`test -s /bin/busybox && echo busybox-nonempty`, and exposes the selected true
edge as `busybox-nonempty\n`. It does not open, read, map, or execute the file.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty\n`,
`pwd\n`, and `exit\n`, with unchanged affine input ownership and prompt
distinction. Every predecessor transition shall complete before the new
nonempty observation.

The immutable inode-five metadata shall report size 2,024,544, equal to the
complete published static-user artifact selected and digest-checked before
boot packaging. Its block count shall be 3,955 512-byte units, the minimal
whole-unit coverage of that byte extent. Every earlier metadata observation
in the same profile shall report the identical values.

After the executable-condition output, the application may repeat the complete
metadata query for exact `/bin/busybox\0`. The provider shall validate the
pathname and metadata candidate, republish the same artifact-bound inode-five
metadata, and record one private ordered nonempty-metadata fact. It shall not
retain a user pointer or consult a host filesystem.

The application shall consume the positive size to select one standard-output
write containing exact `busybox-nonempty\n`. The provider shall validate all
seventeen readable bytes, stage and compare them before publication, return
seventeen, erase staging, and record one private nonempty-condition output
fact. The following prompt and `pwd` shall require that fact. Terminal handling
shall clear both new facts and every predecessor command-time fact.

## X86-64 binding

The repeated metadata query binds to syscall 262 with `AT_FDCWD=-100`, exact
`/bin/busybox\0`, the complete 144-byte candidate, and flags zero. Selected
output binds to syscall 1 on descriptor one with count and result seventeen.
Input results become 11, 5, 176, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Artifact-bound file identity and size, complete metadata, selected condition
output, checked transfers, staging erasure, observation ordering, and terminal
cleanup are portable Linux-personality policy. X86-64, AArch64, and RISC-V may
select different stat layouts, syscall encodings, calling conventions, mapping
layouts, and transfer mechanisms. No native selector, address, register, or
instruction becomes Topal source vocabulary.

## Alternatives rejected

Leaving a zero size while claiming the inode represents the published ELF was
rejected as incoherent. Using a host file size was rejected because boot
artifact provenance already supplies the authoritative identity. Treating the
predecessor metadata as the application observation was rejected because the
pinned application issues a fresh metadata query for `test -s`.

Provider-owned confirmation, arbitrary paths, partial metadata, and immediate
open or execution authority were rejected because they bypass application
behavior or expand authority before file contents and loading are separately
qualified.

## Deliberate bounds

This increment proves one exact artifact-bound nonempty-file observation and
one selected condition output. It does not add opening, contents, mapping,
execution, arbitrary metadata, mutation, dynamic applications, multiple
processes, containers, or virtualization.
