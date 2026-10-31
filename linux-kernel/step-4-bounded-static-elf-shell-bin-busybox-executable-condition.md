# Step 4 increment: bounded static-process bin-busybox executable condition

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable\n`.
After the previously qualified regular-file condition, both the clean fixture
and pinned external BusyBox repeat exact
`newfstatat(AT_FDCWD, "/bin/busybox", ..., 0)`, observe effective UID zero,
emit `busybox-executable\n`, and continue through the qualified `pwd` and exit
sequence.

The provider republishes the identical complete 144-byte inode-five metadata
and admits the credential query only after that ordered repeat. The application
derives execute permission from root ownership and mode `0100555`. Pathname,
metadata, and output staging are read back and erased before the affine
lifecycle advances. No content, descriptor, executable mapping, execution,
mutation, host filesystem, arbitrary permission query, or general VFS
authority is added.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-EXECUTABLE-CONDITION-001`
and decision `TK-DEC-232` bind the transition. Structural qualification
inspects the exact 129-byte command, five input publications, repeated exact
pathname and complete metadata, syscall 107 result zero, exact nineteen-byte
selected output, staging erasure, lifecycle states sixteen through eighteen,
absence of an unobserved `faccessat2` substitution, later `pwd`, terminal
clearing, and rollback.

Artifact `/211`, provider `/201`, provider object `/193`, root object `/200`,
boot adapter `/111`, and gate `/211` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
5,166-byte continuation containing both file metadata observations, the
credential query, selected output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`16bd981114cf9a65119bd090e22cc429d92f3cf6d1268742e946c5c1516a403f`
and
`6297a027dcb1eb99840fc79afdc71d761bdea97824a277a420c2ea04101d4bee`.
They bind boot image
`8157de12fc80c680163801d29e43eaf9b239c5647cb23a57f601aa1e096f7f4a`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the repeated metadata, effective-root credential, and selected
executable-condition output, complete every ordered marker, remain running
afterward, and remain serially quiescent. Their serial transcript hashes are
`07dfa0dfbaa64e3047a0b770de7efbf3be3c822fd06ea1220b2b4bc641dbd387`
and
`6936738ff2c91331160136e63536febb7df15a4398bee452b169bbfb90ca4fab`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because metadata and credentials now affect shell condition
selection and visible output. Review combines exact ABI arguments, complete
stat layout, clean and external mapping profiles, affine staging ownership,
credential ordering, output identity, terminal cleanup, rollback, and physical
runtime evidence.

This proves one successful executable condition for one immutable path. It
does not provide file opening or contents, execution, arbitrary metadata or
permissions, mutation, multiple processes, containers, or VM-host support.
