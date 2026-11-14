# Bounded static-process shell empty-bin glob enumeration

Status: approved systems-feature design (`TK-DEC-229`).

## Purpose

The bounded shell can enumerate sealed root and describe its immutable `bin`
child, but it cannot open or enumerate that child. This increment replaces the
third publication with exact
`echo /*; test -d /bin && echo /bin/*\n`. It preserves the real root match and
named-child metadata test, then proves one application-issued `/bin` open,
coherent empty-directory enumeration, EOF, explicit close, and application
publication of the unmatched literal `/bin/*\n`. It does not add a child entry,
file contents, executable lookup, or a general VFS.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*\n`, `pwd\n`, and `exit\n`, with the
predecessor's affine input ownership and prompt distinction. The complete root
open, coherent `.`, `..`, and `bin` record publication, EOF, explicit close,
exact `/bin\n` match, and exact `/bin` metadata lookup shall occur first during
the third publication.

After consuming the named directory value, the application may open exact
`/bin/\0` read-only as a directory. The provider shall require the named-child
metadata fact, validate the complete pathname and flags, allocate the lowest
free nonstandard descriptor, and bind it to one immutable directory
description for inode four at position zero. Its metadata shall remain inode
four, parent inode three, link count two, directory mode `0755`, root
ownership, zero size and times, and 4,096-byte preferred transfer size.

The application may query descriptor-three metadata and issue one bounded
directory-record read. The provider shall validate the complete writable
candidate, stage coherent ordered 24-byte `.` and `..` `linux_dirent64`
records, copy and verify all 48 bytes, erase staging, advance the description
position, and return 48. A following request shall return EOF without changing
the candidate. The application shall explicitly close descriptor three.

Because no non-dot child exists, the shell shall issue one standard-output
write containing exact `/bin/*\n`. The provider shall require completed EOF and
close facts, validate the complete readable seven-byte candidate, stage and
compare it before publication, return seven, erase staging, and record one
private empty-bin-glob output fact. The following qualified prompt and `pwd`
shall require that fact. Terminal handling shall clear all child metadata,
directory-description, enumeration, output, and command-time facts.

## X86-64 binding

The predecessor root enumeration, `/bin\n` write, and named lookup remain.
Child open binds to syscall 257 with `AT_FDCWD=-100`, exact `/bin/\0`, flags
`O_RDONLY|O_NONBLOCK|O_DIRECTORY|O_CLOEXEC=0x90800`, mode zero, and result
three.
Descriptor metadata binds to syscall 5 and the predecessor's complete 144-byte
inode-four layout. Enumeration binds to syscall 217 with descriptor three, a
complete checked profile-sized candidate, and results 48 then zero. Each
24-byte record uses the native x86-64 layout: inode, next offset, record length,
type, name, and zero padding. Close binds to syscall 3 and result zero. Selected
literal output binds to syscall 1 on descriptor one with count and result
seven. Input results become 11, 5, 37, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Named directory open, immutable description state, coherent records, EOF,
close, application-selected literal output, checked transfers, staging
erasure, observation ordering, and terminal cleanup are portable
Linux-personality policy. X86-64, AArch64, and RISC-V may select different
syscall encodings, native record and metadata layouts, calling conventions,
mapping layouts, and transfer implementations. No native selector, register,
address, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Publishing provider-owned `/bin/*` was rejected because it would bypass shell
glob behavior. Returning immediate EOF without dot records was rejected because
Linux directory enumeration exposes coherent `.` and `..` entries. Reusing the
root description, retaining it across close, consulting host `/bin`, or
accepting arbitrary paths was rejected as incoherent, nondeterministic, or
broader later work. Adding a synthetic executable child now was rejected
because child-directory traversal and empty enumeration should be qualified
before file identity, contents, or execution authority.

## Deliberate bounds

This increment proves one exact open and empty enumeration for the immutable
named directory child. It does not add entries below `/bin`, file metadata or
contents, child execution, arbitrary traversal, mutation, dynamic
applications, multiple processes, containers, or virtualization.
