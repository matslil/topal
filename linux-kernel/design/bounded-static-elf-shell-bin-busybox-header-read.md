# Bounded static-process shell bin-busybox header read

Status: approved systems-feature design (`TK-DEC-235`).

## Purpose

The read-only open creates an immutable description at offset zero, but it
does not prove file-content transfer, description-position advancement, or
application consumption of returned bytes. This increment extends the third
command with exact
`IFS= read -r -n 4 -u 3 _ && echo busybox-read`, admits the pinned shell's
four one-byte reads of the sealed `/bin/busybox` ELF identity, and exposes the
selected success edge as `busybox-read\n`.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read\n`,
`pwd\n`, and `exit\n`, with unchanged affine input ownership and prompt
distinction. Every predecessor transition through `busybox-open\n` shall
complete before content transfer.

The live descriptor-three description shall identify artifact-bound inode
five, remain immutable and read-only, and begin at offset zero. The clean
application shall issue four ordered reads directly, each requesting one byte
into the next byte of one complete writable candidate. The pinned external
application shall first issue two `TCGETS` terminal probes and one `TCSETS`
terminal update against descriptor three. Each shall use its observed complete
stack candidate and fail with `ENOTTY`. Before each read, it shall then issue
`poll` for descriptor three with `POLLIN`, one entry, and an infinite timeout.
The first entry shall have zero `revents`; later entries may retain the
preceding `POLLIN`. Each successful poll shall publish exact `POLLIN`, return
one, and authorize only the immediately following read. The provider shall
validate the calling process, sealed continuation, descriptor owner and
rights, current offset, complete candidate, artifact identity, and
profile-specific probe and readiness order before each transfer. The four read
results shall be one and the bytes shall be exact `0x7f`, `E`, `L`, and `F`,
advancing the description offset from zero to four.

Each byte shall cross private kernel staging before publication. The provider
shall read back the published byte, erase staging, retain no user pointer, and
record only private ordered offset state. A wrong descriptor, count,
candidate, order, offset, content source, duplicate, or post-output read shall
be invalid.

After the fourth successful read, the pinned external application shall issue
one `TCGETS` and one `TCSETS` request against descriptor three with the same
bounded `ENOTTY` behavior. Both applications shall then consume the successful
reads to select one standard-output write containing exact `busybox-read\n`.
The provider shall validate all thirteen readable bytes, stage and compare
them before publication, return thirteen, erase staging, and record one
private ordered read-output fact. The following prompt and `pwd` shall require
that fact while descriptor three remains live at offset four.

The observed pinned application does not explicitly close descriptor three.
Terminal process teardown shall therefore prove and clear the description,
offset, terminal-probe counter, pending-readiness fact, read-output fact, and
every predecessor command-time fact before returning all affine resources.

## X86-64 binding

Each content transfer binds to syscall 0 on descriptor three with count one
and result one. The four writable candidates are consecutive bytes within one
validated user extent. Selected output binds to syscall 1 on descriptor one
with count and result thirteen. The external terminal probes bind to syscall
16 on descriptor three with requests `TCGETS=0x5401` or `TCSETS=0x5402` and
result `-ENOTTY`. Its readiness calls bind to syscall 7, one exact eight-byte
`pollfd`, `nfds=1`, timeout `-1`, result one, and returned `POLLIN`. Input
results become 11, 5, 263, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Immutable description ownership, checked transfers, position advancement,
sealed artifact provenance, ordered terminal-probe failures, readiness
consumption, staging erasure, and terminal revocation are portable
Linux-personality policy. X86-64, AArch64, and RISC-V may select different
syscall numbers, request encodings, calling conventions, mapping layouts, and
copy mechanisms. No native selector, address, register, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

A provider-owned read confirmation was rejected because it would bypass the
application-issued transfers. One four-byte read was rejected because the
pinned BusyBox `read -n 4` builtin issues four observable one-byte reads.
Returning synthetic success without writing the candidates was rejected
because it would not implement Linux read semantics. A host file, an
unvalidated user buffer, and a general descriptor-read implementation were
rejected because they would break sealed provenance, weaken isolation, or
over-expand scope.

Seeking, arbitrary lengths or offsets, EOF, later contents, mapping,
execution, general terminal control or polling, mutation, dynamic
applications, processes, containers, and virtualization remain separate
increments.

## Deliberate bounds

This increment proves the profile-specific bounded terminal-probe and readiness
sequence, four exact one-byte reads of the artifact-bound ELF identity, one
selected output, offset advancement to four, and terminal revocation of the
live description and sequencing facts. It does not add general reads, polling,
terminal control, seeking, mapping, execution, arbitrary opens, mutation,
multiple processes, containers, or virtualization.
