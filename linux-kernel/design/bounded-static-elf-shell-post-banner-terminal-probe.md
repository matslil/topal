# Bounded static-process shell post-banner terminal probe

This record resolves `TK-DEC-212` as the stdin terminal-capability observation
that follows the bounded shell's interactive banner. It preserves the second
observable probe without fabricating terminal attributes or treating the
earlier startup probe as a substitute.

## Decision

The bounded static-process entry may admit exactly one stdin terminal-attribute
query after the completed banner fact and before later current-directory,
identity, profile, prompt, or sealed-input work. The operation shall require the
live process, sealed executable continuation, active checked address-space
hierarchy, retained initial stack, live child and ignored-quit dispositions,
exact shell-input predecessor phase, every prior interactive observation,
standard input, the terminal-attribute request, and one complete writable
candidate. The provider shall return exact non-terminal absence, leave the
candidate and descriptor state unchanged, retain no user pointer, and leave
process phase unchanged.

One provider-private fact shall publish only after complete validation and
shall admit the probe at most once. Later startup work, sealed input, and
terminal rollback shall require that fact. The clean fixture shall place a
sentinel in the complete candidate, issue the query, verify the result and
unchanged sentinel, and erase and verify the candidate. The pinned external
echo-command profile shall omit this optional interactive observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 16, descriptor
zero, `TCGETS=0x5401`, a checked 36-byte candidate in the retained initial
stack, and result `-ENOTTY`. The private completion fact occupies the next byte
of the already qualified observation qword after the banner fact. The clean
observation ends with that byte equal to one; the external echo state remains
the all-zero qword.

## Architecture pressure test

The channel, operation kind, absence result, untouched-candidate effect,
no-retention effect, and observation lifecycle are portable Linux-personality
policy. X86-64, AArch64, and RISC-V may select different syscall and request
encodings, calling conventions, native terminal layouts, and address-space
validators. No selector, structure layout, register, pointer, or instruction
becomes Topal source vocabulary.

## Alternatives rejected

Reusing the earlier stdin probe without a call was rejected because it would
lose observable ordering. Reporting success or fabricating attributes was
rejected because no terminal object exists. Returning `-ENOSYS` was rejected
because it confuses a supported operation with a missing service. Adding
general `ioctl` or retaining the candidate was rejected because either
over-expands scope or weakens isolation.

## Deliberate bounds

This increment admits one exact clean-fixture post-banner terminal miss and
preserves the external echo-command profile. It does not add terminal
attributes, successful terminal operations, arbitrary `ioctl`, later current-
directory or identity queries, profile lookup, a prompt, mutable input,
filesystems, processes, containers, or virtualization.
