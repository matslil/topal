# Step 4 increment: bounded static-process bin-busybox nonempty condition

## Outcome

This increment extends the sealed shell command to
`echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty\n`.
After the previously qualified executable condition, both the clean fixture
and pinned external BusyBox issue a third exact
`newfstatat(AT_FDCWD, "/bin/busybox", ..., 0)`, observe the sealed artifact's
positive size, emit `busybox-nonempty\n`, and continue through the qualified
`pwd` and exit sequence.

The provider binds inode five to the already digest-checked 2,024,544-byte
static-user artifact and publishes 3,955 512-byte blocks, mode `0100555`, one
link, and 4,096-byte preferred I/O blocks on every metadata observation.
Pathname, complete 144-byte metadata, and output staging are read back and
erased before the affine lifecycle advances. No file contents, descriptor,
executable mapping, execution, mutation, host filesystem, arbitrary pathname,
or general VFS authority is added.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-NONEMPTY-CONDITION-001`
and decision `TK-DEC-233` bind the transition. Structural qualification
inspects the exact 176-byte command, five input publications, three exact
pathname and complete metadata observations, the exact 2,024,544-byte size and
3,955-block representation, exact seventeen-byte selected output, staging
erasure, lifecycle states nineteen and twenty, later `pwd`, terminal clearing,
and rollback.

Artifact `/212`, provider `/202`, provider object `/194`, root object `/201`,
boot adapter `/112`, and gate `/212` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte artifact extent and is
byte-identical to the Rust fixture over the full 5,682-byte continuation,
which contains all three file metadata observations and the file, executable,
nonempty, root, and terminal outputs.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`d4d26b1d5e57853a07cdfb8bd95ef33944df296cd0485043c0e780c12b42baca`
and
`8db7224a7bd246ff400871dbc1f7c0fdce79484de04df5c43f4c82d96d181cef`.
They bind boot image
`4ec6ec5532fa0c0c2546027c4abaa68800d6d1d9dd59dc95cbc1022803017d37`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
observe the sealed artifact size, third positive metadata result, and selected
nonempty-condition output, complete every ordered marker, remain running
afterward, and remain serially quiescent. Their serial transcript hashes are
`6e1c82dd40ff78144cf2cb53c84a819c7f16a6534666455f52c0092e46e8dcf0`
and
`d48eb70e63dc801db0a6aa2807d0cfd516b3864b469be79731f31634b8e6647b`.

The five Python lab contract tests, compiler structural suite, full Cargo
workspace suite, formatting check, and repeated clean plus external QEMU
executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because artifact identity and positive metadata now affect shell
condition selection and visible output. Review combines exact ABI arguments,
complete stat layout, artifact-byte binding, clean and external mapping
profiles, affine staging ownership, output identity, terminal cleanup,
rollback, and physical runtime evidence.

This proves one successful nonempty condition for one immutable path. It does
not provide file opening or contents, execution, arbitrary metadata or
permissions, mutation, multiple processes, containers, or VM-host support.
