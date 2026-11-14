# Step 4 increment: bounded static-process bin-busybox file condition

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file\n`.
After the previously qualified root and `/bin/busybox` glob matches, both the
clean fixture and the pinned external BusyBox issue exact
`newfstatat(AT_FDCWD, "/bin/busybox", ..., 0)`, observe one complete regular
file identity, emit `busybox-file\n`, and continue through the qualified `pwd`
and exit sequence.

The provider admits only the sealed inode-five file already published by the
`/bin` directory enumeration. It returns a zero-filled 144-byte x86-64 stat
record with inode five, link count one, mode `0100555`, root ownership, zero
size and times, and 4,096-byte preferred block size. The pathname, stat
record, and output are read back and erased before the affine lifecycle
advances. No content, descriptor, execution, mutation, symlink, arbitrary
path, host filesystem, or general VFS authority is added.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-FILE-CONDITION-001` and
decision `TK-DEC-231` bind the transition. Structural qualification inspects
the exact 80-byte command, five input publications, exact pathname and syscall
arguments, all 144 stat bytes, exact thirteen-byte selected output, staging
erasure, lifecycle states thirteen through fifteen, later `pwd`, terminal
clearing, and rollback.

Artifact `/210`, provider `/200`, provider object `/192`, root object `/199`,
boot adapter `/110`, and gate `/210` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
4,615-byte continuation containing the file lookup, complete metadata checks,
selected output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`ec477e32cfc73c80b6373274fdba23492dbf07f1f735c5d5dd503848ca76fdc2`
and
`72bd0f822114b6bd725d289fe4bd091977aea00bedff5041df71ab5fcd38b5e0`.
They bind boot image
`51364de8019160fef445efd3d6e0f08cc530feee5c6a775a92ce144dbddb33e3`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the exact pathname metadata and selected output, complete every
ordered marker, remain running afterward, and remain serially quiescent.
Their serial transcript hashes are
`396c29e2c1e076466731391283aa699db258d76ac10c45e373766255bd06d2c3`
and
`3f0bc10a16b4474508ca87c28729fc98251852fd3cfb9123cc12323ed7647a73`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because pathname metadata now affects shell condition selection
and visible output. Review combines exact ABI arguments, complete stat layout,
clean and external mapping profiles, affine staging ownership, output identity,
terminal cleanup, rollback, and physical runtime evidence.

This proves one successful regular-file condition for one immutable path. It
does not provide file opening or contents, execution, arbitrary metadata,
mutation, multiple processes, containers, or VM-host support.
