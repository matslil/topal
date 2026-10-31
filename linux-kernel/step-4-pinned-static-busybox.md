# Step 4 increment: pinned static BusyBox activation

## Outcome

This increment admits one external static BusyBox executable identified by
SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.
The file remains an external gate input; repository artifacts contain only its
digest, package-derived resident-header facts, and reproducible observations.

The provider enters the exact `busybox true` initial process at the final Linux
stack, supplies the approved bounded startup syscall sequence, initializes and
restores x86-64 SIMD state, preserves both syscall-visible and provider caller
registers, seals GNU RELRO at the observed request, and restores every mutable
stack, context, and translation-status value before the existing affine
rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PROCESS-ENTRY-001` and decisions
`TK-DEC-193` through `TK-DEC-195` bind this result to one coherent resident
header, argument vector, process stack, syscall sequence, terminal status, and
complete inverse. Structural tests cover exact header selection, staged RELRO,
CR0/CR4 and SIMD setup, every saved Linux syscall register, every provider
callee-saved register, mutable stack-prefix clearing, complete leaf-status
normalization, context clearing, and both initial-stack variants. A wrong
external digest is rejected before any compiler or VM invocation.

Artifact `/174`, provider `/164`, provider object `/156`, root object `/163`,
boot adapter `/74`, and gate `/176` identify the qualified contract.

## Physical evidence

Qualification is recorded separately from the clean-room run in
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. Both gates use
the same kernel source, board manifest, compiler/provider revisions, ordered
markers, and full post-process rollback. The clean-room and external evidence
files have SHA-256 values
`e6a3b406c503aa7dbab4559d02d01de0b4b818b35ce0b7b193dcc91ecca2b71e`
and `8621bd99ed64f794933f6359f64d17aacc3c78258c1d6cd9b11caae06f154390`.
The external run binds boot image
`a6164d5d16ece923e09558c26d4c1160e00087350e324b3116666f98a43fc5e4`,
boot resource
`10845339b3c6028920c2d2849bacf2015f10b74b0d7b2e5826d5f08e98c8a2a8`,
artifact provenance
`e9be0a8087813e632cdce959459b66e9ed23aaa746d0ff0aa62bc36bcfdb7cc9`,
and the exact external image digest above.

## Risk and remaining boundary

Risk is high because ordinary compiled code exercises native privilege return,
SIMD controls, two calling conventions, writable-to-read-only translation
changes, hardware-maintained status bits, and process stack scratch. Review and
qualification therefore combine fail-closed structural checks, focused
compiler and gate tests, full-workspace validation, and independent clean-room
and external QEMU executions.

This proves the pinned external startup path only; it does not yet provide a
shell, general commands, Docker, Podman, or VM-host support.
