# Bounded static-process shell bin-busybox executable condition

Status: approved systems-feature design (`TK-DEC-232`).

## Purpose

Complete regular-file metadata proves the sealed `/bin/busybox` identity but
does not prove the application-visible execute-permission decision. This
increment extends the third command with exact
`test -x /bin/busybox && echo busybox-executable`, proves one application-
issued execute-access observation sequence for the sealed file, and exposes
the selected true edge as `busybox-executable\n`. It does not open, read, map,
or execute the file and does not add a general permission engine or VFS.

## Decision

The bounded static-process entry shall retain five ordered publications:
`echo topal\n`, `cd /\n`, exact
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable\n`,
`pwd\n`, and `exit\n`, with the predecessor's affine input ownership and prompt
distinction. Every root, `/bin`, nested glob, complete file-metadata, and
selected file-condition transition shall occur first during the third
publication.

After consuming the regular-file condition, the application may repeat the
complete metadata query for exact `/bin/busybox\0` and then query its effective
UID. The provider shall validate the complete pathname and metadata candidate,
republish the identical immutable inode-five mode `0555`, require the ordered
predecessor facts, and return effective UID zero. The application derives
execute access from those observed facts; the provider neither consults the
host filesystem nor grants file contents or an executable mapping.

The application shall consume successful access to select one standard-output
write containing exact `busybox-executable\n`. The provider shall validate all
nineteen readable bytes, stage and compare them before publication, return
nineteen, erase staging, and record one private executable-condition output
fact. The following prompt and `pwd` shall require that fact. Terminal handling
shall clear the repeated-metadata, credential, output, and every predecessor
command-time fact.

## X86-64 binding

The repeated metadata query binds to syscall 262 with `AT_FDCWD=-100`, exact
`/bin/busybox\0`, the same complete 144-byte candidate and flags zero. The
credential query binds to syscall 107 with no arguments and result zero.
Selected output binds to syscall 1 on descriptor one with count and result
nineteen. Input results become 11, 5, 129, 4, and 5. The affine stack remains
`[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Exact execute-access policy, immutable identity and credential checks,
condition-selected output, checked transfers, staging erasure, observation
ordering, and terminal cleanup are portable Linux-personality policy. X86-64,
AArch64, and RISC-V may select different syscall encodings, credential and
permission implementations, calling conventions, mapping layouts, and
transfer mechanisms. No native selector, register, address, or instruction
becomes Topal source vocabulary.

## Alternatives rejected

Treating the predecessor metadata alone as proof that the application made its
executable decision was rejected because it bypasses the repeated lookup and
credential query observed from the pinned application.
Provider-owned confirmation, consulting host `/bin/busybox`, accepting
arbitrary paths or access modes, and opening or executing the file were
rejected because they break reproducibility or expand authority before the
access decision is independently qualified.

## Deliberate bounds

This increment proves one exact execute-access observation sequence and one
selected condition output for one immutable root-owned file. It does not add
opening, contents, mapping, execution, arbitrary permission checks, mutation,
dynamic applications, multiple processes, containers, or virtualization.
