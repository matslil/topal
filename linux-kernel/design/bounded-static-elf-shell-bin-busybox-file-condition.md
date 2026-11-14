# Bounded static-process shell bin-busybox file condition

Status: approved systems-feature design (`TK-DEC-231`).

## Purpose

The bounded shell can match the immutable `/bin/busybox` directory entry, but
the entry's native type hint is not a pathname lookup or complete file
metadata. This increment extends the third command with exact
`test -f /bin/busybox && echo busybox-file`, proves one complete application-
issued lookup of the sealed regular file, and exposes the selected true edge as
`busybox-file\n`. It does not add file contents, opening, execution, or a
general VFS.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file\n`,
`pwd\n`, and `exit\n`, with the predecessor's affine input ownership and prompt
distinction. Root and child enumeration, `/bin\n`, `/bin` metadata, exact
`/bin/` open, `.`, `..`, and `busybox` records, EOF, close, and
`/bin/busybox\n` output shall occur first during the third publication.

After consuming the matched child identity, the application may query exact
`/bin/busybox\0`. The provider shall validate the complete pathname and native
metadata candidate, then publish one immutable inode-five regular-file value
with device zero, link count one, mode `0555`, root ownership, zero size,
blocks and times, and 4,096-byte preferred transfer size. The directory entry
and metadata shall agree on name, inode, parent, and regular-file type.

The application shall consume the metadata to select one standard-output write
containing exact `busybox-file\n`. The provider shall require the ordered match
and metadata facts, validate all thirteen readable bytes, stage and compare
them before publication, return thirteen, erase staging, and record one private
file-condition output fact. The following prompt and `pwd` shall require that
fact. Terminal handling shall clear the file metadata, output, and every
predecessor command-time fact.

## X86-64 binding

Child lookup binds to syscall 262 with `AT_FDCWD=-100`, exact
`/bin/busybox\0`, a complete 144-byte checked stack or heap candidate, flags
zero, and result zero. Its 18 qwords are device zero, inode five, link count
one, mode/UID `0x816d`, GID/padding zero, device zero, size zero, block size
4,096, blocks zero, zero timestamps, and zero reserved words. Selected output
binds to syscall 1 on descriptor one with count and result thirteen. Input
results become 11, 5, 80, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Exact lookup, immutable file metadata, condition-selected output, checked
transfers, staging erasure, observation ordering, and terminal cleanup are
portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall encodings, native metadata layouts, calling conventions,
mapping layouts, and transfer implementations. No native selector, register,
address, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Treating `d_type` as complete file metadata was rejected because it is only a
directory-entry hint. Provider-owned confirmation, a fabricated successful
condition without lookup, consulting host `/bin/busybox`, or accepting
arbitrary paths was rejected because each bypasses application behavior or
breaks reproducibility. Adding contents, open, or execution authority now was
rejected because file identity must be qualified first.

## Deliberate bounds

This increment proves one exact immutable regular-file lookup and one selected
condition output. It does not add contents, opening or execution, arbitrary
traversal, mutation, dynamic applications, multiple processes, containers, or
virtualization.
