# Bounded static-process shell bin-busybox reopen

Status: approved systems-feature design (`TK-DEC-237`).

## Purpose

The explicit-close increment destroys the last description bound to sealed
`/bin/busybox`, but it does not prove that the released lowest descriptor can
be reused for a distinct description generation or that a reopened regular
file starts at offset zero. This increment keeps the proven third command
byte-for-byte unchanged, replaces the fourth publication with exact
`exec 3</bin/busybox;pwd\n`, and admits the application's second open before
the existing `pwd` observation.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read; exec 3<&-; echo busybox-close\n`,
`exec 3</bin/busybox;pwd\n`, and `exit\n`, with unchanged affine input
ownership and prompt distinction. Every predecessor transition through
`busybox-close\n`, including the profile-specific following-prompt
observation, shall complete first.

Both qualified profiles shall issue a second exact `AT_FDCWD` read-only open
of `/bin/busybox` with mode zero. The provider shall require descriptor three,
descriptor ten, and the prior description to be absent; validate the complete
immutable pathname and artifact identity; allocate the lowest available slot,
exact descriptor three; and bind it to a new immutable description generation
with read-only rights and offset zero. No state, offset, reference, terminal-
probe count, or readiness fact from the destroyed description may be reused.

After the reopen succeeds, the application shall execute the existing `pwd`
segment from the same publication. Its profile-specific pathname and output
flow and the following prompt shall require exactly the new descriptor-three
mapping and its fresh offset-zero description to remain live. Reopen success
is therefore application-selected by sequencing rather than by a new
provider- or application-owned confirmation line.

Terminal process teardown shall revoke the reopened mapping and description
exactly once, clear their generation, rights, offset, and command-time facts
with every predecessor command-time fact, and return all affine resources.
It shall prove that the destroyed predecessor generation was not revived.

## X86-64 binding

The reopen binds to syscall 257 with `AT_FDCWD=-100`, the complete exact
pathname, flags zero, mode zero, and result three. Input results become 11, 5,
294, 24, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX. The longer fourth
publication deterministically moves the external profile's following complete
144-byte console-stat candidate from the predecessor address to exact
`0x007f_f240` and its following complete 36-byte `TCGETS` candidate from the
predecessor address to exact `0x007f_f1a0`; the clean profile still issues no
command-time console stat or terminal probe.

## Architecture pressure test

Description generations, lowest-available descriptor reuse, fresh offsets,
mapping ownership, last-close destruction, and terminal revocation are
portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall numbers, calling conventions, and flag encodings. No native
selector, register, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Reviving the destroyed description was rejected because a later open creates
a new open-file description with an independent offset. Retaining descriptor
ten, selecting a descriptor other than the lowest free slot, inheriting offset
four, adding a synthetic confirmation line, or fabricating provider-owned
output would misstate observable Linux lifetime or bypass application
behavior. General descriptor tables, arbitrary
open, and execution remain separate increments.

## Deliberate bounds

This increment proves one exact second open, lowest-slot reuse, a distinct
description generation at offset zero, application-selected continuation into
`pwd`, continued lifetime through `pwd`, and terminal revocation. It does not
add arbitrary paths or
flags, general descriptor allocation, seeking, later reads, mapping,
execution, mutation, multiple processes, containers, or virtualization.
