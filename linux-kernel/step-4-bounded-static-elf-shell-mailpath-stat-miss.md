# Step 4 increment: bounded static-process shell `MAILPATH` metadata miss

## Outcome

This increment adds the startup metadata lookup that follows the clean shell's
non-terminal warning. The clean fixture issues
`newfstatat(AT_FDCWD, "/var/spool/mail/root", candidate, 0)` and requires
`-ENOENT`. It prefills the complete native-stat candidate, proves all bytes are
unchanged after the miss, and erases and verifies the complete extent before
sealed command input.

The x86-64 provider requires the completed console, `/dev/tty`, ordered
terminal-probe, and warning observations; exact unchanged shell-input
predecessor phase; live dispositions; checked hierarchy; exact immutable path;
and a complete 144-byte writable initial-stack candidate. It allocates no
descriptor and retains no lookup state. One private byte records completion,
while the pinned external BusyBox echo profile continues to require the full
observation qword to remain zero.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-MAILPATH-STAT-MISS-001` and decision
`TK-DEC-210` bind the transition. Structural tests inspect native syscall 262,
`AT_FDCWD`, the exact path and flags, complete untouched-candidate witness,
exact predecessor and successor observations, result, nonallocation, and full
context restoration.

Artifact `/189`, provider `/179`, provider object `/171`, root object `/178`,
boot adapter `/89`, and gate `/191` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and the same
1,309-byte continuation; it differs from the Rust fixture only at the existing
gate sentinel byte.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`8f444299d76ac6c57101443048c6ca873b0aa4be3572726de8e8345da0c18a28`
and
`7e0230fe38b8c55d6b36493970ec981289b59841bacc39616c5ec9979a5aa614`.
They bind boot image
`5e5c610667d458206ab0ea0db81bf912c1b288d5f552e8782028f8c68bcc232b`,
distinguish the clean exact root-mailbox absence witness from the absent
external observation, complete all ordered markers, remain running afterward,
and remain serially quiescent. Both profiles passed twice. Because the lookup
has no output, their serial transcript hashes remain
`6a97d67d4c2d57c123c3c0efacff0897ce8f1b0af8f058277a2d95ce842989a7`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler, language, and semantic
suites, full Cargo workspace suite, formatting check, and diff check all pass
through the bounded validation runner.

## Risk and remaining boundary

Risk is high because this refines shared native metadata dispatch and extends
ordered private process state. Review combines exact state, pathname, selector,
flags, candidate, no-output, result, and nonallocation checks; full candidate
erasure; physical evidence; mirror parity; full-workspace validation; and
repeated clean plus external QEMU runs.

This does not provide generic pathname lookup, mailbox contents, filesystems,
the shell banner, profile lookup, a prompt, refillable input, general commands,
process creation, containers, or VM-host support.
