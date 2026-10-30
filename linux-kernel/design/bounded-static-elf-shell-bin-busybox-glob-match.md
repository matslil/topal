# Bounded static-process shell bin-busybox glob match

Status: approved systems-feature design (`TK-DEC-230`).

## Purpose

The bounded shell can open and enumerate sealed `/bin`, but that directory is
empty and the second glob therefore publishes the unmatched literal
`/bin/*`. This increment adds one immutable `busybox` regular-file directory
entry so the same application command proves a nested successful pattern
match and publishes `/bin/busybox\n`. It does not add file lookup, metadata,
contents, opening, execution, or a general VFS.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*\n`, `pwd\n`, and `exit\n`, with the
predecessor's affine input ownership and prompt distinction. The complete root
enumeration and `/bin\n` output, exact `/bin` metadata lookup, and exact
`/bin/` directory open and descriptor metadata shall remain unchanged.

The sealed `/bin` directory shall contain one immutable child entry named
`busybox`, with inode five and native regular-file type. Its parent shall
remain inode four with link count two because a regular-file child does not
add a subdirectory link. This increment grants only directory-enumeration
identity and type-hint authority; it grants no pathname lookup, metadata,
content, open, or execution authority for the child.

The application may issue the predecessor's one bounded directory-record read.
The provider shall validate the complete writable candidate, stage coherent
ordered 24-byte `.` and `..` records followed by one aligned 32-byte `busybox`
record, copy and verify all 80 bytes, erase staging, advance the immutable
description position, and return 80. A following request shall return EOF
without changing the candidate. The application shall explicitly close
descriptor three.

After EOF and close, the shell shall issue one standard-output write containing
exact `/bin/busybox\n`. The provider shall require the completed enumeration,
EOF, and close facts, validate the complete readable thirteen-byte candidate,
stage and compare it before publication, return thirteen, erase staging, and
record one private bin-busybox-glob output fact. The following qualified prompt
and `pwd` shall require that fact. Terminal handling shall clear all child
metadata, directory-description, enumeration, output, and command-time facts.

## X86-64 binding

The predecessor root enumeration, `/bin\n` write, `/bin` metadata lookup,
`/bin/` open, and descriptor metadata remain. Enumeration binds to syscall 217
with descriptor three, a complete checked profile-sized candidate, and results
80 then zero. The third record uses inode five, next offset three, record length
32, `DT_REG=8`, exact `busybox\0`, and zero padding in the native x86-64
`linux_dirent64` layout. Close binds to syscall 3 and result zero. Selected
matched output binds to syscall 1 on descriptor one with count and result
thirteen. Input results remain 11, 5, 37, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Immutable child identity, coherent directory records, position, EOF, close,
application-selected matched output, checked transfers, staging erasure,
observation ordering, and terminal cleanup are portable Linux-personality
policy. X86-64, AArch64, and RISC-V may select different syscall encodings,
native record layouts, calling conventions, mapping layouts, and transfer
implementations. No native selector, register, address, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Publishing provider-owned `/bin/busybox` was rejected because it would bypass
shell glob behavior. Retaining the literal output while adding a record,
returning a fabricated match without the record, consulting host `/bin`, or
admitting arbitrary child names was rejected as incoherent, nondeterministic,
or broader later work. Adding lookup, contents, or execution authority now was
rejected because a successful nested match should be qualified before the
child becomes otherwise observable.

## Deliberate bounds

This increment proves one successful nested glob match for one immutable
regular-file directory entry. It does not add file metadata or contents,
opening or execution, arbitrary traversal, mutation, dynamic applications,
multiple processes, containers, or virtualization.
