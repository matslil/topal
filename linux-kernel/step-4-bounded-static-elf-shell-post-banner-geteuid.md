# Step 4 increment: bounded static-process post-banner effective identity

## Outcome

This increment adds the clean shell's effective-user query immediately after
its post-banner current-directory publication. The clean fixture issues native
`geteuid`, requires exact result zero, and proceeds only after the result is
validated.

The x86-64 provider accepts the query only at the exact unchanged shell-input
phase with every prior interactive observation complete. It revalidates the
active hierarchy, retained initial-stack leaf, live dispositions, and all eight
tag/value words of the sealed real/effective user/group credential set before
returning effective UID zero. It retains no argument or mutable credential
state and publishes one fresh private fact after the erased banner-staging
extent. Sealed input and terminal rollback require that fact; rollback clears
and proves it absent. The pinned external BusyBox echo profile requires the
fact and all earlier optional observations to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-POST-BANNER-GETEUID-001` and decision
`TK-DEC-214` bind the transition. Structural tests inspect native syscall 107,
the exact predecessor observation, complete credential revalidation, exact
pointer-free result, private fact publication, successor gating, terminal
clearing, and full context restoration.

Artifact `/193`, provider `/183`, provider object `/175`, root object `/182`,
boot adapter `/93`, and gate `/195` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and carries the
same 1,695-byte continuation shape and exact 12-byte query sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`b9855f4382430d25041aeb62d339cc074077c9bc7fa2484fbc5125fc005b44e8`
and
`201493098153dfcec8afcd4fb546d67480eb1b8df28a64841f1244ac4e4a43cd`.
They bind boot image
`510c34316311ad9117bccf085e4324da4ad9a55708ed88f589dced0b02e32e30`,
distinguish the clean post-banner effective-root witness from the absent
external observation, complete all ordered markers, remain running afterward,
and remain serially quiescent. Both profiles passed twice. Their serial
transcript hashes are
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler, language, and semantic
suites, full Cargo workspace suite, formatting check, and diff check all pass
through the bounded validation runner.

## Risk and remaining boundary

Risk is high because this extends ordered application-controlled identity state
and makes rollback depend on another private lifecycle fact. Review combines
exact phase, predecessor, complete credential-group, pointer-free result,
publication, successor, and erasure checks; physical transcript evidence;
full-workspace validation; and repeated clean plus external QEMU runs.

This does not provide credential mutation, namespaces, profile lookup, a
prompt, refillable input, general commands, filesystems, processes, containers,
or VM-host support.
