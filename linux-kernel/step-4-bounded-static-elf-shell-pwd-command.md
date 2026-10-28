# Step 4 increment: bounded static-process shell `pwd`

## Outcome

This increment adds a second output-producing shell built-in without admitting
an arbitrary command stream. Both qualified profiles receive exact
`echo topal\n`, `pwd\n`, and `exit\n` publications through one checked process
buffer, publish application-owned exact `/\n`, and consume four successive
affine input-owner states. The clean fixture publishes a prompt before every
command and performs a fresh command-time sealed-root query. Pinned BusyBox
keeps prompts absent, revalidates stdout with complete console metadata and an
exact `TCGETS` absence observation, then formats from its already sealed root
fact.

The process stack is now one affine two-base-page extent. Both adjacent pages
are reserved, zeroed, published RW/NX, verified, normalized, revoked, and
released together. The initial Linux ABI value remains in the upper page; the
lower page supplies the bounded working depth required by the pinned shell.
Terminal handling erases the entire lower page and the mutable upper-page
prefix before revoking both leaves.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PWD-COMMAND-001` and decision
`TK-DEC-221` bind the transition. Structural qualification inspects all three
input publications, owner states, clean `getcwd`, external descriptor-one
`fstat` and `TCGETS`, exact profile-specific output candidates, private
staging and erasure, both page-table leaves, complete two-page backing checks,
terminal clearing, hardware-status normalization, and affine rollback.

Artifact `/200`, provider `/190`, provider object `/182`, root object `/189`,
boot adapter `/100`, and gate `/201` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
2,383-byte continuation with three reads, three clean prompts, command-time
root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`7e57e03316dca1b078149dca081133d36680c5f885747bc583d1d628f677ea2f`
and
`1393b2a4972f3530109049721c45fbc334c9e9154f301ffaa9a6800a0adcf413`.
They bind boot image
`c811d140a9105c369a906b51585fd9e9d2742b91b9fdc345e945f64b83c3a5c2`,
distinguish fresh clean root observation from pinned external cached-root use,
complete every ordered marker, remain running afterward, and remain serially
quiescent. Their serial transcript hashes are
`a94cbc182209c6640aea4812f3a355139d04836f61f5c151b1374329a8da1459`
and
`94292fe0ae15f72d0d1413f0a70ea60fbb74574b9970d273b3fa467fc349eb4d`.

The nine Python lab contract tests, compiler and language suites, semantic
suite, full Cargo workspace suite, formatting check, diff check, and repeated
clean plus external QEMU executions pass through the bounded validation
workflow where applicable.

## Risk and remaining boundary

Risk is high because one real application profile differs from the clean
fixture at command time, the initial stack now spans two owned leaves, and
terminal rollback must erase and revoke the complete extent. Review combines
exact profile, predecessor, candidate, result, ownership, output, prompt,
memory, terminal, rollback, and physical runtime evidence.

This does not provide arbitrary input, pathname lookup, mutable directories,
stack growth, demand paging, a general command loop, dynamic applications,
multiple processes, containers, or VM-host support.
