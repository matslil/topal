# Step 4 increment: bounded static ELF initial configuration-stat miss

## Outcome

This increment preserves the optional BusyBox configuration probe without
claiming a filesystem. After the real-UID witness, the clean-room process calls
Linux x86-64 `newfstatat(AT_FDCWD, "/etc/busybox.conf", statbuf, 0)`, receives
exact `-ENOENT`, restores its stack pointer, and completes the existing affine
process rollback.

The provider accepts only the complete immutable pathname, a complete writable
144-byte native metadata destination in the retained stack page, flags zero,
the executable continuation, and the correct startup phase. It publishes no
metadata byte and retains no pathname, destination, or lookup state. The
provider-private resident-header discriminator binds the clean path to
`0x004a6900` and the digest-pinned external path to `0x005c0aa7`; these native
addresses do not enter Topal source.

The currently qualified external BusyBox `true` applet terminates after
`getuid` and therefore does not issue the optional probe. Its header-bound
terminal path remains admitted, while the clean-room header must reach the new
absence phase. This keeps external regression evidence honest and prepares the
next applet startup boundary without overstating executed behavior.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CONFIG-STAT-MISS-001` and decision
`TK-DEC-196` bind the result to one sealed path absence fact. Negative source
tests reject another startup vocabulary, policy, marker, and order. Structural
tests inspect native syscall 262 dispatch, both exact pathname candidates,
complete path bytes, stack-range and continuation validation, phase publication,
`-ENOENT`, and header-dependent terminal state. The clean fixture includes the
exact call and result check; the external run protects the preceding real
process and rollback behavior.

Artifact `/175`, provider `/165`, provider object `/157`, root object `/164`,
boot adapter `/75`, and gate `/177` identify the qualified contract. The root
contains 9,424 typed provider relocations, including 6,800 UART writes, and the
artifact records 65 placements and a 482-transition semantic trace.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. Their SHA-256
values are respectively
`001d7e1ad07e9077036ea58fb3e286d3ae017a64a1410ce55950e5c1a93eb3eb`
and
`ba7f300a34f0de894723fcc635d19522ced8cad979ec50579f2ee97c2e897340`.
Both pinned-QEMU runs complete every ordered marker, remain running afterward,
and remain serially quiescent after the terminal fatal disposition. They bind
the common boot image
`85497641ecd2d2180d2a01ae8d8059f4ba701926cabbbe37d1218c2d080c7a05`
and serial transcript
`8fc1c590a54f5026ccd8f0e06e890acb9d728738f4acaec0af89e15b02ed002a`.
The external file remains outside the repository and is bound only by approved
SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

## Risk and remaining boundary

Risk is high because this extends a native userspace ABI path and conditions
terminal acceptance on a private verified header variant. Review combines
authority-ordered design and specification, fail-closed native structural
checks, focused language/compiler tests, full-workspace validation, and clean
plus external QEMU executions.

This proves one optional configuration absence result. It does not add a VFS,
general metadata queries, the remaining scalar identity calls, command output,
a shell, Docker, Podman, or VM-host support.
