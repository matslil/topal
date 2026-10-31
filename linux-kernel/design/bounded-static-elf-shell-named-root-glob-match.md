# Bounded static-process shell named-root glob match

Status: approved systems-feature design (`TK-DEC-227`).

## Purpose

The bounded shell can enumerate only `.` and `..`, so its `/*` pattern has no
match and remains literal. This increment adds one sealed named root entry and
proves that the application consumes it through native directory records,
selects it as the glob match, and publishes exact `/bin\n`. It does not grant
arbitrary directory contents, pathname traversal, file access, or a general
VFS.

## Decision

The bounded static-process entry retains the five approved command
publications: `echo topal\n`, `cd /\n`, `echo /*\n`, `pwd\n`, and `exit\n`,
including their affine input ownership and profile-specific prompt behavior.

The sealed filesystem model shall refine root with one immutable child named
`bin`. The child has inode four and directory type. Root remains inode three,
but its link count becomes three to account for the named child directory.
This increment exposes only the child's name, inode, and type through root
enumeration; it does not authorize lookup, open, metadata publication, or
enumeration of `/bin`.

The qualified root open, descriptor-three metadata request, directory
description, candidate profiles, EOF request, and explicit close remain in
force. The first `getdents64` publication shall now contain three coherent
24-byte records in order: `.`, `..`, and `bin`. The first two bind to root
inode three, the third binds to child inode four, every record has directory
type, offsets advance through one, two, and three, padding is zero, the result
is 72, and the description advances to its terminal position. The second
request shall return zero without output.

Because `bin` is the one non-dot entry matching `*`, the shell shall expand
`/*` and issue one standard-output write containing exact `/bin\n`. The
provider shall require enumeration EOF and explicit close, validate the
complete readable five-byte candidate, stage and compare it before
publication, return five, erase staging, and record one private matched-glob
fact. The following qualified `pwd` shall require that fact. Terminal handling
shall clear all enumeration, descriptor, output, and command-time facts.

## X86-64 binding

The existing syscall 257 open and syscall 5 metadata bindings remain. Root
metadata changes only its native link-count qword from two to three. Syscall
217 retains descriptor three and profile-qualified count 512 or 32,768. Its
first result becomes 72 and contains three 24-byte `linux_dirent64` records:
inode/offset/name `(3,1,".")`, `(3,2,"..")`, and `(4,3,"bin")`, each with
`DT_DIR` and zero padding. The second result remains zero. Syscall 3 closes the
description with result zero. Syscall 1 publishes exact `/bin\n` on descriptor
one with count and result five. Input results remain 11, 5, 8, 4, and 5. The
affine stack remains `[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Sealed child identity, coherent parent metadata, native record sequencing,
application-selected matching, checked transfer, affine descriptor ownership,
staging erasure, and terminal cleanup are portable Linux-personality policy.
X86-64, AArch64, and RISC-V may select different syscall encodings, record
layouts, flag encodings, calling conventions, mapping layouts, and transfer
implementations. No native selector, register, address, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Publishing `/bin\n` without a named record was rejected because it would
bypass application glob selection. Adding a regular placeholder only to avoid
root link-count refinement was rejected because the next useful sealed object
is a directory. Consulting the host root, exposing several conventional names,
or authorizing lookup or open was rejected as nondeterministic or broader
later work. Repacking records below their already qualified 24-byte extent was
rejected because stable exact records keep this refinement independently
reviewable.

## Deliberate bounds

This increment proves one immutable named directory child through one root
glob match. It does not add arbitrary paths or patterns, child lookup or open,
child contents, file contents, mutation, seeking, concurrent descriptions,
dynamic applications, multiple processes, containers, or virtualization.
