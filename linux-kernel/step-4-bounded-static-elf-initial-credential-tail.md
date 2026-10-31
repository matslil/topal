# Step 4 increment: bounded static ELF startup credential normalization

## Outcome

This increment completes the credential-normalization sequence exercised by
the clean-room static process. After the approved configuration-stat miss, the
process calls Linux x86-64 `getgid`, `setgid(0)`, and `setuid(0)` in order,
receives zero from each call, and completes the existing affine process
rollback.

The provider accepts only those three syscall numbers at their exact startup
phases and executable continuations. Before each result it revalidates the
active address-space hierarchy, retained non-executable stack leaf, and the
complete sealed real/effective user/group auxiliary-vector credential group.
Both setters must request the already sealed zero identity; they return success
without changing credential state or retaining native arguments.

The digest-pinned external BusyBox `true` applet still terminates after its
earlier `getuid` path and therefore does not execute these additional calls.
Its header-bound terminal phase remains admitted, while the clean-room header
must complete all three new phases. This preserves the external regression
boundary without claiming behavior that the selected applet does not execute.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CREDENTIAL-TAIL-001` and decision
`TK-DEC-197` bind the result to the remaining initial scalar identity views.
Negative source tests reject another startup vocabulary, policy, marker, and
order. Structural tests inspect native syscall 104, 106, and 105 dispatch,
phase-specific admission, zero-valued setter arguments, hierarchy and
retained-stack validation, complete credential coherence, zero results, and
header-dependent terminal state. The clean fixture includes the exact ordered
calls and result checks; the external run protects the preceding real process
and rollback behavior.

Artifact `/176`, provider `/166`, provider object `/158`, root object `/165`,
boot adapter `/76`, and gate `/178` identify the qualified contract. The root
contains 9,482 typed provider relocations, including 6,858 UART writes, and the
artifact records 65 placements and a 483-transition semantic trace.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. Their SHA-256
values are respectively
`8691e9edd0273356d5ac46e5728acd18ab91dea5551aec033ec3776fc72a23bf`
and
`33e9bd49fa3fd1dac24007dbd24d54ce6562e3e5b6b64cec8092bb180aed4b87`.
Both pinned-QEMU runs complete every ordered marker, remain running afterward,
and remain serially quiescent after the terminal fatal disposition. They bind
the common boot image
`c46d55ebd81093444c6273c6e0bd8a60f4703fd2f4a451bc1cf58da049651803`
and serial transcript
`8fe946139fc397c9bf92082b215ddc2332e7f55d795f0a8bbacaa2eafed45f86`.
The external file remains outside the repository and is bound only by approved
SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

## Risk and remaining boundary

Risk is high because this extends native userspace ABI behavior and depends on
page-table and sealed-credential validation across repeated syscall returns.
Review combines authority-ordered design and specification, fail-closed native
structural checks, focused language/compiler tests, full-workspace validation,
and clean plus external QEMU executions.

This proves only one credential observation and two same-value startup
credential selections. It does not add arbitrary credential mutation, a VFS,
command output, a shell, Docker, Podman, or VM-host support.
