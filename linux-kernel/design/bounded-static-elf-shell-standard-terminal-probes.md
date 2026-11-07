# Bounded static-process shell standard-terminal probes

This record resolves `TK-DEC-208` as the three ordered terminal-capability
probes immediately following the bounded shell's `/dev/tty` absence result. It
reports that none of the inherited standard descriptions provides terminal
attributes in this shell profile, without changing those descriptions or
claiming general terminal support.

## Decision

The bounded static-process entry may admit exactly three ordered terminal
attribute probes after the completed console-metadata and `/dev/tty`-absence
observations and before warning output or sealed command input. They shall name
standard error, standard output, and standard input in that order. Every probe
shall require the live process, sealed executable continuation, active checked
address-space hierarchy, retained initial stack, live child and ignored-quit
dispositions, exact shell-input predecessor phase, both prior observations,
the next expected description, a complete writable candidate, and the
terminal-attributes request.

Each operation shall return the not-a-terminal result without reading or
changing the candidate, descriptor state, process phase, or any process
resource. One provider-private count shall advance only after all checks and
shall admit each description at most once. Later warning output, sealed input,
and terminal rollback shall require the complete count. The clean fixture
shall issue and validate all three misses. The pinned external echo-command
profile shall omit the optional interactive prerequisites.

## X86-64 binding

The x86-64 realization binds the operations to native syscall 16 with
`TCGETS=0x5401`, descriptors 2, 1, and 0, a checked 36-byte candidate within
the retained initial stack, and exact result `-ENOTTY` (`-25`). The private
count shares the already qualified observation qword after the console-`fstat`
and `/dev/tty` bytes. The complete clean observation is two one-byte facts plus
count three; the external echo state remains the all-zero qword. Existing
initial-zero and complete qualification-restoration proofs cover the storage.

## Architecture pressure test

The ordered standard-description identities, terminal-capability request,
complete output extent, absence result, no-output effect, and observation
lifecycle are portable Linux-personality policy. X86-64, AArch64, and RISC-V
may select different syscall encodings, request encodings, native output
layouts, register assignments, and provider adapters. No selector, request
number, pointer, register, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Publishing fabricated terminal attributes was rejected because this bounded
shell profile does not implement their behavior. Treating the earlier
descriptor-zero probe as proof for all three descriptions was rejected because
BusyBox observes each description independently and in a different phase.
Returning success without output, skipping the calls, or coalescing them into
one observation was rejected because each changes application-visible control
flow or loses order. Adding general `ioctl` or mutable terminal state was
rejected because the observed prerequisite needs only three exact absence
facts. Activating interactive external arguments here was rejected because the
following warning output remains a separate testable dependency.

## Deliberate bounds

This increment admits three exact clean-fixture terminal misses and preserves
the external echo-command profile. It does not add terminal attributes,
successful terminal operations, arbitrary requests or descriptions, warning
or banner output, `MAILPATH` lookup, profile lookup, a prompt, mutable input,
processes, filesystems, containers, or virtualization.
