# Pinned static BusyBox `echo` profile

Status: approved systems-feature design (`TK-DEC-199`).

## Purpose

The first real external application profile advances from a silent BusyBox
`true` process to BusyBox `echo`. The executable remains the exact external,
digest-pinned Debian BusyBox artifact; only the sealed second argument changes.
This makes the already qualified one-byte newline operation mandatory for the
external process while retaining the root-specific shortcut around optional
configuration and credential normalization.

## Portable contract

Static process construction shall seal the semantic argument vector
`["busybox", "echo"]`, an empty environment, and the existing bounded auxiliary
vector. Every later stack-content fact, process-entry validation, syscall
return, and rollback check shall preserve that same argument identity until the
complete stack block is cleared. The profile name is
`bounded-static-elf-busybox-echo`.

The pinned external header variant shall no longer terminate after `getuid`.
Because that sealed result is root identity zero, BusyBox skips its optional
configuration and idempotent credential-normalization branch and proceeds
directly to exactly one staged line-feed write to standard output followed by
`exit_group(0)`. The clean-room fixture shall retain the longer configuration
and credential path before reaching the same output and terminal phase.

The sealed argument selection and required terminal phase are process-profile
facts. Native pointers, string layout, syscall selectors, instruction
addresses, and output-device mechanics remain provider-private. A different
argument, digest, call sequence, output byte, exit status, or terminal phase is
a fatal profile violation.

## X86-64 binding

The existing in-stack second string occupies the same five-byte extent and
pointer as `true`; its bytes become `echo\0`. The exact pinned BusyBox image
then consumes sealed root `getuid` result zero and executes
`write(1, candidate, 1)` followed by `exit_group(0)`. A differential native
trace that forces the `getuid` result to zero confirms this root-only branch.
Existing address-space, readable-backing, private-staging, native-context, and
affine rollback checks remain mandatory.

## Cross-architecture mapping

AArch64 or RISC-V uses the same semantic argument vector and process phases,
with its own Linux entry layout, syscall convention, executable build, and
console provider. The abstraction is selection and complete execution of one
digest-bound application profile, not an x86 string address or instruction
sequence.

## Exclusions

This profile does not add arbitrary arguments, more output bytes, option
parsing, environment variables, a shell, filesystem access, dynamic linking,
multiple processes, Docker, Podman, or virtualization-host support.
