# Step 4 increment: bounded static-process root-glob enumeration

## Outcome

This increment replaces the third shell condition with exact `echo /*\n`.
Both profiles consume the same five ordered command publications, open the
sealed root directory, receive complete deterministic directory metadata,
enumerate the exact `.` and `..` records, observe end of directory, close the
description, and publish the application-selected literal `/*\n` output.

The provider accepts only `AT_FDCWD`, `/\0`, the pinned read-only directory
flags, descriptor three, and one complete native metadata or directory-record
candidate. It validates the clean stack or pinned BusyBox heap candidates,
stages every returned byte before comparison or publication, erases staging,
and advances one private affine lifecycle through open, metadata, records,
end-of-directory, close, output, and terminal clearing. No arbitrary pathname,
general glob evaluator, mutable directory, output stream, or general VFS is
introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-ROOT-GLOB-ENUMERATION-001` and decision
`TK-DEC-226` bind the transition. Structural qualification inspects all five
input publications, the complete eight-byte third command and zero tail, exact
open selector, pathname and flags, full 144-byte directory metadata, exact
512-byte clean and 32,768-byte external enumeration candidates, two 24-byte
`linux_dirent64` records, zero padding, explicit EOF, exact close and output,
private staging and erasure, the complete lifecycle, later `pwd`, terminal
clearing, and rollback.

Artifact `/205`, provider `/195`, provider object `/187`, root object `/194`,
boot adapter `/105`, and gate `/205` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
3,164-byte continuation with five reads, five clean prompts, root `chdir`, root
directory enumeration, literal glob output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`52a093385ce04942c6a38d6dfcde59f3a44af3f78b1926b3ed0baac068a12432`
and
`5c77fdd4242c6dae4c264855334205aacb05f9e448a2207bd285a5a0a5b853b9`.
They bind boot image
`9cf09ed50d3f853de6160f467f56d7a165df3860bb9c8b136e60ab0efd01a4a9`,
distinguish the clean stack witness from the pinned BusyBox heap witness,
complete every ordered marker, remain running afterward, and remain serially
quiescent. Their serial transcript hashes are
`de6f067007340b26be60e09cd65d420e2a5ec099fd269b148aa397f9423f51bd`
and
`064e14b14c6268a8502495532b966f19de1a839224e8993de749bbb4076aa7ff`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because the real application must consume native directory
metadata and records while descriptor state, candidate authority,
address-space state, staging, observation, and teardown remain bounded. Review
combines exact request bytes, complete metadata and records, clean and external
mapping profiles, EOF and close, transfer extent, affine ownership, terminal
cleanup, rollback, and physical runtime evidence.

This proves one successful enumeration of one sealed root directory containing
only `.` and `..`. It does not provide arbitrary glob patterns, relative paths,
traversal, named root children, file contents, mutable filesystem objects,
multiple processes, containers, or VM-host support.
