# Bounded static ELF startup credential normalization

Status: approved systems-feature design (`TK-DEC-197`).

## Purpose

After its optional `/etc/busybox.conf` lookup, the pinned BusyBox applet startup
path queries its real group ID and performs idempotent group-ID and user-ID
selections. The initial process already owns a complete sealed root credential
group. This increment admits that exact normalization without introducing
mutable credential or process-table machinery.

## Portable contract

Following the configuration-absence witness, the live initial process may
perform exactly three ordered pointer-free operations:

1. real group identity;
2. select the same sealed real/effective group identity; and
3. select the same sealed real/effective user identity.

The observation and both requests are derived from the corresponding immutable
auxiliary-vector facts. The provider revalidates the complete `AT_UID`,
`AT_EUID`, `AT_GID`, and `AT_EGID` group for every call, accepts only requested
identity zero, returns zero, retains no argument or identity state, and leaves
the credential group unchanged while advancing only the private startup phase.
Failure preserves the phase and every process resource.

The provider owns native syscall selectors, entry registers, return placement,
and phase representation. Portable source sees the ordered credential facts,
not Linux syscall numbers or a target register file.

## Initial x86-64 binding

The provider binds the sequence to Linux x86-64 `getgid` number 104, `setgid`
number 106, and `setuid` number 105. The clean-room process issues them in that
order, supplies zero to both setters, requires zero after each call, and then
exits. The digest-pinned root `echo` applet skips this optional branch after
`getuid(0)` and proceeds directly to its output dependency.

## Cross-architecture mapping

An AArch64 or RISC-V provider binds the same observation and idempotent
selections to its own Linux selectors, entry mechanism, argument/result
registers, and return instruction. It need not share x86-64 numbers,
instruction encodings, or private phase layout. Credential coherence,
no-change selection, and affine lifecycle are the abstraction.

## Exclusions

This increment adds no credential change, arbitrary setter, supplementary
groups, namespaces, capabilities, process table, multiple processes or
threads, filesystem, application output, shell, prompt, or BusyBox
redistribution.
