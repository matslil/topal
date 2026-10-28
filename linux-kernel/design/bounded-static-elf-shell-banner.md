# Bounded static-process shell banner

This record resolves `TK-DEC-211` as the exact interactive banner emitted after
the bounded shell's root-mailbox metadata miss. It advances the clean shell to
later profile and prompt startup without replacing application output with a
provider-owned notice.

## Decision

The bounded static-process entry may admit exactly one standard-output write
after the completed `MAILPATH` absence observation and before later profile
lookup, prompt output, or sealed command input. The user process shall supply
the complete 109-byte sequence
`\n\nBusyBox v1.37.0 (Debian 1:1.37.0-6+b9) built-in shell (ash)\nEnter 'help' for a list of built-in commands.\n\n`
in one call. The operation shall require the live process, sealed executable
continuation, active checked address-space hierarchy, retained initial stack,
live child and ignored-quit dispositions, exact shell-input predecessor phase,
all prior interactive observations, descriptor one, exact length, and one
complete readable candidate.

The provider shall copy and validate all 109 bytes in private staging before
output becomes observable, publish only those copied bytes to the existing
console-output transport, return 109, erase and prove erasure of the complete
staging extent, retain no user pointer or payload, and leave descriptor state
and process phase unchanged. One provider-private fact shall publish only after
complete output and shall admit the banner at most once. Later startup work,
sealed input, and terminal rollback shall require that fact. The clean fixture
shall stage, issue, verify, and erase the complete banner. The pinned external
echo-command profile shall omit this optional interactive output.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 1, descriptor 1,
length and result 109, a checked candidate in the retained initial stack, a
fixed 112-byte provider-private staging extent whose final three bytes remain
zero, and the existing polled serial transport. The private completion fact
occupies the next byte of the already qualified observation qword after the
mailbox fact. The clean observation ends with that byte equal to one; the
external echo state remains the all-zero qword.

## Architecture pressure test

The exact payload, output channel, complete-copy-before-output rule, result,
staging erasure, no-retention effect, and observation lifecycle are portable
Linux-personality policy. X86-64, AArch64, and RISC-V may select different
syscall encodings, staging mechanisms, calling conventions, and physical
console-output adapters. No selector, register, pointer, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Emitting a provider-owned banner was rejected because it would bypass the
application write and user-copy evidence. Splitting or rewriting the payload
was rejected because the pinned executable performs one call with exact bytes
and result. Publishing before the complete copy and validation was rejected
because it could expose partial or substituted output. Reusing user memory as
staging or retaining its pointer was rejected because either weakens isolation.
Activating external interactive arguments here was rejected because profile
lookup, prompt output, and refillable input remain separate dependencies.

## Deliberate bounds

This increment admits the exact clean-fixture banner and preserves the external
echo-command profile. It does not add general standard-output formatting,
banner variants, profile lookup, a prompt, mutable input, filesystems,
processes, containers, or virtualization.
