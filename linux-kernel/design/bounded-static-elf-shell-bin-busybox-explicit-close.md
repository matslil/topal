# Bounded static-process shell bin-busybox explicit close

Status: approved systems-feature design (`TK-DEC-236`).

## Purpose

The header-read increment leaves descriptor three live until terminal process
teardown. That proves implicit process-exit revocation but not application-
issued descriptor removal, preservation of the underlying description while a
saved mapping remains, or destruction on the last close. This increment
extends the third command with exact
`exec 3<&-; echo busybox-close`, admits the pinned shell's observed bounded
save-and-close sequence, and exposes its selected success edge as
`busybox-close\n`.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read; exec 3<&-; echo busybox-close\n`,
`pwd\n`, and `exit\n`, with unchanged affine input ownership and prompt
distinction. Every predecessor transition through `busybox-read\n` shall
complete first.

Both qualified profiles shall implement the observed shell-redirection
transaction. The application shall request a close-on-exec duplicate of live
descriptor three using a minimum descriptor of ten. The provider shall bind
the lowest available result, exact descriptor ten, to the same immutable
artifact description and current offset four, return ten, and retain the
description once for the two mappings. A wrong source, command, minimum,
result, lifecycle, owner, description, rights, offset, or duplicate request
shall be invalid.

The application shall then close descriptor three. The description shall
remain live solely through descriptor ten with its identity, rights, and
offset unchanged. It shall then close descriptor ten. That last mapping
removal shall destroy the description and revoke its offset, ordered terminal-
probe count, and readiness state. Both close calls shall return zero. A wrong
descriptor, order, duplicate close, or access after its mapping is removed
shall be invalid.

After the last close, the application shall issue one standard-output write
containing exact `busybox-close\n`. The provider shall validate all fourteen
readable bytes, stage and compare them before publication, return fourteen,
erase staging, and record one private ordered close-output fact. The following
prompt and `pwd` shall require that fact and require both nonstandard mappings
and their description to be absent.

Terminal process teardown shall prove and clear the close-output and bounded
redirection facts with every predecessor command-time fact before returning
all affine resources. It shall not perform a second description revocation.

## X86-64 binding

The duplicate binds to syscall 72 with descriptor three,
`F_DUPFD_CLOEXEC=0x406`, minimum ten, and result ten. Mapping removal binds to
syscall 3 first on descriptor three and then on descriptor ten, each with
result zero. Selected output binds to syscall 1 on descriptor one with count
and result fourteen. Input results become 11, 5, 294, 4, and 5. The affine
stack remains `[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Description identity, shared offset, mapping ownership, close-on-exec
metadata, lowest-available selection, ordered mapping removal, last-close
destruction, and terminal proof are portable Linux-personality policy. X86-64,
AArch64, and RISC-V may select different syscall numbers, calling conventions,
flag encodings, and lowering. No native selector, register, or instruction
becomes Topal source vocabulary.

## Alternatives rejected

Closing descriptor three directly was rejected because it would omit the
pinned application's observed save mapping and would fail to prove shared
description lifetime. Treating the duplicate as an independent description
was rejected because Linux duplication shares the open-file description and
offset. Destroying the description on the first close, retaining it after the
last close, forcing terminal cleanup, or synthesizing provider-owned output
would misstate observable ownership or bypass application behavior. General
`fcntl`, arbitrary descriptor allocation, fork inheritance, and close-on-exec
execution remain separate increments.

## Deliberate bounds

This increment proves one exact descriptor-three to descriptor-ten close-on-
exec duplication, two ordered closes, last-mapping destruction, one selected
output, and terminal proof of absence. It does not add general duplication,
descriptor tables, exec-time processing, arbitrary close, reads, seeking,
mapping, execution, mutation, multiple processes, containers, or
virtualization.
