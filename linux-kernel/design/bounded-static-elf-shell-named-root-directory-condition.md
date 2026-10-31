# Bounded static-process shell named-root directory condition

Status: approved systems-feature design (`TK-DEC-228`).

## Purpose

The bounded shell can match sealed `/bin` through root enumeration but cannot
look up or describe that child. This increment replaces the third publication
with exact `echo /*; test -d /bin && echo bin-directory\n`. It first preserves
the real glob match, then proves one successful application-issued lookup,
complete metadata transfer, true conditional edge, and selected output for the
same sealed child. It does not grant arbitrary traversal, child enumeration or
contents, or a general VFS.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo bin-directory\n`, `pwd\n`, and `exit\n`, with
the predecessor's affine input ownership and prompt distinction. The complete
root open, coherent `.`, `..`, and `bin` record publication, EOF, explicit
close, and exact `/bin\n` matched output shall occur first during the third
publication.

After the matched output, the application may issue one metadata request for
exact `/bin\0` relative to `AT_FDCWD`, with zero flags and one complete native
output candidate. The provider shall require the named-entry and matched-output
facts, validate the complete pathname and writable candidate, and publish one
deterministic immutable directory value with inode four, link count two,
directory mode `0755`, root ownership, zero size and times, and 4,096-byte
preferred transfer size. It shall copy and verify every byte, return zero,
erase staging, and retain only one private named-child-metadata fact.

After consuming that value, the shell shall select its true edge and issue one
standard-output write containing exact `bin-directory\n`. The provider shall
require the metadata fact, validate the complete readable 14-byte candidate,
stage and compare it before publication, return 14, erase staging, and record
one private named-condition-output fact. The following qualified prompt and
`pwd` shall require the completed named condition. Terminal handling shall
clear all child metadata, enumeration, descriptor, output, and command-time
facts.

## X86-64 binding

The predecessor's open, descriptor metadata, `getdents64`, close, and `/bin\n`
write bindings remain. Named lookup binds to syscall 262 with `AT_FDCWD=-100`,
exact `/bin\0`, a complete 144-byte stack or heap candidate, flags zero, and
result zero. Its 18 qwords are device zero, inode four, link count two,
mode/UID `0x41ed`, GID/padding zero, device zero, size zero, block size 4,096,
blocks zero, zero timestamps, and zero reserved words. Selected conditional
output binds to syscall 1 on descriptor one with count and result 14. Input
results become 11, 5, 44, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Sealed child lookup, coherent metadata, application-selected condition flow,
checked transfers, staging erasure, observation ordering, and terminal cleanup
are portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall encodings, native metadata layouts, calling conventions,
mapping layouts, and transfer implementations. No native selector, register,
address, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Publishing a provider-owned child confirmation was rejected because it would
bypass lookup and application control flow. Reusing the enumeration record as
if it were complete metadata was rejected because `d_type` does not satisfy a
native metadata request. Consulting host `/bin`, accepting relative or
arbitrary paths, or granting open or enumeration authority was rejected as
nondeterministic or broader later work. Splitting the condition into another
input publication was rejected because the shell already supports ordered
commands within one bounded publication and the combined line preserves the
existing five-publication lifecycle.

## Deliberate bounds

This increment proves one successful exact lookup and directory condition for
the one sealed named child. It does not add arbitrary traversal, child open or
enumeration, child contents, file contents, mutation, dynamic applications,
multiple processes, containers, or virtualization.
