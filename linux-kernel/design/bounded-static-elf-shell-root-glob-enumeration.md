# Bounded static-process shell root-glob enumeration

Status: approved systems-feature design (`TK-DEC-226`).

## Purpose

The bounded shell can query fixed pathnames but cannot enumerate a directory.
This increment replaces the third command with exact `echo /*\n`. It proves
that the application opens the sealed root as a directory, consumes native
directory entries for `.` and `..`, observes end-of-directory, closes the
description, resolves the no-match glob to its literal form, and publishes
exact `/*\n` without granting a general VFS, arbitrary globbing, or arbitrary
directory contents.

## Decision

The bounded static-process entry may admit exactly five ordered command
publications through one checked process buffer: `echo topal\n`, `cd /\n`,
`echo /*\n`, `pwd\n`, and `exit\n`. Each refill shall replace a consumed
publication, zero and validate its sealed tail, retain no user pointer, and
advance one exclusive private input-owner state. The clean profile shall
publish exact `/ # ` before every command; the pinned external BusyBox profile
shall require those optional prompts absent.

After consuming the third command, both applications may revalidate sealed-root
metadata through the already qualified exact-root request. They shall open
exact `/\0` relative to `AT_FDCWD` with read-only, nonblocking, close-on-exec,
directory-only flags. The provider shall validate the complete pathname and
selectors, allocate lowest-free descriptor three, retain one immutable root
directory description at position zero, and return three. No host directory or
entry shall be consulted.

Before enumeration, the application shall request native metadata for
descriptor three into one complete checked 144-byte stack candidate. The
provider shall publish one deterministic root-directory value with inode three,
link count two, directory mode `0755`, root ownership, zero size and times, and
4,096-byte preferred transfer size, then erase its private staging. This value
describes the sealed root and shall not incorporate host metadata.

The application shall issue `getdents64` on descriptor three with a complete
checked writable candidate. The clean fixture may request 512 bytes from its
owned stack; pinned BusyBox may request 32,768 bytes from its owned heap. The
provider shall stage and publish exactly two coherent 24-byte Linux directory
records for `.` and `..`, both directory typed and bound to the sealed root,
advance the private description position, return 48, and erase staging. A
second request on the same description and candidate shall return zero without
output and record end-of-directory. The application shall explicitly close
descriptor three, consuming the description and freeing the slot.

Because enumeration exposes no child matching `*`, the shell shall retain the
literal glob and issue one standard-output write containing exact `/*\n`. The
provider shall require enumeration EOF and explicit close, validate the
complete readable three-byte candidate, stage and compare it before
publication, return three, erase staging, and record one private glob-output
fact. The following qualified `pwd` shall require that fact. Terminal handling
shall clear all enumeration, descriptor, output, and command-time facts.

## X86-64 binding

The x86-64 realization binds open to syscall 257 with `AT_FDCWD=-100`, exact
`/\0`, flags `O_RDONLY|O_NONBLOCK|O_CLOEXEC|O_DIRECTORY` (`0x90800`), mode
zero, and result three. Directory metadata binds to syscall 5, descriptor
three, a complete 144-byte candidate, and result zero. Its 18 qwords are device
zero, inode three, link count two, mode/UID `0x41ed`, GID/padding zero, device
zero, size zero, block size 4,096, blocks zero, zero timestamps, and zero
reserved words. Enumeration binds to syscall 217, descriptor three,
profile-qualified count 512 or 32,768, first result 48, and second result
zero. Each native record has 24-byte length and `DT_DIR`; the first carries
name `.\0`, the second `..\0`, and all padding is zero. Close binds to syscall
3 and result zero. Selected output binds to syscall 1 on descriptor one with
count and result three. Input results are 11, 5, 8, 4, and 5. The affine stack
remains `[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Directory-description ownership, coherent entry identity, position and EOF,
checked transfer, explicit close, no-match glob selection, staging erasure,
and terminal cleanup are portable Linux-personality policy. X86-64, AArch64,
and RISC-V may select different syscall encodings, native directory-record
layouts, flag encodings, calling conventions, mapping layouts, and transfer
implementations. No native selector, register, address, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Publishing provider-owned entry text was rejected because it bypasses
application enumeration and glob selection. Returning EOF immediately was
rejected because Linux root enumeration includes `.` and `..`. Omitting the
descriptor metadata request, second entry request, or explicit close was
rejected because it loses application compatibility, position, EOF, or
lifecycle evidence. Inheriting host-root entries or metadata, accepting arbitrary
paths, entries, buffer sizes, flags, descriptors, or output, or adding a
general glob evaluator was rejected as nondeterministic or broader later work.

## Deliberate bounds

This increment proves one immutable root enumeration with only `.` and `..`
and one no-match glob. It does not add arbitrary paths or patterns, child
objects, file contents, mutation, seeking, concurrent descriptions, dynamic
applications, multiple processes, containers, or virtualization.
